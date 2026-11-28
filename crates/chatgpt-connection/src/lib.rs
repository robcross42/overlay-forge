//! ChatGPT plan authorization and inference shared by desktop and browser hosts.
use serde::{Deserialize, Serialize};

pub mod oauth;
pub mod transport;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}
