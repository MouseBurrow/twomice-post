use crate::errors::PostError;
use crate::service;
use axum::extract::{Path, State};
use axum::Json;
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use serde_json::json;
use slugify::slugify;

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
) -> Result<Json<serde_json::Value>, PostError> {
    let slug = service::create_post(
        &app.pool,
        user_id.into(),
        &topic_name,
        &body.title,
        slugify!(&body.title),
        &body.content,
        &body.image_url,
    )
    .await?;

    Ok(Json(json!({"final_slug": slug})))
}

pub async fn get_all_posts(
    State(app): State<AppData>,
    Path(topic_name): Path<String>,
) -> Result<Json<Vec<service::PostData>>, PostError> {
    let posts = service::get_all_post(&app.pool, &topic_name).await?;
    Ok(Json(posts))
}

pub async fn get_post(
    State(app): State<AppData>,
    Path((topic_name, post_slug)): Path<(String, String)>,
) -> Result<Json<service::PostData>, PostError> {
    let post = service::get_post(&app.pool, &topic_name, &post_slug).await?;
    Ok(Json(post))
}
