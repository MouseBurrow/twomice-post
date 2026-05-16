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
