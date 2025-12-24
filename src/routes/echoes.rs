use crate::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
struct ReplyBody {
    content: String,
}

#[post("/mcf/{topic}/nib/{post}/sqk/{comment}/echoes")]
pub async fn create_reply(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
    user_id: UserId,
    body: web::Json<ReplyBody>,
) -> Result<HttpResponse, PostError> {
    let (topic_name, post_slug, comment_hash) = path.into_inner();

    let _: () = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_reply($1, $2, $3, $4, $5)",
        binds = [user_id, comment_hash, post_slug,topic_name, &body.content]
    )?;

    Ok(HttpResponse::Ok().finish())
}

// #[post("/mcf/{topic}/nib/{post}/sqk/{comment}/echoes/{reply}/echo")]
// pub async fn reply_a_reply(
//     app: web::Data<AppData>,
//     path: web::Path<(String, String, String, String)>,
//     user_id: UserId,
//     body: web::Json<ReplyBody>,
// ) -> HttpResponse {
//     let (topic_name, post_slug, comment_hash, reply_hash) = path.into_inner();
//     let content = &body.comment_content;
//     let result: Result<(), PostError> = db_call!(
//         pool = &app.pool,
//         query = ONE COLUMN "SELECT create_reply($1, $2, $3, $4, $5, $6)",
//         binds = [user_id, reply_hash, comment_hash, post_slug,topic_name, content]
//     );
//
//     match result {
//         Ok(_) => HttpResponse::Ok().finish(),
//         Err(PostError::TopicNotFound) => HttpResponse::NotFound().json(json!({
//             "error": "not_found",
//             "message": "Topic not found"
//         })),
//         Err(PostError::PostNotFound) => HttpResponse::NotFound().json(json!({
//             "error": "not_found",
//             "message": "Post not found"
//         })),
//         Err(PostError::CommentNotFound) => HttpResponse::NotFound().json(json!({
//             "error": "not_found",
//             "message": "Comment not found"
//         })),
//         Err(_) => HttpResponse::InternalServerError().finish(),
//     }
// }

#[derive(FromRow, Serialize)]
struct ReplyData {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[get("/mcf/{topic}/nib/{post}/sqk/{comment}/echoes")]
pub async fn get_replies(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, PostError> {
    let (topic_name, post_slug, comment_hash) = path.into_inner();

    let replies: Vec<ReplyData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_replies($1, $2, $3)",
        binds = [topic_name, post_slug, comment_hash]
    )?;

    Ok(HttpResponse::Ok().json(replies))
}
