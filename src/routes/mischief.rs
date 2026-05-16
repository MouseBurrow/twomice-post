use crate::errors::PostError;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

#[derive(Deserialize)]
struct TopicBody {
    name: String,
    description: String,
}

pub async fn create_topic(
    State(app): State<AppData>,
    _user_id: UserId,
    Json(body): Json<TopicBody>,
) -> impl IntoResponse {
    let name = &body.name;
    let desc = &body.description;

    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_name",
                "message": "Topic name may contain only letters, digits, and underscores"
            })),
        )
            .into_response();
    }

    let result: Result<(), PostError> = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_topic($1, $2)",
        binds = [name, desc]
    );

    match result {
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

#[allow(dead_code)]
#[derive(FromRow, Serialize)]
struct TopicData {
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

pub async fn get_all_topics(
    State(app): State<AppData>,
) -> Result<Json<Vec<TopicData>>, PostError> {
    let all_topics: Vec<TopicData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_topics()"
    )?;

    Ok(Json(all_topics))
}

pub async fn get_topic(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
) -> impl IntoResponse {
    let result: Result<TopicData, PostError> = db_call!(
        pool = &app.pool,
        query = ONE ROW "SELECT * FROM get_topic($1)",
        binds = [&topic_name]
    );

    match result {
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
