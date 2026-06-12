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

use super::ContentBody;

#[derive(Deserialize, Default)]
pub struct CommentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort: Option<String>,
}

pub async fn create_comment(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
    user_id: UserId,
    Json(body): Json<ContentBody>,
) -> Result<StatusCode, PostError> {
    service::create_comment(
        &app.pool,
        user_id.into(),
        &topic_name,
        &post_slug,
        &body.content,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_comments(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
    OptionalUserId(maybe_user_id): OptionalUserId,
    Query(query): Query<CommentsQuery>,
) -> Result<Json<PaginatedResponse<service::CommentData>>, PostError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let sort = query.sort.as_deref().unwrap_or("hot");

    let comments = service::get_all_comments(
        &app.pool,
        &topic_name,
        &post_slug,
        maybe_user_id,
        limit,
        offset,
        sort,
    )
    .await?;
    Ok(Json(comments))
}
