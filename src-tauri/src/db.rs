use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
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
                context_type.trim(),
                source_id,
                label.trim()
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

        Ok(context)
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
