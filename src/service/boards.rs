use super::{BoardData, BoardSummary, PostError};
use easy_errors::map_sqlx_error;
use sqlx::{Pool, Postgres};

pub async fn create_board(
    pool: &Pool<Postgres>,
    name: &str,
    description: &str,
) -> Result<(), PostError> {
    sqlx::query("INSERT INTO topics (name, description) VALUES ($1, $2)")
        .bind(name)
        .bind(description)
        .execute(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;

    Ok(())
}

pub async fn get_board(pool: &Pool<Postgres>, name: &str) -> Result<BoardData, PostError> {
    let board: Option<BoardData> =
        sqlx::query_as("SELECT name, description, created_at, deleted FROM topics WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;

    board.ok_or(PostError::TopicNotFound)
}

pub async fn get_all_boards(pool: &Pool<Postgres>) -> Result<Vec<BoardData>, PostError> {
    let boards: Vec<BoardData> = sqlx::query_as(
        "SELECT name, description, created_at, deleted FROM topics ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(boards)
}

pub async fn get_active_boards(
    pool: &Pool<Postgres>,
    limit: i64,
) -> Result<Vec<BoardSummary>, PostError> {
    let boards: Vec<BoardSummary> = sqlx::query_as(
        "SELECT t.name, t.description, COUNT(p.id)::BIGINT as post_count
         FROM topics t
         LEFT JOIN posts p ON p.topic_id = t.id AND p.deleted = false
         WHERE t.deleted = false
         GROUP BY t.id, t.name, t.description
         ORDER BY post_count DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(boards)
}

pub async fn get_board_tags(
    pool: &Pool<Postgres>,
    topic_name: &str,
) -> Result<Vec<String>, PostError> {
    let topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(topic_name)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::TopicNotFound)?;

    let tags: Vec<String> =
        sqlx::query_scalar("SELECT tag_name FROM topic_tags WHERE topic_id = $1 ORDER BY tag_name")
            .bind(topic_id)
            .fetch_all(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;

    Ok(tags)
}
