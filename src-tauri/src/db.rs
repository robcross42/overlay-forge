use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize)]
pub struct TaskRecord {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub deadline: String,
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct NoteRecord {
    pub id: i64,
    pub title: String,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct CalendarEventRecord {
    pub id: i64,
    pub title: String,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub notes: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

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

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                deadline TEXT NOT NULL DEFAULT '',
                is_completed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS calendar_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                start_date TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_date TEXT NOT NULL,
                end_time TEXT NOT NULL,
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )?;
        Self::ensure_column(&connection, "tasks", "body", "TEXT NOT NULL DEFAULT ''")?;
        Self::ensure_column(&connection, "tasks", "deadline", "TEXT NOT NULL DEFAULT ''")?;

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

    pub fn list_tasks(&self) -> Result<Vec<TaskRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, title, body, deadline, is_completed, created_at, updated_at
            FROM tasks
            ORDER BY
                CASE WHEN deadline = '' THEN 1 ELSE 0 END,
                deadline ASC,
                updated_at DESC,
                id DESC
            ",
        )?;

        let tasks = statement
            .query_map([], |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    deadline: row.get(3)?,
                    is_completed: row.get::<_, i64>(4)? == 1,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tasks)
    }

    pub fn create_task(&self, title: &str, body: &str, deadline: &str) -> Result<TaskRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "INSERT INTO tasks (title, body, deadline) VALUES (?1, ?2, ?3)",
            params![title.trim(), body, deadline.trim()],
        )?;
        let id = connection.last_insert_rowid();
        Self::get_task_by_id(&connection, id)
    }

    pub fn update_task(
        &self,
        id: i64,
        title: Option<&str>,
        body: Option<&str>,
        deadline: Option<&str>,
        is_completed: Option<bool>,
    ) -> Result<TaskRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");

        if let Some(next_title) = title {
            connection.execute(
                "
                UPDATE tasks
                SET title = ?1,
                    body = COALESCE(?2, body),
                    deadline = COALESCE(?3, deadline),
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?4
                ",
                params![next_title.trim(), body, deadline.map(str::trim), id],
            )?;
        } else if body.is_some() || deadline.is_some() {
            connection.execute(
                "
                UPDATE tasks
                SET body = COALESCE(?1, body),
                    deadline = COALESCE(?2, deadline),
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?3
                ",
                params![body, deadline.map(str::trim), id],
            )?;
        }

        if let Some(next_state) = is_completed {
            connection.execute(
                "UPDATE tasks SET is_completed = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![if next_state { 1 } else { 0 }, id],
            )?;
        }

        Self::get_task_by_id(&connection, id)
    }

    pub fn delete_task(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, title, body, created_at, updated_at
            FROM notes
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let notes = statement
            .query_map([], |row| {
                Ok(NoteRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(notes)
    }

    pub fn create_note(&self, title: &str, body: &str) -> Result<NoteRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "INSERT INTO notes (title, body) VALUES (?1, ?2)",
            params![title.trim(), body],
        )?;
        let id = connection.last_insert_rowid();
        Self::get_note_by_id(&connection, id)
    }

    pub fn update_note(&self, id: i64, title: &str, body: &str) -> Result<NoteRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            UPDATE notes
            SET title = ?1, body = ?2, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?3
            ",
            params![title.trim(), body, id],
        )?;

        Self::get_note_by_id(&connection, id)
    }

    pub fn delete_note(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_calendar_events(&self) -> Result<Vec<CalendarEventRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, title, start_date, start_time, end_date, end_time, notes, created_at, updated_at
            FROM calendar_events
            ORDER BY start_date ASC, start_time ASC, id ASC
            ",
        )?;

        let events = statement
            .query_map([], |row| {
                Ok(CalendarEventRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    start_date: row.get(2)?,
                    start_time: row.get(3)?,
                    end_date: row.get(4)?,
                    end_time: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(events)
    }

    pub fn create_calendar_event(
        &self,
        title: &str,
        start_date: &str,
        start_time: &str,
        end_date: &str,
        end_time: &str,
        notes: &str,
    ) -> Result<CalendarEventRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            INSERT INTO calendar_events (title, start_date, start_time, end_date, end_time, notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                title.trim(),
                start_date.trim(),
                start_time.trim(),
                end_date.trim(),
                end_time.trim(),
                notes
            ],
        )?;
        let id = connection.last_insert_rowid();
        Self::get_calendar_event_by_id(&connection, id)
    }

    pub fn update_calendar_event(
        &self,
        id: i64,
        title: &str,
        start_date: &str,
        start_time: &str,
        end_date: &str,
        end_time: &str,
        notes: &str,
    ) -> Result<CalendarEventRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            UPDATE calendar_events
            SET title = ?1,
                start_date = ?2,
                start_time = ?3,
                end_date = ?4,
                end_time = ?5,
                notes = ?6,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?7
            ",
            params![
                title.trim(),
                start_date.trim(),
                start_time.trim(),
                end_date.trim(),
                end_time.trim(),
                notes,
                id
            ],
        )?;

        Self::get_calendar_event_by_id(&connection, id)
    }

    pub fn delete_calendar_event(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute("DELETE FROM calendar_events WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_task_by_id(connection: &Connection, id: i64) -> Result<TaskRecord> {
        connection.query_row(
            "
            SELECT id, title, body, deadline, is_completed, created_at, updated_at
            FROM tasks
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    deadline: row.get(3)?,
                    is_completed: row.get::<_, i64>(4)? == 1,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
    }

    fn ensure_column(
        connection: &Connection,
        table_name: &str,
        column_name: &str,
        column_definition: &str,
    ) -> Result<()> {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
        let exists = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?
            .iter()
            .any(|existing_column| existing_column == column_name);

        if !exists {
            connection.execute(
                &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
                [],
            )?;
        }

        Ok(())
    }

    fn get_note_by_id(connection: &Connection, id: i64) -> Result<NoteRecord> {
        connection.query_row(
            "
            SELECT id, title, body, created_at, updated_at
            FROM notes
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(NoteRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
    }

    fn get_calendar_event_by_id(connection: &Connection, id: i64) -> Result<CalendarEventRecord> {
        connection.query_row(
            "
            SELECT id, title, start_date, start_time, end_date, end_time, notes, created_at, updated_at
            FROM calendar_events
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(CalendarEventRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    start_date: row.get(2)?,
                    start_time: row.get(3)?,
                    end_date: row.get(4)?,
                    end_time: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
    }
}
