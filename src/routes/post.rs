use crate::db::errors::PostError;
use crate::db::post::PostHandler;
use actix_web::dev::Payload;
use actix_web::http::header::{Header, TryIntoHeaderValue};
use actix_web::{post, web, FromRequest, HttpRequest, HttpResponse, Responder};
use config::app_data::AppData;
use serde::Deserialize;
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserId(pub Uuid);

impl FromRequest for UserId {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let header = match req.headers().get("X-User-Id") {
            Some(v) => v,
            None => return ready(Err(actix_web::error::ErrorBadRequest("missing X-User-Id"))),
        };

        let s = match header.to_str() {
            Ok(s) => s,
            Err(_) => return ready(Err(actix_web::error::ErrorBadRequest("invalid X-User-Id"))),
        };

        let uuid = match Uuid::parse_str(s) {
            Ok(u) => u,
            Err(_) => return ready(Err(actix_web::error::ErrorBadRequest("invalid X-User-Id"))),
        };

        ready(Ok(UserId(uuid)))
    }
}

#[derive(Deserialize)]
pub struct PostBody {
    pub post_content: String,
    pub image_url: String,
}

#[post("/post")]
pub async fn post(
    app: web::Data<AppData>,
    user_id: UserId,
    body: web::Json<PostBody>,
) -> impl Responder {
    let user_id = &user_id.0;
    let post_content = &body.post_content;
    let image_url = &body.image_url;

    match PostHandler::create_post(&app.pool, user_id, post_content, image_url).await {
        Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
            "created_at": created_at
        })),
        Err(PostError::UserNotFound) => HttpResponse::NotFound().body("User not found!"),
        Err(e) => HttpResponse::InternalServerError().finish(),
    }
}
