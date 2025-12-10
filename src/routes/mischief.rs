use crate::db::errors::PostError;
use actix_web::{get, post, web, HttpResponse};
use burrow_db::db_call;
use config::app_data::AppData;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct TopicBody {
    name: String,
    description: String,
}

#[post("/mcf")]
pub async fn create_topic(app: web::Data<AppData>, body: web::Json<TopicBody>) -> HttpResponse {
    let name = &body.name;
    let desc = &body.description;

    let valid_name = regex::Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
    if !valid_name.is_match(name) {
        return HttpResponse::BadRequest().json(json!({
            "error": "invalid_name",
            "message": "Topic name may contain only letters, digits, and underscores"
        }));
    }

    let result = db_call!(
        pool = &app.pool,
        query = sqlx::query(r#"SELECT create_topic($1, $2)"#),
        binds = [&name, &desc],
        error = PostError
    );

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(PostError::UniqueViolation) => HttpResponse::Conflict().json(json!({
            "error": "unique_violation",
            "message": "Topic already exists"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/mcf/{topic}")]
pub async fn get_topic(app: web::Data<AppData>, path: web::Path<String>) -> HttpResponse {
    let topic_name = path.into_inner();

    let result: Result<String, PostError> = db_call!(
        pool = &app.pool,
        query = sqlx::query_scalar(r#"SELECT get_topic($1)"#),
        binds = [topic_name],
        error = PostError
    );

    match result {
        Ok(description) => HttpResponse::Ok().json(json!({
            "description": description,
        })),
        Err(PostError::TopicNotFound) => HttpResponse::NotFound().json(json!({
            "error": "not_found",
            "message": "Topic not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
