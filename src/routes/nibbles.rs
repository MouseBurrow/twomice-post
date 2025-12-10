use actix_web::{get, post, web, HttpResponse};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;

#[derive(Deserialize)]
struct PostBody {
    post_content: String,
    image_url: String,
}

#[post("/bur/{topic}/nib")]
pub async fn create_post(
    app: web::Data<AppData>,
    path: web::Path<String>,
    user_id: UserId,
    body: web::Json<PostBody>,
) -> HttpResponse {
    todo!("This path is for CREATING a post.")
    // let post_content = &body.post_content;
    // let image_url = &body.image_url;
    //
    // match PostHandler::create_post(&app.pool, user_id.into(), post_content, image_url).await {
    //     Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
    //         "created_at": created_at
    //     })),
    //     Err(PostError::UserNotFound) => HttpResponse::NotFound().body("User not found!"),
    //     Err(_) => HttpResponse::InternalServerError().finish(),
    // }
}

#[get("/bur/{topic}/nib/{post_id}")]
pub async fn get_post(app: web::Data<AppData>, path: web::Path<(String, String)>) -> HttpResponse {
    todo!("This path is for RECEIVING the post information.")
}
