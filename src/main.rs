use crate::routes::echoes::{create_reply, get_replies};
use crate::routes::mischief::{create_topic, get_all_topics, get_topic};
use crate::routes::nibbles::{create_post, get_all_posts, get_post};
use crate::routes::squeaks::{create_comment, get_all_comments};
use config::launch_service;

pub(crate) mod errors;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    launch_service!(
        service: "post",
        routes: [create_topic, get_all_topics, get_topic, create_post, get_post, get_all_posts, create_comment, get_all_comments, create_reply, get_replies]
    );
    Ok(())
}
