use crate::errors::PostError;
use chrono::{DateTime, Utc};
use easy_errors::map_sqlx_error;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

const B62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn random_b62(len: usize) -> String {
    use rand::Rng;
    (0..len)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0..B62_CHARS.len());
            B62_CHARS[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_b62_has_correct_length() {
        for len in [1, 5, 10, 32] {
            let s = random_b62(len);
            assert_eq!(s.len(), len, "length {len}");
        }
    }

    #[test]
    fn random_b62_uses_valid_chars() {
        let s = random_b62(1000);
        for c in s.chars() {
            assert!(c.is_ascii_alphanumeric(), "invalid char '{c}'");
        }
    }

    #[test]
    fn random_b62_produces_different_values() {
        let a = random_b62(10);
        let b = random_b62(10);
        assert_ne!(a, b);
    }
}

#[derive(FromRow, Serialize)]
pub struct TopicData {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
}

pub async fn create_topic(
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

pub async fn get_topic(pool: &Pool<Postgres>, name: &str) -> Result<TopicData, PostError> {
    let topic: Option<TopicData> =
        sqlx::query_as("SELECT name, description, created_at, deleted FROM topics WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;

    topic.ok_or(PostError::TopicNotFound)
}

pub async fn get_all_topics(pool: &Pool<Postgres>) -> Result<Vec<TopicData>, PostError> {
    let topics: Vec<TopicData> = sqlx::query_as(
        "SELECT name, description, created_at, deleted FROM topics ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(topics)
}

#[derive(FromRow, Serialize)]
pub struct PostData {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
}

pub async fn create_post(
    pool: &Pool<Postgres>,
    creator_id: Uuid,
    topic_name: &str,
    title: &str,
    slug: &str,
    content: &str,
    image_url: &Option<String>,
) -> Result<String, PostError> {
    let topic_id: Uuid = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(topic_name)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::TopicNotFound)?;

    loop {
        let final_slug = format!("{}-{}", slug, random_b62(5));

        let result = sqlx::query(
            "INSERT INTO posts (creator_id, topic_id, title, slug, content, image_url)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(creator_id)
        .bind(topic_id)
        .bind(title)
        .bind(&final_slug)
        .bind(content)
        .bind(image_url)
        .execute(pool)
        .await;

        match result {
            Ok(_) => return Ok(final_slug),
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                continue;
            }
            Err(e) => return Err(map_sqlx_error(e)),
        }
    }
}

pub async fn get_post(
    pool: &Pool<Postgres>,
    topic_name: &str,
    post_slug: &str,
) -> Result<PostData, PostError> {
    let post: Option<PostData> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted
         FROM posts p
         JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1 AND p.slug = $2",
    )
    .bind(topic_name)
    .bind(post_slug)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    post.ok_or(PostError::PostNotFound)
}

pub async fn get_all_post(
    pool: &Pool<Postgres>,
    topic_name: &str,
) -> Result<Vec<PostData>, PostError> {
    let posts: Vec<PostData> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted
         FROM posts p
         JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1
         ORDER BY p.created_at",
    )
    .bind(topic_name)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(posts)
}

#[derive(FromRow, Serialize)]
pub struct CommentData {
    pub hash: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
}

pub async fn create_comment(
    pool: &Pool<Postgres>,
    sender_id: Uuid,
    topic_name: &str,
    post_slug: &str,
    content: &str,
) -> Result<(), PostError> {
    let post_id: Uuid = sqlx::query_scalar(
        "SELECT p.id FROM posts p JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1 AND p.slug = $2",
    )
    .bind(topic_name)
    .bind(post_slug)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .ok_or(PostError::PostNotFound)?;

    loop {
        let hash = random_b62(5);

        let result = sqlx::query(
            "INSERT INTO comments (hash, sender_id, post_id, content)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&hash)
        .bind(sender_id)
        .bind(post_id)
        .bind(content)
        .execute(pool)
        .await;

        match result {
            Ok(_) => return Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                continue;
            }
            Err(e) => return Err(map_sqlx_error(e)),
        }
    }
}

pub async fn get_all_comments(
    pool: &Pool<Postgres>,
    topic_name: &str,
    post_slug: &str,
) -> Result<Vec<CommentData>, PostError> {
    let comments: Vec<CommentData> = sqlx::query_as(
        "SELECT c.hash, c.content, c.created_at, c.deleted
         FROM comments c
         JOIN posts p ON p.id = c.post_id
         JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1 AND p.slug = $2
         ORDER BY c.created_at",
    )
    .bind(topic_name)
    .bind(post_slug)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(comments)
}

#[derive(FromRow, Serialize)]
pub struct ReplyData {
    pub hash: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
}

pub async fn create_reply(
    pool: &Pool<Postgres>,
    sender_id: Uuid,
    topic_name: &str,
    post_slug: &str,
    comment_hash: &str,
    content: &str,
) -> Result<(), PostError> {
    let comment_id: Uuid = sqlx::query_scalar("SELECT id FROM comments WHERE hash = $1")
        .bind(comment_hash)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::CommentNotFound)?;

    let post_id: Uuid = sqlx::query_scalar(
        "SELECT p.id FROM posts p JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1 AND p.slug = $2",
    )
    .bind(topic_name)
    .bind(post_slug)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .ok_or(PostError::PostNotFound)?;

    loop {
        let hash = random_b62(5);

        let result = sqlx::query(
            "INSERT INTO replies (hash, sender_id, post_id, comment_id, content)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&hash)
        .bind(sender_id)
        .bind(post_id)
        .bind(comment_id)
        .bind(content)
        .execute(pool)
        .await;

        match result {
            Ok(_) => return Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                continue;
            }
            Err(e) => return Err(map_sqlx_error(e)),
        }
    }
}

pub async fn get_replies(
    pool: &Pool<Postgres>,
    topic_name: &str,
    post_slug: &str,
    comment_hash: &str,
) -> Result<Vec<ReplyData>, PostError> {
    let replies: Vec<ReplyData> = sqlx::query_as(
        "SELECT r.hash, r.content, r.created_at, r.deleted
         FROM replies r
         JOIN posts p ON p.id = r.post_id
         JOIN topics t ON t.id = p.topic_id
         JOIN comments c ON c.id = r.comment_id
         WHERE t.name = $1 AND p.slug = $2 AND c.hash = $3
         ORDER BY r.created_at",
    )
    .bind(topic_name)
    .bind(post_slug)
    .bind(comment_hash)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(replies)
}
