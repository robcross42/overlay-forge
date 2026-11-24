use overlay_forge_google_identity::GoogleIdentity;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{path::Path, sync::Mutex};

pub const SESSION_SECONDS: u64 = 8 * 60 * 60;
const IDLE_SECONDS: u64 = 60 * 60;
pub struct AccessRepository {
    connection: Mutex<Connection>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSession {
    pub email: String,
    pub name: Option<String>,
    pub csrf_token: String,
    pub navigation_collapsed: bool,
}

pub fn normalize_email(email: &str) -> Result<String, &'static str> {
    let email = email.trim().to_ascii_lowercase();
    let parts: Vec<_> = email.split('@').collect();
    if email.len() > 320
        || !email.is_ascii()
        || email.chars().any(|c| c.is_whitespace() || c.is_control())
        || parts.len() != 2
        || parts[0].is_empty()
        || !parts[1].contains('.')
        || parts[1].starts_with('.')
        || parts[1].ends_with('.')
    {
        return Err("Enter a complete email address.");
    }
    Ok(email)
}
impl AccessRepository {
    pub fn new(path: &Path) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("
            PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS browser_access (
                email TEXT PRIMARY KEY, approved INTEGER NOT NULL DEFAULT 0 CHECK(approved IN (0,1)),
                google_subject TEXT, navigation_collapsed INTEGER NOT NULL DEFAULT 0 CHECK(navigation_collapsed IN (0,1))
            );
            CREATE TABLE IF NOT EXISTS browser_session (
                token_hash TEXT PRIMARY KEY, email TEXT NOT NULL REFERENCES browser_access(email),
                google_subject TEXT NOT NULL, display_name TEXT, csrf_token TEXT NOT NULL,
                expires_at INTEGER NOT NULL, last_seen_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS browser_session_email ON browser_session(email);
        ")?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
    fn connection(&self) -> rusqlite::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)
    }
    pub fn approve(&self, email: &str) -> rusqlite::Result<()> {
        self.connection()?.execute(
            "INSERT INTO browser_access(email,approved) VALUES(?1,1)
            ON CONFLICT(email) DO UPDATE SET approved=1",
            params![email],
        )?;
        Ok(())
    }
    pub fn revoke(&self, email: &str) -> rusqlite::Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE browser_access SET approved=0 WHERE email=?1",
            params![email],
        )?;
        transaction.execute("DELETE FROM browser_session WHERE email=?1", params![email])?;
        transaction.commit()
    }
    pub fn list(&self) -> rusqlite::Result<Vec<(String, bool, bool)>> {
        let connection = self.connection()?;
        let mut query = connection.prepare(
            "SELECT email, approved, google_subject IS NOT NULL FROM browser_access ORDER BY email",
        )?;
        let rows = query.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        rows.collect()
    }
    pub fn create_session(
        &self,
        identity: &GoogleIdentity,
        token_hash: &str,
        csrf: &str,
        now: u64,
    ) -> rusqlite::Result<bool> {
        let Some(email) = identity
            .email
            .as_deref()
            .and_then(|email| normalize_email(email).ok())
        else {
            return Ok(false);
        };
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let grant: Option<(bool, Option<String>)> = transaction
            .query_row(
                "SELECT approved,google_subject FROM browser_access WHERE email=?1",
                params![email],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if !matches!(grant, Some((true, ref subject)) if subject.as_deref().is_none_or(|value| value == identity.subject))
        {
            return Ok(false);
        }
        transaction.execute(
            "UPDATE browser_access SET google_subject=?2 WHERE email=?1 AND google_subject IS NULL",
            params![email, identity.subject],
        )?;
        transaction.execute(
            "DELETE FROM browser_session WHERE expires_at<=?1 OR last_seen_at<=?2",
            params![now, now.saturating_sub(IDLE_SECONDS)],
        )?;
        let count: i64 =
            transaction.query_row("SELECT COUNT(*) FROM browser_session", [], |row| row.get(0))?;
        if count >= 2048 {
            return Ok(false);
        }
        transaction.execute("INSERT INTO browser_session(token_hash,email,google_subject,display_name,csrf_token,expires_at,last_seen_at)
            VALUES(?1,?2,?3,?4,?5,?6,?7)", params![token_hash,email,identity.subject,identity.name,csrf,now+SESSION_SECONDS,now])?;
        transaction.commit()?;
        Ok(true)
    }
    pub fn session(&self, hash: &str, now: u64) -> rusqlite::Result<Option<BrowserSession>> {
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM browser_session WHERE expires_at<=?1 OR last_seen_at<=?2",
            params![now, now.saturating_sub(IDLE_SECONDS)],
        )?;
        let session = connection
            .query_row(
                "SELECT s.email,s.display_name,s.csrf_token,a.navigation_collapsed
            FROM browser_session s JOIN browser_access a ON a.email=s.email
            WHERE s.token_hash=?1 AND a.approved=1 AND a.google_subject=s.google_subject
            AND s.expires_at>?2 AND s.last_seen_at>?3",
                params![hash, now, now.saturating_sub(IDLE_SECONDS)],
                |row| {
                    Ok(BrowserSession {
                        email: row.get(0)?,
                        name: row.get(1)?,
                        csrf_token: row.get(2)?,
                        navigation_collapsed: row.get(3)?,
                    })
                },
            )
            .optional()?;
        if session.is_some() {
            connection.execute(
                "UPDATE browser_session SET last_seen_at=?2 WHERE token_hash=?1",
                params![hash, now],
            )?;
        }
        Ok(session)
    }
    pub fn logout(&self, hash: &str) -> rusqlite::Result<()> {
        self.connection()?.execute(
            "DELETE FROM browser_session WHERE token_hash=?1",
            params![hash],
        )?;
        Ok(())
    }
    pub fn save_navigation(&self, email: &str, collapsed: bool) -> rusqlite::Result<()> {
        self.connection()?.execute(
            "UPDATE browser_access SET navigation_collapsed=?2 WHERE email=?1 AND approved=1",
            params![email, collapsed],
        )?;
        Ok(())
    }
}
