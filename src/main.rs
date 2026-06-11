mod errors;
mod routes;
pub(crate) mod service;

use axum::routing::{get, post};
use axum::Router;
use config::server;
use routes::boards::{create_board, get_active_boards, get_all_boards, get_board, get_board_tags};
use routes::comments::{create_comment, get_all_comments};
use routes::feed::get_feed;
use routes::posts::{create_post, get_all_posts, get_post};
use routes::replies::{create_reply, get_replies};
use routes::stats::get_internal_user_stats;
use routes::user_posts::get_user_posts;
use routes::votes::{cast_comment_vote, cast_post_vote};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::serve(
        "post",
        Router::new()
            .route("/b/active", get(get_active_boards))
            .route("/b", post(create_board).get(get_all_boards))
            .route("/b/:topic", get(get_board))
            .route("/b/:topic/tags", get(get_board_tags))
            .route("/b/:topic/nib", post(create_post).get(get_all_posts))
            .route("/b/:topic/nib/:post_id", get(get_post))
            .route("/b/:topic/nib/:post_id/vote", post(cast_post_vote))
            .route(
                "/b/:topic/nib/:post_id/sqk",
                post(create_comment).get(get_all_comments),
            )
            .route(
                "/b/:topic/nib/:post_id/sqk/:hash/vote",
                post(cast_comment_vote),
            )
            .route(
                "/b/:topic/nib/:post_id/sqk/:hash/echoes",
                post(create_reply).get(get_replies),
            )
            .route("/feed", get(get_feed))
            .route("/users/me/nibs", get(get_user_posts))
            .route("/internal/stats/:user_id", get(get_internal_user_stats)),
    )
    .await
}
