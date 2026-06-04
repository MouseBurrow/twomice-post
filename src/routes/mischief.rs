use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TopicBody {
    name: String,
    description: String,
}

pub async fn create_topic(
    State(app): State<AppData>,
    _user_id: UserId,
    Json(body): Json<TopicBody>,
) -> Result<StatusCode, PostError> {
    if !body
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(PostError::InvalidTopicName);
    }

    service::create_topic(&app.pool, &body.name, &body.description).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_topics(
    State(app): State<AppData>,
) -> Result<Json<Vec<service::TopicData>>, PostError> {
    let topics = service::get_all_topics(&app.pool).await?;
    Ok(Json(topics))
}

pub async fn get_topic(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
) -> Result<Json<service::TopicData>, PostError> {
    let topic = service::get_topic(&app.pool, &topic_name).await?;
    Ok(Json(topic))
}

#[derive(Deserialize)]
pub struct ActiveQuery {
    #[serde(default = "default_active_limit")]
    pub limit: i64,
}

fn default_active_limit() -> i64 {
    8
}

pub async fn get_active_topics(
    State(app): State<AppData>,
    Query(query): Query<ActiveQuery>,
) -> Result<Json<Vec<service::BoardSummary>>, PostError> {
    let limit = query.limit.min(50);
    let topics = service::get_active_topics(&app.pool, limit).await?;
    Ok(Json(topics))
}
