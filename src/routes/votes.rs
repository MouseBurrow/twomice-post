use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VoteBody {
    pub direction: i8,
}

pub async fn cast_post_vote(
    State(app): State<AppData>,
    Path((_topic_name, post_slug)): Path<(String, String)>,
    user_id: UserId,
    Json(body): Json<VoteBody>,
) -> Result<Json<serde_json::Value>, PostError> {
    let post_id = service::resolve_post_b62(&app.pool, &post_slug).await?;
    let vote_count =
        service::cast_post_vote(&app.pool, user_id.into(), post_id, body.direction).await?;
    Ok(Json(serde_json::json!({ "vote_count": vote_count })))
}

pub async fn cast_comment_vote(
    State(app): State<AppData>,
    Path((_topic_name, _post_slug, comment_hash)): Path<(String, String, String)>,
    user_id: UserId,
    Json(body): Json<VoteBody>,
) -> Result<Json<serde_json::Value>, PostError> {
    let comment_id = service::resolve_comment_id(&app.pool, &comment_hash).await?;
    let vote_count =
        service::cast_comment_vote(&app.pool, user_id.into(), comment_id, body.direction).await?;
    Ok(Json(serde_json::json!({ "vote_count": vote_count })))
}

pub async fn cast_reply_vote(
    State(app): State<AppData>,
    Path((_topic_name, _post_slug, _comment_hash, reply_hash)): Path<(
        String,
        String,
        String,
        String,
    )>,
    user_id: UserId,
    Json(body): Json<VoteBody>,
) -> Result<Json<serde_json::Value>, PostError> {
    let reply_id = service::resolve_reply_id(&app.pool, &reply_hash).await?;
    let vote_count =
        service::cast_reply_vote(&app.pool, user_id.into(), reply_id, body.direction).await?;
    Ok(Json(serde_json::json!({ "vote_count": vote_count })))
}
