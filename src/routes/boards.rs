use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BoardBody {
    name: String,
    description: String,
}

pub async fn create_board(
    State(app): State<AppData>,
    _user_id: UserId,
    Json(body): Json<BoardBody>,
) -> Result<StatusCode, PostError> {
    if !body
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(PostError::InvalidTopicName);
    }

    service::create_board(&app.pool, &body.name, &body.description).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_boards(
    State(app): State<AppData>,
) -> Result<Json<Vec<service::BoardData>>, PostError> {
    let boards = service::get_all_boards(&app.pool).await?;
    Ok(Json(boards))
}

pub async fn get_board(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
) -> Result<Json<service::BoardData>, PostError> {
    let board = service::get_board(&app.pool, &topic_name).await?;
    Ok(Json(board))
}

#[derive(Deserialize)]
pub struct ActiveQuery {
    #[serde(default = "default_active_limit")]
    pub limit: i64,
}

fn default_active_limit() -> i64 {
    8
}

pub async fn get_active_boards(
    State(app): State<AppData>,
    Query(query): Query<ActiveQuery>,
) -> Result<Json<Vec<service::BoardSummary>>, PostError> {
    let limit = query.limit.min(50);
    let boards = service::get_active_boards(&app.pool, limit).await?;
    Ok(Json(boards))
}

pub async fn get_board_tags(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
) -> Result<Json<Vec<String>>, PostError> {
    let tags = service::get_board_tags(&app.pool, &topic_name).await?;
    Ok(Json(tags))
}
