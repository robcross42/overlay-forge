use super::{
    domain::{Project, ProjectDraft, ProjectQuery},
    seeds,
};
use crate::db::HostDatabase;
use rusqlite::{params, Connection, OptionalExtension};

pub struct ProjectRepository<'a> {
    database: &'a HostDatabase,
}

fn database_error(_: rusqlite::Error) -> String {
    "Could not access the project catalog. Please try again.".into()
}

impl<'a> ProjectRepository<'a> {
    pub fn new(database: &'a HostDatabase) -> Self {
        Self { database }
    }

    pub fn initialize(connection: &Connection) -> rusqlite::Result<()> {
        connection.execute_batch("CREATE TABLE IF NOT EXISTS obj_host_project (
            id TEXT PRIMARY KEY, body TEXT NOT NULL, archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1))
        );
        CREATE TABLE IF NOT EXISTS obj_host_project_seed (
            seed_key TEXT PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TRIGGER IF NOT EXISTS project_delete_requires_archive
        BEFORE DELETE ON obj_host_project WHEN OLD.archived=0
        BEGIN SELECT RAISE(ABORT, 'Archive the project before permanently deleting its tracker record.'); END;")
    }

    pub fn seed_once(&self) -> Result<(), String> {
        let mut connection = self.database.connection().map_err(database_error)?;
        let transaction = connection.transaction().map_err(database_error)?;
        // A separate ledger survives tracker deletion. The complete catalog is one atomic seed.
        let inserted = transaction.execute(
            "INSERT OR IGNORE INTO obj_host_project_seed(seed_key) VALUES('initial-catalog-v1')", [],
        ).map_err(database_error)?;
        if inserted != 0 {
            for (key, draft) in seeds::catalog() {
                let body = serde_json::to_string(&draft.validate()?)
                    .map_err(|_| "Invalid catalog seed.")?;
                transaction
                    .execute(
                        "INSERT OR IGNORE INTO obj_host_project(id,body) VALUES(?1,?2)",
                        params![format!("seed-{key}"), body],
                    )
                    .map_err(database_error)?;
            }
        }
        transaction.commit().map_err(database_error)
    }

    pub fn list(&self, query: &ProjectQuery) -> Result<Vec<Project>, String> {
        if query.search.len() > 2000 {
            return Err("Search is too long.".into());
        }
        self.seed_once()?;
        let connection = self.database.connection().map_err(database_error)?;
        let mut statement = connection
            .prepare("SELECT id,body,archived FROM obj_host_project")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], Self::from_row)
            .map_err(database_error)?;
        let mut projects = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(database_error)?;
        projects.retain(|project| query.matches(project));
        projects.sort_by_cached_key(|project| project.draft.title.to_lowercase());
        Ok(projects)
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
        let body: String = row.get(1)?;
        let draft = serde_json::from_str(&body).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        Ok(Project {
            id: row.get(0)?,
            draft,
            archived: row.get(2)?,
        })
    }

    pub fn save(&self, id: Option<&str>, draft: ProjectDraft) -> Result<String, String> {
        self.seed_once()?;
        let draft = draft.validate()?;
        let body = serde_json::to_string(&draft).map_err(|_| "Could not encode project.")?;
        let connection = self.database.connection().map_err(database_error)?;
        if let Some(id) = id {
            let count = connection
                .execute(
                    "UPDATE obj_host_project SET body=?2 WHERE id=?1",
                    params![id, body],
                )
                .map_err(database_error)?;
            if count == 0 {
                return Err("This project no longer exists. Refresh the list.".into());
            }
            Ok(id.to_owned())
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            connection
                .execute(
                    "INSERT INTO obj_host_project(id,body) VALUES(?1,?2)",
                    params![id, body],
                )
                .map_err(database_error)?;
            Ok(id)
        }
    }

    pub fn set_archived(&self, id: &str, archived: bool) -> Result<(), String> {
        let count = self
            .database
            .connection()
            .map_err(database_error)?
            .execute(
                "UPDATE obj_host_project SET archived=?2 WHERE id=?1",
                params![id, archived],
            )
            .map_err(database_error)?;
        if count == 0 {
            return Err("This project no longer exists. Refresh the list.".into());
        }
        Ok(())
    }

    pub fn delete(&self, id: &str, confirmation: &str) -> Result<(), String> {
        let mut connection = self.database.connection().map_err(database_error)?;
        let transaction = connection.transaction().map_err(database_error)?;
        let project = transaction
            .query_row(
                "SELECT id,body,archived FROM obj_host_project WHERE id=?1",
                [id],
                Self::from_row,
            )
            .optional()
            .map_err(database_error)?
            .ok_or("This project no longer exists. Refresh the list.")?;
        if !project.archived {
            return Err(
                "Archive the project before permanently deleting its tracker record.".into(),
            );
        }
        if confirmation != project.draft.title {
            return Err("Type the project's exact title to confirm deletion.".into());
        }
        transaction
            .execute(
                "DELETE FROM obj_host_project WHERE id=?1 AND archived=1",
                [id],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }
}
