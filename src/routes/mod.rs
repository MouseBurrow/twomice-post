pub(crate) mod boards;
pub(crate) mod comments;
pub(crate) mod feed;
pub(crate) mod posts;
pub(crate) mod replies;
pub(crate) mod stats;
pub(crate) mod user_posts;
pub(crate) mod votes;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ContentBody {
    pub content: String,
}

#[derive(Deserialize)]
pub(crate) struct EchoBody {
    pub content: String,
    pub reply_hash: Option<String>,
}
