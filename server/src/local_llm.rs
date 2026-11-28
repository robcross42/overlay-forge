use crate::error::ApiError;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::Semaphore;

const OLLAMA_CHAT_URL: &str = "http://127.0.0.1:11434/api/chat";
const MODEL: &str = "qwen3:8b";

pub struct LocalLlm {
    http: reqwest::Client,
    inference: Semaphore,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Role {
    User,
    Assistant,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatInput {
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: &'static str,
    messages: Vec<Message>,
    stream: bool,
    think: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: ChatReply,
}

#[derive(Deserialize, Serialize)]
pub struct ChatReply {
    content: String,
}

impl LocalLlm {
    pub fn new() -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self {
            http,
            inference: Semaphore::new(1),
        })
    }

    pub async fn chat(&self, input: ChatInput) -> Result<ChatReply, ApiError> {
        let messages = input.messages;
        if messages.is_empty()
            || messages.len() > 13
            || messages.len() % 2 == 0
            || messages
                .iter()
                .zip([Role::User, Role::Assistant].into_iter().cycle())
                .any(|(message, expected)| {
                    message.role != expected || message.content.trim().is_empty()
                })
        {
            return Err(ApiError(
                StatusCode::BAD_REQUEST,
                "Send a local chat message.",
            ));
        }
        let _permit = self.inference.try_acquire().map_err(|_| {
            ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "Local chat is already answering a prompt. Try again shortly.",
            )
        })?;
        let response = self
            .http
            .post(OLLAMA_CHAT_URL)
            .json(&OllamaRequest {
                model: MODEL,
                messages,
                stream: false,
                think: false,
                options: OllamaOptions { num_predict: 512 },
            })
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    ApiError(
                        StatusCode::GATEWAY_TIMEOUT,
                        "Ollama took too long to answer. Try again.",
                    )
                } else {
                    ApiError(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Local chat cannot reach Ollama on Ubuntu. Check that Ollama is running.",
                    )
                }
            })?;
        match response.status() {
            StatusCode::NOT_FOUND => {
                return Err(ApiError(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "The local model qwen3:8b is unavailable in Ollama on Ubuntu.",
                ));
            }
            StatusCode::OK => {}
            _ => {
                return Err(ApiError(
                    StatusCode::BAD_GATEWAY,
                    "Ollama could not answer the local chat request.",
                ))
            }
        }
        let reply = response
            .json::<OllamaResponse>()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    ApiError(
                        StatusCode::GATEWAY_TIMEOUT,
                        "Ollama took too long to answer. Try again.",
                    )
                } else {
                    ApiError(
                        StatusCode::BAD_GATEWAY,
                        "Ollama returned an invalid chat reply. Try again.",
                    )
                }
            })?
            .message;
        if reply.content.trim().is_empty() {
            return Err(ApiError(
                StatusCode::BAD_GATEWAY,
                "Ollama returned an empty chat reply. Try again.",
            ));
        }
        Ok(reply)
    }
}
