mod secrets;
use overlay_forge_chatgpt_connection::{oauth, transport};
use crate::{
    ai_hub::{ChatMessage, CompletedReply, PendingChat, SavedChatConversation},
    db::HostDatabase,
    AppState,
};
use oauth::{seconds, AuthAttempt, Credentials};
use reqwest::blocking::Client;
use serde::Serialize;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    Disconnected,
    Authorizing,
    Connected,
    Limited,
    Verifying,
    Verified,
    Stopping,
    Unknown,
    Failed,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSnapshot {
    pub state: ConnectionState,
    pub account_label: Option<String>,
    pub registration: Option<String>,
    pub detail: String,
    pub models: Vec<transport::Model>,
    pub verification_text: String,
    pub plan_granted: bool,
    pub last_update: u64,
    pub provider_sequence: Option<u64>,
    pub correlation: Option<String>,
    pub busy: bool,
    pub verification: crate::status_watch::VerificationSnapshot,
}
pub struct ChatgptService {
    http: Client,
    status: Mutex<ConnectionSnapshot>,
    busy: AtomicBool,
    cancel: AtomicBool,
}
impl ChatgptService {
    pub fn new() -> Result<Self, String> {
        let http = Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_secs(25))
            .build()
            .map_err(|_| "Could not initialize ChatGPT secure transport.")?;
        Ok(Self {
            http,
            status: Mutex::new(ConnectionSnapshot {
                state: ConnectionState::Disconnected,
                account_label: None,
                registration: None,
                detail: "No ChatGPT account connected. API-key billing is disabled.".into(),
                models: Vec::new(),
                verification_text: String::new(),
                plan_granted: false,
                last_update: seconds(),
                provider_sequence: None,
                correlation: None,
                busy: false,
                verification: crate::status_watch::VerificationSnapshot::stream(
                    None,
                    crate::status_watch::now_ms(),
                    crate::status_watch::now_ms(),
                    false,
                ),
            }),
            busy: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
        })
    }
    pub fn snapshot(&self) -> Result<ConnectionSnapshot, String> {
        let mut snapshot = self
            .status
            .lock()
            .map_err(|_| "ChatGPT status unavailable.")?
            .clone();
        snapshot.busy = self.busy.load(Ordering::SeqCst);
        snapshot.verification = crate::status_watch::VerificationSnapshot::stream(
            snapshot.verification.last_verified_at,
            snapshot.last_update.saturating_mul(1000),
            crate::status_watch::now_ms(),
            matches!(snapshot.state, ConnectionState::Verified),
        );
        Ok(snapshot)
    }
    fn set(&self, app: &AppHandle, state: ConnectionState, detail: impl Into<String>) {
        if let Ok(mut status) = self.status.lock() {
            if matches!(
                state,
                ConnectionState::Authorizing
                    | ConnectionState::Verifying
                    | ConnectionState::Disconnected
            ) {
                status.verification.last_verified_at = None;
            }
            status.state = state;
            status.detail = detail.into();
            status.last_update = seconds();
        }
        self.publish(app);
    }
    fn publish(&self, app: &AppHandle) {
        if let Ok(mut hub) = app.state::<AppState>().hub.lock() {
            hub.touch();
        }
        let _ = app.emit("hub-changed", ());
    }
    fn load(&self, db: &HostDatabase) -> Result<Option<Credentials>, String> {
        db.ai_credentials()
            .map_err(|_| "Could not read protected ChatGPT credentials.")?
            .map(|encrypted| {
                let host_id = db
                    .ai_host_id()
                    .map_err(|_| "Could not resolve the host credential identity.")?;
                let plain = secrets::unprotect(&host_id, &encrypted)?;
                serde_json::from_slice(&plain)
                    .map_err(|_| "Protected ChatGPT credential record could not be read.".into())
            })
            .transpose()
    }
    fn save(&self, db: &HostDatabase, credentials: &Credentials) -> Result<(), String> {
        let serialized = Zeroizing::new(
            serde_json::to_vec(credentials)
                .map_err(|_| "Could not encode protected credentials.")?,
        );
        let host_id = db
            .ai_host_id()
            .map_err(|_| "Could not resolve the host credential identity.")?;
        let encrypted = secrets::protect(&host_id, &serialized)?;
        db.save_ai_credentials(Some(&encrypted))
            .map_err(|_| "Could not save protected ChatGPT credentials.".into())
    }
    pub fn restore(&self, db: &HostDatabase) -> Result<(), String> {
        db.ai_host_id()
            .map_err(|_| "Could not prepare the stable ChatGPT host identity.")?;
        if let Some(credentials) = self.load(db)? {
            self.describe(&credentials, false)?;
        }
        Ok(())
    }

    pub fn restore_conversations(app: &AppHandle) -> Result<(), String> {
        let state = app.state::<AppState>();
        let db = &state.database;
        let rows = db
            .chatgpt_conversations()
            .map_err(|_| "Could not read saved ChatGPT conversations.")?;
        if rows.is_empty() {
            return Ok(());
        }
        let host = db
            .ai_host_id()
            .map_err(|_| "Could not open saved ChatGPT conversations.")?;
        let mut conversations = Vec::new();
        let mut unreadable = false;
        for (id, protected) in rows {
            let conversation = secrets::unprotect(&host, &protected)
                .ok()
                .and_then(|plain| serde_json::from_slice::<SavedChatConversation>(&plain).ok())
                .filter(|conversation| conversation.id == id);
            match conversation {
                Some(conversation) => conversations.push(conversation),
                None => unreadable = true,
            }
        }
        state
            .hub
            .lock()
            .map_err(|_| "Could not restore the ChatGPT conversation list.")?
            .restore_chatgpt(conversations);
        if unreadable {
            return Err(
                "Some saved ChatGPT conversations could not be opened with this profile.".into(),
            );
        }
        Ok(())
    }

    fn save_chatgpt_reply(state: &AppState, id: u64, reply: &str) -> Result<(), String> {
        let conversation = state
            .hub
            .lock()
            .map_err(|_| "AI hub is unavailable.")?
            .completed_chatgpt_transcript(id, reply)
            .ok_or("ChatGPT conversation is unavailable.")?;
        let plain = Zeroizing::new(
            serde_json::to_vec(&conversation)
                .map_err(|_| "Could not encode the ChatGPT conversation.")?,
        );
        let db = &state.database;
        let host = db
            .ai_host_id()
            .map_err(|_| "Could not open ChatGPT conversation storage.")?;
        let protected = secrets::protect(&host, &plain)?;
        db.save_chatgpt_conversation(&conversation.id, &protected)
            .map_err(|_| "Could not write the saved ChatGPT conversation.".into())
    }
    fn describe(&self, credentials: &Credentials, verified: bool) -> Result<(), String> {
        let mut status = self
            .status
            .lock()
            .map_err(|_| "ChatGPT status unavailable.")?;
        status.account_label = Some(
            credentials
                .email
                .clone()
                .unwrap_or_else(|| credentials.subject.clone()),
        );
        status.registration = Some(credentials.client_id.clone());
        status.plan_granted = credentials.plan_allowed() && !credentials.access_token.is_empty();
        let signed_out = credentials.access_token.is_empty() && credentials.id_token.is_empty();
        status.state = if signed_out {
            ConnectionState::Disconnected
        } else if !status.plan_granted {
            ConnectionState::Limited
        } else if verified {
            ConnectionState::Verified
        } else {
            ConnectionState::Connected
        };
        status.detail=if signed_out{"Signed out locally; registration retained for reauthorization."}else if !status.plan_granted{"Identity verified, but ChatGPT plan use was not granted. Inference is disabled."}else if verified{"Stream completion verified using this ChatGPT registration. No API-key billing was used."}else{"ChatGPT identity and plan-use grant validated. Send a chat prompt or run the verification test."}.into();
        status.last_update = seconds();
        Ok(())
    }
    fn reserve(&self) -> Result<(), String> {
        self.busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| "A ChatGPT connection operation is already in progress.")?;
        self.cancel.store(false, Ordering::SeqCst);
        Ok(())
    }
    pub fn start(app: AppHandle, operation: Operation) -> Result<(), String> {
        let state = app.state::<AppState>();
        state.chatgpt.reserve()?;
        match operation {Operation::SignIn=>state.chatgpt.set(&app,ConnectionState::Authorizing,"Waiting for you: complete ChatGPT sign-in and review plan-use consent in your browser. This attempt expires after five minutes."),Operation::Verify(_)=>state.chatgpt.set(&app,ConnectionState::Verifying,"Assistant: checking this registration's model catalog before the fixed verification prompt."),Operation::Disconnect=>state.chatgpt.set(&app,ConnectionState::Stopping,"Ending the selected ChatGPT renewable session and clearing local tokens.")}
        std::thread::spawn(move || {
            let state = app.state::<AppState>();
            let service = &state.chatgpt;
            let result = match operation {
                Operation::SignIn => service.sign_in(&app, &state.database),
                Operation::Verify(model) => service.verify(&app, &state.database, model),
                Operation::Disconnect => service.disconnect(&app, &state.database),
            };
            if let Err(error) = result {
                service.set(
                    &app,
                    if service.cancel.load(Ordering::SeqCst) {
                        ConnectionState::Unknown
                    } else {
                        ConnectionState::Failed
                    },
                    error,
                );
            }
            service.busy.store(false, Ordering::SeqCst);
            service.publish(&app);
        });
        Ok(())
    }
    pub fn stop(&self, app: &AppHandle) {
        if self.busy.load(Ordering::SeqCst) {
            self.cancel.store(true, Ordering::SeqCst);
            let account_job = self.status.lock().is_ok_and(|status| {
                matches!(
                    status.state,
                    ConnectionState::Authorizing
                        | ConnectionState::Verifying
                        | ConnectionState::Stopping
                )
            });
            if account_job {
                self.set(app,ConnectionState::Stopping,"Stop requested. Waiting for the owned operation to return; network waits are bounded to 90 seconds. Remote cancellation is not yet confirmed.");
            }
        }
    }
    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }

    pub fn start_chat(
        app: &AppHandle,
        expected_draft: &str,
        selected_conversation: Option<&str>,
    ) -> Result<String, String> {
        let state = app.state::<AppState>();
        let service = &state.chatgpt;
        let (registration, pending) = {
            let mut hub = state.hub.lock().map_err(|_| "AI hub is unavailable.")?;
            let connection = service.snapshot()?;
            let registration = connection
                .registration
                .filter(|_| connection.plan_granted)
                .ok_or("Connect a ChatGPT account and allow ChatGPT plan use before sending.")?;
            service.reserve()?;
            match hub.start_chatgpt(expected_draft, selected_conversation, &registration) {
                Ok(pending) => (registration, pending),
                Err(error) => {
                    service.busy.store(false, Ordering::SeqCst);
                    return Err(error);
                }
            }
        };
        let PendingChat {
            request_id: id,
            conversation_id,
            input,
        } = pending;
        let worker_app = app.clone();
        let worker = std::thread::Builder::new().spawn(move || {
            let state = worker_app.state::<AppState>();
            let service = &state.chatgpt;
            let outcome = service
                .chat(&worker_app, id, &registration, &input)
                .map(|text| {
                    let saved_locally = Self::save_chatgpt_reply(&state, id, &text).is_ok();
                    CompletedReply {
                        text,
                        saved_locally,
                    }
                });
            if let Ok(mut hub) = state.hub.lock() {
                hub.finish_chatgpt(id, outcome, service.cancel.load(Ordering::SeqCst));
            }
            service.busy.store(false, Ordering::SeqCst);
            service.publish(&worker_app);
        });
        if worker.is_err() {
            if let Ok(mut hub) = state.hub.lock() {
                hub.finish_chatgpt(
                    id,
                    Err("Could not start the ChatGPT worker. The prompt was not sent.".into()),
                    false,
                );
            }
            service.busy.store(false, Ordering::SeqCst);
        }
        service.publish(app);
        Ok(conversation_id)
    }

    fn chat(
        &self,
        app: &AppHandle,
        id: u64,
        registration: &str,
        input: &[ChatMessage],
    ) -> Result<String, String> {
        let state = app.state::<AppState>();
        let credentials = self.refreshed(&state.database)?;
        if credentials.client_id != registration {
            return Err(
                "The connected ChatGPT account changed before the conversation could be sent."
                    .into(),
            );
        }
        let models = transport::models(&self.http, &credentials)?;
        if self.cancel.load(Ordering::SeqCst) {
            return Err("Stopped before sending the prompt.".into());
        }
        let mut last_publish = None::<Instant>;
        let result = transport::stream_reply(
            &self.http,
            &credentials,
            &models[0].slug,
            transport::Inference::Chat(input),
            &self.cancel,
            |result| {
                if let Ok(mut hub) = state.hub.lock() {
                    hub.update_chatgpt(id, &result.text, result.sequence);
                }
                let observed_at = Instant::now();
                if last_publish.is_none_or(|last| {
                    observed_at.duration_since(last) >= Duration::from_millis(75)
                }) {
                    last_publish = Some(observed_at);
                    let _ = app.emit("hub-changed", ());
                }
            },
        )
        .map_err(|error| error.detail)?;
        Ok(result.text)
    }
    fn sign_in(&self, app: &AppHandle, db: &HostDatabase) -> Result<(), String> {
        let host = db
            .ai_host_id()
            .map_err(|_| "Stable host identity is unavailable.")?;
        let saved = self.load(db)?;
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .map_err(|_| "Could not bind the loopback sign-in callback.")?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "Could not prepare the loopback sign-in callback.")?;
        let port = listener
            .local_addr()
            .map_err(|_| "Loopback callback address unavailable.")?
            .port();
        let mut attempt = AuthAttempt::new(
            port,
            saved.as_ref().map(|c| c.client_id.clone()),
            saved.as_ref().map(|c| c.subject.clone()),
        );
        open_sign_in_browser(&attempt, &host)?;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(300) {
            if self.cancel.load(Ordering::SeqCst) {
                return Err("Sign-in cancelled locally; callback listener closed. Browser consent, if already granted, must be reviewed in ChatGPT settings.".into());
            }
            match listener.accept() {
                Ok((mut socket, peer)) => {
                    if !peer.ip().is_loopback() {
                        continue;
                    }
                    socket
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .map_err(|_| "Could not bound callback read.")?;
                    let mut request = Zeroizing::new(Vec::new());
                    let mut chunk = [0u8; 1024];
                    loop {
                        let n = socket
                            .read(&mut chunk)
                            .map_err(|_| "Callback request timed out.")?;
                        if n == 0 {
                            break;
                        }
                        request.extend_from_slice(&chunk[..n]);
                        chunk.zeroize();
                        if request.windows(4).any(|v| v == b"\r\n\r\n") {
                            break;
                        }
                        if request.len() > 16_384 {
                            return Err("Callback request exceeded its size limit.".into());
                        }
                    }
                    let text = std::str::from_utf8(&request)
                        .map_err(|_| "Callback was not valid HTTP.")?;
                    let first = text.lines().next().unwrap_or_default();
                    let mut parts = first.split_whitespace();
                    let method = parts.next().unwrap_or_default();
                    let target = parts.next().unwrap_or_default();
                    let expected_host = format!("127.0.0.1:{port}");
                    let host_ok = text.lines().filter_map(|line| line.split_once(':')).any(
                        |(name, value)| {
                            name.eq_ignore_ascii_case("host") && value.trim() == expected_host
                        },
                    );
                    if method != "GET" || !target.starts_with("/auth/callback?") || !host_ok {
                        let _=socket.write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
                        continue;
                    }
                    let callback = attempt.callback(target);
                    let _=socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\nContent-Length: 61\r\n\r\nCallback received. Return to Overlay Forge for final status.\n");
                    drop(listener);
                    let (code, client) = callback?;
                    self.set(app,ConnectionState::Authorizing,"Assistant: validating OAuth callback, identity signature and granted scopes.");
                    let credentials = attempt.exchange(&self.http, &code, &client)?;
                    if self.cancel.load(Ordering::SeqCst) {
                        let revoked = self.revoke(&credentials).is_ok();
                        return Err(if revoked{"Sign-in cancelled; renewable session revocation confirmed."}else{"Sign-in cancelled; remote revocation was not confirmed. Review this app in ChatGPT settings."}.into());
                    }
                    self.save(db, &credentials)?;
                    self.describe(&credentials, false)?;
                    self.publish(app);
                    return Ok(());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100))
                }
                Err(_) => return Err("Loopback callback listener failed.".into()),
            }
        }
        Err("Sign-in timed out after five minutes. Start a fresh attempt.".into())
    }
    fn refreshed(&self, db: &HostDatabase) -> Result<Credentials, String> {
        let mut credentials = self
            .load(db)?
            .ok_or("Continue with ChatGPT before sending a prompt.")?;
        if credentials.access_token.is_empty() {
            return Err("Sign in again before sending a prompt.".into());
        }
        if credentials.expires_at <= seconds() + 60 {
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
                .map_err(|_| {
                    "ChatGPT token refresh could not connect. Existing credentials were preserved."
                })?;
            if !response.status().is_success() {
                let failure = transport::RemoteFailure::read(response);
                if failure.terminal_refresh() {
                    credentials.access_token.zeroize();
                    credentials.refresh_token.zeroize();
                    credentials.id_token.zeroize();
                    credentials.scope.clear();
                    credentials.expires_at = 0;
                    self.save(db, &credentials)?;
                    self.describe(&credentials, false)?;
                }
                return Err(failure.message());
            }
            let mut tokens = oauth::read_tokens(response)?;
            credentials.access_token.zeroize();
            credentials.access_token = std::mem::take(&mut tokens.access_token);
            if !tokens.refresh_token.is_empty() {
                credentials.refresh_token.zeroize();
                credentials.refresh_token = std::mem::take(&mut tokens.refresh_token);
            }
            if !tokens.scope.is_empty() {
                credentials.scope = std::mem::take(&mut tokens.scope);
            }
            credentials.expires_at = seconds()
                .checked_add(tokens.expires_in)
                .ok_or("ChatGPT expiry was invalid.")?;
            self.save(db, &credentials)?;
            if !credentials.plan_allowed() || credentials.access_token.is_empty() {
                self.describe(&credentials, false)?;
            }
        }
        Ok(credentials)
    }
    fn verify(
        &self,
        app: &AppHandle,
        db: &HostDatabase,
        selected: Option<String>,
    ) -> Result<(), String> {
        let credentials = self.refreshed(db)?;
        let models = transport::models(&self.http, &credentials)?;
        if self.cancel.load(Ordering::SeqCst) {
            return Err("Verification stopped before inference; no prompt was sent.".into());
        }
        let model = selected
            .as_ref()
            .filter(|slug| models.iter().any(|model| &model.slug == *slug))
            .cloned()
            .unwrap_or_else(|| models[0].slug.clone());
        if selected.is_some() && selected.as_ref() != Some(&model) {
            return Err("Selected model is not in this account's current catalog. Review the available models.".into());
        }
        if let Ok(mut status) = self.status.lock() {
            status.models = models;
            status.verification_text.clear();
            status.provider_sequence = None;
            status.correlation = None;
            status.verification.last_verified_at = None;
        }
        self.set(app,ConnectionState::Verifying,format!("Assistant: sending the fixed harmless verification prompt using {model}. Funding: this ChatGPT registration; API-key billing is disabled."));
        let result = transport::stream_reply(
            &self.http,
            &credentials,
            &model,
            transport::Inference::Verification,
            &self.cancel,
            |result| {
                if let Ok(mut status) = self.status.lock() {
                    let previous_sequence = status.provider_sequence;
                    status.verification_text = result.text.clone();
                    status.provider_sequence = result.sequence;
                    status.correlation = result.correlation.clone();
                    status.last_update = seconds();
                    // The stream parser validated provider sequence/correlation before this callback.
                    status.verification.observe_stream(
                        previous_sequence,
                        result.sequence,
                        crate::status_watch::now_ms(),
                    );
                }
                self.publish(app);
            },
        );
        match result {
            Ok(_) => {
                self.describe(&credentials, true)?;
                self.publish(app);
                Ok(())
            }
            Err(error) => {
                self.set(
                    app,
                    if error.confirmed_failure {
                        ConnectionState::Failed
                    } else {
                        ConnectionState::Unknown
                    },
                    error.detail,
                );
                Ok(())
            }
        }
    }
    fn revoke(&self, credentials: &Credentials) -> Result<(), String> {
        if credentials.refresh_token.is_empty() {
            return Err("No renewable session token was available to revoke.".into());
        }
        let discovery: serde_json::Value = self
            .http
            .get("https://auth.openai.com/.well-known/openid-configuration")
            .send()
            .map_err(|_| "Revocation discovery unavailable.")?
            .error_for_status()
            .map_err(|_| "Revocation discovery unavailable.")?
            .json()
            .map_err(|_| "Revocation discovery invalid.")?;
        let endpoint = discovery
            .get("revocation_endpoint")
            .and_then(serde_json::Value::as_str)
            .ok_or("Revocation endpoint unavailable.")?;
        let url = reqwest::Url::parse(endpoint).map_err(|_| "Revocation endpoint invalid.")?;
        if url.scheme() != "https"
            || url.host_str() != Some("auth.openai.com")
            || url.port_or_known_default() != Some(443)
        {
            return Err("Revocation endpoint was outside the trusted issuer.".into());
        }
        self.http
            .post(url)
            .form(&[
                ("token", credentials.refresh_token.as_str()),
                ("token_type_hint", "refresh_token"),
                ("client_id", credentials.client_id.as_str()),
            ])
            .send()
            .map_err(|_| "Remote revocation was not confirmed.")?
            .error_for_status()
            .map_err(|_| "Remote revocation was not confirmed.")?;
        Ok(())
    }
    fn disconnect(&self, app: &AppHandle, db: &HostDatabase) -> Result<(), String> {
        let Some(mut credentials) = self.load(db)? else {
            self.set(
                app,
                ConnectionState::Disconnected,
                "No account was connected.",
            );
            return Ok(());
        };
        let remote = self.revoke(&credentials).is_ok();
        credentials.access_token.zeroize();
        credentials.refresh_token.zeroize();
        credentials.id_token.zeroize();
        credentials.scope.clear();
        credentials.expires_at = 0;
        self.save(db, &credentials)?;
        self.describe(&credentials, false)?;
        self.set(app,ConnectionState::Disconnected,if remote{"Local tokens removed and renewable session revocation confirmed. Registration retained for later sign-in."}else{"Local tokens removed; remote revocation was NOT confirmed. Disconnect the app in ChatGPT settings if needed."});
        Ok(())
    }
}
pub enum Operation {
    SignIn,
    Verify(Option<String>),
    Disconnect,
}
#[cfg(windows)]
fn open_sign_in_browser(attempt: &AuthAttempt, host: &str) -> Result<(), String> {
    crate::browser_auth::open_system_browser(&attempt.authorization_url(host))
}

#[cfg(not(windows))]
fn open_sign_in_browser(_attempt: &AuthAttempt, _host: &str) -> Result<(), String> {
    Err("ChatGPT browser sign-in is currently available on Windows.".into())
}

#[tauri::command]
pub fn chatgpt_sign_in(app: AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    ChatgptService::start(app.clone(), Operation::SignIn)
}
#[tauri::command]
pub fn chatgpt_verify(app: AppHandle, model: Option<String>, session_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    ChatgptService::start(app.clone(), Operation::Verify(model))
}
#[tauri::command]
pub fn chatgpt_disconnect(app: AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    ChatgptService::start(app.clone(), Operation::Disconnect)
}
