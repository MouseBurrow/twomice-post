use crate::utils::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

#[derive(Deserialize)]
struct CommentBody {
    content: String,
}

#[post("/mcf/{topic}/nib/{post}/sqk")]
pub async fn create_comment(
    app: web::Data<AppData>,
    path: web::Path<(String, String)>,
    user_id: UserId,
    body: web::Json<CommentBody>,
) -> HttpResponse {
    let (topic_name, post_slug) = path.into_inner();
    let content = &body.content;

    let result: Result<(), PostError> = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_comment($1, $2, $3, $4)",
        binds = [user_id, topic_name, post_slug, content]
    );

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
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

#[derive(FromRow, Serialize)]
struct CommentData {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[get("/mcf/{topic}/nib/{post}/sqk")]
pub async fn get_all_comments(
    app: web::Data<AppData>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (topic_name, post_slug) = path.into_inner();

    let result: Result<Vec<CommentData>, PostError> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_comments($1, $2)",
        binds = [topic_name, post_slug]
    );

    match result {
        Ok(comments) => HttpResponse::Ok().json(comments),
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
