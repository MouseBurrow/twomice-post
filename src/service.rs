use crate::errors::PostError;
use chrono::{DateTime, Utc};
use easy_errors::{insert_retry_on_duplicate, map_sqlx_error};
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Pool, Postgres};
use std::sync::OnceLock;

const MAX_TITLE_LEN: usize = 200;
const MAX_CONTENT_LEN: usize = 50000;
const MAX_TAGS_PER_POST: usize = 5;

async fn resolve_post_b62(pool: &Pool<Postgres>, post_b62_or_slug: &str) -> Result<i64, PostError> {
    if let Some(id) = utils::decode_b62(post_b62_or_slug) {
        return Ok(id);
    }
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM posts WHERE slug = $1")
        .bind(post_b62_or_slug)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    id.ok_or(PostError::PostNotFound)
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
pub struct BoardSummary {
    pub name: String,
    pub description: String,
    pub post_count: i64,
}

#[derive(FromRow)]
struct CommentRow {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
    vote_count: i64,
    sender_id: i64,
}

#[derive(FromRow)]
struct ReplyRow {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
    sender_id: i64,
}

#[derive(FromRow)]
struct PostRow {
    title: String,
    slug: String,
    content: String,
    image_url: Option<String>,
    created_at: DateTime<Utc>,
    deleted: bool,
    vote_count: i64,
    tags: Vec<String>,
    reply_count: i64,
    view_count: i64,
    board_id: Option<String>,
    creator_id: Option<i64>,
}

#[derive(FromRow, Serialize)]
pub struct PostData {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
    #[sqlx(default)]
    pub vote_count: i64,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anon_token: Option<String>,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mine: Option<bool>,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[sqlx(default)]
    pub reply_count: i64,
    #[sqlx(default)]
    pub view_count: i64,
    #[sqlx(default)]
    pub is_hot: bool,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_id: Option<String>,
}

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

    Ok(slug)
}

async fn validate_tags(pool: &Pool<Postgres>, topic_id: i64, tags: &[String]) -> Result<(), PostError> {
    let allowed: Vec<String> = sqlx::query_scalar(
        "SELECT tag_name FROM topic_tags WHERE topic_id = $1",
    )
    .bind(topic_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    if allowed.is_empty() {
        return Ok(());
    }

    for tag in tags {
        if !allowed.contains(tag) {
            return Err(PostError::InvalidTags);
        }
    }

    Ok(())
}

pub async fn get_topic_tags(
    pool: &Pool<Postgres>,
    topic_name: &str,
) -> Result<Vec<String>, PostError> {
    let topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(topic_name)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?
        .ok_or(PostError::TopicNotFound)?;

    let tags: Vec<String> = sqlx::query_scalar(
        "SELECT tag_name FROM topic_tags WHERE topic_id = $1 ORDER BY tag_name",
    )
    .bind(topic_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    Ok(tags)
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
                COALESCE(p.tags, '{{}}') as tags,
                (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT as reply_count,
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

    let _ = sqlx::query("UPDATE posts SET view_count = view_count + 1 WHERE id = $1")
        .bind(post_id)
        .execute(pool)
        .await;

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

pub async fn get_all_post(
    pool: &Pool<Postgres>,
    topic_name: &str,
    maybe_user_id: Option<i64>,
) -> Result<Vec<PostData>, PostError> {
    let rows: Vec<PostRow> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                COALESCE(p.tags, '{{}}') as tags,
                (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT as reply_count,
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

    let result = rows.into_iter().map(|row| {
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
    }).collect();

    Ok(result)
}

#[derive(FromRow, Serialize)]
pub struct CommentData {
    pub hash: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
    #[sqlx(default)]
    pub vote_count: i64,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anon_token: Option<String>,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mine: Option<bool>,
}
pub type NoteData = CommentData;
pub type ReplyData = NoteData;

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
        Ok(())
    })
    .await
}

pub async fn get_all_comments(
    pool: &Pool<Postgres>,
    topic_name: &str,
    post_b62_or_slug: &str,
    maybe_user_id: Option<i64>,
) -> Result<Vec<CommentData>, PostError> {
    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let rows: Vec<CommentRow> = sqlx::query_as(
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
         ORDER BY c.created_at",
    )
    .bind(post_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let result = rows.into_iter().map(|row| {
        let (is_mine, anon_token) = if let Some(uid) = maybe_user_id {
            let mine = row.sender_id == uid;
            (
                Some(mine),
                Some(compute_squeak_anon_token(uid, row.sender_id, topic_name, post_b62_or_slug)),
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
    }).collect();

    Ok(result)
}

pub async fn create_reply(
    pool: &Pool<Postgres>,
    sender_id: i64,
    _topic_name: &str,
    post_b62_or_slug: &str,
    comment_hash: &str,
    content: &str,
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
    post_b62_or_slug: &str,
    comment_hash: &str,
    maybe_user_id: Option<i64>,
) -> Result<Vec<ReplyData>, PostError> {
    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let rows: Vec<ReplyRow> = sqlx::query_as(
        "SELECT r.hash, r.content, r.created_at, r.deleted, r.sender_id
         FROM replies r
         JOIN comments c ON c.id = r.comment_id
         WHERE r.post_id = $1 AND c.hash = $2
         ORDER BY r.created_at",
    )
    .bind(post_id)
    .bind(comment_hash)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?;

    let result = rows.into_iter().map(|row| {
        let (is_mine, anon_token) = if let Some(uid) = maybe_user_id {
            let mine = row.sender_id == uid;
            (
                Some(mine),
                Some(compute_squeak_anon_token(uid, row.sender_id, topic_name, post_b62_or_slug)),
            )
        } else {
            (None, None)
        };

        ReplyData {
            hash: row.hash,
            content: row.content,
            created_at: row.created_at,
            deleted: row.deleted,
            vote_count: 0,
            anon_token,
            is_mine,
        }
    }).collect();

    Ok(result)
}

#[derive(Serialize)]
pub struct InternalUserStats {
    pub nib_count: i64,
    pub squeak_count: i64,
    pub upvote_count: i64,
}

fn get_anon_salt() -> &'static str {
    static SALT: OnceLock<String> = OnceLock::new();
    SALT.get_or_init(|| {
        std::env::var("ANON_SALT").unwrap_or_else(|_| {
            utils::random_b62(32)
        })
    })
}

fn compute_anon_token(user_id: i64, board_name: &str, post_slug: &str) -> String {
    use sha2::{Digest, Sha256};
    let salt = get_anon_salt();
    let input = format!("{}:{}:{}:{}", user_id, board_name, post_slug, salt);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..8])
}

fn compute_squeak_anon_token(viewer_id: i64, sender_id: i64, board_name: &str, post_slug: &str) -> String {
    use sha2::{Digest, Sha256};
    let salt = get_anon_salt();
    let input = format!("sqk:{}:{}:{}:{}:{}", viewer_id, sender_id, board_name, post_slug, salt);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..8])
}

pub async fn resolve_post_id(
    pool: &Pool<Postgres>,
    _topic_name: &str,
    post_b62_or_slug: &str,
) -> Result<i64, PostError> {
    resolve_post_b62(pool, post_b62_or_slug).await
}

pub async fn resolve_comment_id(
    pool: &Pool<Postgres>,
    comment_hash: &str,
) -> Result<i64, PostError> {
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM comments WHERE hash = $1")
        .bind(comment_hash)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    id.ok_or(PostError::CommentNotFound)
}

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

    Ok(count)
}

pub async fn get_active_topics(
    pool: &Pool<Postgres>,
    limit: i64,
) -> Result<Vec<BoardSummary>, PostError> {
    let topics: Vec<BoardSummary> = sqlx::query_as(
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

    Ok(topics)
}

pub async fn get_feed_nibs(
    pool: &Pool<Postgres>,
    sort: &str,
    _app_env: &config::app_envs::AppEnvs,
) -> Result<Vec<PostData>, PostError> {
    let rows: Vec<PostRow> = match sort {
        "new" => sqlx::query_as(
            "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                    COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                    COALESCE(p.tags, '{{}}') as tags,
                    (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT as reply_count,
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
             WHERE p.deleted = false AND t.deleted = false
             ORDER BY p.created_at DESC
             LIMIT 100",
        )
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?,

        "top" => sqlx::query_as(
            "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                    COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                    COALESCE(p.tags, '{{}}') as tags,
                    (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT as reply_count,
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
             WHERE p.deleted = false AND t.deleted = false
             ORDER BY COALESCE(pv.vote_count, 0) DESC, p.created_at DESC
             LIMIT 100",
        )
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?,

        _ => sqlx::query_as(
            "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                    COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                    COALESCE(p.tags, '{{}}') as tags,
                    (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT as reply_count,
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
             WHERE p.deleted = false AND t.deleted = false
             ORDER BY COALESCE(pv.vote_count, 0) DESC, p.created_at DESC
             LIMIT 100",
        )
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?,
    };

    let nibs = rows.into_iter().map(|row| {
        PostData {
            title: row.title,
            slug: row.slug,
            content: row.content,
            image_url: row.image_url,
            created_at: row.created_at,
            deleted: row.deleted,
            vote_count: row.vote_count,
            anon_token: None,
            is_mine: None,
            tags: row.tags,
            reply_count: row.reply_count,
            view_count: row.view_count,
            is_hot: row.vote_count > 10 || row.view_count > 100,
            board_id: row.board_id,
        }
    }).collect();

    Ok(nibs)
}

pub async fn get_user_nibs(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<Vec<PostData>, PostError> {
    let mut nibs: Vec<PostData> = sqlx::query_as(
        "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted,
                COALESCE(pv.vote_count, 0)::BIGINT as vote_count,
                NULL::TEXT as anon_token,
                true as is_mine,
                COALESCE(p.tags, '{{}}') as tags,
                (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT as reply_count,
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

    for nib in &mut nibs {
        nib.is_hot = nib.vote_count > 10 || nib.view_count > 100;
        let board = nib.board_id.as_deref().unwrap_or("");
        nib.anon_token = Some(compute_anon_token(user_id, board, &nib.slug));
    }

    Ok(nibs)
}

pub async fn get_user_content_stats(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<InternalUserStats, PostError> {
    let nib_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM posts WHERE creator_id = $1 AND deleted = false",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error::<PostError>)?
    .unwrap_or(0);

    let squeak_count: i64 = sqlx::query_scalar(
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
        nib_count,
        squeak_count,
        upvote_count: post_upvotes + comment_upvotes,
    })
}
