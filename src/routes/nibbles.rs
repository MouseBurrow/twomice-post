use crate::utils::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slugify::slugify;
use sqlx::FromRow;

#[derive(Deserialize)]
struct PostBody {
    title: String,
    content: String,
    image_url: Option<String>,
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
        query = ONE COLUMN "SELECT create_post($1, $2, $3, $4, $5, $6)",
        binds = [user_id, topic_name, title, slug, content, image_url]
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
struct PostData {
    title: String,
    slug: String,
    content: String,
    image_url: Option<String>,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[get("/mcf/{topic}")]
pub async fn get_all_posts(app: web::Data<AppData>, path: web::Path<String>) -> HttpResponse {
    let topic_name = path.into_inner();

    let result: Result<Vec<PostData>, PostError> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_post($1)",
        binds = [topic_name]
    );

    match result {
        Ok(posts) => HttpResponse::Ok().json(posts),
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

#[get("/mcf/{topic}/nib/{post_id}")]
pub async fn get_post(app: web::Data<AppData>, path: web::Path<(String, String)>) -> HttpResponse {
    let (topic_name, post_slug) = path.into_inner();

    let result: Result<PostData, PostError> = db_call!(
        pool = &app.pool,
        query = ONE ROW "SELECT get_post($1, $2)",
        binds = [topic_name, post_slug]
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
