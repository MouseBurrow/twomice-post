use crate::db::errors::PostError;
use crate::db::post::PostHandler;
use actix_web::{post, web, HttpResponse, Responder};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
struct PostBody {
    post_content: String,
    image_url: String,
}

#[post("/post")]
pub async fn post(
    app: web::Data<AppData>,
    user_id: UserId,
    body: web::Json<PostBody>,
) -> impl Responder {
    let post_content = &body.post_content;
    let image_url = &body.image_url;

    match PostHandler::create_post(&app.pool, user_id.into(), post_content, image_url).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::UserNotFound) => HttpResponse::NotFound().body("User not found!"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
