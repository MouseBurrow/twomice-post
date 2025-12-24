use crate::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slugify::slugify;
use sqlx::FromRow;

#[derive(Deserialize)]
struct PostBody {
    title: String,
    content: String,
    image_url: Option<String>,
}

#[post("/mcf/{topic}/nib")]
pub async fn create_post(
    app: web::Data<AppData>,
    path: web::Path<String>,
    user_id: UserId,
    body: web::Json<PostBody>,
) -> Result<HttpResponse, PostError> {
    let topic_name = path.into_inner();

    let slug: String = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_post($1, $2, $3, $4, $5, $6)",
        binds = [user_id, topic_name, &body.title, slugify!(&body.title), &body.content, &body.image_url]
    )?;

    Ok(HttpResponse::Ok().json(json!({"final_slug": slug})))
}

#[derive(FromRow, Serialize)]
struct PostData {
    title: String,
    slug: String,
    content: String,
    image_url: Option<String>,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[get("/mcf/{topic}/nib")]
pub async fn get_all_posts(
    app: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, PostError> {
    let topic_name = path.into_inner();

    let posts: Vec<PostData> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_post($1)",
        binds = [topic_name]
    )?;

    Ok(HttpResponse::Ok().json(posts))
}

#[get("/mcf/{topic}/nib/{post_id}")]
pub async fn get_post(
    app: web::Data<AppData>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, PostError> {
    let (topic_name, post_slug) = path.into_inner();

    let post: PostData = db_call!(
        pool = &app.pool,
        query = ONE ROW "SELECT * FROM get_post($1, $2)",
        binds = [topic_name, post_slug]
    )?;

    Ok(HttpResponse::Ok().json(post))
}
