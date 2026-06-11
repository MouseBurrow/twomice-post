use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::optional_user_id::OptionalUserId;
use custom_headers::user_id::UserId;

use super::EchoBody;

pub async fn create_reply(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
    user_id: UserId,
    Json(body): Json<EchoBody>,
) -> Result<StatusCode, PostError> {
    service::create_reply(
        &app.pool,
        user_id.into(),
        &topic_name,
        &post_slug,
        &comment_hash,
        &body.content,
        body.reply_hash.as_deref(),
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_replies(
    State(app): State<AppData>,
    Path((topic_name, post_slug, comment_hash)): Path<(String, String, String)>,
    OptionalUserId(maybe_user_id): OptionalUserId,
) -> Result<Json<Vec<service::ReplyData>>, PostError> {
    let replies = service::get_replies(
        &app.pool,
        &topic_name,
        &post_slug,
        &comment_hash,
        maybe_user_id,
    )
    .await?;
    Ok(Json(replies))
}
