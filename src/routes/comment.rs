use crate::db::comment::CommentHandler;
use crate::db::errors::PostError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CommentRequest {
    pub token: String,
    pub comment_content: String,
    pub post_id: Uuid,
}

#[derive(Deserialize)]
pub struct ReplyRequest {
    pub token: String,
    pub comment_content: String,
    pub comment_id: Uuid,
}

#[post("/post/comment")]
pub async fn add_comment(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    body: web::Json<CommentRequest>,
) -> impl Responder {
    let token = &body.token;
    let comment_content = &body.comment_content;
    let post_id = &body.post_id;

    match CommentHandler::add_comment(&pool, token, comment_content, *post_id).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::TokenNotFound) => HttpResponse::NotFound().body("Token not found!"),
        Err(PostError::PostNotFound) => HttpResponse::NotFound().body("Post not found!"),

        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/post/reply")]
pub async fn reply_comment(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    body: web::Json<ReplyRequest>,
) -> impl Responder {
    let token = &body.token;
    let comment_content = &body.comment_content;
    let comment_id = &body.comment_id;

    match CommentHandler::reply_comment(&pool, token, comment_content, *comment_id).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::TokenNotFound) => HttpResponse::NotFound().body("Token not found!"),
        Err(PostError::PostNotFound) => HttpResponse::NotFound().body("Post not found!"),
        Err(PostError::CommentNotFound) => HttpResponse::NotFound().body("Comment not found!"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
