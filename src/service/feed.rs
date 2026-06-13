use super::{PostData, PostError, PostRow};
use easy_errors::map_sqlx_error;
use sqlx::{Pool, Postgres};

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
