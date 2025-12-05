use crate::db::errors::PostError;
use crate::db::post::PostHandler;
use actix_web::{post, web, web::Json, App, HttpResponse, HttpServer, Responder};
use config::Config;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use std::env;

mod db;

#[derive(Deserialize)]
pub struct PostRequest {
    pub token: String,
    pub post_content: String,
    pub image_url: String,
}
#[post("/post")]
async fn post(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    body: web::Json<PostRequest>,
) -> impl Responder {
    let token = &body.token;
    let post_content = &body.post_content;
    let image_url = &body.image_url;

    match PostHandler::create_post(&pool, &token, &post_content, &image_url).await {
        Ok((user_id, content, url, created_at)) => {
            return HttpResponse::Ok().json(serde_json::json!({
                "user id": user_id,
                "content": content,
                "Image url ": url,
                "created_at": created_at
            }));
        }
        Err(PostError::TokenNotFound) => {
            return HttpResponse::NotFound().body("Token not found!");
        }
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // If it is dev mode, load env variables from .env
    #[cfg(debug_assertions)]
    Config::load_local_env();

    // Connect to the database
    let config: Config = Config::init("POST");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.database_url.as_str())
        .await?;

    let port = env::var("PORT")?;
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting Gateway at http://{}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(post)
    })
    .bind(addr)?
    .run()
    .await?;

    Ok(())
}
