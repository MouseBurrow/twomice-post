use super::{InternalUserStats, PostError};
use easy_errors::map_sqlx_error;
use sqlx::{Pool, Postgres};

pub async fn get_user_content_stats(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<InternalUserStats, PostError> {
    let post_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM posts WHERE creator_id = $1 AND deleted = false",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .unwrap_or(0);

    let comment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM comments WHERE sender_id = $1 AND deleted = false",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .unwrap_or(0);

    let post_upvotes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(pv.direction), 0)::BIGINT
         FROM post_votes pv
         JOIN posts p ON p.id = pv.post_id
         WHERE p.creator_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .unwrap_or(0);

    let comment_upvotes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(cv.direction), 0)::BIGINT
         FROM comment_votes cv
         JOIN comments c ON c.id = cv.comment_id
         WHERE c.sender_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .unwrap_or(0);

    Ok(InternalUserStats {
        post_count,
        comment_count,
        upvote_count: post_upvotes + comment_upvotes,
    })
}
