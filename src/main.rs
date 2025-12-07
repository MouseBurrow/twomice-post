use crate::routes::comment::{add_comment, reply_comment};
use crate::routes::post::post;

use config::launch_service;

mod db;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    launch_service!(
        service: "post",
        routes: [post, add_comment, reply_comment]
    );
    Ok(())
}
