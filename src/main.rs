use axum::routing::{get, post};
use axum::Router;
use config::server;
use post::routes::boards::{create_board, get_active_boards, get_all_boards, get_board, get_board_tags};
use post::routes::comments::{create_comment, get_all_comments};
use post::routes::feed::get_feed;
use post::routes::posts::{create_post, get_all_posts, get_post};
use post::routes::replies::{create_reply, get_replies};
use post::routes::stats::get_internal_user_stats;
use post::routes::user_posts::get_user_posts;
use post::routes::votes::{cast_comment_vote, cast_post_vote, cast_reply_vote};

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
            .route(
                "/b/:topic/nib/:post_id/sqk/:comment_hash/echoes/:reply_hash/vote",
                post(cast_reply_vote),
            )
            .route("/feed", get(get_feed))
            .route("/users/me/nibs", get(get_user_posts))
            .route("/internal/stats/:user_id", get(get_internal_user_stats)),
    )
    .await
}
