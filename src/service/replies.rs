use super::{
    compute_anon_token, resolve_post_b62, PostError, ReplyData, ReplyRow, MAX_CONTENT_LEN,
};
use easy_errors::{insert_retry_on_duplicate, map_sqlx_error};
use sqlx::{Pool, Postgres};
use tracing::info;
use utils::PaginatedResponse;

pub async fn create_reply(
    pool: &Pool<Postgres>,
    sender_id: i64,
    _topic_name: &str,
    post_b62_or_slug: &str,
    comment_hash: &str,
    content: &str,
    reply_hash: Option<&str>,
) -> Result<(), PostError> {
    if content.len() > MAX_CONTENT_LEN {
        return Err(PostError::ContentTooLong);
    }

    let comment_id: i64 = sqlx::query_scalar("SELECT id FROM comments WHERE hash = $1")
        .bind(comment_hash)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::CommentNotFound)?;

    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let reply_id: Option<i64> = if let Some(r_hash) = reply_hash {
        let r_id: Option<i64> = sqlx::query_scalar(
            "SELECT r.id FROM replies r
             WHERE r.hash = $1 AND r.post_id = $2 AND r.comment_id = $3 AND r.deleted = false",
        )
        .bind(r_hash)
        .bind(post_id)
        .bind(comment_id)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;

        Some(r_id.ok_or(PostError::ReplyNotFound)?)
    } else {
        None
    };

    insert_retry_on_duplicate::<PostError, _, _>(|| async {
        let hash = utils::random_b62(5);
        sqlx::query(
            "INSERT INTO replies (hash, sender_id, post_id, comment_id, reply_id, content)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&hash)
        .bind(sender_id)
        .bind(post_id)
        .bind(comment_id)
        .bind(reply_id)
        .bind(content)
        .execute(pool)
        .await?;
        info!(target: "post", "reply created sender_id={} post_id={} comment_id={} hash={}", sender_id, post_id, comment_id, hash);
        Ok(())
    })
    .await
}

pub async fn get_replies(
    pool: &Pool<Postgres>,
    topic_name: &str,
    post_b62_or_slug: &str,
    comment_hash: &str,
    maybe_user_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<PaginatedResponse<ReplyData>, PostError> {
    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT
         FROM replies r
         JOIN comments c ON c.id = r.comment_id
         WHERE r.post_id = $1 AND c.hash = $2 AND r.reply_id IS NULL",
    )
    .bind(post_id)
    .bind(comment_hash)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let top_ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT r.id
         FROM replies r
         JOIN comments c ON c.id = r.comment_id
         WHERE r.post_id = $1 AND c.hash = $2 AND r.reply_id IS NULL
         ORDER BY r.created_at, r.id
         LIMIT $3 OFFSET $4",
    )
    .bind(post_id)
    .bind(comment_hash)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let top_id_list: Vec<i64> = top_ids.into_iter().map(|(id,)| id).collect();

    let rows: Vec<ReplyRow> = if !top_id_list.is_empty() {
        sqlx::query_as(
            "WITH RECURSIVE reply_tree AS (
                 SELECT r.id, r.hash, r.content, r.created_at, r.deleted,
                        r.sender_id, r.reply_id, NULL::VARCHAR(5) as parent_hash,
                        COALESCE(rv.vote_count, 0)::BIGINT as vote_count
                 FROM replies r
                 LEFT JOIN LATERAL (
                     SELECT COALESCE(SUM(direction), 0) as vote_count
                     FROM reply_votes
                     WHERE reply_id = r.id
                 ) rv ON true
                 WHERE r.id = ANY($1)

                 UNION ALL

                 SELECT r.id, r.hash, r.content, r.created_at, r.deleted,
                        r.sender_id, r.reply_id, pr.hash as parent_hash,
                        COALESCE(rv.vote_count, 0)::BIGINT as vote_count
                 FROM replies r
                 JOIN reply_tree rt ON r.reply_id = rt.id
                 LEFT JOIN replies pr ON pr.id = r.reply_id
                 LEFT JOIN LATERAL (
                     SELECT COALESCE(SUM(direction), 0) as vote_count
                     FROM reply_votes
                     WHERE reply_id = r.id
                 ) rv ON true
             )
             SELECT id, hash, content, created_at, deleted, sender_id,
                    reply_id, parent_hash, vote_count
             FROM reply_tree
             ORDER BY created_at",
        )
        .bind(&top_id_list[..])
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
    } else {
        Vec::new()
    };

    let top_level = build_reply_tree(rows, maybe_user_id, topic_name, post_b62_or_slug);

    Ok(PaginatedResponse::new(top_level)
        .with_total(total)
        .with_pagination(limit, offset))
}

fn build_reply_tree(
    rows: Vec<ReplyRow>,
    maybe_user_id: Option<i64>,
    topic_name: &str,
    post_b62_or_slug: &str,
) -> Vec<ReplyData> {
    use std::collections::HashMap;

    let mut reply_map: HashMap<String, ReplyData> = HashMap::new();
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
    let mut roots: Vec<String> = Vec::new();

    for row in rows {
        let (is_mine, anon_token) = if let Some(uid) = maybe_user_id {
            let mine = row.sender_id == uid;
            (
                Some(mine),
                Some(compute_anon_token(
                    row.sender_id,
                    topic_name,
                    post_b62_or_slug,
                )),
            )
        } else {
            (None, None)
        };

        let hash = row.hash.clone();
        let parent_hash = row.parent_hash.clone();

        reply_map.insert(
            hash.clone(),
            ReplyData {
                hash: row.hash,
                content: row.content,
                created_at: row.created_at,
                deleted: row.deleted,
                vote_count: row.vote_count,
                anon_token,
                is_mine,
                children: Vec::new(),
            },
        );

        if let Some(ph) = parent_hash {
            children_of.entry(ph).or_default().push(hash.clone());
        } else {
            roots.push(hash);
        }
    }

    fn attach_children(
        hash: &str,
        reply_map: &mut HashMap<String, ReplyData>,
        children_of: &HashMap<String, Vec<String>>,
    ) -> ReplyData {
        let mut node = reply_map.remove(hash).expect("reply must exist");
        if let Some(child_hashes) = children_of.get(hash) {
            for child_hash in child_hashes {
                if reply_map.contains_key(child_hash) {
                    node.children
                        .push(attach_children(child_hash, reply_map, children_of));
                }
            }
        }
        node.children.sort_by_key(|a| a.created_at);
        node
    }

    let mut top_level: Vec<ReplyData> = Vec::new();
    for root_hash in roots {
        if reply_map.contains_key(&root_hash) {
            top_level.push(attach_children(&root_hash, &mut reply_map, &children_of));
        }
    }

    top_level.sort_by_key(|a| a.created_at);
    top_level
}
