use actix_web::{get, post, web, HttpResponse};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct ReplyBody {
    comment_content: String,
    comment_id: Uuid,
}

#[post("/bur/{topic}/nib/{post}/sqk/{comment}/echo")]
pub async fn create_reply(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
    user_id: UserId,
    body: web::Json<ReplyBody>,
) -> HttpResponse {
    todo!("This path is for CREATING the reply.")
}

#[get("/bur/{topic}/nib/{post}/sqk{comment}/echo")]
pub async fn get_replies(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    todo!("This path is for RECEIVING the replies. It will return the whole tree of replies!")
}
