use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_errors::json_empty;

use super::ContentBody;

pub async fn create_reply(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
    user_id: UserId,
    Json(body): Json<ContentBody>,
) -> Result<Json<serde_json::Value>, PostError> {
    service::create_reply(
        &app.pool,
        user_id.into(),
        &topic_name,
        &post_slug,
        &comment_hash,
        &body.content,
    )
    .await?;

    Ok(json_empty())
}

pub async fn get_replies(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
) -> Result<Json<Vec<service::ReplyData>>, PostError> {
    let replies = service::get_replies(&app.pool, &topic_name, &post_slug, &comment_hash).await?;
    Ok(Json(replies))
}
