mod errors;
mod routes;
pub(crate) mod service;

use axum::routing::{get, post};
use axum::Router;
use config::server;
use routes::echoes::{create_reply, get_replies};
use routes::feed::get_feed;
use routes::mischief::{create_topic, get_active_topics, get_all_topics, get_topic, get_topic_tags};
use routes::nibbles::{create_post, get_all_posts, get_post};
use routes::squeaks::{create_comment, get_all_comments};
use routes::stats::get_internal_user_stats;
use routes::user_nibs::get_user_nibs;
use routes::votes::{cast_comment_vote, cast_post_vote};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::serve(
        "post",
        Router::new()
            .route("/mcf/active", get(get_active_topics))
            .route("/mcf", post(create_topic).get(get_all_topics))
            .route("/mcf/:topic", get(get_topic))
            .route("/mcf/:topic/tags", get(get_topic_tags))
            .route("/mcf/:topic/nib", post(create_post).get(get_all_posts))
            .route("/mcf/:topic/nib/:post_id", get(get_post))
            .route("/mcf/:topic/nib/:post_id/vote", post(cast_post_vote))
            .route(
                "/mcf/:topic/nib/:post_id/sqk",
                post(create_comment).get(get_all_comments),
            )
            .route(
                "/mcf/:topic/nib/:post_id/sqk/:hash/vote",
                post(cast_comment_vote),
            )
            .route(
                "/mcf/:topic/nib/:post_id/sqk/:hash/echoes",
                post(create_reply).get(get_replies),
            )
            .route("/feed", get(get_feed))
            .route("/users/me/nibs", get(get_user_nibs))
            .route("/internal/stats/:user_id", get(get_internal_user_stats)),
    )
    .await
}
