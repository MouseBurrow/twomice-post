use crate::errors::PostError;
use crate::service;
use axum::extract::State;
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;

pub async fn get_user_posts(
    State(app): State<AppData>,
    user_id: UserId,
) -> Result<Json<Vec<service::PostData>>, PostError> {
    let posts = service::get_user_posts(&app.pool, user_id.into()).await?;
    Ok(Json(posts))
}
