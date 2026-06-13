use crate::errors::PostError;
use chrono::{DateTime, Utc};
use easy_errors::map_sqlx_error;
use serde::Serialize;
use sqlx::{FromRow, Pool, Postgres};
use std::sync::OnceLock;

const MAX_TITLE_LEN: usize = 200;
const MAX_CONTENT_LEN: usize = 50000;
const MAX_TAGS_PER_POST: usize = 5;

const POST_BASE: &str = concat!(
    "SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted, ",
    "COALESCE(pv.vote_count, 0)::BIGINT as vote_count, ",
    "COALESCE(p.tags, '{}') as tags, ",
    "(SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.deleted = false)::BIGINT ",
    "+ (SELECT COUNT(*) FROM replies r WHERE r.post_id = p.id AND r.deleted = false)::BIGINT ",
    "as reply_count, ",
    "p.view_count, ",
    "t.name as board_id, ",
    "p.creator_id ",
    "FROM posts p ",
    "JOIN topics t ON t.id = p.topic_id ",
    "LEFT JOIN LATERAL (",
    "    SELECT COALESCE(SUM(direction), 0) as vote_count ",
    "    FROM post_votes ",
    "    WHERE post_id = p.id ",
    ") pv ON true"
);

fn post_auth_fields(
    maybe_user_id: Option<i64>,
    creator_id: Option<i64>,
    slug: &str,
    board_name: &str,
) -> (Option<bool>, Option<String>) {
    match (maybe_user_id, creator_id) {
        (Some(uid), Some(cid)) => {
            let mine = cid == uid;
            let token = if mine {
                Some(compute_anon_token(uid, board_name, slug))
            } else {
                None
            };
            (Some(mine), token)
        }
        _ => (None, None),
    }
}

#[derive(FromRow, Serialize)]
pub struct BoardData {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
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
    #[allow(dead_code)]
    id: i64,
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
    sender_id: i64,
    #[allow(dead_code)]
    reply_id: Option<i64>,
    parent_hash: Option<String>,
    #[sqlx(default)]
    vote_count: i64,
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

#[derive(Serialize)]
pub struct ReplyData {
    pub hash: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
    #[serde(default)]
    pub vote_count: i64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anon_token: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mine: Option<bool>,
    #[serde(default)]
    pub children: Vec<ReplyData>,
}

#[derive(Serialize)]
pub struct InternalUserStats {
    pub post_count: i64,
    pub comment_count: i64,
    pub upvote_count: i64,
}

pub async fn resolve_post_b62(
    pool: &Pool<Postgres>,
    post_b62_or_slug: &str,
) -> Result<i64, PostError> {
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

async fn validate_tags(
    pool: &Pool<Postgres>,
    topic_id: i64,
    tags: &[String],
) -> Result<(), PostError> {
    let allowed: Vec<String> =
        sqlx::query_scalar("SELECT tag_name FROM topic_tags WHERE topic_id = $1")
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

fn get_anon_salt() -> &'static str {
    static SALT: OnceLock<String> = OnceLock::new();
    SALT.get_or_init(|| std::env::var("ANON_SALT").unwrap_or_else(|_| utils::random_b62(32)))
}

fn compute_anon_token(user_id: i64, board_name: &str, post_slug: &str) -> String {
    use sha2::{Digest, Sha256};
    let salt = get_anon_salt();
    let input = format!("{}:{}:{}:{}", user_id, board_name, post_slug, salt);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..8])
}

fn compute_squeak_anon_token(
    viewer_id: i64,
    sender_id: i64,
    board_name: &str,
    post_slug: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let salt = get_anon_salt();
    let input = format!(
        "sqk:{}:{}:{}:{}:{}",
        viewer_id, sender_id, board_name, post_slug, salt
    );
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..8])
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

pub async fn resolve_reply_id(pool: &Pool<Postgres>, reply_hash: &str) -> Result<i64, PostError> {
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM replies WHERE hash = $1")
        .bind(reply_hash)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    id.ok_or(PostError::ReplyNotFound)
}

pub mod boards;
pub mod comments;
pub mod feed;
pub mod posts;
pub mod replies;
pub mod stats;
pub mod votes;

pub use self::boards::*;
pub use self::comments::*;
pub use self::feed::*;
pub use self::posts::*;
pub use self::replies::*;
pub use self::stats::*;
pub use self::votes::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anon_token_is_deterministic() {
        let a = compute_anon_token(1, "general", "abc123");
        let b = compute_anon_token(1, "general", "abc123");
        assert_eq!(a, b, "same inputs must produce same token");
    }

    #[test]
    fn anon_token_differs_per_user() {
        let a = compute_anon_token(1, "general", "abc123");
        let b = compute_anon_token(2, "general", "abc123");
        assert_ne!(a, b, "different users must produce different tokens");
    }

    #[test]
    fn anon_token_differs_per_board() {
        let a = compute_anon_token(1, "general", "abc123");
        let b = compute_anon_token(1, "random", "abc123");
        assert_ne!(a, b, "different boards must produce different tokens");
    }

    #[test]
    fn anon_token_differs_per_post() {
        let a = compute_anon_token(1, "general", "abc123");
        let b = compute_anon_token(1, "general", "xyz789");
        assert_ne!(a, b, "different posts must produce different tokens");
    }

    #[test]
    fn anon_token_is_16_hex_chars() {
        let token = compute_anon_token(42, "board", "slug123");
        assert_eq!(token.len(), 16);
        assert!(
            token.chars().all(|c| c.is_ascii_hexdigit()),
            "token must be hex"
        );
    }

    #[test]
    fn squeak_anon_token_is_deterministic() {
        let a = compute_squeak_anon_token(1, 5, "general", "abc123");
        let b = compute_squeak_anon_token(1, 5, "general", "abc123");
        assert_eq!(a, b, "same inputs must produce same squeak token");
    }

    #[test]
    fn squeak_anon_token_differs_per_sender() {
        let a = compute_squeak_anon_token(1, 5, "general", "abc123");
        let b = compute_squeak_anon_token(1, 99, "general", "abc123");
        assert_ne!(
            a, b,
            "different commenters on the same post must get different tokens per viewer"
        );
    }

    #[test]
    fn squeak_anon_token_differs_per_viewer() {
        let a = compute_squeak_anon_token(1, 5, "general", "abc123");
        let b = compute_squeak_anon_token(2, 5, "general", "abc123");
        assert_ne!(
            a, b,
            "different viewers must see different tokens for the same commenter"
        );
    }

    #[test]
    fn squeak_anon_token_differs_per_board() {
        let a = compute_squeak_anon_token(1, 5, "general", "abc123");
        let b = compute_squeak_anon_token(1, 5, "random", "abc123");
        assert_ne!(
            a, b,
            "different boards must produce different squeak tokens"
        );
    }

    #[test]
    fn squeak_anon_token_differs_from_post_token() {
        let post_token = compute_anon_token(1, "general", "abc123");
        let squeak_token = compute_squeak_anon_token(1, 5, "general", "abc123");
        assert_ne!(
            post_token, squeak_token,
            "post tokens and squeak tokens must use different hash domains"
        );
    }

    #[test]
    fn squeak_anon_token_is_16_hex_chars() {
        let token = compute_squeak_anon_token(42, 7, "board", "slug123");
        assert_eq!(token.len(), 16);
        assert!(
            token.chars().all(|c| c.is_ascii_hexdigit()),
            "token must be hex"
        );
    }

    #[test]
    fn anon_salt_is_consistent() {
        let a = get_anon_salt();
        let b = get_anon_salt();
        assert_eq!(
            a, b,
            "salt must be consistent within the same process lifetime"
        );
        assert!(!a.is_empty(), "salt must not be empty");
        assert_eq!(a.len(), 32, "default random salt must be 32 chars");
    }

    #[test]
    fn constants_are_sane() {
        assert!(
            MAX_TITLE_LEN > 0 && MAX_TITLE_LEN <= 1000,
            "MAX_TITLE_LEN out of range"
        );
        assert!(
            MAX_CONTENT_LEN >= MAX_TITLE_LEN,
            "MAX_CONTENT_LEN must be >= MAX_TITLE_LEN"
        );
        assert!(
            MAX_TAGS_PER_POST > 0 && MAX_TAGS_PER_POST <= 20,
            "MAX_TAGS_PER_POST out of range"
        );
    }

    #[test]
    fn token_domain_isolation() {
        for user_id in [1, 2, 100] {
            for board in ["general", "random", "tech"] {
                for slug in ["abc", "xyz", "123"] {
                    let post_tok = compute_anon_token(user_id, board, slug);
                    let sqk_tok = compute_squeak_anon_token(user_id, user_id, board, slug);
                    assert_ne!(
                        post_tok, sqk_tok,
                        "domain collision: user={user_id}, board={board}, slug={slug}"
                    );
                }
            }
        }
    }
}
