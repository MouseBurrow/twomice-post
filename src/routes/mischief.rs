use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct TopicBody {
    name: String,
    description: String,
}

pub async fn create_topic(
    State(app): State<AppData>,
    _user_id: UserId,
    Json(body): Json<TopicBody>,
) -> Result<Json<serde_json::Value>, PostError> {
    if !body
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(PostError::InvalidTopicName);
    }

    service::create_topic(&app.pool, &body.name, &body.description).await?;

    Ok(Json(json!({})))
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
