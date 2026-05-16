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
struct ReplyBody {
    content: String,
}

pub async fn create_reply(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
    user_id: UserId,
    Json(body): Json<ReplyBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), PostError> {
    let _: () = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_reply($1, $2, $3, $4, $5)",
        binds = [user_id, &comment_hash, &post_slug, &topic_name, &body.content]
    )?;

    Ok((StatusCode::OK, Json(serde_json::json!({}))))
}

#[derive(FromRow, Serialize)]
struct ReplyData {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

pub async fn get_replies(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
) -> Result<Json<Vec<ReplyData>>, PostError> {
    let replies: Vec<ReplyData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_replies($1, $2, $3)",
        binds = [topic_name, post_slug, comment_hash]
    )?;

    Ok(Json(replies))
}
