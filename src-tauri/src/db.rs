use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MARKDOWN_CONTEXT_PER_FILE_LIMIT: usize = 200_000;
const MARKDOWN_CONTEXT_TOTAL_LIMIT: usize = 650_000;

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

#[derive(Serialize)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PlanningMessageRecord {
    pub id: i64,
    #[serde(rename = "conversationId")]
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Serialize)]
pub struct PlanningConversationRecord {
    pub id: i64,
    #[serde(rename = "projectId")]
    pub project_id: i64,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct PlanningConversationContextRecord {
    pub id: i64,
    #[serde(rename = "conversationId")]
    pub conversation_id: i64,
    #[serde(rename = "contextType")]
    pub context_type: String,
    #[serde(rename = "sourceId")]
    pub source_id: Option<i64>,
    pub label: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Serialize)]
pub struct ProjectGitHubRepositoryRecord {
    pub id: i64,
    #[serde(rename = "projectId")]
    pub project_id: i64,
    #[serde(rename = "repositoryFullName")]
    pub repository_full_name: String,
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,
    #[serde(rename = "defaultBranch")]
    pub default_branch: String,
    pub visibility: String,
    #[serde(rename = "lastFetchedAt")]
    pub last_fetched_at: String,
    #[serde(rename = "lastFetchStatus")]
    pub last_fetch_status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct ProjectMarkdownContextRecord {
    pub id: i64,
    #[serde(rename = "projectId")]
    pub project_id: i64,
    #[serde(rename = "rootPath")]
    pub root_path: String,
    #[serde(rename = "readmePath")]
    pub readme_path: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct ProjectMarkdownContextFile {
    #[serde(rename = "relativePath")]
    pub relative_path: String,
    pub included: bool,
    pub content: String,
    pub warning: String,
}

#[derive(Clone, Serialize)]
pub struct ProjectMarkdownContextPayload {
    pub files: Vec<ProjectMarkdownContextFile>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct YouTubeReferenceRecord {
    pub id: i64,
    pub title: String,
    pub url: String,
    #[serde(rename = "videoId")]
    pub video_id: String,
    #[serde(rename = "channelName")]
    pub channel_name: String,
    pub notes: String,
    pub tags: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameRecord {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub summary: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameCatalogObjectRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    pub name: String,
    #[serde(rename = "objectType")]
    pub object_type: String,
    pub category: String,
    #[serde(rename = "categoryIcon")]
    pub category_icon: String,
    #[serde(rename = "categoryIconPath")]
    pub category_icon_path: String,
    pub description: String,
    pub notes: String,
    pub tags: String,
    #[serde(rename = "thumbnailPath")]
    pub thumbnail_path: String,
    #[serde(rename = "sourceScreenshotPath")]
    pub source_screenshot_path: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct GameScreenshotCaptureRequestRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    pub title: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "requestPath")]
    pub request_path: String,
    #[serde(rename = "captureStatus")]
    pub capture_status: String,
    #[serde(rename = "capturedAt")]
    pub captured_at: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct GameChatConversationRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameChatMessageRecord {
    pub id: i64,
    #[serde(rename = "conversationId")]
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Serialize)]
pub struct PromptPreviewContextItem {
    pub id: i64,
    #[serde(rename = "contextType")]
    pub context_type: String,
    pub label: String,
    pub included: bool,
    pub content: String,
    pub warning: String,
}

#[derive(Clone)]
pub struct PlanningContextPayload {
    pub content: String,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct PlanningPromptPreviewRecord {
    #[serde(rename = "projectLabel")]
    pub project_label: String,
    #[serde(rename = "projectStatus")]
    pub project_status: String,
    #[serde(rename = "projectDescription")]
    pub project_description: String,
    #[serde(rename = "conversationLabel")]
    pub conversation_label: String,
    #[serde(rename = "messageCount")]
    pub message_count: i64,
    #[serde(rename = "draftMessage")]
    pub draft_message: String,
    #[serde(rename = "projectMarkdownContextItems")]
    pub project_markdown_context_items: Vec<ProjectMarkdownContextFile>,
    #[serde(rename = "attachedContextItems")]
    pub attached_context_items: Vec<PromptPreviewContextItem>,
    #[serde(rename = "assembledPrompt")]
    pub assembled_prompt: String,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct BridgeFileDraftRecord {
    pub id: i64,
    #[serde(rename = "projectId")]
    pub project_id: i64,
    #[serde(rename = "conversationId")]
    pub conversation_id: i64,
    pub title: String,
    pub content: String,
    pub status: String,
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

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

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

            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'ACTIVE',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS planning_conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT 'Planning conversation',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS planning_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS planning_conversation_context (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                context_type TEXT NOT NULL,
                source_id INTEGER,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS project_github_repositories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL UNIQUE,
                repository_full_name TEXT NOT NULL,
                repository_url TEXT NOT NULL DEFAULT '',
                default_branch TEXT NOT NULL DEFAULT '',
                visibility TEXT NOT NULL DEFAULT '',
                last_fetched_at TEXT NOT NULL DEFAULT '',
                last_fetch_status TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS project_markdown_context (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL UNIQUE,
                root_path TEXT NOT NULL,
                readme_path TEXT NOT NULL DEFAULT 'README.md',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS youtube_references (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                video_id TEXT NOT NULL DEFAULT '',
                channel_name TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS bridge_file_drafts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                conversation_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                slug TEXT NOT NULL UNIQUE COLLATE NOCASE,
                summary TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS game_catalog_objects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                object_type TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS game_catalog_references (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                object_id INTEGER,
                title TEXT NOT NULL,
                reference_type TEXT NOT NULL DEFAULT '',
                url TEXT NOT NULL DEFAULT '',
                local_path TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS game_catalog_screenshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                object_id INTEGER,
                title TEXT NOT NULL DEFAULT '',
                file_path TEXT NOT NULL,
                request_id TEXT NOT NULL DEFAULT '',
                request_path TEXT NOT NULL DEFAULT '',
                capture_status TEXT NOT NULL DEFAULT 'captured',
                captured_at TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS game_chat_conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT 'Game chat',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS game_chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_game_catalog_objects_game_id
                ON game_catalog_objects (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_objects_game_name
                ON game_catalog_objects (game_id, name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_references_game_id
                ON game_catalog_references (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_references_object_id
                ON game_catalog_references (object_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_screenshots_game_id
                ON game_catalog_screenshots (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_screenshots_object_id
                ON game_catalog_screenshots (object_id);
            CREATE INDEX IF NOT EXISTS idx_game_chat_conversations_game_id
                ON game_chat_conversations (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_chat_messages_conversation_id
                ON game_chat_messages (conversation_id);

            INSERT OR IGNORE INTO games (name, slug, summary)
            VALUES (
                'GearBlocks',
                'gearblocks',
                'Game-specific workspace section for GearBlocks planning, object cataloging, references, and screenshots.'
            );
            ",
        )?;
        Self::ensure_column(&connection, "tasks", "body", "TEXT NOT NULL DEFAULT ''")?;
        Self::ensure_column(&connection, "tasks", "deadline", "TEXT NOT NULL DEFAULT ''")?;
        Self::ensure_column(
            &connection,
            "game_catalog_screenshots",
            "request_id",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "game_catalog_screenshots",
            "request_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "game_catalog_screenshots",
            "capture_status",
            "TEXT NOT NULL DEFAULT 'captured'",
        )?;
        Self::ensure_column(
            &connection,
            "game_catalog_objects",
            "category_icon",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "game_catalog_objects",
            "category_icon_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "game_catalog_objects",
            "thumbnail_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "game_catalog_objects",
            "source_screenshot_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        connection.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_game_catalog_screenshots_request_id
                ON game_catalog_screenshots (request_id)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_catalog_objects_game_name_unique
                ON game_catalog_objects (game_id, name COLLATE NOCASE)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_catalog_objects_game_name_exact_unique
                ON game_catalog_objects (game_id, name)
            ",
            [],
        )?;

        Ok(Self {
            connection: Mutex::new(connection),
            ready: true,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![key.trim()],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn save_app_setting(&self, key: &str, value: &str) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            INSERT INTO app_settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![key.trim(), value],
        )?;
        Ok(())
    }

    pub fn delete_app_setting(&self, key: &str) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "DELETE FROM app_settings WHERE key = ?1",
            params![key.trim()],
        )?;
        Ok(())
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

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, name, description, status, created_at, updated_at
            FROM projects
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let projects = statement
            .query_map([], |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(projects)
    }

    pub fn create_project(
        &self,
        name: &str,
        description: &str,
        status: &str,
    ) -> Result<ProjectRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "INSERT INTO projects (name, description, status) VALUES (?1, ?2, ?3)",
            params![name.trim(), description, status.trim()],
        )?;
        let id = connection.last_insert_rowid();
        Self::get_project_by_id(&connection, id)
    }

    pub fn update_project(
        &self,
        id: i64,
        name: &str,
        description: &str,
        status: &str,
    ) -> Result<ProjectRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            UPDATE projects
            SET name = ?1,
                description = ?2,
                status = ?3,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?4
            ",
            params![name.trim(), description, status.trim(), id],
        )?;

        Self::get_project_by_id(&connection, id)
    }

    pub fn delete_project(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "DELETE FROM bridge_file_drafts WHERE project_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM project_markdown_context WHERE project_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM project_github_repositories WHERE project_id = ?1",
            params![id],
        )?;
        connection.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_project(&self, id: i64) -> Result<ProjectRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, id)
    }

    pub fn get_project_github_repository(
        &self,
        project_id: i64,
    ) -> Result<Option<ProjectGitHubRepositoryRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        Self::get_project_github_repository_by_project_id(&connection, project_id).optional()
    }

    pub fn save_project_github_repository(
        &self,
        project_id: i64,
        repository_full_name: &str,
    ) -> Result<ProjectGitHubRepositoryRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        connection.execute(
            "
            INSERT INTO project_github_repositories (
                project_id,
                repository_full_name,
                repository_url,
                default_branch,
                visibility,
                last_fetched_at,
                last_fetch_status,
                updated_at
            )
            VALUES (?1, ?2, '', '', '', '', 'Repository link saved', CURRENT_TIMESTAMP)
            ON CONFLICT(project_id) DO UPDATE SET
                repository_full_name = excluded.repository_full_name,
                repository_url = '',
                default_branch = '',
                visibility = '',
                last_fetched_at = '',
                last_fetch_status = 'Repository link saved',
                updated_at = CURRENT_TIMESTAMP
            ",
            params![project_id, repository_full_name.trim()],
        )?;

        Self::get_project_github_repository_by_project_id(&connection, project_id)
    }

    pub fn delete_project_github_repository(&self, project_id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        connection.execute(
            "DELETE FROM project_github_repositories WHERE project_id = ?1",
            params![project_id],
        )?;
        Ok(())
    }

    pub fn update_project_github_metadata(
        &self,
        project_id: i64,
        repository_full_name: &str,
        repository_url: &str,
        default_branch: &str,
        visibility: &str,
        last_fetch_status: &str,
    ) -> Result<ProjectGitHubRepositoryRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            UPDATE project_github_repositories
            SET repository_full_name = ?1,
                repository_url = ?2,
                default_branch = ?3,
                visibility = ?4,
                last_fetched_at = CURRENT_TIMESTAMP,
                last_fetch_status = ?5,
                updated_at = CURRENT_TIMESTAMP
            WHERE project_id = ?6
            ",
            params![
                repository_full_name.trim(),
                repository_url.trim(),
                default_branch.trim(),
                visibility.trim(),
                last_fetch_status.trim(),
                project_id
            ],
        )?;

        Self::get_project_github_repository_by_project_id(&connection, project_id)
    }

    pub fn update_project_github_fetch_status(
        &self,
        project_id: i64,
        last_fetch_status: &str,
    ) -> Result<ProjectGitHubRepositoryRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            UPDATE project_github_repositories
            SET last_fetched_at = CURRENT_TIMESTAMP,
                last_fetch_status = ?1,
                updated_at = CURRENT_TIMESTAMP
            WHERE project_id = ?2
            ",
            params![last_fetch_status.trim(), project_id],
        )?;

        Self::get_project_github_repository_by_project_id(&connection, project_id)
    }

    pub fn get_project_markdown_context(
        &self,
        project_id: i64,
    ) -> Result<Option<ProjectMarkdownContextRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        Self::get_project_markdown_context_by_project_id(&connection, project_id).optional()
    }

    pub fn save_project_markdown_context(
        &self,
        project_id: i64,
        root_path: &str,
        readme_path: &str,
    ) -> Result<ProjectMarkdownContextRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        let clean_readme_path = if readme_path.trim().is_empty() {
            "README.md"
        } else {
            readme_path.trim()
        };

        connection.execute(
            "
            INSERT INTO project_markdown_context (
                project_id,
                root_path,
                readme_path,
                updated_at
            )
            VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
            ON CONFLICT(project_id) DO UPDATE SET
                root_path = excluded.root_path,
                readme_path = excluded.readme_path,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![project_id, root_path.trim(), clean_readme_path],
        )?;

        Self::get_project_markdown_context_by_project_id(&connection, project_id)
    }

    pub fn delete_project_markdown_context(&self, project_id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        connection.execute(
            "DELETE FROM project_markdown_context WHERE project_id = ?1",
            params![project_id],
        )?;
        Ok(())
    }

    pub fn load_project_markdown_context(
        &self,
        project_id: i64,
    ) -> Result<ProjectMarkdownContextPayload> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        Self::load_project_markdown_context_for_project(&connection, project_id)
    }

    pub fn list_planning_conversations(
        &self,
        project_id: Option<i64>,
    ) -> Result<Vec<PlanningConversationRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");

        if let Some(project_id) = project_id {
            let mut statement = connection.prepare(
                "
                SELECT id, project_id, title, created_at, updated_at
                FROM planning_conversations
                WHERE project_id = ?1
                ORDER BY updated_at DESC, id DESC
                ",
            )?;

            return statement
                .query_map(params![project_id], planning_conversation_from_row)?
                .collect::<Result<Vec<_>>>();
        }

        let mut statement = connection.prepare(
            "
            SELECT id, project_id, title, created_at, updated_at
            FROM planning_conversations
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let conversations = statement
            .query_map([], planning_conversation_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(conversations)
    }

    pub fn create_planning_conversation(
        &self,
        project_id: i64,
        title: &str,
    ) -> Result<PlanningConversationRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        let clean_title = if title.trim().is_empty() {
            "Planning conversation"
        } else {
            title.trim()
        };

        connection.execute(
            "
            INSERT INTO planning_conversations (project_id, title)
            VALUES (?1, ?2)
            ",
            params![project_id, clean_title],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_planning_conversation_by_id(&connection, id)
    }

    pub fn get_planning_conversation(&self, id: i64) -> Result<PlanningConversationRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_planning_conversation_by_id(&connection, id)
    }

    pub fn list_planning_messages(
        &self,
        conversation_id: i64,
    ) -> Result<Vec<PlanningMessageRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        Self::list_planning_messages_for_connection(&connection, conversation_id)
    }

    pub fn recent_planning_messages(
        &self,
        conversation_id: i64,
        limit: i64,
    ) -> Result<Vec<PlanningMessageRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM (
                SELECT id, conversation_id, role, content, created_at
                FROM planning_messages
                WHERE conversation_id = ?1
                ORDER BY id DESC
                LIMIT ?2
            )
            ORDER BY id ASC
            ",
        )?;

        let messages = statement
            .query_map(params![conversation_id, limit], planning_message_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    pub fn create_planning_message(
        &self,
        conversation_id: i64,
        role: &str,
        content: &str,
    ) -> Result<PlanningMessageRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        connection.execute(
            "
            INSERT INTO planning_messages (conversation_id, role, content)
            VALUES (?1, ?2, ?3)
            ",
            params![conversation_id, role, content],
        )?;
        connection.execute(
            "
            UPDATE planning_conversations
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            ",
            params![conversation_id],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_planning_message_by_id(&connection, id)
    }

    pub fn delete_planning_conversation(&self, conversation_id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "DELETE FROM bridge_file_drafts WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM planning_conversation_context WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM planning_messages WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM planning_conversations WHERE id = ?1",
            params![conversation_id],
        )?;
        Ok(())
    }

    pub fn list_planning_conversation_context(
        &self,
        conversation_id: i64,
    ) -> Result<Vec<PlanningConversationContextRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        Self::list_planning_conversation_context_for_connection(&connection, conversation_id)
    }

    pub fn attach_planning_conversation_context(
        &self,
        conversation_id: i64,
        context_type: &str,
        source_id: Option<i64>,
        label: &str,
    ) -> Result<PlanningConversationContextRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_planning_conversation_by_id(&connection, conversation_id)?;

        let normalized_context_type = context_type.trim();
        let normalized_label = label.trim();
        let existing = Self::find_existing_planning_conversation_context(
            &connection,
            conversation_id,
            normalized_context_type,
            source_id,
            normalized_label,
        )?;
        if let Some(existing) = existing {
            return Ok(existing);
        }

        connection.execute(
            "
            INSERT INTO planning_conversation_context (
                conversation_id,
                context_type,
                source_id,
                label
            )
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                conversation_id,
                normalized_context_type,
                source_id,
                normalized_label
            ],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_planning_conversation_context_by_id(&connection, id)
    }

    pub fn remove_planning_conversation_context(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "DELETE FROM planning_conversation_context WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn preview_planning_chat_prompt(
        &self,
        conversation_id: i64,
        draft_message: &str,
        system_instruction: &str,
    ) -> Result<PlanningPromptPreviewRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let conversation = Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        let project = Self::get_project_by_id(&connection, conversation.project_id)?;
        let contexts =
            Self::list_planning_conversation_context_for_connection(&connection, conversation_id)?;
        let markdown_context =
            Self::load_project_markdown_context_for_project(&connection, project.id)?;
        let message_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM planning_messages WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        )?;

        let mut warnings = vec![
            "Prompt Preview does not call OpenAI.".to_string(),
            "Project Markdown context and manual attachments are included in actual project chat sends."
                .to_string(),
        ];
        warnings.extend(markdown_context.warnings.clone());
        let mut attached_context_items = Vec::new();

        for context in contexts {
            let resolved = Self::resolve_context_preview_content(&connection, &context, &project)?;
            if !resolved.warning.is_empty() {
                warnings.push(format!("{}: {}", context.label, resolved.warning));
            }
            attached_context_items.push(resolved);
        }
        Self::add_project_github_context_if_missing(
            &connection,
            &project,
            &mut attached_context_items,
        )?;

        let attached_context_text = if attached_context_items.is_empty() {
            "No attached context.".to_string()
        } else {
            attached_context_items
                .iter()
                .map(|item| {
                    let status = if item.included {
                        "Included: Yes"
                    } else {
                        "Included: No"
                    };
                    let body = if item.content.trim().is_empty() {
                        item.warning.as_str()
                    } else {
                        item.content.as_str()
                    };
                    format!(
                        "Type: {}\nLabel: {}\n{}\nContent:\n{}",
                        item.context_type, item.label, status, body
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };
        let markdown_context_text = build_project_markdown_context_text(&markdown_context.files);

        let assembled_prompt = format!(
            "System intent:\n{}\n\nProject Markdown Context:\n{}\n\nAttached context:\n{}\n\nProject context:\nName: {}\nStatus: {}\nDescription: {}\n\nConversation:\nTitle: {}\nExisting message count: {}\n\nCurrent user message:\n{}",
            system_instruction,
            markdown_context_text,
            attached_context_text,
            project.name,
            project.status,
            if project.description.trim().is_empty() {
                "No description"
            } else {
                project.description.as_str()
            },
            conversation.title,
            message_count,
            draft_message
        );

        Ok(PlanningPromptPreviewRecord {
            project_label: project.name,
            project_status: project.status,
            project_description: project.description,
            conversation_label: conversation.title,
            message_count,
            draft_message: draft_message.to_string(),
            project_markdown_context_items: markdown_context.files,
            attached_context_items,
            assembled_prompt,
            warnings,
        })
    }

    pub fn planning_conversation_context_payload(
        &self,
        conversation_id: i64,
    ) -> Result<PlanningContextPayload> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let conversation = Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        let project = Self::get_project_by_id(&connection, conversation.project_id)?;
        let contexts =
            Self::list_planning_conversation_context_for_connection(&connection, conversation_id)?;
        let markdown_context =
            Self::load_project_markdown_context_for_project(&connection, project.id)?;
        let mut attached_context_items = Vec::new();

        for context in contexts {
            attached_context_items.push(Self::resolve_context_preview_content(
                &connection,
                &context,
                &project,
            )?);
        }
        Self::add_project_github_context_if_missing(
            &connection,
            &project,
            &mut attached_context_items,
        )?;

        Ok(build_planning_context_payload(
            &markdown_context,
            &attached_context_items,
        ))
    }

    pub fn list_bridge_file_drafts(&self, project_id: i64) -> Result<Vec<BridgeFileDraftRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_project_by_id(&connection, project_id)?;
        let mut statement = connection.prepare(
            "
            SELECT id, project_id, conversation_id, title, content, status, created_at, updated_at
            FROM bridge_file_drafts
            WHERE project_id = ?1
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let drafts = statement
            .query_map(params![project_id], bridge_file_draft_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(drafts)
    }

    pub fn get_bridge_file_draft(&self, id: i64) -> Result<BridgeFileDraftRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_bridge_file_draft_by_id(&connection, id)
    }

    pub fn create_bridge_file_draft_from_conversation(
        &self,
        conversation_id: i64,
    ) -> Result<BridgeFileDraftRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let conversation = Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        let project = Self::get_project_by_id(&connection, conversation.project_id)?;
        let messages = Self::list_planning_messages_for_connection(&connection, conversation_id)?;
        let contexts =
            Self::list_planning_conversation_context_for_connection(&connection, conversation_id)?;
        let markdown_context =
            Self::load_project_markdown_context_for_project(&connection, project.id)?;

        let mut attached_context_items = Vec::new();
        for context in contexts {
            attached_context_items.push(Self::resolve_context_preview_content(
                &connection,
                &context,
                &project,
            )?);
        }
        Self::add_project_github_context_if_missing(
            &connection,
            &project,
            &mut attached_context_items,
        )?;

        let generated_at: String =
            connection.query_row("SELECT CURRENT_TIMESTAMP", [], |row| row.get(0))?;
        let title = format!("Bridge Draft - {} - {}", conversation.title, generated_at);
        let content = build_bridge_file_draft_markdown(
            &project,
            &conversation,
            &messages,
            &markdown_context,
            &attached_context_items,
        );

        connection.execute(
            "
            INSERT INTO bridge_file_drafts (
                project_id,
                conversation_id,
                title,
                content,
                status
            )
            VALUES (?1, ?2, ?3, ?4, 'draft')
            ",
            params![project.id, conversation.id, title, content],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_bridge_file_draft_by_id(&connection, id)
    }

    pub fn delete_bridge_file_draft(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute("DELETE FROM bridge_file_drafts WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_youtube_references(&self) -> Result<Vec<YouTubeReferenceRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, title, url, video_id, channel_name, notes, tags, created_at, updated_at
            FROM youtube_references
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let references = statement
            .query_map([], youtube_reference_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(references)
    }

    pub fn get_youtube_reference(&self, id: i64) -> Result<YouTubeReferenceRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_youtube_reference_by_id(&connection, id)
    }

    pub fn create_youtube_reference(
        &self,
        title: &str,
        url: &str,
        video_id: &str,
        channel_name: &str,
        notes: &str,
        tags: &str,
    ) -> Result<YouTubeReferenceRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            INSERT INTO youtube_references (title, url, video_id, channel_name, notes, tags)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                title.trim(),
                url.trim(),
                video_id.trim(),
                channel_name.trim(),
                notes,
                tags.trim()
            ],
        )?;
        let id = connection.last_insert_rowid();
        Self::get_youtube_reference_by_id(&connection, id)
    }

    pub fn update_youtube_reference(
        &self,
        id: i64,
        title: &str,
        url: &str,
        video_id: &str,
        channel_name: &str,
        notes: &str,
        tags: &str,
    ) -> Result<YouTubeReferenceRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            UPDATE youtube_references
            SET title = ?1,
                url = ?2,
                video_id = ?3,
                channel_name = ?4,
                notes = ?5,
                tags = ?6,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?7
            ",
            params![
                title.trim(),
                url.trim(),
                video_id.trim(),
                channel_name.trim(),
                notes,
                tags.trim(),
                id
            ],
        )?;

        Self::get_youtube_reference_by_id(&connection, id)
    }

    pub fn delete_youtube_reference(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute("DELETE FROM youtube_references WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<GameRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, name, slug, summary, created_at, updated_at
            FROM games
            ORDER BY name COLLATE NOCASE ASC, id ASC
            ",
        )?;

        let games = statement
            .query_map([], game_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_game(&self, id: i64) -> Result<GameRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_by_id(&connection, id)
    }

    pub fn create_game(&self, name: &str, summary: &str) -> Result<GameRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let trimmed_name = name.trim();
        let slug = game_slug(trimmed_name);
        connection.execute(
            "
            INSERT INTO games (name, slug, summary)
            VALUES (?1, ?2, ?3)
            ",
            params![trimmed_name, slug, summary.trim()],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_by_id(&connection, id)
    }

    pub fn delete_game(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            DELETE FROM game_chat_messages
            WHERE conversation_id IN (
                SELECT id FROM game_chat_conversations WHERE game_id = ?1
            )
            ",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM game_chat_conversations WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM game_catalog_screenshots WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM game_catalog_references WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM game_catalog_objects WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute("DELETE FROM games WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_game_catalog_objects(&self, game_id: i64) -> Result<Vec<GameCatalogObjectRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                name,
                object_type,
                category,
                category_icon,
                category_icon_path,
                description,
                notes,
                tags,
                thumbnail_path,
                source_screenshot_path,
                created_at,
                updated_at
            FROM game_catalog_objects
            WHERE game_id = ?1
            ORDER BY category COLLATE NOCASE ASC, name COLLATE NOCASE ASC
            ",
        )?;

        let objects = statement
            .query_map(params![game_id], game_catalog_object_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(objects)
    }

    pub fn delete_game_screenshot_catalog_objects(&self, game_id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            DELETE FROM game_catalog_objects
            WHERE game_id = ?1
                AND tags LIKE '%screenshot-catalog%'
            ",
            params![game_id],
        )?;
        Ok(())
    }

    pub fn upsert_game_catalog_object(
        &self,
        game_id: i64,
        name: &str,
        object_type: &str,
        category: &str,
        category_icon: &str,
        category_icon_path: &str,
        description: &str,
        notes: &str,
        tags: &str,
        thumbnail_path: &str,
        source_screenshot_path: &str,
    ) -> Result<GameCatalogObjectRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            INSERT INTO game_catalog_objects (
                game_id,
                name,
                object_type,
                category,
                category_icon,
                category_icon_path,
                description,
                notes,
                tags,
                thumbnail_path,
                source_screenshot_path
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(game_id, name) DO UPDATE SET
                object_type = excluded.object_type,
                category = excluded.category,
                category_icon = excluded.category_icon,
                category_icon_path = excluded.category_icon_path,
                description = excluded.description,
                notes = excluded.notes,
                tags = excluded.tags,
                thumbnail_path = excluded.thumbnail_path,
                source_screenshot_path = excluded.source_screenshot_path,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                name.trim(),
                object_type.trim(),
                category.trim(),
                category_icon.trim(),
                category_icon_path.trim(),
                description.trim(),
                notes.trim(),
                tags.trim(),
                thumbnail_path.trim(),
                source_screenshot_path.trim()
            ],
        )?;

        Self::get_game_catalog_object_by_name(&connection, game_id, name)
    }

    pub fn create_game_screenshot_capture_request(
        &self,
        game_id: i64,
        title: &str,
        file_path: &str,
        request_id: &str,
        request_path: &str,
        capture_status: &str,
        captured_at: &str,
        notes: &str,
    ) -> Result<GameScreenshotCaptureRequestRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            INSERT INTO game_catalog_screenshots (
                game_id,
                title,
                file_path,
                request_id,
                request_path,
                capture_status,
                captured_at,
                notes
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                game_id,
                title.trim(),
                file_path.trim(),
                request_id.trim(),
                request_path.trim(),
                capture_status.trim(),
                captured_at.trim(),
                notes
            ],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_screenshot_capture_request_by_id(&connection, id)
    }

    pub fn list_game_chat_conversations(
        &self,
        game_id: i64,
    ) -> Result<Vec<GameChatConversationRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT id, game_id, title, created_at, updated_at
            FROM game_chat_conversations
            WHERE game_id = ?1
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let conversations = statement
            .query_map(params![game_id], game_chat_conversation_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(conversations)
    }

    pub fn create_game_chat_conversation(
        &self,
        game_id: i64,
        title: &str,
    ) -> Result<GameChatConversationRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_by_id(&connection, game_id)?;
        let clean_title = if title.trim().is_empty() {
            "Game chat"
        } else {
            title.trim()
        };

        connection.execute(
            "
            INSERT INTO game_chat_conversations (game_id, title)
            VALUES (?1, ?2)
            ",
            params![game_id, clean_title],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_chat_conversation_by_id(&connection, id)
    }

    pub fn get_game_chat_conversation(&self, id: i64) -> Result<GameChatConversationRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_chat_conversation_by_id(&connection, id)
    }

    pub fn list_game_chat_messages(
        &self,
        conversation_id: i64,
    ) -> Result<Vec<GameChatMessageRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_chat_conversation_by_id(&connection, conversation_id)?;
        Self::list_game_chat_messages_for_connection(&connection, conversation_id)
    }

    pub fn recent_game_chat_messages(
        &self,
        conversation_id: i64,
        limit: i64,
    ) -> Result<Vec<GameChatMessageRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM (
                SELECT id, conversation_id, role, content, created_at
                FROM game_chat_messages
                WHERE conversation_id = ?1
                ORDER BY id DESC
                LIMIT ?2
            )
            ORDER BY id ASC
            ",
        )?;

        let messages = statement
            .query_map(params![conversation_id, limit], game_chat_message_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    pub fn create_game_chat_message(
        &self,
        conversation_id: i64,
        role: &str,
        content: &str,
    ) -> Result<GameChatMessageRecord> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_chat_conversation_by_id(&connection, conversation_id)?;
        connection.execute(
            "
            INSERT INTO game_chat_messages (conversation_id, role, content)
            VALUES (?1, ?2, ?3)
            ",
            params![conversation_id, role.trim(), content.trim()],
        )?;
        connection.execute(
            "
            UPDATE game_chat_conversations
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            ",
            params![conversation_id],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_chat_message_by_id(&connection, id)
    }

    pub fn delete_game_chat_conversation(&self, conversation_id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "DELETE FROM game_chat_messages WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM game_chat_conversations WHERE id = ?1",
            params![conversation_id],
        )?;
        Ok(())
    }

    pub fn list_game_screenshots(
        &self,
        game_id: i64,
    ) -> Result<Vec<GameScreenshotCaptureRequestRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                title,
                file_path,
                request_id,
                request_path,
                capture_status,
                captured_at,
                created_at,
                updated_at
            FROM game_catalog_screenshots
            WHERE game_id = ?1
                AND capture_status != 'requested'
            ORDER BY created_at DESC, id DESC
            ",
        )?;

        let screenshots = statement
            .query_map(params![game_id], game_screenshot_capture_request_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(screenshots)
    }

    pub fn get_game_screenshot(
        &self,
        id: i64,
    ) -> Result<Option<GameScreenshotCaptureRequestRecord>> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        Self::get_game_screenshot_capture_request_by_id(&connection, id).optional()
    }

    pub fn delete_game_screenshot_references(
        &self,
        game_id: i64,
        file_path: &str,
        request_path: &str,
    ) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "
            DELETE FROM game_catalog_references
            WHERE game_id = ?1
                AND (
                    local_path = ?2
                    OR local_path = ?3
                )
            ",
            params![game_id, file_path.trim(), request_path.trim()],
        )?;
        Ok(())
    }

    pub fn delete_game_screenshot(&self, id: i64) -> Result<()> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        connection.execute(
            "DELETE FROM game_catalog_screenshots WHERE id = ?1",
            params![id],
        )?;
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

    fn get_project_by_id(connection: &Connection, id: i64) -> Result<ProjectRecord> {
        connection.query_row(
            "
            SELECT id, name, description, status, created_at, updated_at
            FROM projects
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
    }

    fn get_project_github_repository_by_project_id(
        connection: &Connection,
        project_id: i64,
    ) -> Result<ProjectGitHubRepositoryRecord> {
        connection.query_row(
            "
            SELECT
                id,
                project_id,
                repository_full_name,
                repository_url,
                default_branch,
                visibility,
                last_fetched_at,
                last_fetch_status,
                created_at,
                updated_at
            FROM project_github_repositories
            WHERE project_id = ?1
            ",
            params![project_id],
            |row| {
                Ok(ProjectGitHubRepositoryRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    repository_full_name: row.get(2)?,
                    repository_url: row.get(3)?,
                    default_branch: row.get(4)?,
                    visibility: row.get(5)?,
                    last_fetched_at: row.get(6)?,
                    last_fetch_status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
    }

    fn get_project_markdown_context_by_project_id(
        connection: &Connection,
        project_id: i64,
    ) -> Result<ProjectMarkdownContextRecord> {
        connection.query_row(
            "
            SELECT id, project_id, root_path, readme_path, created_at, updated_at
            FROM project_markdown_context
            WHERE project_id = ?1
            ",
            params![project_id],
            project_markdown_context_from_row,
        )
    }

    fn get_planning_conversation_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<PlanningConversationRecord> {
        connection.query_row(
            "
            SELECT id, project_id, title, created_at, updated_at
            FROM planning_conversations
            WHERE id = ?1
            ",
            params![id],
            planning_conversation_from_row,
        )
    }

    fn get_planning_message_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<PlanningMessageRecord> {
        connection.query_row(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM planning_messages
            WHERE id = ?1
            ",
            params![id],
            planning_message_from_row,
        )
    }

    fn list_planning_messages_for_connection(
        connection: &Connection,
        conversation_id: i64,
    ) -> Result<Vec<PlanningMessageRecord>> {
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM planning_messages
            WHERE conversation_id = ?1
            ORDER BY id ASC
            ",
        )?;

        let messages = statement
            .query_map(params![conversation_id], planning_message_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    fn get_planning_conversation_context_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<PlanningConversationContextRecord> {
        connection.query_row(
            "
            SELECT id, conversation_id, context_type, source_id, label, created_at
            FROM planning_conversation_context
            WHERE id = ?1
            ",
            params![id],
            planning_conversation_context_from_row,
        )
    }

    fn list_planning_conversation_context_for_connection(
        connection: &Connection,
        conversation_id: i64,
    ) -> Result<Vec<PlanningConversationContextRecord>> {
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, context_type, source_id, label, created_at
            FROM planning_conversation_context
            WHERE conversation_id = ?1
            ORDER BY created_at ASC, id ASC
            ",
        )?;

        let context = statement
            .query_map(
                params![conversation_id],
                planning_conversation_context_from_row,
            )?
            .collect::<Result<Vec<_>>>()?;

        let mut seen = HashSet::new();
        let deduped = context
            .into_iter()
            .filter(|item| seen.insert(planning_context_dedupe_key(item)))
            .collect();

        Ok(deduped)
    }

    fn find_existing_planning_conversation_context(
        connection: &Connection,
        conversation_id: i64,
        context_type: &str,
        source_id: Option<i64>,
        label: &str,
    ) -> Result<Option<PlanningConversationContextRecord>> {
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, context_type, source_id, label, created_at
            FROM planning_conversation_context
            WHERE conversation_id = ?1
              AND context_type = ?2
              AND (
                source_id = ?3
                OR (source_id IS NULL AND ?3 IS NULL AND label = ?4)
                OR (?2 = 'github_repository' AND label = ?4)
              )
            ORDER BY created_at ASC, id ASC
            LIMIT 1
            ",
        )?;

        statement
            .query_row(
                params![conversation_id, context_type, source_id, label],
                planning_conversation_context_from_row,
            )
            .optional()
    }

    fn get_youtube_reference_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<YouTubeReferenceRecord> {
        connection.query_row(
            "
            SELECT id, title, url, video_id, channel_name, notes, tags, created_at, updated_at
            FROM youtube_references
            WHERE id = ?1
            ",
            params![id],
            youtube_reference_from_row,
        )
    }

    fn get_game_by_id(connection: &Connection, id: i64) -> Result<GameRecord> {
        connection.query_row(
            "
            SELECT id, name, slug, summary, created_at, updated_at
            FROM games
            WHERE id = ?1
            ",
            params![id],
            game_from_row,
        )
    }

    fn get_game_catalog_object_by_name(
        connection: &Connection,
        game_id: i64,
        name: &str,
    ) -> Result<GameCatalogObjectRecord> {
        connection.query_row(
            "
            SELECT
                id,
                game_id,
                name,
                object_type,
                category,
                category_icon,
                category_icon_path,
                description,
                notes,
                tags,
                thumbnail_path,
                source_screenshot_path,
                created_at,
                updated_at
            FROM game_catalog_objects
            WHERE game_id = ?1
                AND name = ?2 COLLATE NOCASE
            ",
            params![game_id, name.trim()],
            game_catalog_object_from_row,
        )
    }

    fn get_game_screenshot_capture_request_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<GameScreenshotCaptureRequestRecord> {
        connection.query_row(
            "
            SELECT
                id,
                game_id,
                title,
                file_path,
                request_id,
                request_path,
                capture_status,
                captured_at,
                created_at,
                updated_at
            FROM game_catalog_screenshots
            WHERE id = ?1
            ",
            params![id],
            game_screenshot_capture_request_from_row,
        )
    }

    fn get_game_chat_conversation_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<GameChatConversationRecord> {
        connection.query_row(
            "
            SELECT id, game_id, title, created_at, updated_at
            FROM game_chat_conversations
            WHERE id = ?1
            ",
            params![id],
            game_chat_conversation_from_row,
        )
    }

    fn get_game_chat_message_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<GameChatMessageRecord> {
        connection.query_row(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM game_chat_messages
            WHERE id = ?1
            ",
            params![id],
            game_chat_message_from_row,
        )
    }

    fn list_game_chat_messages_for_connection(
        connection: &Connection,
        conversation_id: i64,
    ) -> Result<Vec<GameChatMessageRecord>> {
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM game_chat_messages
            WHERE conversation_id = ?1
            ORDER BY id ASC
            ",
        )?;

        let messages = statement
            .query_map(params![conversation_id], game_chat_message_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    fn get_bridge_file_draft_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<BridgeFileDraftRecord> {
        connection.query_row(
            "
            SELECT id, project_id, conversation_id, title, content, status, created_at, updated_at
            FROM bridge_file_drafts
            WHERE id = ?1
            ",
            params![id],
            bridge_file_draft_from_row,
        )
    }

    fn resolve_context_preview_content(
        connection: &Connection,
        context: &PlanningConversationContextRecord,
        conversation_project: &ProjectRecord,
    ) -> Result<PromptPreviewContextItem> {
        let missing_warning = "Attached source could not be resolved.".to_string();
        let (included, content, warning) = match context.context_type.as_str() {
            "project" => {
                let project = match context.source_id {
                    Some(id) => Self::get_project_by_id(connection, id).optional()?,
                    None => Some(ProjectRecord {
                        id: conversation_project.id,
                        name: conversation_project.name.clone(),
                        description: conversation_project.description.clone(),
                        status: conversation_project.status.clone(),
                        created_at: conversation_project.created_at.clone(),
                        updated_at: conversation_project.updated_at.clone(),
                    }),
                };
                match project {
                    Some(project) => (
                        true,
                        format!(
                            "Name: {}\nStatus: {}\nDescription: {}",
                            project.name, project.status, project.description
                        ),
                        String::new(),
                    ),
                    None => (false, String::new(), missing_warning),
                }
            }
            "github_repository" => match context.source_id {
                Some(id) => {
                    let repository =
                        Self::get_project_github_repository_by_id(connection, id).optional()?;
                    let repository = match repository {
                        Some(repository) => Some(repository),
                        None => Self::get_project_github_repository_by_project_id(
                            connection,
                            conversation_project.id,
                        )
                        .optional()?,
                    };
                    match repository {
                        Some(repository) => (
                            true,
                            github_repository_context_content(&repository),
                            String::new(),
                        ),
                        None => (false, String::new(), missing_warning),
                    }
                }
                None => match Self::get_project_github_repository_by_project_id(
                    connection,
                    conversation_project.id,
                )
                .optional()?
                {
                    Some(repository) => (
                        true,
                        github_repository_context_content(&repository),
                        String::new(),
                    ),
                    None => (false, String::new(), missing_warning),
                },
            },
            "note" => match context.source_id {
                Some(id) => match Self::get_note_by_id(connection, id).optional()? {
                    Some(note) => (
                        true,
                        format!("Title: {}\nBody:\n{}", note.title, note.body),
                        String::new(),
                    ),
                    None => (false, String::new(), missing_warning),
                },
                None => (false, String::new(), missing_warning),
            },
            "task" => match context.source_id {
                Some(id) => match Self::get_task_by_id(connection, id).optional()? {
                    Some(task) => (
                        true,
                        format!(
                            "Title: {}\nBody:\n{}\nDeadline: {}\nCompleted: {}",
                            task.title, task.body, task.deadline, task.is_completed
                        ),
                        String::new(),
                    ),
                    None => (false, String::new(), missing_warning),
                },
                None => (false, String::new(), missing_warning),
            },
            "calendar_event" => match context.source_id {
                Some(id) => match Self::get_calendar_event_by_id(connection, id).optional()? {
                    Some(event) => (
                        true,
                        format!(
                            "Title: {}\nStart: {} {}\nEnd: {} {}\nNotes:\n{}",
                            event.title,
                            event.start_date,
                            event.start_time,
                            event.end_date,
                            event.end_time,
                            event.notes
                        ),
                        String::new(),
                    ),
                    None => (false, String::new(), missing_warning),
                },
                None => (false, String::new(), missing_warning),
            },
            "youtube_reference" => match context.source_id {
                Some(id) => match Self::get_youtube_reference_by_id(connection, id).optional()? {
                    Some(reference) => (
                        true,
                        format!(
                            "Title: {}\nURL: {}\nVideo ID: {}\nChannel: {}\nTags: {}\nNotes:\n{}",
                            reference.title,
                            reference.url,
                            reference.video_id,
                            reference.channel_name,
                            reference.tags,
                            reference.notes
                        ),
                        String::new(),
                    ),
                    None => (false, String::new(), missing_warning),
                },
                None => (false, String::new(), missing_warning),
            },
            "scratchpad" => {
                let content: String = connection.query_row(
                    "SELECT content FROM scratchpad WHERE id = 1",
                    [],
                    |row| row.get(0),
                )?;
                if content.trim().is_empty() {
                    (false, String::new(), "Scratchpad is empty.".to_string())
                } else {
                    (true, content, String::new())
                }
            }
            _ => (
                false,
                String::new(),
                "Unsupported context type.".to_string(),
            ),
        };

        Ok(PromptPreviewContextItem {
            id: context.id,
            context_type: context.context_type.clone(),
            label: context.label.clone(),
            included,
            content,
            warning,
        })
    }

    fn add_project_github_context_if_missing(
        connection: &Connection,
        project: &ProjectRecord,
        context_items: &mut Vec<PromptPreviewContextItem>,
    ) -> Result<()> {
        let repository =
            Self::get_project_github_repository_by_project_id(connection, project.id).optional()?;
        let Some(repository) = repository else {
            return Ok(());
        };

        let already_present = context_items.iter().any(|item| {
            item.context_type == "github_repository"
                && item.label == repository.repository_full_name
        });
        if already_present {
            return Ok(());
        }

        context_items.push(PromptPreviewContextItem {
            id: -repository.id,
            context_type: "github_repository".to_string(),
            label: repository.repository_full_name.clone(),
            included: true,
            content: github_repository_context_content(&repository),
            warning: String::new(),
        });

        Ok(())
    }

    fn load_project_markdown_context_for_project(
        connection: &Connection,
        project_id: i64,
    ) -> Result<ProjectMarkdownContextPayload> {
        let config =
            Self::get_project_markdown_context_by_project_id(connection, project_id).optional()?;
        Ok(match config {
            Some(config) => load_project_markdown_context_from_config(&config),
            None => ProjectMarkdownContextPayload {
                files: Vec::new(),
                warnings: vec!["No project Markdown context root is configured.".to_string()],
            },
        })
    }

    fn get_project_github_repository_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<ProjectGitHubRepositoryRecord> {
        connection.query_row(
            "
            SELECT
                id,
                project_id,
                repository_full_name,
                repository_url,
                default_branch,
                visibility,
                last_fetched_at,
                last_fetch_status,
                created_at,
                updated_at
            FROM project_github_repositories
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(ProjectGitHubRepositoryRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    repository_full_name: row.get(2)?,
                    repository_url: row.get(3)?,
                    default_branch: row.get(4)?,
                    visibility: row.get(5)?,
                    last_fetched_at: row.get(6)?,
                    last_fetch_status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
    }
}

fn planning_conversation_from_row(row: &rusqlite::Row<'_>) -> Result<PlanningConversationRecord> {
    Ok(PlanningConversationRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn planning_message_from_row(row: &rusqlite::Row<'_>) -> Result<PlanningMessageRecord> {
    Ok(PlanningMessageRecord {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn planning_conversation_context_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<PlanningConversationContextRecord> {
    Ok(PlanningConversationContextRecord {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        context_type: row.get(2)?,
        source_id: row.get(3)?,
        label: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn project_markdown_context_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<ProjectMarkdownContextRecord> {
    Ok(ProjectMarkdownContextRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        root_path: row.get(2)?,
        readme_path: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn bridge_file_draft_from_row(row: &rusqlite::Row<'_>) -> Result<BridgeFileDraftRecord> {
    Ok(BridgeFileDraftRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        conversation_id: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        status: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn build_bridge_file_draft_markdown(
    project: &ProjectRecord,
    conversation: &PlanningConversationRecord,
    messages: &[PlanningMessageRecord],
    markdown_context: &ProjectMarkdownContextPayload,
    context_items: &[PromptPreviewContextItem],
) -> String {
    let description = if project.description.trim().is_empty() {
        "No description provided."
    } else {
        project.description.as_str()
    };
    let goal = first_user_message(messages)
        .map(|message| {
            format!("Review and refine this inferred goal from the conversation:\n\n{message}")
        })
        .unwrap_or_else(|| "TODO: User review required.".to_string());
    let transcript = if messages.is_empty() {
        "No conversation messages were saved when this draft was generated.".to_string()
    } else {
        messages
            .iter()
            .map(|message| {
                format!(
                    "### {}\n\n{}\n",
                    message.role.to_uppercase(),
                    message.content.trim()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let project_markdown_context = build_project_markdown_context_text(&markdown_context.files);
    let relevant_context = if context_items.is_empty() {
        "No attached context was linked to this conversation.".to_string()
    } else {
        context_items
            .iter()
            .map(|item| {
                let included = if item.included { "Yes" } else { "No" };
                let body = if !item.content.trim().is_empty() {
                    item.content.as_str()
                } else if !item.warning.trim().is_empty() {
                    item.warning.as_str()
                } else {
                    "TODO: User review required."
                };
                format!(
                    "### {}: {}\n\nIncluded: {}\n\n{}\n",
                    bridge_context_type_label(&item.context_type),
                    item.label,
                    included,
                    body
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "# Project Bridge Draft\n\n\
## Project\n\n\
- Name: {}\n\
- Status: {}\n\
- Description: {}\n\n\
## Conversation Source\n\n\
- Conversation: {}\n\
- Conversation ID: {}\n\
- Message count: {}\n\n\
{}\n\n\
## Goal\n\n\
{}\n\n\
## Relevant Context\n\n\
### Project Markdown Context\n\n\
{}\n\n\
### Conversation Manual Attachments\n\n\
{}\n\n\
## Implementation Instructions\n\n\
TODO: User review required.\n\n\
Use the project, project Markdown context, conversation transcript, and manual attachments above as source material. Do not invent repository files, external services, or implementation details that are not present in the provided context.\n\n\
## Validation Checklist\n\n\
- Review this bridge draft for accuracy before using it in Codex.\n\
- Confirm the selected project and conversation are correct.\n\
- Confirm attached context is relevant and safe to include.\n\
- Run the repository's normal validation commands after implementation.\n\
- Manually validate the changed app workflow.\n\n\
## Deferred Items\n\n\
- Full bridge-file editor\n\
- Approval workflow\n\
- Obsolete status workflow\n\
- Export to local Markdown files\n\
- Direct Codex handoff\n\
- GitHub write operations\n\
- Chat streaming\n\
- Model picker UI\n\
- Token budgeting\n\
- Vector stores or semantic search\n\
- ChatGPT import\n\n\
## Notes\n\n\
- This is a local SQLite bridge draft generated by Overlay Forge.\n\
- User review remains required before using this draft as an implementation prompt.\n\
- TODO: Add any unresolved questions or assumptions before handoff.\n",
        project.name,
        project.status,
        description,
        conversation.title,
        conversation.id,
        messages.len(),
        transcript,
        goal,
        project_markdown_context,
        relevant_context
    )
}

fn build_planning_context_payload(
    markdown_context: &ProjectMarkdownContextPayload,
    context_items: &[PromptPreviewContextItem],
) -> PlanningContextPayload {
    let mut warnings = markdown_context.warnings.clone();
    let included_context = context_items
        .iter()
        .filter_map(|item| {
            if !item.warning.trim().is_empty() {
                warnings.push(format!("{}: {}", item.label, item.warning));
            }

            if !item.included || item.content.trim().is_empty() {
                return None;
            }

            Some(format!(
                "Type: {}\nLabel: {}\nContent:\n{}",
                bridge_context_type_label(&item.context_type),
                item.label,
                item.content
            ))
        })
        .collect::<Vec<_>>();

    let markdown_text = build_project_markdown_context_text(&markdown_context.files);
    let attachment_text = if included_context.is_empty() {
        "No conversation manual attachments included.".to_string()
    } else {
        included_context.join("\n\n---\n\n")
    };

    let content = if markdown_context.files.is_empty() && included_context.is_empty() {
        String::new()
    } else {
        format!(
            "Local repository Markdown context:\n\
The following Markdown files were read from the selected project's configured local repository root and are available as source context for this chat. Use them when answering questions about the project files, README, docs, bridge files, project plan, or milestone state.\n\n{}\n\nConversation manual attachments:\n\n{}",
            markdown_text, attachment_text
        )
    };

    PlanningContextPayload { content, warnings }
}

fn build_project_markdown_context_text(files: &[ProjectMarkdownContextFile]) -> String {
    if files.is_empty() {
        return "No project Markdown context loaded.".to_string();
    }

    files
        .iter()
        .map(|file| {
            let status = if file.included {
                "Included: Yes"
            } else {
                "Included: No"
            };
            let body = if file.included && !file.content.trim().is_empty() {
                file.content.as_str()
            } else if !file.warning.trim().is_empty() {
                file.warning.as_str()
            } else {
                "No content."
            };
            format!(
                "File: {}\n{}\nContent:\n{}",
                file.relative_path, status, body
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn load_project_markdown_context_from_config(
    config: &ProjectMarkdownContextRecord,
) -> ProjectMarkdownContextPayload {
    let mut warnings = Vec::new();
    let root_path = PathBuf::from(config.root_path.trim());

    let root = match fs::canonicalize(&root_path) {
        Ok(path) if path.is_dir() => path,
        Ok(_) => {
            return ProjectMarkdownContextPayload {
                files: vec![ProjectMarkdownContextFile {
                    relative_path: ".".to_string(),
                    included: false,
                    content: String::new(),
                    warning: "Configured Markdown context root is not a directory.".to_string(),
                }],
                warnings: vec!["Configured Markdown context root is not a directory.".to_string()],
            };
        }
        Err(error) => {
            return ProjectMarkdownContextPayload {
                files: vec![ProjectMarkdownContextFile {
                    relative_path: ".".to_string(),
                    included: false,
                    content: String::new(),
                    warning: format!("Configured Markdown context root could not be read: {error}"),
                }],
                warnings: vec![format!(
                    "Configured Markdown context root could not be read: {error}"
                )],
            };
        }
    };

    let readme_path = normalize_relative_markdown_path(&config.readme_path)
        .unwrap_or_else(|| "README.md".to_string());
    let mut relative_paths = Vec::new();
    push_unique_path(&mut relative_paths, readme_path.clone());
    for path in known_markdown_context_paths(&root) {
        push_unique_path(&mut relative_paths, path);
    }

    let readme_full_path = root.join(&readme_path);
    match fs::read_to_string(&readme_full_path) {
        Ok(content) => {
            let (references, reference_warnings) = extract_markdown_references(&content);
            warnings.extend(reference_warnings);
            for path in references {
                push_unique_path(&mut relative_paths, path);
            }
        }
        Err(error) => warnings.push(format!("{readme_path}: could not be read: {error}")),
    }

    let mut files = Vec::new();
    let mut total_bytes = 0usize;

    for relative_path in relative_paths {
        let Some(file) = load_markdown_context_file(&root, &relative_path, &mut total_bytes) else {
            continue;
        };
        if !file.warning.trim().is_empty() {
            warnings.push(format!("{}: {}", file.relative_path, file.warning));
        }
        files.push(file);
    }

    ProjectMarkdownContextPayload { files, warnings }
}

fn known_markdown_context_paths(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    for relative_path in ["README.md", "CHANGELOG.md"] {
        if root.join(relative_path).is_file() {
            paths.push(relative_path.to_string());
        }
    }

    for directory in ["docs", "bridge-files"] {
        let full_directory = root.join(directory);
        let Ok(entries) = fs::read_dir(full_directory) else {
            continue;
        };
        let mut names = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && is_markdown_path(&path) {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| format!("{directory}/{name}"))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        names.sort();
        paths.extend(names);
    }

    paths
}

fn load_markdown_context_file(
    root: &Path,
    relative_path: &str,
    total_bytes: &mut usize,
) -> Option<ProjectMarkdownContextFile> {
    let normalized = match normalize_relative_markdown_path(relative_path) {
        Some(path) => path,
        None => {
            return Some(ProjectMarkdownContextFile {
                relative_path: relative_path.to_string(),
                included: false,
                content: String::new(),
                warning: "Skipped unsafe or non-Markdown path.".to_string(),
            });
        }
    };

    let full_path = root.join(&normalized);
    let canonical = match fs::canonicalize(&full_path) {
        Ok(path) => path,
        Err(error) => {
            return Some(ProjectMarkdownContextFile {
                relative_path: normalized,
                included: false,
                content: String::new(),
                warning: format!("Markdown file could not be read: {error}"),
            });
        }
    };

    if !canonical.starts_with(root) {
        return Some(ProjectMarkdownContextFile {
            relative_path: normalized,
            included: false,
            content: String::new(),
            warning: "Skipped because the resolved path is outside the configured root."
                .to_string(),
        });
    }

    if !canonical.is_file() || !is_markdown_path(&canonical) {
        return Some(ProjectMarkdownContextFile {
            relative_path: normalized,
            included: false,
            content: String::new(),
            warning: "Skipped because the resolved path is not a Markdown file.".to_string(),
        });
    }

    let content = match fs::read_to_string(&canonical) {
        Ok(content) => content,
        Err(error) => {
            return Some(ProjectMarkdownContextFile {
                relative_path: normalized,
                included: false,
                content: String::new(),
                warning: format!("Markdown file could not be read: {error}"),
            });
        }
    };

    let mut warning = String::new();
    let mut included_content = content;
    if included_content.len() > MARKDOWN_CONTEXT_PER_FILE_LIMIT {
        included_content.truncate(MARKDOWN_CONTEXT_PER_FILE_LIMIT);
        warning = format!(
            "File was truncated to {} bytes for context assembly.",
            MARKDOWN_CONTEXT_PER_FILE_LIMIT
        );
    }

    if *total_bytes >= MARKDOWN_CONTEXT_TOTAL_LIMIT {
        return Some(ProjectMarkdownContextFile {
            relative_path: normalized,
            included: false,
            content: String::new(),
            warning: "Skipped because the project Markdown context size limit was reached."
                .to_string(),
        });
    }

    let remaining = MARKDOWN_CONTEXT_TOTAL_LIMIT - *total_bytes;
    if included_content.len() > remaining {
        included_content.truncate(remaining);
        warning = format!(
            "File was truncated because the project Markdown context total size limit is {} bytes.",
            MARKDOWN_CONTEXT_TOTAL_LIMIT
        );
    }
    *total_bytes += included_content.len();

    Some(ProjectMarkdownContextFile {
        relative_path: normalized,
        included: true,
        content: included_content,
        warning,
    })
}

fn extract_markdown_references(content: &str) -> (Vec<String>, Vec<String>) {
    let mut references = Vec::new();
    let mut warnings = Vec::new();
    for line in content.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let after_start = &rest[start + 2..];
            let Some(end) = after_start.find(')') else {
                break;
            };
            let target = after_start[..end].trim();
            if let Some(path) = normalize_relative_markdown_path(target) {
                push_unique_path(&mut references, path);
            } else if looks_like_markdown_reference(target) {
                warnings.push(format!(
                    "{target}: skipped unsafe or unsupported Markdown reference."
                ));
            }
            rest = &after_start[end + 1..];
        }

        for raw_part in line.split_whitespace() {
            let cleaned = raw_part.trim_matches(|character: char| {
                matches!(
                    character,
                    '"' | '\'' | '`' | '<' | '>' | '(' | ')' | '[' | ']' | ',' | '.'
                )
            });
            if let Some(path) = normalize_relative_markdown_path(cleaned) {
                push_unique_path(&mut references, path);
            }
        }
    }
    (references, warnings)
}

fn normalize_relative_markdown_path(value: &str) -> Option<String> {
    let without_anchor = value.split('#').next().unwrap_or_default();
    let without_query = without_anchor.split('?').next().unwrap_or_default();
    let cleaned = without_query.trim().replace('\\', "/");
    if cleaned.is_empty()
        || cleaned.starts_with('/')
        || cleaned.contains(':')
        || cleaned.contains("://")
        || cleaned.contains('*')
        || cleaned.contains('?')
        || cleaned.split('/').any(|part| part == "..")
    {
        return None;
    }

    let lower = cleaned.to_ascii_lowercase();
    if !(lower.ends_with(".md") || lower.ends_with(".markdown")) {
        return None;
    }

    Some(cleaned)
}

fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
        .unwrap_or(false)
}

fn looks_like_markdown_reference(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains(".md") || lower.contains(".markdown")
}

fn push_unique_path(paths: &mut Vec<String>, path: String) {
    let key = path.to_ascii_lowercase();
    if !paths
        .iter()
        .any(|existing| existing.to_ascii_lowercase() == key)
    {
        paths.push(path);
    }
}

fn first_user_message(messages: &[PlanningMessageRecord]) -> Option<String> {
    messages
        .iter()
        .find(|message| message.role == "user" && !message.content.trim().is_empty())
        .map(|message| message.content.trim().to_string())
}

fn github_repository_context_content(repository: &ProjectGitHubRepositoryRecord) -> String {
    format!(
        "Repository: {}\nURL: {}\nDefault branch: {}\nVisibility: {}\nLast fetched: {}\nFetch status: {}",
        repository.repository_full_name,
        repository.repository_url,
        repository.default_branch,
        repository.visibility,
        repository.last_fetched_at,
        repository.last_fetch_status
    )
}

fn bridge_context_type_label(context_type: &str) -> &'static str {
    match context_type {
        "project" => "Project",
        "github_repository" => "GitHub Repository",
        "note" => "Note",
        "task" => "Task",
        "calendar_event" => "Calendar Event",
        "youtube_reference" => "YouTube Reference",
        "scratchpad" => "Scratchpad",
        _ => "Context",
    }
}

fn planning_context_dedupe_key(context: &PlanningConversationContextRecord) -> String {
    if context.context_type == "github_repository" {
        return format!(
            "{}:label:{}",
            context.context_type,
            context.label.trim().to_lowercase()
        );
    }

    match context.source_id {
        Some(source_id) => format!("{}:source:{}", context.context_type, source_id),
        None => format!(
            "{}:label:{}",
            context.context_type,
            context.label.trim().to_lowercase()
        ),
    }
}

fn youtube_reference_from_row(row: &rusqlite::Row<'_>) -> Result<YouTubeReferenceRecord> {
    Ok(YouTubeReferenceRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        video_id: row.get(3)?,
        channel_name: row.get(4)?,
        notes: row.get(5)?,
        tags: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn game_from_row(row: &rusqlite::Row<'_>) -> Result<GameRecord> {
    Ok(GameRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        summary: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn game_catalog_object_from_row(row: &rusqlite::Row<'_>) -> Result<GameCatalogObjectRecord> {
    Ok(GameCatalogObjectRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        name: row.get(2)?,
        object_type: row.get(3)?,
        category: row.get(4)?,
        category_icon: row.get(5)?,
        category_icon_path: row.get(6)?,
        description: row.get(7)?,
        notes: row.get(8)?,
        tags: row.get(9)?,
        thumbnail_path: row.get(10)?,
        source_screenshot_path: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn game_screenshot_capture_request_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<GameScreenshotCaptureRequestRecord> {
    Ok(GameScreenshotCaptureRequestRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        title: row.get(2)?,
        file_path: row.get(3)?,
        request_id: row.get(4)?,
        request_path: row.get(5)?,
        capture_status: row.get(6)?,
        captured_at: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn game_chat_conversation_from_row(row: &rusqlite::Row<'_>) -> Result<GameChatConversationRecord> {
    Ok(GameChatConversationRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        title: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn game_chat_message_from_row(row: &rusqlite::Row<'_>) -> Result<GameChatMessageRecord> {
    Ok(GameChatMessageRecord {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn game_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('-');
            previous_was_separator = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "game".to_string()
    } else {
        slug
    }
}
