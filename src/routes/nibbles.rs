use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use config::app_data::AppData;
use custom_headers::optional_user_id::OptionalUserId;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PostBody {
    title: String,
    content: String,
    image_url: Option<String>,
}

pub async fn create_post(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
    user_id: UserId,
    Json(body): Json<PostBody>,
) -> Result<StatusCode, PostError> {
    let _slug = service::create_post(
        &app.pool,
        user_id.into(),
        &topic_name,
        &body.title,
        &body.content,
        &body.image_url,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_posts(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
    OptionalUserId(maybe_user_id): OptionalUserId,
) -> Result<Json<Vec<service::PostData>>, PostError> {
    let posts = service::get_all_post(&app.pool, &topic_name, maybe_user_id).await?;
    Ok(Json(posts))
}

pub async fn get_post(
    State(app): State<AppData>,
    Path((_topic_name, post_slug)): Path<(String, String)>,
    OptionalUserId(maybe_user_id): OptionalUserId,
) -> Result<Json<service::PostData>, PostError> {
    let post = service::get_post(&app.pool, &post_slug, maybe_user_id).await?;
    Ok(Json(post))
}
