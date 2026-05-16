use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
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
) -> impl IntoResponse {
    if !body
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_name",
                "message": "Topic name may contain only letters, digits, and underscores"
            })),
        )
            .into_response();
    }

    match service::create_topic(&app.pool, &body.name, &body.description).await {
        Ok(_) => (StatusCode::OK, Json(json!({}))).into_response(),
        Err(PostError::UniqueViolation) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "topic_already_exists",
                "message": "Topic already exists"
            })),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))).into_response(),
    }
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
) -> impl IntoResponse {
    match service::get_topic(&app.pool, &topic_name).await {
        Ok(topic) => (StatusCode::OK, Json(json!(topic))).into_response(),
        Err(PostError::TopicNotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Topic not found"
            })),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))).into_response(),
    }
}
