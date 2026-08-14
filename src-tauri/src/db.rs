use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

pub struct HostDatabase {
    connection: Mutex<Connection>,
}

impl HostDatabase {
    pub fn new(path: PathBuf) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(r#"
            PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS obj_host_launcher_state (
                id INTEGER PRIMARY KEY CHECK(id=1), last_product_key TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT OR IGNORE INTO obj_host_launcher_state(id,last_product_key) VALUES(1,NULL);
            CREATE TABLE IF NOT EXISTS obj_host_window_state (
                window_key TEXT PRIMARY KEY, x INTEGER, y INTEGER, width INTEGER, height INTEGER,
                is_visible INTEGER NOT NULL DEFAULT 0, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
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
}
