use crate::utils::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use easy_db::db_call;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

#[derive(Deserialize)]
struct TopicBody {
    name: String,
    description: String,
}

#[post("/mcf")]
pub async fn create_topic(
    app: web::Data<AppData>,
    body: web::Json<TopicBody>,
    _user_id: UserId,
) -> HttpResponse {
    let name = &body.name;
    let desc = &body.description;

    let valid_name = regex::Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
    if !valid_name.is_match(name) {
        return HttpResponse::BadRequest().json(json!({
            "error": "invalid_name",
            "message": "Topic name may contain only letters, digits, and underscores"
        }));
    }

    let result: Result<(), PostError> = db_call!(
        pool = &app.pool,
        query = ONE COLUMN "SELECT create_topic($1, $2)",
        binds = [&name, &desc]
    );

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(PostError::UniqueViolation) => HttpResponse::Conflict().json(json!({
            "error": "topic_already_exists",
            "message": "Topic already exists"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[allow(dead_code)]
#[derive(FromRow, Serialize)]
struct TopicData {
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[get("/mcf")]
pub async fn get_all_topics(app: web::Data<AppData>) -> HttpResponse {
    let result: Result<Vec<TopicData>, PostError> = db_call!(
        pool = &app.pool,
        query = ALL ROW "SELECT * FROM get_all_topics()"
    );

    match result {
        Ok(all_topics) => HttpResponse::Ok().json(all_topics),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
