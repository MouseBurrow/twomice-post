use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::Json;
use config::app_data::AppData;
use serde::Serialize;

#[derive(Serialize)]
pub struct InternalUserStats {
    pub nib_count: i64,
    pub squeak_count: i64,
    pub upvote_count: i64,
}

pub async fn get_internal_user_stats(
    State(app): State<AppData>,
    Path(user_id): Path<i64>,
) -> Result<Json<InternalUserStats>, PostError> {
    let stats = service::get_user_content_stats(&app.pool, user_id).await?;
    Ok(Json(InternalUserStats {
        nib_count: stats.nib_count,
        squeak_count: stats.squeak_count,
        upvote_count: stats.upvote_count,
    }))
}
