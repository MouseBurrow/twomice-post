use crate::utils::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use burrow_db::db_call;
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slugify::slugify;
use sqlx::FromRow;

#[derive(Deserialize)]
struct PostBody {
    title: String,
    content: String,
    image_url: String,
}

#[post("/mcf/{topic}/nib")]
pub async fn create_post(
    app: web::Data<AppData>,
    path: web::Path<String>,
    user_id: UserId,
    body: web::Json<PostBody>,
) -> HttpResponse {
    let topic_name = path.into_inner();

    let title = &body.title;
    let content = &body.content;
    let image_url = &body.image_url;

    let slug = slugify!(title);

    let result: Result<String, PostError> = db_call!(
        pool = &app.pool,
        query = sqlx::query_scalar(r#"SELECT create_post($1, $2, $3, $4, $5, $6)"#),
        binds = [user_id, topic_name, title, slug, content, image_url],
        error = PostError
    );

    match result {
        Ok(final_slug) => HttpResponse::Ok().json(json!({
            "final_slug": final_slug
        })),
        Err(PostError::TopicNotFound) => HttpResponse::NotFound().json(json!({
            "error": "not_found",
            "message": "Topic not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(FromRow, Serialize)]
struct GetPostResult {
    title: String,
    content: String,
    image_url: String,
    created_at: DateTime<Utc>,
}

#[get("/mcf/{topic}/nib/{post_id}")]
pub async fn get_post(app: web::Data<AppData>, path: web::Path<(String, String)>) -> HttpResponse {
    let (topic_name, post_slug) = path.into_inner();

    let result: Result<GetPostResult, PostError> = db_call!(
        pool = &app.pool,
        query = sqlx::query_as(r#"SELECT * FROM get_post($1, $2)"#),
        binds = [topic_name, post_slug],
        error = PostError
    );

    match result {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(PostError::TopicNotFound) => HttpResponse::NotFound().json(json!({
            "error": "not_found",
            "message": "Topic not found"
        })),
        Err(PostError::PostNotFound) => HttpResponse::NotFound().json(json!({
            "error": "not_found",
            "message": "Post not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
