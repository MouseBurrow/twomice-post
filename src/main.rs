mod errors;
mod routes;
pub(crate) mod service;

use axum::routing::{get, post};
use axum::Router;
use config::server;
use routes::echoes::{create_reply, get_replies};
use routes::mischief::{create_topic, get_all_topics, get_topic};
use routes::nibbles::{create_post, get_all_posts, get_post};
use routes::squeaks::{create_comment, get_all_comments};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::serve("post", Router::new()
        .route("/mcf", post(create_topic).get(get_all_topics))
        .route("/mcf/:topic", get(get_topic))
        .route("/mcf/:topic/nib", post(create_post).get(get_all_posts))
        .route("/mcf/:topic/nib/:post_id", get(get_post))
        .route(
            "/mcf/:topic/nib/:post/sqk",
            post(create_comment).get(get_all_comments),
        )
        .route(
            "/mcf/:topic/nib/:post/sqk/:comment/echoes",
            post(create_reply).get(get_replies),
        )
    ).await
}
