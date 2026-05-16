use crate::errors::PostError;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slugify::slugify;
use sqlx::FromRow;

#[derive(Deserialize)]
struct PostBody {
    title: String,
    content: String,
    image_url: Option<String>,
}

pub async fn create_post(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
    user_id: UserId,
    Json(body): Json<PostBody>,
) -> Result<Json<serde_json::Value>, PostError> {
    let slug: String = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_post($1, $2, $3, $4, $5, $6)",
        binds = [user_id, &topic_name, &body.title, slugify!(&body.title), &body.content, &body.image_url]
    )?;

    Ok(Json(json!({"final_slug": slug})))
}

#[derive(FromRow, Serialize)]
struct PostData {
    title: String,
    slug: String,
    content: String,
    image_url: Option<String>,
    created_at: DateTime<Utc>,
    deleted: bool,
}

pub async fn get_all_posts(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
) -> Result<Json<Vec<PostData>>, PostError> {
    let posts: Vec<PostData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_post($1)",
        binds = [topic_name]
    )?;

    Ok(Json(posts))
}

pub async fn get_post(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
) -> Result<Json<PostData>, PostError> {
    let post: PostData = db_call!(
        pool = &app.pool,
        query = ONE ROW "SELECT * FROM get_post($1, $2)",
        binds = [topic_name, post_slug]
    )?;

    Ok(Json(post))
}
