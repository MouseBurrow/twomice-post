use super::{PostData, PostError, PostRow, POST_BASE};
use easy_errors::map_sqlx_error;
use sqlx::{AssertSqlSafe, Pool, Postgres};

pub async fn get_feed_posts(pool: &Pool<Postgres>, sort: &str) -> Result<Vec<PostData>, PostError> {
    let order = match sort {
        "new" => "ORDER BY p.created_at DESC",
        _ => "ORDER BY COALESCE(pv.vote_count, 0) DESC, p.created_at DESC",
    };
    let sql = format!(
        "{} WHERE p.deleted = false AND t.deleted = false {} LIMIT 100",
        POST_BASE, order
    );
    let rows: Vec<PostRow> = sqlx::query_as(AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error::<PostError>)?;

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
