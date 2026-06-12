use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::optional_user_id::OptionalUserId;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use utils::PaginatedResponse;

use super::EchoBody;

#[derive(Deserialize, Default)]
pub struct RepliesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

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
    Query(query): Query<RepliesQuery>,
) -> Result<Json<PaginatedResponse<service::ReplyData>>, PostError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let replies = service::get_replies(
        &app.pool,
        &topic_name,
        &post_slug,
        &comment_hash,
        maybe_user_id,
        limit,
        offset,
    )
    .await?;
    Ok(Json(replies))
}
