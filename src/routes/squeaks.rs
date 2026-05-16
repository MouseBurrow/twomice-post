use crate::errors::PostError;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
struct CommentBody {
    content: String,
}

pub async fn create_comment(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
    user_id: UserId,
    Json(body): Json<CommentBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), PostError> {
    let _: () = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_comment($1, $2, $3, $4)",
        binds = [user_id, &topic_name, &post_slug, &body.content]
    )?;

    Ok((StatusCode::OK, Json(serde_json::json!({}))))
}

#[derive(FromRow, Serialize)]
struct CommentData {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

pub async fn get_all_comments(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
) -> Result<Json<Vec<CommentData>>, PostError> {
    let comments: Vec<CommentData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_comments($1, $2)",
        binds = [topic_name, post_slug]
    )?;

    Ok(Json(comments))
}
