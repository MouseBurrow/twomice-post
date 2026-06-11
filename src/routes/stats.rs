use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Serialize;

#[derive(Serialize)]
pub struct InternalUserStats {
    pub post_count: i64,
    pub comment_count: i64,
    pub upvote_count: i64,
}

pub async fn get_internal_user_stats(
    State(app): State<AppData>,
    Path(user_id): Path<i64>,
    _user_id: UserId,
) -> Result<Json<InternalUserStats>, PostError> {
    let stats = service::get_user_content_stats(&app.pool, user_id).await?;
    Ok(Json(InternalUserStats {
        post_count: stats.post_count,
        comment_count: stats.comment_count,
        upvote_count: stats.upvote_count,
    }))
}
