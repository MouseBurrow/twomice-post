pub(crate) mod echoes;
pub(crate) mod mischief;
pub(crate) mod nibbles;
pub(crate) mod squeaks;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ContentBody {
    pub content: String,
}
