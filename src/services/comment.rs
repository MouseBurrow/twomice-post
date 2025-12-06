use crate::db::comment::CommentHandler;
use crate::db::errors::PostError;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use config::Config;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CommentRequest {
    pub token: String,
    pub comment_content: String,
    pub post_id: Uuid,
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
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
