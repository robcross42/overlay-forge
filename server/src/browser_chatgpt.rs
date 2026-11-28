use crate::{error::ApiError, routes, Shared};
use axum::{
    extract::{FromRequest, Request, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
    Json,
};
use overlay_forge_chatgpt_connection::{
    oauth::{self, AuthAttempt, Credentials},
    transport::{self, Inference, Model},
    ChatMessage, ChatRole,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use zeroize::Zeroize;

struct Pending {
    browser_session: String,
    attempt: AuthAttempt,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: &'static str,
    pub detail: String,
    pub observed_at: Option<u64>,
    pub account: Option<String>,
    pub models: Vec<Model>,
}

pub struct BrowserChatgpt {
    port: u16,
    host: String,
    http: reqwest::blocking::Client,
    pending: Mutex<Option<Pending>>,
    credentials: Mutex<Option<Credentials>>,
    status: Mutex<Snapshot>,
    operation: Arc<Semaphore>,
    active_reply: Mutex<Option<Arc<AtomicBool>>>,
}

impl BrowserChatgpt {
    pub fn new(port: u16, data_dir: &Path) -> Result<Self, String> {
        Ok(Self {
            port,
            host: host_id(data_dir)?,
            http: reqwest::blocking::Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(20))
                .build()
                .map_err(|_| "Could not initialize ChatGPT transport.")?,
            pending: Mutex::new(None),
            credentials: Mutex::new(None),
            status: Mutex::new(Snapshot {
                state: "disconnected",
                detail: "Continue with ChatGPT to connect this Ubuntu service.".into(),
                observed_at: None,
                account: None,
                models: Vec::new(),
            }),
            operation: Arc::new(Semaphore::new(1)),
            active_reply: Mutex::new(None),
        })
    }

    fn reserve(&self) -> Result<OwnedSemaphorePermit, ApiError> {
        self.operation.clone().try_acquire_owned().map_err(|_| {
            ApiError(
                StatusCode::CONFLICT,
                "Another ChatGPT operation is running.",
            )
        })
    }

    fn event(&self, state: &'static str, detail: impl Into<String>) {
        if let Ok(mut status) = self.status.lock() {
            status.state = state;
            status.detail = detail.into();
            status.observed_at = Some(oauth::seconds());
        }
    }

    fn snapshot(&self) -> Result<Snapshot, ApiError> {
        let mut pending = self.pending.lock().map_err(|_| ApiError::unavailable())?;
        if pending.as_ref().is_some_and(|p| p.attempt.expired()) {
            *pending = None;
            self.event("failed", "ChatGPT sign-in expired. Start again.");
        }
        self.status
            .lock()
            .map(|s| s.clone())
            .map_err(|_| ApiError::unavailable())
    }

    fn begin(&self, browser_session: String) -> Result<String, ApiError> {
        let _permit = self.reserve()?;
        let registration = self
            .credentials
            .lock()
            .map_err(|_| ApiError::unavailable())?;
        let attempt = AuthAttempt::for_browser(
            self.port,
            registration.as_ref().map(|c| c.client_id.clone()),
            registration.as_ref().map(|c| c.subject.clone()),
        );
        let url = attempt.authorization_url(&self.host);
        let mut pending = self.pending.lock().map_err(|_| ApiError::unavailable())?;
        *pending = Some(Pending {
            browser_session,
            attempt,
        });
        self.event(
            "awaitingConsent",
            "Waiting for ChatGPT sign-in in your browser.",
        );
        Ok(url)
    }

    fn take_callback(&self, session: &str, state: &str) -> Result<AuthAttempt, ApiError> {
        let mut pending = self.pending.lock().map_err(|_| ApiError::unavailable())?;
        if !pending
            .as_ref()
            .is_some_and(|p| p.browser_session == session && p.attempt.matches_state(state))
        {
            return Err(ApiError::forbidden());
        }
        let attempt = pending.take().ok_or_else(ApiError::forbidden)?.attempt;
        self.event(
            "authorizing",
            "Verifying the ChatGPT account and plan permissions.",
        );
        Ok(attempt)
    }

    fn connect(
        &self,
        mut attempt: AuthAttempt,
        target: &str,
        can_connect: impl FnOnce() -> bool,
    ) -> Result<(), String> {
        let (code, client) = attempt.callback(target)?;
        let credentials = attempt.exchange(&self.http, &code, &client)?;
        let models = credentials
            .plan_allowed()
            .then(|| transport::models(&self.http, &credentials));
        let (state, detail, models) = match models {
            Some(Ok(models)) => (
                "connected",
                "ChatGPT plan access is connected.".into(),
                models,
            ),
            Some(Err(detail)) => ("limited", detail, Vec::new()),
            None => (
                "limited",
                "ChatGPT signed in without permission to use the plan.".into(),
                Vec::new(),
            ),
        };
        if !can_connect() {
            return Err("Your browser session ended before ChatGPT sign-in completed.".into());
        }
        let account = credentials.email.clone();
        *self
            .credentials
            .lock()
            .map_err(|_| "ChatGPT connection is unavailable.")? = Some(credentials);
        let mut status = self
            .status
            .lock()
            .map_err(|_| "ChatGPT status is unavailable.")?;
        status.account = account;
        status.models = models;
        status.state = state;
        status.detail = detail;
        status.observed_at = Some(oauth::seconds());
        Ok(())
    }

    fn cancel_sign_in(&self, session: &str) -> Result<(), ApiError> {
        let _permit = self.reserve()?;
        let mut pending = self.pending.lock().map_err(|_| ApiError::unavailable())?;
        if !pending
            .as_ref()
            .is_some_and(|attempt| attempt.browser_session == session)
        {
            return Err(ApiError::forbidden());
        }
        *pending = None;
        let state = if self
            .credentials
            .lock()
            .map_err(|_| ApiError::unavailable())?
            .is_some()
        {
            "connected"
        } else {
            "disconnected"
        };
        self.event(state, "The pending ChatGPT sign-in was cancelled.");
        Ok(())
    }

    fn disconnect(&self) -> Result<(), ApiError> {
        let _permit = self.reserve()?;
        *self.pending.lock().map_err(|_| ApiError::unavailable())? = None;
        *self
            .credentials
            .lock()
            .map_err(|_| ApiError::unavailable())? = None;
        let mut status = self.status.lock().map_err(|_| ApiError::unavailable())?;
        status.models.clear();
        status.account = None;
        status.state = "disconnected";
        status.detail = "ChatGPT was disconnected from this Ubuntu service.".into();
        status.observed_at = Some(oauth::seconds());
        Ok(())
    }

    fn reply(&self, input: Input, cancel: &AtomicBool) -> Result<String, String> {
        if cancel.load(Ordering::SeqCst) {
            return Err("Stopped before the prompt was sent.".into());
        }
        let mut credentials = self
            .credentials
            .lock()
            .map_err(|_| "ChatGPT connection is unavailable.")?;
        let credentials = credentials
            .as_mut()
            .ok_or("Continue with ChatGPT before sending a prompt.")?;
        if credentials.expires_at <= oauth::seconds() + 60 {
            self.refresh(credentials)?;
        }
        if cancel.load(Ordering::SeqCst) {
            return Err("Stopped before the prompt was sent.".into());
        }
        let result = transport::stream_reply(
            &self.http,
            credentials,
            &input.model,
            Inference::Chat(&input.messages),
            cancel,
            |_| {},
        )
        .map_err(|error| error.detail)?;
        Ok(result.text)
    }

    fn refresh(&self, credentials: &mut Credentials) -> Result<(), String> {
        if credentials.refresh_token.is_empty() {
            return Err("ChatGPT access expired. Sign in again.".into());
        }
        let response = self
            .http
            .post(oauth::TOKEN_ENDPOINT)
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", credentials.client_id.as_str()),
                ("refresh_token", credentials.refresh_token.as_str()),
                ("resource", oauth::RESOURCE),
            ])
            .send()
            .map_err(|_| "ChatGPT could not refresh the connection.")?;
        if !response.status().is_success() {
            let failure = transport::RemoteFailure::read(response);
            if failure.terminal_refresh() {
                credentials.access_token.zeroize();
                credentials.refresh_token.zeroize();
                credentials.id_token.zeroize();
                credentials.scope.zeroize();
                credentials.expires_at = 0;
                if let Ok(mut status) = self.status.lock() {
                    status.models.clear();
                }
            }
            return Err(failure.message());
        }
        let mut tokens = oauth::read_tokens(response)?;
        let expiry = oauth::seconds()
            .checked_add(tokens.expires_in)
            .ok_or("ChatGPT expiry was invalid.")?;
        credentials.access_token.zeroize();
        credentials.access_token = std::mem::take(&mut tokens.access_token);
        if !tokens.refresh_token.is_empty() {
            credentials.refresh_token.zeroize();
            credentials.refresh_token = std::mem::take(&mut tokens.refresh_token);
        }
        if !tokens.scope.is_empty() {
            credentials.scope = std::mem::take(&mut tokens.scope);
        }
        credentials.expires_at = expiry;
        if !credentials.plan_allowed() || credentials.access_token.is_empty() {
            if let Ok(mut status) = self.status.lock() {
                status.models.clear();
            }
            return Err("ChatGPT plan permission is no longer available. Sign in again.".into());
        }
        Ok(())
    }
}

/// Only a non-secret host identifier survives service restart.
fn host_id(directory: &Path) -> Result<String, &'static str> {
    let path = directory.join("chatgpt-browser-host-id");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let unavailable = "Could not access the browser ChatGPT host identifier.";
    match options.open(&path) {
        Ok(mut file) => {
            let value = uuid::Uuid::new_v4().urn().to_string();
            file.write_all(value.as_bytes()).map_err(|_| unavailable)?;
            Ok(value)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = std::fs::symlink_metadata(&path).map_err(|_| unavailable)?;
            if !metadata.is_file() || metadata.len() > 128 {
                return Err(unavailable);
            }
            let value = std::fs::read_to_string(&path).map_err(|_| unavailable)?;
            let id = uuid::Uuid::parse_str(&value).map_err(|_| unavailable)?;
            if id.urn().to_string() != value {
                return Err(unavailable);
            }
            Ok(value)
        }
        Err(_) => Err(unavailable),
    }
}

#[derive(Serialize)]
pub struct SignInUrl {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    model: String,
    messages: Vec<ChatMessage>,
}

impl Input {
    fn valid(&self) -> bool {
        !self.messages.is_empty()
            && self.messages.len() <= 13
            && self.messages.len() % 2 == 1
            && self.messages.iter().enumerate().all(|(index, message)| {
                !message.content.trim().is_empty()
                    && message.content.len() <= 16_000
                    && matches!(
                        (&message.role, index % 2),
                        (ChatRole::User, 0) | (ChatRole::Assistant, 1)
                    )
            })
    }
}

#[derive(Serialize)]
pub struct Reply {
    content: String,
}

fn service(state: &Shared) -> Result<&BrowserChatgpt, ApiError> {
    state.chatgpt.as_ref().ok_or_else(ApiError::forbidden)
}

pub async fn status(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Snapshot>, ApiError> {
    routes::authorize_admin(&state, &headers)?;
    service(&state)?.snapshot().map(Json)
}

pub async fn start(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<SignInUrl>, ApiError> {
    let session = routes::authorize_admin(&state, &headers)?;
    let url = service(&state)?.begin(session)?;
    Ok(Json(SignInUrl { url }))
}

pub async fn callback(
    state: Shared,
    headers: &HeaderMap,
    oauth_state: &str,
    target: String,
) -> Redirect {
    let original = Redirect::to("/?chatgpt=returned");
    let rejected = || Redirect::to("/?chatgpt=denied");
    let Ok(session) = routes::authenticate_admin(&state, headers) else {
        return rejected();
    };
    let Ok(chatgpt) = service(&state) else {
        return rejected();
    };
    if target.len() > 8192 {
        return rejected();
    }
    let Ok(permit) = chatgpt.reserve() else {
        return rejected();
    };
    let Ok(attempt) = chatgpt.take_callback(&session, oauth_state) else {
        return rejected();
    };
    let outcome = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        if let Ok(chatgpt) = service(&state) {
            if let Err(detail) = chatgpt.connect(attempt, &target, || {
                state
                    .repository
                    .session(&session, oauth::seconds())
                    .ok()
                    .flatten()
                    .is_some_and(|browser| state.config.is_browser_admin(&browser.email))
            }) {
                chatgpt.event("failed", detail);
            }
        }
    })
    .await;
    if outcome.is_err() {
        rejected()
    } else {
        original
    }
}

pub async fn cancel_sign_in(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let session = routes::authorize_admin(&state, &headers)?;
    service(&state)?.cancel_sign_in(&session)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn disconnect(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    routes::authorize_admin(&state, &headers)?;
    service(&state)?.disconnect()?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stop(State(state): State<Shared>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    routes::authorize_admin(&state, &headers)?;
    let chatgpt = service(&state)?;
    if let Some(cancel) = chatgpt
        .active_reply
        .lock()
        .map_err(|_| ApiError::unavailable())?
        .as_ref()
    {
        cancel.store(true, Ordering::SeqCst);
        chatgpt.event(
            "stopping",
            "Stop requested. Waiting for the local stream to close.",
        );
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn chat(State(state): State<Shared>, request: Request) -> Result<Json<Reply>, ApiError> {
    routes::authorize_admin(&state, request.headers())?;
    let Json(input) = Json::<Input>::from_request(request, &state)
        .await
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "Invalid ChatGPT conversation."))?;
    let chatgpt = service(&state)?;
    let permit = chatgpt.reserve()?;
    let snapshot = chatgpt.snapshot()?;
    if !input.valid()
        || !snapshot
            .models
            .iter()
            .any(|model| model.slug == input.model)
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Choose a connected ChatGPT model and enter a valid conversation.",
        ));
    }
    if snapshot.state == "awaitingConsent" {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "Finish connecting ChatGPT before sending a message.",
        ));
    }
    let cancel = Arc::new(AtomicBool::new(false));
    *chatgpt
        .active_reply
        .lock()
        .map_err(|_| ApiError::unavailable())? = Some(cancel.clone());
    chatgpt.event("requesting", "Waiting for a response from ChatGPT.");
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let chatgpt = service(&state)?;
        let reply = chatgpt.reply(input, &cancel);
        *chatgpt
            .active_reply
            .lock()
            .map_err(|_| ApiError::unavailable())? = None;
        match reply {
            Ok(content) => {
                chatgpt.event("completed", "ChatGPT confirmed completion.");
                Ok(Json(Reply { content }))
            }
            Err(detail) => {
                chatgpt.event(
                    if cancel.load(Ordering::SeqCst) {
                        "stopped"
                    } else {
                        "failed"
                    },
                    detail,
                );
                Err(ApiError(
                    StatusCode::BAD_GATEWAY,
                    "ChatGPT did not complete this reply. See connection status for details.",
                ))
            }
        }
    })
    .await
    .map_err(|_| ApiError::unavailable())?
}
