use super::{
    compute_squeak_anon_token, resolve_post_b62, CommentData, CommentRow, PostError,
    MAX_CONTENT_LEN,
};
use easy_errors::{insert_retry_on_duplicate, map_sqlx_error};
use sqlx::{AssertSqlSafe, Pool, Postgres};
use tracing::info;
use utils::PaginatedResponse;

pub async fn create_comment(
    pool: &Pool<Postgres>,
    sender_id: i64,
    _topic_name: &str,
    post_b62_or_slug: &str,
    content: &str,
) -> Result<(), PostError> {
    if content.len() > MAX_CONTENT_LEN {
        return Err(PostError::ContentTooLong);
    }

    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    insert_retry_on_duplicate::<PostError, _, _>(|| async {
        let hash = utils::random_b62(5);
        sqlx::query(
            "INSERT INTO comments (hash, sender_id, post_id, content)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&hash)
        .bind(sender_id)
        .bind(post_id)
        .bind(content)
        .execute(pool)
        .await?;
        info!(target: "post", "comment created sender_id={} post_id={} hash={}", sender_id, post_id, hash);
        Ok(())
    })
    .await
}

pub async fn get_all_comments(
    pool: &Pool<Postgres>,
    topic_name: &str,
    post_b62_or_slug: &str,
    maybe_user_id: Option<i64>,
    limit: i64,
    offset: i64,
    sort: &str,
) -> Result<PaginatedResponse<CommentData>, PostError> {
    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM comments WHERE post_id = $1")
        .bind(post_id)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;

    let order = match sort {
        "new" => "ORDER BY c.created_at DESC, c.id ASC",
        _ => "ORDER BY COALESCE(cv.vote_count, 0) DESC, c.created_at DESC, c.id ASC",
    };
    let rows: Vec<CommentRow> = {
        let sql = format!(
            "SELECT c.hash, c.content, c.created_at, c.deleted,
                    COALESCE(cv.vote_count, 0)::BIGINT as vote_count,
                    c.sender_id
             FROM comments c
             LEFT JOIN LATERAL (
                 SELECT COALESCE(SUM(direction), 0) as vote_count
                 FROM comment_votes
                 WHERE comment_id = c.id
             ) cv ON true
             WHERE c.post_id = $1
             {}
             LIMIT $2 OFFSET $3",
            order
        );
        sqlx::query_as(AssertSqlSafe(sql))
            .bind(post_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)? 
    };

    let data = rows
        .into_iter()
        .map(|row| {
            let (is_mine, anon_token) = if let Some(uid) = maybe_user_id {
                let mine = row.sender_id == uid;
                (
                    Some(mine),
                    Some(compute_squeak_anon_token(
                        uid,
                        row.sender_id,
                        topic_name,
                        post_b62_or_slug,
                    )),
                )
            } else {
                (None, None)
            };

            CommentData {
                hash: row.hash,
                content: row.content,
                created_at: row.created_at,
                deleted: row.deleted,
                vote_count: row.vote_count,
                anon_token,
                is_mine,
            }
        })
        .collect();

    Ok(PaginatedResponse::new(data)
        .with_total(total)
        .with_pagination(limit, offset))
}
