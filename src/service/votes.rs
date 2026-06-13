use super::PostError;
use easy_errors::map_sqlx_error;
use sqlx::{Pool, Postgres};
use tracing::info;

pub async fn cast_post_vote(
    pool: &Pool<Postgres>,
    user_id: i64,
    post_id: i64,
    direction: i8,
) -> Result<i64, PostError> {
    if direction == 0 {
        sqlx::query("DELETE FROM post_votes WHERE user_id = $1 AND post_id = $2")
            .bind(user_id)
            .bind(post_id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;
    } else {
        let dir: i16 = if direction > 0 { 1 } else { -1 };
        sqlx::query(
            "INSERT INTO post_votes (user_id, post_id, direction)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, post_id) DO UPDATE SET direction = $3",
        )
        .bind(user_id)
        .bind(post_id)
        .bind(dir)
        .execute(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(direction), 0)::BIGINT FROM post_votes WHERE post_id = $1",
    )
    .bind(post_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    info!(target: "post", "post vote cast user_id={} post_id={} direction={} count={}", user_id, post_id, direction, count);
    Ok(count)
}

pub async fn cast_comment_vote(
    pool: &Pool<Postgres>,
    user_id: i64,
    comment_id: i64,
    direction: i8,
) -> Result<i64, PostError> {
    if direction == 0 {
        sqlx::query("DELETE FROM comment_votes WHERE user_id = $1 AND comment_id = $2")
            .bind(user_id)
            .bind(comment_id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;
    } else {
        let dir: i16 = if direction > 0 { 1 } else { -1 };
        sqlx::query(
            "INSERT INTO comment_votes (user_id, comment_id, direction)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, comment_id) DO UPDATE SET direction = $3",
        )
        .bind(user_id)
        .bind(comment_id)
        .bind(dir)
        .execute(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(direction), 0)::BIGINT FROM comment_votes WHERE comment_id = $1",
    )
    .bind(comment_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    info!(target: "post", "comment vote cast user_id={} comment_id={} direction={} count={}", user_id, comment_id, direction, count);
    Ok(count)
}

pub async fn cast_reply_vote(
    pool: &Pool<Postgres>,
    user_id: i64,
    reply_id: i64,
    direction: i8,
) -> Result<i64, PostError> {
    if direction == 0 {
        sqlx::query("DELETE FROM reply_votes WHERE user_id = $1 AND reply_id = $2")
            .bind(user_id)
            .bind(reply_id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;
    } else {
        let dir: i16 = if direction > 0 { 1 } else { -1 };
        sqlx::query(
            "INSERT INTO reply_votes (user_id, reply_id, direction)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, reply_id) DO UPDATE SET direction = $3",
        )
        .bind(user_id)
        .bind(reply_id)
        .bind(dir)
        .execute(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(direction), 0)::BIGINT FROM reply_votes WHERE reply_id = $1",
    )
    .bind(reply_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    info!(target: "post", "reply vote cast user_id={} reply_id={} direction={} count={}", user_id, reply_id, direction, count);
    Ok(count)
}
