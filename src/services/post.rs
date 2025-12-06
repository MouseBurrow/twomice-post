use crate::db::errors::PostError;
use crate::db::post::PostHandler;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use config::Config;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PostRequest {
    pub token: String,
    pub post_content: String,
    pub image_url: String,
}

#[post("post/post")]
pub async fn post(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    body: web::Json<PostRequest>,
) -> impl Responder {
    let token = &body.token;
    let post_content = &body.post_content;
    let image_url = &body.image_url;

    match PostHandler::create_post(&pool, token, post_content, image_url).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::TokenNotFound) => HttpResponse::NotFound().body("Token not found!"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
