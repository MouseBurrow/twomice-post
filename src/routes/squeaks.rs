use crate::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
struct CommentBody {
    content: String,
}

#[post("/mcf/{topic}/nib/{post}/sqk")]
pub async fn create_comment(
    app: web::Data<AppData>,
    path: web::Path<(String, String)>,
    user_id: UserId,
    body: web::Json<CommentBody>,
) -> Result<HttpResponse, PostError> {
    let (topic_name, post_slug) = path.into_inner();
    let content = &body.content;

    let _: () = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_comment($1, $2, $3, $4)",
        binds = [user_id, topic_name, post_slug, content]
    )?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(FromRow, Serialize)]
struct CommentData {
    hash: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[get("/mcf/{topic}/nib/{post}/sqk")]
pub async fn get_all_comments(
    app: web::Data<AppData>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, PostError> {
    let (topic_name, post_slug) = path.into_inner();

    let comments: Vec<CommentData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_comments($1, $2)",
        binds = [topic_name, post_slug]
    )?;

    Ok(HttpResponse::Ok().json(comments))
}
