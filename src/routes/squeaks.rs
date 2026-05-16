use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommentBody {
    content: String,
}

pub async fn create_comment(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
    user_id: UserId,
    Json(body): Json<CommentBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), PostError> {
    service::create_comment(
        &app.pool,
        user_id.into(),
        &topic_name,
        &post_slug,
        &body.content,
    )
    .await?;

    Ok((StatusCode::OK, Json(serde_json::json!({}))))
}

pub async fn get_all_comments(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
) -> Result<Json<Vec<service::CommentData>>, PostError> {
    let comments = service::get_all_comments(&app.pool, &topic_name, &post_slug).await?;
    Ok(Json(comments))
}
