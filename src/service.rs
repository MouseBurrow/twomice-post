use crate::errors::PostError;
use chrono::{DateTime, Utc};
use easy_errors::{insert_retry_on_duplicate, map_sqlx_error};
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Pool, Postgres};

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
    creator_id: i64,
    topic_name: &str,
    title: &str,
    slug: &str,
    content: &str,
    image_url: &Option<String>,
) -> Result<String, PostError> {
    let topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(topic_name)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::TopicNotFound)?;

    let mut final_slug = String::new();

    insert_retry_on_duplicate::<PostError, _, _>(|| {
        let fs = format!("{}-{}", slug, utils::random_b62(5));
        final_slug = fs.clone();
        async move {
            sqlx::query(
                "INSERT INTO posts (creator_id, topic_id, title, slug, content, image_url)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(creator_id)
            .bind(topic_id)
            .bind(title)
            .bind(&fs)
            .bind(content)
            .bind(image_url)
            .execute(pool)
            .await?;
            Ok(())
        }
    })
    .await?;

    Ok(final_slug)
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
pub struct NoteData {
    pub hash: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
}
pub type CommentData = NoteData;
pub type ReplyData = NoteData;

pub async fn create_comment(
    pool: &Pool<Postgres>,
    sender_id: i64,
    topic_name: &str,
    post_slug: &str,
    content: &str,
) -> Result<(), PostError> {
    let post_id: i64 = sqlx::query_scalar(
        "SELECT p.id FROM posts p JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1 AND p.slug = $2",
    )
    .bind(topic_name)
    .bind(post_slug)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .ok_or(PostError::PostNotFound)?;

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
        Ok(())
    })
    .await
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

pub async fn create_reply(
    pool: &Pool<Postgres>,
    sender_id: i64,
    topic_name: &str,
    post_slug: &str,
    comment_hash: &str,
    content: &str,
) -> Result<(), PostError> {
    let comment_id: i64 = sqlx::query_scalar("SELECT id FROM comments WHERE hash = $1")
        .bind(comment_hash)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::CommentNotFound)?;

    let post_id: i64 = sqlx::query_scalar(
        "SELECT p.id FROM posts p JOIN topics t ON t.id = p.topic_id
         WHERE t.name = $1 AND p.slug = $2",
    )
    .bind(topic_name)
    .bind(post_slug)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .ok_or(PostError::PostNotFound)?;

    insert_retry_on_duplicate::<PostError, _, _>(|| async {
        let hash = utils::random_b62(5);
        sqlx::query(
            "INSERT INTO replies (hash, sender_id, post_id, comment_id, content)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&hash)
        .bind(sender_id)
        .bind(post_id)
        .bind(comment_id)
        .bind(content)
        .execute(pool)
        .await?;
        Ok(())
    })
    .await
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
