use actix_web::{get, post, web, HttpResponse};
use config::app_data::AppData;
use custom_headers::user_id::UserId;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct CommentBody {
    comment_content: String,
    post_id: Uuid,
    // Probably a lot more stuff here
}

#[post("/bur/{topic}/nib/{post}/sqk")]
pub async fn create_comment(
    app: web::Data<AppData>,
    path: web::Path<(String, String)>,
    user_id: UserId,
    body: web::Json<CommentBody>,
) -> HttpResponse {
    todo!("This path is for CREATING the comment.")
    // let comment_content = &body.comment_content;
    // let comment_id = &body.comment_id;
    //
    // match CommentHandler::reply_comment(&pool, user_id.into(), comment_content, *comment_id).await {
    //     Ok(created_at) => HttpResponse::Ok().json(serde_json::json!({
    //         "created_at": created_at
    //     })),
    //     Err(PostError::UserNotFound) => HttpResponse::NotFound().body("Token not found!"),
    //     Err(PostError::PostNotFound) => HttpResponse::NotFound().body("Post not found!"),
    //     Err(PostError::CommentNotFound) => HttpResponse::NotFound().body("Comment not found!"),
    //     Err(_) => HttpResponse::InternalServerError().finish(),
    // }
}

#[get("/bur/{topic}/nib/{post}/sqk")]
pub async fn get_comments(
    app: web::Data<AppData>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    todo!("This path is for RECEIVING the comments. It will return the top level comments only!")
}
