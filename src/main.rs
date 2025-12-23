use crate::routes::echoes::{create_reply, get_replies};
use crate::routes::mischief::{create_topic, get_all_topics};
use crate::routes::nibbles::{create_post, get_all_posts, get_post};
use crate::routes::squeaks::{create_comment, get_all_comments};
use config::launch_service;

mod routes;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    launch_service!(
        service: "post",
        routes: [create_topic, get_all_topics, create_post, get_post, get_all_posts, create_comment, get_all_comments, reply_reply, reply_comment]
    );
    Ok(())
}
