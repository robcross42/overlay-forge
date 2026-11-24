use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

pub struct HostDatabase {
    connection: Mutex<Connection>,
}

impl HostDatabase {
    pub fn new(path: PathBuf) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }
    fn initialize(connection: Connection) -> rusqlite::Result<Self> {
        connection.execute_batch(r#"
            PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS obj_host_ui_preferences (
                id INTEGER PRIMARY KEY CHECK(id=1),
                navigation_collapsed INTEGER NOT NULL DEFAULT 0 CHECK(navigation_collapsed IN (0,1))
            );
            INSERT OR IGNORE INTO obj_host_ui_preferences(id) VALUES(1);
            CREATE TABLE IF NOT EXISTS obj_host_launcher_state (
                id INTEGER PRIMARY KEY CHECK(id=1), last_product_key TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT OR IGNORE INTO obj_host_launcher_state(id,last_product_key) VALUES(1,NULL);
            CREATE TABLE IF NOT EXISTS obj_host_window_state (
                window_key TEXT PRIMARY KEY, x INTEGER, y INTEGER, width INTEGER, height INTEGER,
                is_visible INTEGER NOT NULL DEFAULT 0, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS obj_host_ai_connection (
                id INTEGER PRIMARY KEY CHECK(id=1), host_id TEXT NOT NULL,
                protected_credentials BLOB, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS obj_host_chatgpt_conversation (
                conversation_id TEXT PRIMARY KEY NOT NULL,
                protected_transcript BLOB NOT NULL
            );
            CREATE TABLE IF NOT EXISTS obj_host_google_client (
                id INTEGER PRIMARY KEY CHECK(id=1), protected_config BLOB NOT NULL
            );
        "#)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> rusqlite::Result<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)
    }
    pub fn navigation_collapsed(&self) -> rusqlite::Result<bool> {
        self.connection()?.query_row(
            "SELECT navigation_collapsed FROM obj_host_ui_preferences WHERE id=1",
            [],
            |row| row.get(0),
        )
    }
    pub fn save_navigation_collapsed(&self, collapsed: bool) -> rusqlite::Result<()> {
        self.connection()?.execute(
            "UPDATE obj_host_ui_preferences SET navigation_collapsed=?1 WHERE id=1",
            params![collapsed],
        )?;
        Ok(())
    }
    pub fn google_client_config(&self) -> rusqlite::Result<Option<Vec<u8>>> {
        self.connection()?.query_row("SELECT protected_config FROM obj_host_google_client WHERE id=1", [], |row| row.get(0)).optional()
    }
    pub fn save_google_client_config(&self, protected: &[u8]) -> rusqlite::Result<()> {
        self.connection()?.execute("INSERT INTO obj_host_google_client(id,protected_config) VALUES(1,?1)
            ON CONFLICT(id) DO UPDATE SET protected_config=excluded.protected_config", params![protected])?;
        Ok(())
    }
    #[cfg(test)]
    pub fn last_product(&self) -> rusqlite::Result<Option<String>> {
        self.connection()?
            .query_row(
                "SELECT last_product_key FROM obj_host_launcher_state WHERE id=1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map(|v| v.flatten())
    }
    pub fn remember_product(&self, key: &str) -> rusqlite::Result<()> {
        self.connection()?.execute("UPDATE obj_host_launcher_state SET last_product_key=?1,updated_at=CURRENT_TIMESTAMP WHERE id=1",params![key])?;
        Ok(())
    }
    pub fn ai_host_id(&self) -> rusqlite::Result<String> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT OR IGNORE INTO obj_host_ai_connection(id,host_id) VALUES(1,?1)",
            params![format!("urn:uuid:{}", uuid::Uuid::new_v4())],
        )?;
        connection.query_row(
            "SELECT host_id FROM obj_host_ai_connection WHERE id=1",
            [],
            |row| row.get(0),
        )
    }
    pub fn ai_credentials(&self) -> rusqlite::Result<Option<Vec<u8>>> {
        self.connection()?
            .query_row(
                "SELECT protected_credentials FROM obj_host_ai_connection WHERE id=1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map(|value| value.flatten())
    }
    pub fn save_ai_credentials(&self, encrypted: Option<&[u8]>) -> rusqlite::Result<()> {
        self.connection()?.execute("UPDATE obj_host_ai_connection SET protected_credentials=?1,updated_at=CURRENT_TIMESTAMP WHERE id=1",params![encrypted])?;
        Ok(())
    }

    pub fn chatgpt_conversations(&self) -> rusqlite::Result<Vec<(String, Vec<u8>)>> {
        let connection = self.connection()?;
        let mut query = connection.prepare(
            "SELECT conversation_id, protected_transcript FROM obj_host_chatgpt_conversation",
        )?;
        let rows = query.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    pub fn save_chatgpt_conversation(&self, id: &str, protected: &[u8]) -> rusqlite::Result<()> {
        self.connection()?.execute(
            "INSERT INTO obj_host_chatgpt_conversation(conversation_id, protected_transcript)
             VALUES(?1, ?2) ON CONFLICT(conversation_id) DO UPDATE
             SET protected_transcript=excluded.protected_transcript",
            params![id, protected],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ai_migration_preserves_launcher_and_is_idempotent() {
        let database = HostDatabase::initialize(Connection::open_in_memory().unwrap()).unwrap();
        database.remember_product("journal").unwrap();
        let host = database.ai_host_id().unwrap();
        database
            .save_ai_credentials(Some(b"synthetic-encrypted-blob"))
            .unwrap();
        let connection = database.connection.into_inner().unwrap();
        let database = HostDatabase::initialize(connection).unwrap();
        assert_eq!(database.last_product().unwrap().as_deref(), Some("journal"));
        assert_eq!(database.ai_host_id().unwrap(), host);
        assert_eq!(
            database.ai_credentials().unwrap().unwrap(),
            b"synthetic-encrypted-blob"
        );
        database.save_ai_credentials(None).unwrap();
        assert!(database.ai_credentials().unwrap().is_none());
        assert_eq!(database.ai_host_id().unwrap(), host);
    }
}
