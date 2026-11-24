//! Google identifies the local session; only the pinned administrator can use AI.
use crate::{google_oauth::GoogleIdentity, AppState};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, MutexGuard,
    },
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    id: String,
    pub email: Option<String>,
    name: Option<String>,
    pub admin: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSnapshot {
    revision: u64,
    session_id: String,
    configured: bool,
    busy: bool,
    profile: Option<Profile>,
    detail: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupConfig {
    admin_email: String,
}

struct ProfileRepository {
    connection: Connection,
}
impl ProfileRepository {
    fn new(root: &Path) -> Result<Self, String> {
        let connection = Connection::open(root.join("overlay-forge-identities.sqlite3"))
            .map_err(|_| "Could not open the Google identity registry.")?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS identity_config (
                id INTEGER PRIMARY KEY CHECK(id=1), admin_email TEXT NOT NULL,
                admin_subject TEXT, admin_issuer TEXT
            );
            CREATE TABLE IF NOT EXISTS google_profile (
                id TEXT PRIMARY KEY, issuer TEXT NOT NULL, subject TEXT NOT NULL,
                email TEXT, display_name TEXT, UNIQUE(issuer,subject)
            );",
            )
            .map_err(|_| "Could not initialize the Google identity registry.")?;
        let configured: bool = connection
            .query_row("SELECT EXISTS(SELECT 1 FROM identity_config)", [], |row| {
                row.get(0)
            })
            .map_err(|_| "Could not read administrator configuration.")?;
        if !configured && root.join("google-auth.json").exists() {
            let bytes = std::fs::read(root.join("google-auth.json"))
                .map_err(|_| "Could not read local Google setup.")?;
            if bytes.len() > 4096 {
                return Err("Local Google setup is too large.".into());
            }
            let config: SetupConfig = serde_json::from_slice(&bytes)
                .map_err(|_| "Invalid local Google setup. Expected admin_email only.")?;
            let email = config.admin_email.trim().to_ascii_lowercase();
            // Bootstrap is restricted to Google-managed Gmail addresses. Later checks use issuer/sub.
            if !email.ends_with("@gmail.com")
                || email.len() > 254
                || email.split('@').count() != 2
                || email.starts_with('@')
                || email.chars().any(char::is_whitespace)
            {
                return Err("Administrator setup requires a valid Gmail address.".into());
            }
            connection
                .execute(
                    "INSERT INTO identity_config(id,admin_email) VALUES(1,?1)",
                    [email],
                )
                .map_err(|_| "Could not save administrator setup.")?;
        }
        Ok(Self { connection })
    }

    fn configured(&self) -> Result<bool, String> {
        self.connection
            .query_row("SELECT EXISTS(SELECT 1 FROM identity_config)", [], |row| {
                row.get(0)
            })
            .map_err(|_| "Could not read Google setup.".into())
    }

    fn register(&mut self, identity: GoogleIdentity) -> Result<Profile, String> {
        if identity.expires_at <= crate::status_watch::now_ms() / 1000 {
            return Err(
                "Google sign-in expired before it could be committed. Sign in again.".into(),
            );
        }
        let tx = self
            .connection
            .transaction()
            .map_err(|_| "Could not open the identity transaction.")?;
        let (admin_email, admin_issuer, admin_subject): (String, Option<String>, Option<String>) =
            tx.query_row(
                "SELECT admin_email,admin_issuer,admin_subject FROM identity_config WHERE id=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| "Administrator setup is required before sign-in.")?;
        let admin = match (admin_issuer, admin_subject) {
            (Some(issuer), Some(subject)) => {
                issuer == identity.issuer && subject == identity.subject
            }
            (None, None)
                if identity
                    .email
                    .as_deref()
                    .is_some_and(|email| email.eq_ignore_ascii_case(&admin_email)) =>
            {
                tx.execute(
                    "UPDATE identity_config SET admin_issuer=?1,admin_subject=?2 WHERE id=1",
                    params![identity.issuer, identity.subject],
                )
                .map_err(|_| "Could not bind the administrator identity.")?;
                true
            }
            (None, None) => false,
            _ => {
                return Err("Administrator identity binding is inconsistent. Access denied.".into())
            }
        };
        let id: Option<String> = tx
            .query_row(
                "SELECT id FROM google_profile WHERE issuer=?1 AND subject=?2",
                params![identity.issuer, identity.subject],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Could not resolve the Google profile.")?;
        let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        tx.execute("INSERT INTO google_profile(id,issuer,subject,email,display_name) VALUES(?1,?2,?3,?4,?5)
            ON CONFLICT(issuer,subject) DO UPDATE SET email=excluded.email,display_name=excluded.display_name",
            params![id,identity.issuer,identity.subject,identity.email,identity.name])
            .map_err(|_| "Could not save the Google profile.")?;
        tx.commit()
            .map_err(|_| "Could not commit the Google profile.")?;
        Ok(Profile {
            id,
            email: identity.email,
            name: identity.name,
            admin,
        })
    }
}

pub struct ProfileService {
    repository: ProfileRepository,
    active: Option<Profile>,
    session_id: String,
    revision: u64,
    busy: bool,
    cancel: Arc<AtomicBool>,
    detail: String,
}
impl ProfileService {
    pub fn new(root: &Path) -> Result<Self, String> {
        Ok(Self {
            repository: ProfileRepository::new(root)?,
            active: None,
            session_id: uuid::Uuid::new_v4().to_string(),
            revision: 0,
            busy: false,
            cancel: Arc::new(AtomicBool::new(false)),
            detail: "Sign in with Google to open Overlay Forge. ChatGPT is administrator-only."
                .into(),
        })
    }
    fn check_session(&self, session: &str) -> Result<(), String> {
        if session != self.session_id {
            return Err("The Google session changed. Refresh before trying again.".into());
        }
        Ok(())
    }
    pub fn require_admin<'a>(
        state: &'a AppState,
        session: &str,
    ) -> Result<MutexGuard<'a, Self>, String> {
        let profiles = state
            .profiles
            .lock()
            .map_err(|_| "Google session is unavailable.")?;
        profiles.check_session(session)?;
        if profiles.busy
            || !profiles
                .active
                .as_ref()
                .is_some_and(|profile| profile.admin)
        {
            return Err("This action requires the signed-in Overlay Forge administrator.".into());
        }
        Ok(profiles)
    }
    pub fn require_signed_in<'a>(
        state: &'a AppState,
        session: Option<&str>,
    ) -> Result<MutexGuard<'a, Self>, String> {
        let profiles = state
            .profiles
            .lock()
            .map_err(|_| "Google session is unavailable.")?;
        if let Some(session) = session {
            profiles.check_session(session)?;
        }
        if profiles.busy || profiles.active.is_none() {
            return Err("Sign in with Google before opening a product.".into());
        }
        Ok(profiles)
    }
    pub fn is_busy(&self) -> bool {
        self.busy
    }
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
    fn snapshot(&self) -> Result<ProfileSnapshot, String> {
        Ok(ProfileSnapshot {
            revision: self.revision,
            session_id: self.session_id.clone(),
            configured: self.repository.configured()?,
            busy: self.busy,
            profile: self.active.clone(),
            detail: self.detail.clone(),
        })
    }
    fn ready_to_switch(state: &AppState) -> Result<(), String> {
        if state.chatgpt.is_busy() {
            return Err("Stop or finish ChatGPT work before changing Google profiles.".into());
        }
        state
            .hub
            .lock()
            .map_err(|_| "AI hub is unavailable.")?
            .can_switch_profile()
    }
    fn publish(app: &AppHandle) {
        let _ = app.emit("identity-changed", ());
        let _ = app.emit("hub-changed", ());
    }
    fn finish_sign_in(app: &AppHandle, result: Result<GoogleIdentity, String>) {
        let state = app.state::<AppState>();
        let Ok(mut profiles) = state.profiles.lock() else {
            return;
        };
        let outcome = (|| -> Result<(), String> {
            if profiles.cancel.load(Ordering::SeqCst) {
                return Err("Google sign-in cancelled. The previous profile was retained.".into());
            }
            Self::ready_to_switch(&state)?;
            let identity = result?;
            let profile = profiles.repository.register(identity)?;
            *state.hub.lock().map_err(|_| "AI hub is unavailable.")? =
                crate::ai_hub::AiHub::default();
            profiles.session_id = uuid::Uuid::new_v4().to_string();
            profiles.active = Some(profile.clone());
            if profile.admin {
                if let Err(error) = crate::chatgpt::ChatgptService::restore_conversations(app) {
                    state
                        .hub
                        .lock()
                        .map_err(|_| "AI hub is unavailable.")?
                        .startup_message(error);
                }
            }
            profiles.detail=if profile.admin { "Signed in as administrator. Your existing ChatGPT connection and saved conversations are available." }
                else { "Signed in as a family member. Products are available; ChatGPT is administrator-only." }.into();
            Ok(())
        })();
        let signed_in = outcome.is_ok();
        if let Err(error) = outcome {
            profiles.detail = error;
        }
        profiles.busy = false;
        profiles.revision += 1;
        if signed_in {
            if let Err(error) = crate::commands::launch_requested_or_last(state.inner()) {
                profiles.detail = format!("Signed in. Product launch failed: {error}");
                if let Ok(mut hub) = state.hub.lock() {
                    hub.startup_message(error);
                }
            }
        }
        drop(profiles);
        Self::publish(app);
    }
}

#[tauri::command]
pub fn identity_snapshot(state: tauri::State<'_, AppState>) -> Result<ProfileSnapshot, String> {
    state
        .profiles
        .lock()
        .map_err(|_| "Google session is unavailable.")?
        .snapshot()
}
#[tauri::command]
pub fn identity_sign_in(app: AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut profiles = state
        .profiles
        .lock()
        .map_err(|_| "Google session is unavailable.")?;
    profiles.check_session(&session_id)?;
    if profiles.busy {
        return Err("Google sign-in is already running.".into());
    }
    if !profiles.repository.configured()? {
        return Err("Configure the local administrator email before Google sign-in.".into());
    }
    ProfileService::ready_to_switch(&state)?;
    profiles.busy = true;
    profiles.cancel.store(false, Ordering::SeqCst);
    profiles.detail =
        "Complete Google sign-in in your system browser. This attempt expires after five minutes."
            .into();
    profiles.revision += 1;
    let cancel = profiles.cancel.clone();
    drop(profiles);
    ProfileService::publish(&app);
    let worker_app = app.clone();
    if std::thread::Builder::new()
        .name("google-sign-in".into())
        .spawn(move || {
            let result = crate::google_oauth::sign_in(&cancel);
            ProfileService::finish_sign_in(&worker_app, result);
        })
        .is_err()
    {
        ProfileService::finish_sign_in(
            &app,
            Err("Could not start Google sign-in. Try again.".into()),
        );
    }
    Ok(())
}
#[tauri::command]
pub fn identity_cancel(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let profiles = state
        .profiles
        .lock()
        .map_err(|_| "Google session is unavailable.")?;
    profiles.check_session(&session_id)?;
    profiles.cancel();
    Ok(())
}
#[tauri::command]
pub fn identity_sign_out(app: AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut profiles = state
        .profiles
        .lock()
        .map_err(|_| "Google session is unavailable.")?;
    profiles.check_session(&session_id)?;
    if profiles.busy {
        return Err("Cancel and finish Google sign-in before signing out.".into());
    }
    ProfileService::ready_to_switch(&state)?;
    *state.hub.lock().map_err(|_| "AI hub is unavailable.")? = crate::ai_hub::AiHub::default();
    profiles.active = None;
    profiles.session_id = uuid::Uuid::new_v4().to_string();
    profiles.revision += 1;
    profiles.detail="Signed out of Overlay Forge. Saved data and protected ChatGPT credentials remain on this device. This does not sign out of Google in your browser.".into();
    drop(profiles);
    ProfileService::publish(&app);
    Ok(())
}
