use actix_web::{get, post, web, HttpResponse};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use easy_db::db_call;
use crate::utils::errors::PostError;

#[derive(Deserialize)]
struct ReplyBody {
    comment_content: String,
}

#[post("/mcf/{topic}/nib/{post}/sqk/{comment}/echoes")]
pub async fn create_reply(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
    user_id: UserId,
    body: web::Json<ReplyBody>,
) -> HttpResponse {
    let (topic_name, post_slug, comment_hash) = path.into_inner();
    let content = &body.comment_content;
    let result: Result<(), PostError> = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_reply($1, $2, $3, $4, $5)",
        binds = [user_id, comment_hash, post_slug,topic_name, content]
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
        Err(PostError::CommentNotFound) => HttpResponse::NotFound().json(json!({
            "error": "not_found",
            "message": "Comment not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/mcf/{topic}/nib/{post}/sqk/{comment}/echoes/{reply}/echo")]
pub async fn reply_a_reply(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
    user_id: UserId,
    body: web::Json<ReplyBody>,
) -> HttpResponse {
    let (topic_name, post_slug, comment_hash, reply_hash) = path.into_inner();
    let content = &body.comment_content;
    let result: Result<(), PostError> = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_reply($1, $2, $3, $4, $5, $6)",
        binds = [user_id, reply_hash, comment_hash, post_slug,topic_name, content]
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
        Err(PostError::CommentNotFound) => HttpResponse::NotFound().json(json!({
            "error": "not_found",
            "message": "Comment not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/mcf/{topic}/nib/{post}/sqk{comment}/echoes")]
pub async fn get_replies(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    todo!("This path is for RECEIVING the replies. It will return the whole tree of replies!")
}
