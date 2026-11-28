use super::oauth::Credentials;
use crate::{ChatMessage, ChatRole};
use reqwest::{
    blocking::{Client, Response},
    header::{ACCEPT, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Read},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub slug: String,
    pub display_name: String,
}
#[derive(Deserialize)]
struct Catalog {
    models: Vec<CatalogEntry>,
}
#[derive(Deserialize)]
struct CatalogEntry {
    slug: String,
    display_name: String,
    visibility: String,
}
fn safe_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        .take(160)
        .collect()
}
fn request_id(response: &Response) -> String {
    response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(safe_identifier)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unavailable".into())
}
pub struct RemoteFailure {
    pub status: u16,
    pub code: String,
    pub request_id: String,
}
impl RemoteFailure {
    pub fn read(mut response: Response) -> Self {
        let status = response.status().as_u16();
        let request_id = request_id(&response);
        let mut body = Vec::new();
        let _ = response.by_ref().take(65_536).read_to_end(&mut body);
        let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        let code = parsed
            .pointer("/error/code")
            .or_else(|| parsed.get("error").filter(|v| v.is_string()))
            .and_then(Value::as_str)
            .map(safe_identifier)
            .unwrap_or_else(|| "unstructured_admission_error".into());
        Self {
            status,
            code,
            request_id,
        }
    }
    pub fn terminal_refresh(&self) -> bool {
        matches!(
            self.code.as_str(),
            "invalid_grant"
                | "invalid_refresh_token"
                | "token_expired"
                | "refresh_token_expired"
                | "refresh_token_invalidated"
                | "refresh_token_reused"
        )
    }
    pub fn message(&self) -> String {
        format!(
            "ChatGPT HTTP {}; code {}; request {}. No API-key fallback was used.",
            self.status, self.code, self.request_id
        )
    }
}
pub fn http_error(response: Response) -> String {
    RemoteFailure::read(response).message()
}
pub fn models(http: &Client, credentials: &Credentials) -> Result<Vec<Model>, String> {
    if !credentials.plan_allowed() {
        return Err(
            "Signed in, but ChatGPT plan usage was not granted. Reauthorize in the browser.".into(),
        );
    }
    let response = http
        .get("https://api.openai.com/v1/models")
        .bearer_auth(&credentials.access_token)
        .send()
        .map_err(|_| "ChatGPT model catalog connection failed.")?;
    if !response.status().is_success() {
        return Err(http_error(response));
    }
    let catalog: Catalog = response
        .json()
        .map_err(|_| "Unexpected ChatGPT model catalog format; inference remains disabled.")?;
    let models: Vec<_> = catalog
        .models
        .into_iter()
        .filter(|m| m.visibility == "list")
        .map(|m| Model {
            slug: m.slug,
            display_name: m.display_name,
        })
        .collect();
    if models.is_empty() {
        return Err("This ChatGPT registration returned no available models.".into());
    }
    Ok(models)
}
pub enum Inference<'a> {
    Verification,
    Chat(&'a [ChatMessage]),
}

fn request_body(model: &str, input: &[ChatMessage]) -> Value {
    json!({"model":model,"input":input,"store":false,"stream":true})
}

fn verification_body(model: &str) -> Value {
    request_body(
        model,
        &[ChatMessage {
            role: ChatRole::User,
            content: "Say exactly: Overlay Forge connection verified.".into(),
        }],
    )
}
#[derive(Default)]
pub struct StreamResult {
    pub text: String,
    pub completed: bool,
    pub sequence: Option<u64>,
    pub correlation: Option<String>,
    require_completion_identity: bool,
}
#[derive(Debug)]
pub struct StreamFailure {
    pub confirmed_failure: bool,
    pub detail: String,
}
impl StreamFailure {
    fn failed(detail: String) -> Self {
        Self {
            confirmed_failure: true,
            detail,
        }
    }
}
impl From<&str> for StreamFailure {
    fn from(detail: &str) -> Self {
        Self {
            confirmed_failure: false,
            detail: detail.into(),
        }
    }
}
impl From<String> for StreamFailure {
    fn from(detail: String) -> Self {
        Self {
            confirmed_failure: false,
            detail,
        }
    }
}
impl StreamResult {
    pub fn event(&mut self, data: &str) -> Result<(), StreamFailure> {
        if self.completed {
            return Ok(());
        }
        if data == "[DONE]" {
            return Ok(());
        }
        let event: Value =
            serde_json::from_str(data).map_err(|_| "Invalid ChatGPT stream event.")?;
        let sequence = event.get("sequence_number").and_then(Value::as_u64);
        if let Some(sequence) = sequence {
            if self.sequence.is_some_and(|old| sequence <= old) {
                return Ok(());
            }
            self.sequence = Some(sequence);
        }
        let response_id = event.pointer("/response/id").and_then(Value::as_str);
        match event.get("type").and_then(Value::as_str) {
            Some("response.output_text.delta" | "response.refusal.delta") => {
                if let Some(text) = event.get("delta").and_then(Value::as_str) {
                    if self.text.len() + text.len() > 65_536 {
                        return Err("ChatGPT output exceeded the local reply limit.".into());
                    }
                    self.text.push_str(text);
                }
            }
            Some("response.completed") => {
                if self.require_completion_identity
                    && (sequence.is_none() || response_id.is_none_or(str::is_empty))
                {
                    return Err("The response had no content type and the completion event lacked a sequence or response ID.".into());
                }
                self.completed = true;
            }
            Some("response.failed" | "response.incomplete" | "error") => {
                let code = event
                    .pointer("/response/error/code")
                    .or_else(|| event.get("code"))
                    .and_then(Value::as_str)
                    .map(safe_identifier)
                    .unwrap_or_else(|| "incomplete_or_failed".into());
                return Err(StreamFailure::failed(format!(
                    "ChatGPT stream ended with {code}. No funding path was changed."
                )));
            }
            _ => (),
        }
        if let Some(id) = response_id {
            self.correlation = Some(safe_identifier(id));
        }
        Ok(())
    }
}
pub fn stream_reply(
    http: &Client,
    credentials: &Credentials,
    model: &str,
    inference: Inference<'_>,
    cancel: &AtomicBool,
    mut update: impl FnMut(&StreamResult),
) -> Result<StreamResult, StreamFailure> {
    if !credentials.plan_allowed() || credentials.access_token.is_empty() {
        return Err(StreamFailure::failed(
            "ChatGPT plan use is not authorized.".into(),
        ));
    }
    let body = match inference {
        Inference::Verification => verification_body(model),
        Inference::Chat(input) => request_body(model, input),
    };
    let response = http
        .post("https://api.openai.com/v1/responses")
        .timeout(Duration::from_secs(90))
        .bearer_auth(&credentials.access_token)
        .header(ACCEPT, "text/event-stream")
        .json(&body)
        .send()
        .map_err(|_| "ChatGPT connection failed or timed out; no automatic replay occurred.")?;
    if !response.status().is_success() {
        return Err(StreamFailure::failed(http_error(response)));
    }
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    let missing_content_type = !response.headers().contains_key(CONTENT_TYPE);
    if !missing_content_type
        && !content_type.is_some_and(|value| value.eq_ignore_ascii_case("text/event-stream"))
    {
        let media_type = content_type
            .and_then(|value| value.split_once('/'))
            .map(|(kind, subtype)| {
                format!("{}/{}", safe_identifier(kind), safe_identifier(subtype))
            })
            .unwrap_or_else(|| "missing or invalid".into());
        return Err(format!(
            "ChatGPT returned HTTP {} with content type {}. Expected a ChatGPT event stream. Request ID: {}.",
            response.status().as_u16(), media_type, request_id(&response),
        ).into());
    }
    let missing_header_detail = missing_content_type.then(|| {
        format!(
            "ChatGPT returned HTTP {} without a content type. Request ID: {}.",
            response.status().as_u16(),
            request_id(&response),
        )
    });
    let mut reader = BufReader::new(response.take(1_048_576));
    let mut result = StreamResult {
        require_completion_identity: missing_content_type,
        ..Default::default()
    };
    let mut data = String::new();
    let mut line = String::new();
    let mut saw_body = false;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err("Stream stopped locally; remote cancellation is unconfirmed.".into());
        }
        line.clear();
        if reader
            .read_line(&mut line)
            .map_err(|_| "ChatGPT stream disconnected; remote completion is unknown.")?
            == 0
        {
            break;
        }
        saw_body = true;
        if line.len() > 65_536 {
            return Err("ChatGPT stream event exceeded the local limit.".into());
        }
        let line = line.trim_start_matches('\u{feff}');
        if line.trim_end().is_empty() {
            if !data.is_empty() {
                result.event(data.trim_end())?;
                data.clear();
                update(&result);
                if result.completed {
                    return Ok(result);
                }
            }
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push_str(value.trim_start());
            data.push('\n');
        } else if let Some(detail) = &missing_header_detail {
            if !line.starts_with(':')
                && !["event:", "id:", "retry:"]
                    .iter()
                    .any(|field| line.starts_with(*field))
            {
                return Err(format!("{detail} The body was not framed as an event stream.").into());
            }
        }
    }
    if let Some(detail) = missing_header_detail {
        let body_detail = if saw_body {
            "The body ended before a usable completion event arrived."
        } else {
            "The response body was empty."
        };
        return Err(format!("{detail} {body_detail}").into());
    }
    Err(
        "Stream ended without response.completed. Completion is unknown; no request was replayed."
            .into(),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plan_request_has_no_unsupported_or_paid_fallback_fields() {
        let body = verification_body("account-model");
        assert_eq!(body["store"], false);
        assert_eq!(body["stream"], true);
        assert_eq!(body.as_object().unwrap().len(), 4);
        assert!(body["input"].is_array());
    }
    #[test]
    fn stream_requires_terminal_evidence_and_rejects_late_duplicate_deltas() {
        let mut r = StreamResult::default();
        r.event(r#"{"type":"response.output_text.delta","sequence_number":1,"delta":"OK"}"#)
            .unwrap();
        r.event(r#"{"type":"response.output_text.delta","sequence_number":1,"delta":"bad"}"#)
            .unwrap();
        assert_eq!(r.text, "OK");
        assert!(!r.completed);
        r.event(r#"{"type":"response.completed","sequence_number":2,"response":{"id":"test"}}"#)
            .unwrap();
        assert!(r.completed);
    }
    #[test]
    fn usage_limit_failure_is_not_success() {
        let mut r = StreamResult::default();
        assert!(r.event(r#"{"type":"response.failed","response":{"error":{"code":"subscription_sharing_usage_limit_exceeded"}}}"#).is_err());
        assert!(!r.completed);
    }
}
