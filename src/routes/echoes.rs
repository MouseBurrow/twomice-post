use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReplyBody {
    content: String,
}

pub async fn create_reply(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
    user_id: UserId,
    Json(body): Json<ReplyBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), PostError> {
    service::create_reply(
        &app.pool,
        user_id.into(),
        &topic_name,
        &post_slug,
        &comment_hash,
        &body.content,
    )
    .await?;

    Ok((StatusCode::OK, Json(serde_json::json!({}))))
}

pub async fn get_replies(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
) -> Result<Json<Vec<service::ReplyData>>, PostError> {
    let replies = service::get_replies(&app.pool, &topic_name, &post_slug, &comment_hash).await?;
    Ok(Json(replies))
}
