use actix_web::{get, post, web, HttpResponse};
use config::app_data::AppData;
use custom_headers::user_id::UserId;

struct TopicBody {
    // A LOT of stuff will be required here probably!
}

#[post("/bur")]
pub async fn create_topic(app: web::Data<AppData>, user_id: UserId) -> HttpResponse {
    todo!("This path is for CREATING the topic.")
}

#[get("/bur/{topic}")]
pub async fn get_topic(app: web::Data<AppData>, path: web::Path<String>) -> HttpResponse {
    todo!("This path is for RECEIVING the topic information.")
}
