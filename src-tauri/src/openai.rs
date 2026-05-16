use crate::db::{PlanningMessageRecord, ProjectRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const DEFAULT_MODEL: &str = "gpt-5";
pub const PLANNING_SYSTEM_INSTRUCTION: &str = "You are helping plan the selected Overlay Forge local project. Keep responses concise, practical, and implementation-oriented. Prefer Codex-ready structure when the user asks for implementation planning. Do not claim repository access unless repository content was explicitly provided to the model request.";

#[derive(Serialize)]
struct ResponsesRequest {
    model: &'static str,
    instructions: &'static str,
    input: Vec<ResponsesInputMessage>,
    store: bool,
}

#[derive(Serialize)]
struct ResponsesInputMessage {
    role: String,
    content: String,
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
    project: &ProjectRecord,
    messages: &[PlanningMessageRecord],
) -> Result<String, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();

    if api_key.is_empty() {
        return Err(
            "OpenAI API key is not configured. Set OPENAI_API_KEY and restart Overlay Forge."
                .to_string(),
        );
    }

    let request = ResponsesRequest {
        model: DEFAULT_MODEL,
        instructions: PLANNING_SYSTEM_INSTRUCTION,
        input: build_input(project, messages),
        store: false,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key)
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
) -> Vec<ResponsesInputMessage> {
    let mut input = vec![ResponsesInputMessage {
        role: "user".to_string(),
        content: format!(
            "Selected local project context:\nName: {}\nStatus: {}\nDescription: {}",
            project.name, project.status, project.description
        ),
    }];

    input.extend(messages.iter().map(|message| ResponsesInputMessage {
        role: message.role.clone(),
        content: message.content.clone(),
    }));

    input
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
