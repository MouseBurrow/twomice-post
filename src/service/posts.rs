use super::{
    compute_anon_token, resolve_post_b62, validate_tags, PostData, PostError, PostRow,
    MAX_CONTENT_LEN, MAX_TAGS_PER_POST, MAX_TITLE_LEN,
};
use easy_errors::map_sqlx_error;
use sqlx::{Pool, Postgres};
use tracing::info;

pub async fn create_post(
    pool: &Pool<Postgres>,
    creator_id: i64,
    topic_name: &str,
    title: &str,
    content: &str,
    image_url: &Option<String>,
    tags: &Option<Vec<String>>,
) -> Result<String, PostError> {
    if title.len() > MAX_TITLE_LEN || content.len() > MAX_CONTENT_LEN {
        return Err(PostError::ContentTooLong);
    }

    let topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(topic_name)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::TopicNotFound)?;

    if let Some(tags) = tags {
        if tags.len() > MAX_TAGS_PER_POST {
            return Err(PostError::InvalidTags);
        }
        validate_tags(pool, topic_id, tags).await?;
    }

    let post_id: i64 = sqlx::query_scalar(
        "INSERT INTO posts (creator_id, topic_id, title, slug, content, image_url, tags)
         VALUES ($1, $2, $3, '', $4, $5, $6)
         RETURNING id",
    )
    .bind(creator_id)
    .bind(topic_id)
    .bind(title)
    .bind(content)
    .bind(image_url)
    .bind(tags.as_deref().unwrap_or(&[]))
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let slug = utils::encode_b62(post_id);
    sqlx::query("UPDATE posts SET slug = $1 WHERE id = $2")
        .bind(&slug)
        .bind(post_id)
        .execute(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;

    info!(target: "post", "post created creator_id={} slug={}", creator_id, slug);
    Ok(slug)
}

pub async fn get_post(
    pool: &Pool<Postgres>,
    post_b62_or_slug: &str,
    maybe_user_id: Option<i64>,
) -> Result<PostData, PostError> {
    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let row: Option<PostRow> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                COALESCE(p.tags, '{}') as tags,
                (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT
                + (SELECT COUNT(*) FROM replies r WHERE r.post_id = p.id AND r.deleted = false)::BIGINT
                as reply_count,
                p.view_count,
                t.name as board_id,
                p.creator_id
         FROM posts p
         JOIN topics t ON t.id = p.topic_id
         LEFT JOIN LATERAL (
             SELECT COALESCE(SUM(direction), 0) as vote_count
             FROM post_votes
             WHERE post_id = p.id
         ) pv ON true
         WHERE p.id = $1",
    )
    .bind(post_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let row = row.ok_or(PostError::PostNotFound)?;

    if let Err(e) = sqlx::query("UPDATE posts SET view_count = view_count + 1 WHERE id = $1")
        .bind(post_id)
        .execute(pool)
        .await
    {
        tracing::warn!(target: "post", "failed to increment view_count for post_id={}: {e}", post_id);
    }

    let is_hot = row.vote_count > 10 || row.view_count > 100;

    let (is_mine, anon_token) = if let Some(uid) = maybe_user_id {
        if let Some(cid) = row.creator_id {
            let mine = cid == uid;
            let token = if mine {
                let board = row.board_id.as_deref().unwrap_or("");
                Some(compute_anon_token(uid, board, &row.slug))
            } else {
                None
            };
            (Some(mine), token)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(PostData {
        title: row.title,
        slug: row.slug,
        content: row.content,
        image_url: row.image_url,
        created_at: row.created_at,
        deleted: row.deleted,
        vote_count: row.vote_count,
        anon_token,
        is_mine,
        tags: row.tags,
        reply_count: row.reply_count,
        view_count: row.view_count,
        is_hot,
        board_id: row.board_id,
    })
}

pub async fn get_all_posts(
    pool: &Pool<Postgres>,
    topic_name: &str,
    maybe_user_id: Option<i64>,
) -> Result<Vec<PostData>, PostError> {
    let rows: Vec<PostRow> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                COALESCE(p.tags, '{}') as tags,
                (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT
                + (SELECT COUNT(*) FROM replies r WHERE r.post_id = p.id AND r.deleted = false)::BIGINT
                as reply_count,
                p.view_count,
                t.name as board_id,
                p.creator_id
         FROM posts p
         JOIN topics t ON t.id = p.topic_id
         LEFT JOIN LATERAL (
             SELECT COALESCE(SUM(direction), 0) as vote_count
             FROM post_votes
             WHERE post_id = p.id
         ) pv ON true
         WHERE t.name = $1
         ORDER BY p.created_at",
    )
    .bind(topic_name)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let result = rows
        .into_iter()
        .map(|row| {
            let is_hot = row.vote_count > 10 || row.view_count > 100;

            let (is_mine, anon_token) = if let Some(uid) = maybe_user_id {
                if let Some(cid) = row.creator_id {
                    let mine = cid == uid;
                    let token = if mine {
                        Some(compute_anon_token(uid, topic_name, &row.slug))
                    } else {
                        None
                    };
                    (Some(mine), token)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            PostData {
                title: row.title,
                slug: row.slug,
                content: row.content,
                image_url: row.image_url,
                created_at: row.created_at,
                deleted: row.deleted,
                vote_count: row.vote_count,
                anon_token,
                is_mine,
                tags: row.tags,
                reply_count: row.reply_count,
                view_count: row.view_count,
                is_hot,
                board_id: row.board_id,
            }
        })
        .collect();

    Ok(result)
}

pub async fn get_user_posts(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<Vec<PostData>, PostError> {
    let mut posts: Vec<PostData> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                NULL::TEXT as anon_token,
                true as is_mine,
                COALESCE(p.tags, '{}') as tags,
                (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT
                + (SELECT COUNT(*) FROM replies r WHERE r.post_id = p.id AND r.deleted = false)::BIGINT
                as reply_count,
                p.view_count,
                false as is_hot,
                t.name as board_id
         FROM posts p
         JOIN topics t ON t.id = p.topic_id
         LEFT JOIN LATERAL (
             SELECT COALESCE(SUM(direction), 0) as vote_count
             FROM post_votes
             WHERE post_id = p.id
         ) pv ON true
         WHERE p.creator_id = $1
         ORDER BY p.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    for post in &mut posts {
        post.is_hot = post.vote_count > 10 || post.view_count > 100;
        let board = post.board_id.as_deref().unwrap_or("");
        post.anon_token = Some(compute_anon_token(user_id, board, &post.slug));
    }

    Ok(posts)
}
