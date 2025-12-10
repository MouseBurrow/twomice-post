use crate::routes::burrows::{create_topic, get_topic};
use crate::routes::echoes::{create_reply, get_replies};
use crate::routes::nibbles::{create_post, get_post};
use crate::routes::squeaks::{create_comment, get_comments};
use config::launch_service;

mod db;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    launch_service!(
        service: "post",
        routes: [create_topic, get_topic, create_post, get_post, create_comment, get_comments, create_reply, get_replies]
    );
    Ok(())
}
