use crate::errors::PostError;
use chrono::{DateTime, Utc};
use easy_errors::{insert_retry_on_duplicate, map_sqlx_error};
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::OnceLock;
use utils::PaginatedResponse;

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
pub struct BoardData {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
}

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
#[allow(dead_code)]
pub type NoteData = CommentData;

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
    limit: i64,
    offset: i64,
    sort: &str,
) -> Result<PaginatedResponse<CommentData>, PostError> {
    let post_id = resolve_post_b62(pool, post_b62_or_slug).await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM comments WHERE post_id = $1")
            .bind(post_id)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx_error::<PostError>)?;

    let order = match sort {
        "new" => "new",
        _ => "hot",
    };

    let rows: Vec<CommentRow> = match order {
        "new" => sqlx::query_as(
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
             ORDER BY c.created_at DESC, c.id ASC
             LIMIT $2 OFFSET $3",
        )
        .bind(post_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?,

        _ => sqlx::query_as(
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
             ORDER BY COALESCE(cv.vote_count, 0) DESC, c.created_at DESC, c.id ASC
             LIMIT $2 OFFSET $3",
        )
        .bind(post_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?,
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

    let mut reply_map: HashMap<String, ReplyData> = HashMap::new();
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
    let mut roots: Vec<String> = Vec::new();

    for row in rows {
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

    Ok(PaginatedResponse::new(top_level)
        .with_total(total)
        .with_pagination(limit, offset))
}

#[derive(Serialize)]
pub struct InternalUserStats {
    pub post_count: i64,
    pub comment_count: i64,
    pub upvote_count: i64,
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

pub async fn resolve_reply_id(pool: &Pool<Postgres>, reply_hash: &str) -> Result<i64, PostError> {
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM replies WHERE hash = $1")
        .bind(reply_hash)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;
    id.ok_or(PostError::ReplyNotFound)
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

    Ok(count)
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

pub async fn get_feed_posts(
    pool: &Pool<Postgres>,
    sort: &str,
    _app_env: &config::app_envs::AppEnvs,
) -> Result<Vec<PostData>, PostError> {
    let rows: Vec<PostRow> = match sort {
        "new" => sqlx::query_as(
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
             WHERE p.deleted = false AND t.deleted = false
             ORDER BY COALESCE(pv.vote_count, 0) DESC, p.created_at DESC
             LIMIT 100",
        )
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?,
    };

    let posts = rows
        .into_iter()
        .map(|row| PostData {
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
        })
        .collect();

    Ok(posts)
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
