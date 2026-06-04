use crate::errors::PostError;
use crate::service;
use axum::extract::State;
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;

pub async fn get_user_nibs(
    State(app): State<AppData>,
    user_id: UserId,
) -> Result<Json<Vec<service::PostData>>, PostError> {
    let nibs = service::get_user_nibs(&app.pool, user_id.into()).await?;
    Ok(Json(nibs))
}
