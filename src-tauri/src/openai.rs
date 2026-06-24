use crate::db::{GameChatMessageRecord, GameRecord, PlanningMessageRecord, ProjectRecord};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const DEFAULT_MODEL: &str = "gpt-5";
pub const PLANNING_SYSTEM_INSTRUCTION: &str = "You are helping plan the selected Overlay Forge local project. Keep responses concise, practical, and implementation-oriented. Prefer Codex-ready structure when the user asks for implementation planning. Do not claim repository access unless repository content was explicitly provided to the model request.";
pub const GAME_SYSTEM_INSTRUCTION: &str = "You are helping plan, analyze, and document the selected game workspace in Overlay Forge. Keep responses concise, practical, and grounded in the provided game context. When discussing visible parts, builds, screenshots, or physics behavior, distinguish observed facts from assumptions.";
pub const GAME_BUILD_GUIDE_SYSTEM_INSTRUCTION: &str = "You create practical GearBlocks build guides for Overlay Forge. Output only Markdown, with no conversational preface and no fenced code blocks. Use GearBlocks units and centimeters, never imperial units unless explicitly requested. Prefer known GearBlocks catalog part names from the provided context. Keep the guide phased, buildable, and focused on readable in-game assembly guidance.";

#[derive(Serialize)]
struct ResponsesRequest {
    model: &'static str,
    instructions: &'static str,
    input: Vec<ResponsesInputMessage>,
    reasoning: ResponsesReasoning,
    text: ResponsesText,
    store: bool,
}

#[derive(Serialize)]
struct ResponsesReasoning {
    effort: &'static str,
}

#[derive(Serialize)]
struct ResponsesText {
    verbosity: &'static str,
}

#[derive(Serialize)]
struct ResponsesInputMessage {
    role: String,
    content: Value,
}

pub struct GameChatImageInput {
    pub label: String,
    pub data_url: String,
}

#[derive(Deserialize)]
struct ResponsesErrorBody {
    error: Option<ResponsesError>,
}

#[derive(Deserialize)]
struct ResponsesError {
    message: String,
}

pub async fn create_planning_response(
    api_key: &str,
    project: &ProjectRecord,
    messages: &[PlanningMessageRecord],
    attached_context: &str,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(
            "OpenAI API key is not configured. Save one in Settings or set OPENAI_API_KEY."
                .to_string(),
        );
    }

    let request = ResponsesRequest {
        model: DEFAULT_MODEL,
        instructions: PLANNING_SYSTEM_INSTRUCTION,
        input: build_input(project, messages, attached_context),
        reasoning: low_latency_reasoning(),
        text: concise_text(),
        store: false,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key.trim())
        .json(&request)
        .send()
        .await
        .map_err(|error| format!("OpenAI request failed: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("OpenAI response could not be read: {error}"))?;

    if !status.is_success() {
        if let Ok(error_body) = serde_json::from_str::<ResponsesErrorBody>(&body) {
            if let Some(error) = error_body.error {
                return Err(format!("OpenAI request failed: {}", error.message));
            }
        }

        return Err(format!("OpenAI request failed with status {status}"));
    }

    extract_output_text(&body)
}

pub async fn create_game_response(
    api_key: &str,
    game: &GameRecord,
    messages: &[GameChatMessageRecord],
    attached_context: &str,
    images: &[GameChatImageInput],
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(
            "OpenAI API key is not configured. Save one in Settings or set OPENAI_API_KEY."
                .to_string(),
        );
    }

    let request = ResponsesRequest {
        model: DEFAULT_MODEL,
        instructions: GAME_SYSTEM_INSTRUCTION,
        input: build_game_input(game, messages, attached_context, images),
        reasoning: low_latency_reasoning(),
        text: concise_text(),
        store: false,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key.trim())
        .json(&request)
        .send()
        .await
        .map_err(|error| format!("OpenAI request failed: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("OpenAI response could not be read: {error}"))?;

    if !status.is_success() {
        if let Ok(error_body) = serde_json::from_str::<ResponsesErrorBody>(&body) {
            if let Some(error) = error_body.error {
                return Err(format!("OpenAI request failed: {}", error.message));
            }
        }

        return Err(format!("OpenAI request failed with status {status}"));
    }

    extract_output_text(&body)
}

pub async fn create_game_build_guide_response(
    api_key: &str,
    game: &GameRecord,
    messages: &[GameChatMessageRecord],
    attached_context: &str,
    build_goal: &str,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(
            "OpenAI API key is not configured. Save one in Settings or set OPENAI_API_KEY."
                .to_string(),
        );
    }

    let request = ResponsesRequest {
        model: DEFAULT_MODEL,
        instructions: GAME_BUILD_GUIDE_SYSTEM_INSTRUCTION,
        input: build_game_build_guide_input(game, messages, attached_context, build_goal),
        reasoning: low_latency_reasoning(),
        text: concise_text(),
        store: false,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key.trim())
        .json(&request)
        .send()
        .await
        .map_err(|error| format!("OpenAI request failed: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("OpenAI response could not be read: {error}"))?;

    if !status.is_success() {
        if let Ok(error_body) = serde_json::from_str::<ResponsesErrorBody>(&body) {
            if let Some(error) = error_body.error {
                return Err(format!("OpenAI request failed: {}", error.message));
            }
        }

        return Err(format!("OpenAI request failed with status {status}"));
    }

    extract_output_text(&body)
}

fn build_input(
    project: &ProjectRecord,
    messages: &[PlanningMessageRecord],
    attached_context: &str,
) -> Vec<ResponsesInputMessage> {
    let mut input = vec![ResponsesInputMessage {
        role: "user".to_string(),
        content: text_content(format!(
            "Selected local project context:\nName: {}\nStatus: {}\nDescription: {}",
            project.name, project.status, project.description
        )),
    }];

    if !attached_context.trim().is_empty() {
        input.push(ResponsesInputMessage {
            role: "user".to_string(),
            content: text_content(attached_context.to_string()),
        });
    }

    input.extend(messages.iter().map(|message| ResponsesInputMessage {
        role: message.role.clone(),
        content: text_content(message.content.clone()),
    }));

    input
}

fn build_game_input(
    game: &GameRecord,
    messages: &[GameChatMessageRecord],
    attached_context: &str,
    images: &[GameChatImageInput],
) -> Vec<ResponsesInputMessage> {
    let mut input = vec![ResponsesInputMessage {
        role: "user".to_string(),
        content: text_content(format!(
            "Selected game workspace context:\nName: {}\nSlug: {}\nSummary: {}",
            game.name, game.slug, game.summary
        )),
    }];

    if !attached_context.trim().is_empty() {
        input.push(ResponsesInputMessage {
            role: "user".to_string(),
            content: text_content(attached_context.to_string()),
        });
    }

    for (index, message) in messages.iter().enumerate() {
        let is_latest_user_message = index + 1 == messages.len() && message.role == "user";
        input.push(ResponsesInputMessage {
            role: message.role.clone(),
            content: if is_latest_user_message && !images.is_empty() {
                image_prompt_content(&message.content, images)
            } else {
                text_content(message.content.clone())
            },
        });
    }

    input
}

fn build_game_build_guide_input(
    game: &GameRecord,
    messages: &[GameChatMessageRecord],
    attached_context: &str,
    build_goal: &str,
) -> Vec<ResponsesInputMessage> {
    let mut input = vec![ResponsesInputMessage {
        role: "user".to_string(),
        content: text_content(format!(
            "Selected game workspace context:\nName: {}\nSlug: {}\nSummary: {}",
            game.name, game.slug, game.summary
        )),
    }];

    if !attached_context.trim().is_empty() {
        input.push(ResponsesInputMessage {
            role: "user".to_string(),
            content: text_content(attached_context.to_string()),
        });
    }

    if !messages.is_empty() {
        input.push(ResponsesInputMessage {
            role: "user".to_string(),
            content: text_content("Recent conversation context follows.".to_string()),
        });
        input.extend(messages.iter().map(|message| ResponsesInputMessage {
            role: message.role.clone(),
            content: text_content(message.content.clone()),
        }));
    }

    input.push(ResponsesInputMessage {
        role: "user".to_string(),
        content: text_content(format!(
            "{}\n\n{}\n\n{}\n{}\n{}\n{}\n{}",
            "Create an Overlay Forge GearBlocks build guide from this goal:",
            build_goal.trim(),
            "Required Markdown structure:",
            "# <short build guide title>",
            "## Build Goal\n## Scale Reference\n## Current Chosen Geometry\n## Main Parts List\n### <category>\n| Qty | Part | Purpose |\n| --- | --- | --- |",
            "## Assembly Instructions\n### 1. <step title>\n## First Test Checklist",
            "Output only the Markdown guide. Do not wrap it in triple backticks."
        )),
    });

    input
}

fn text_content(text: String) -> Value {
    json!(text)
}

fn low_latency_reasoning() -> ResponsesReasoning {
    ResponsesReasoning { effort: "low" }
}

fn concise_text() -> ResponsesText {
    ResponsesText { verbosity: "low" }
}

fn image_prompt_content(text: &str, images: &[GameChatImageInput]) -> Value {
    let mut content = vec![json!({
        "type": "input_text",
        "text": text,
    })];

    for image in images {
        content.push(json!({
            "type": "input_text",
            "text": format!("Attached screenshot: {}", image.label),
        }));
        content.push(json!({
            "type": "input_image",
            "image_url": image.data_url,
            "detail": "low",
        }));
    }

    json!(content)
}

fn extract_output_text(body: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("OpenAI response JSON was invalid: {error}"))?;

    if let Some(output_text) = value.get("output_text").and_then(Value::as_str) {
        let trimmed = output_text.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let mut text_parts = Vec::new();
    if let Some(output_items) = value.get("output").and_then(Value::as_array) {
        for item in output_items {
            if item.get("type").and_then(Value::as_str) != Some("message") {
                continue;
            }

            if let Some(content_items) = item.get("content").and_then(Value::as_array) {
                for content_item in content_items {
                    if let Some(text) = content_item.get("text").and_then(Value::as_str) {
                        text_parts.push(text.trim().to_string());
                    }
                }
            }
        }
    }

    let output = text_parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    if output.is_empty() {
        Err("OpenAI returned no assistant text.".to_string())
    } else {
        Ok(output)
    }
}
