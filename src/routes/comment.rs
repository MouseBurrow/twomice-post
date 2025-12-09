use crate::db::comment::CommentHandler;
use crate::db::errors::PostError;
use actix_web::{post, web, HttpResponse, Responder};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct CommentBody {
    comment_content: String,
    post_id: Uuid,
}

#[derive(Deserialize)]
struct ReplyBody {
    comment_content: String,
    comment_id: Uuid,
}

#[post("/post/comment")]
pub async fn add_comment(
    app: web::Data<AppData>,
    user_id: UserId,
    body: web::Json<CommentBody>,
) -> impl Responder {
    let user_id = &user_id.0;
    let comment_content = &body.comment_content;
    let post_id = &body.post_id;

    match CommentHandler::add_comment(&app.pool, user_id, comment_content, *post_id).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::UserNotFound) => HttpResponse::NotFound().body("Token not found!"),
        Err(PostError::PostNotFound) => HttpResponse::NotFound().body("Post not found!"),

        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/post/reply")]
pub async fn reply_comment(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    user_id: UserId,
    body: web::Json<ReplyBody>,
) -> impl Responder {
    let comment_content = &body.comment_content;
    let comment_id = &body.comment_id;

    match CommentHandler::reply_comment(&pool, user_id.into(), comment_content, *comment_id).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::UserNotFound) => HttpResponse::NotFound().body("Token not found!"),
        Err(PostError::PostNotFound) => HttpResponse::NotFound().body("Post not found!"),
        Err(PostError::CommentNotFound) => HttpResponse::NotFound().body("Comment not found!"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
