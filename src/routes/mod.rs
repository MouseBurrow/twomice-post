pub(crate) mod echoes;
pub(crate) mod feed;
pub(crate) mod mischief;
pub(crate) mod nibbles;
pub(crate) mod squeaks;
pub(crate) mod stats;
pub(crate) mod user_nibs;
pub(crate) mod votes;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ContentBody {
    pub content: String,
}
