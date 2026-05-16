use crate::routes::echoes::{create_reply, get_replies};
use crate::routes::mischief::{create_topic, get_all_topics, get_topic};
use crate::routes::nibbles::{create_post, get_all_posts, get_post};
use crate::routes::squeaks::{create_comment, get_all_comments};
use actix_web::{web, App, HttpServer};
use config::app_data::AppData;
use config::config::Config;
use config::logger;

pub(crate) mod errors;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init();

    let config = Config::load("post");
    let app_data = AppData::new(config.clone()).await?;
    let addr = format!("0.0.0.0:{}", config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .service(create_topic)
            .service(get_all_topics)
            .service(get_topic)
            .service(create_post)
            .service(get_post)
            .service(get_all_posts)
            .service(create_comment)
            .service(get_all_comments)
            .service(create_reply)
            .service(get_replies)
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
