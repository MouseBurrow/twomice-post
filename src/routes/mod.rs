pub mod boards;
pub mod comments;
pub mod feed;
pub mod posts;
pub mod replies;
pub mod stats;
pub mod user_posts;
pub mod votes;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ContentBody {
    pub content: String,
}

#[derive(Deserialize)]
pub struct EchoBody {
    pub content: String,
    pub reply_hash: Option<String>,
}
