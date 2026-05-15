use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppDatabase {
    connection: Mutex<Connection>,
    ready: bool,
}

impl AppDatabase {
    pub fn new(path: PathBuf) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS scratchpad (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            INSERT OR IGNORE INTO scratchpad (id, content) VALUES (1, '');
            ",
        )?;

        Ok(Self {
            connection: Mutex::new(connection),
            ready: true,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn get_scratchpad(&self) -> Result<String> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.query_row("SELECT content FROM scratchpad WHERE id = 1", [], |row| {
            row.get(0)
        })
    }

    pub fn save_scratchpad(&self, content: &str) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            INSERT INTO scratchpad (id, content, updated_at)
            VALUES (1, ?1, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![content],
        )?;

        Ok(())
    }
}
