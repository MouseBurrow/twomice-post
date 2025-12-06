use crate::services::comment::add_comment;
use crate::services::post::post;

use actix_web::{web, App, HttpServer};
use config::launch_service;

mod db;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    launch_service!(
        service: "POST",
        routes: [post, add_comment]
    );
    Ok(())
}
