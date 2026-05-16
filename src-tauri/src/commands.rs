use crate::db::{
    CalendarEventRecord, NoteRecord, PlanningConversationRecord, PlanningMessageRecord,
    ProjectGitHubRepositoryRecord, ProjectRecord, TaskRecord,
};
use crate::github;
use crate::openai;
use crate::AppState;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct MilestoneStatus {
    milestone: String,
    hotkey: String,
    #[serde(rename = "databaseReady")]
    database_ready: bool,
}

#[tauri::command]
pub fn get_scratchpad(state: State<'_, AppState>) -> Result<String, String> {
    state
        .database
        .get_scratchpad()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_scratchpad(content: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .save_scratchpad(&content)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_milestone_status(state: State<'_, AppState>) -> Result<MilestoneStatus, String> {
    Ok(MilestoneStatus {
        milestone: "Milestone 4".to_string(),
        hotkey: "Ctrl+Shift+Space".to_string(),
        database_ready: state.database.is_ready(),
    })
}

#[tauri::command]
pub fn shutdown_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<TaskRecord>, String> {
    state
        .database
        .list_tasks()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_task(
    title: String,
    body: String,
    deadline: String,
    state: State<'_, AppState>,
) -> Result<TaskRecord, String> {
    require_text(&title, "Task title")?;
    state
        .database
        .create_task(&title, &body, &deadline)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_task(
    id: i64,
    title: Option<String>,
    body: Option<String>,
    deadline: Option<String>,
    is_completed: Option<bool>,
    state: State<'_, AppState>,
) -> Result<TaskRecord, String> {
    if let Some(next_title) = title.as_ref() {
        require_text(next_title, "Task title")?;
    }

    state
        .database
        .update_task(
            id,
            title.as_deref(),
            body.as_deref(),
            deadline.as_deref(),
            is_completed,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_task(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_task(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<Vec<NoteRecord>, String> {
    state
        .database
        .list_notes()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_note(
    title: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<NoteRecord, String> {
    require_text(&title, "Note title")?;
    state
        .database
        .create_note(&title, &body)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_note(
    id: i64,
    title: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<NoteRecord, String> {
    require_text(&title, "Note title")?;
    state
        .database
        .update_note(id, &title, &body)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_note(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_note(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_calendar_events(
    state: State<'_, AppState>,
) -> Result<Vec<CalendarEventRecord>, String> {
    state
        .database
        .list_calendar_events()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_calendar_event(
    title: String,
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    notes: String,
    state: State<'_, AppState>,
) -> Result<CalendarEventRecord, String> {
    validate_calendar_event(&title, &start_date, &start_time, &end_date, &end_time)?;
    state
        .database
        .create_calendar_event(
            &title,
            &start_date,
            &start_time,
            &end_date,
            &end_time,
            &notes,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_calendar_event(
    id: i64,
    title: String,
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    notes: String,
    state: State<'_, AppState>,
) -> Result<CalendarEventRecord, String> {
    validate_calendar_event(&title, &start_date, &start_time, &end_date, &end_time)?;
    state
        .database
        .update_calendar_event(
            id,
            &title,
            &start_date,
            &start_time,
            &end_date,
            &end_time,
            &notes,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_calendar_event(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_calendar_event(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectRecord>, String> {
    state
        .database
        .list_projects()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_project(
    name: String,
    description: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<ProjectRecord, String> {
    validate_project(&name, &status)?;
    state
        .database
        .create_project(&name, &description, &status)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_project(
    id: i64,
    name: String,
    description: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<ProjectRecord, String> {
    validate_project(&name, &status)?;
    state
        .database
        .update_project(id, &name, &description, &status)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_project(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_project(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_project_github_repository(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<Option<ProjectGitHubRepositoryRecord>, String> {
    state
        .database
        .get_project_github_repository(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_project_github_repository(
    project_id: i64,
    repository_full_name: String,
    state: State<'_, AppState>,
) -> Result<ProjectGitHubRepositoryRecord, String> {
    let normalized = github::normalize_repository_full_name(&repository_full_name)?;
    state
        .database
        .save_project_github_repository(project_id, &normalized)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_project_github_repository(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_project_github_repository(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn fetch_project_github_metadata(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<ProjectGitHubRepositoryRecord, String> {
    let link = state
        .database
        .get_project_github_repository(project_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Link a GitHub repository before fetching metadata.".to_string())?;

    match github::fetch_repository_metadata(&link.repository_full_name).await {
        Ok(metadata) => state
            .database
            .update_project_github_metadata(
                project_id,
                &metadata.repository_full_name,
                &metadata.repository_url,
                &metadata.default_branch,
                &metadata.visibility,
                "Fetched GitHub repository metadata successfully",
            )
            .map_err(|error| error.to_string()),
        Err(error) => {
            let _ = state
                .database
                .update_project_github_fetch_status(project_id, &error);
            Err(error)
        }
    }
}

#[tauri::command]
pub fn list_planning_conversations(
    project_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningConversationRecord>, String> {
    state
        .database
        .list_planning_conversations(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_planning_conversation(
    project_id: i64,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<PlanningConversationRecord, String> {
    state
        .database
        .create_planning_conversation(project_id, title.as_deref().unwrap_or_default())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_planning_messages(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningMessageRecord>, String> {
    state
        .database
        .list_planning_messages(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn send_planning_message(
    conversation_id: i64,
    content: String,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningMessageRecord>, String> {
    require_text(&content, "Message")?;
    let conversation = state
        .database
        .get_planning_conversation(conversation_id)
        .map_err(|error| error.to_string())?;
    let project = state
        .database
        .get_project(conversation.project_id)
        .map_err(|error| error.to_string())?;

    state
        .database
        .create_planning_message(conversation_id, "user", &content)
        .map_err(|error| error.to_string())?;

    let recent_messages = state
        .database
        .recent_planning_messages(conversation_id, 20)
        .map_err(|error| error.to_string())?;
    let assistant_content = openai::create_planning_response(&project, &recent_messages).await?;

    state
        .database
        .create_planning_message(conversation_id, "assistant", &assistant_content)
        .map_err(|error| error.to_string())?;

    state
        .database
        .list_planning_messages(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_planning_conversation(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_planning_conversation(conversation_id)
        .map_err(|error| error.to_string())
}

fn validate_calendar_event(
    title: &str,
    start_date: &str,
    start_time: &str,
    end_date: &str,
    end_time: &str,
) -> Result<(), String> {
    require_text(title, "Event title")?;
    require_text(start_date, "Start date")?;
    require_text(start_time, "Start time")?;
    require_text(end_date, "End date")?;
    require_text(end_time, "End time")?;
    Ok(())
}

fn validate_project(name: &str, status: &str) -> Result<(), String> {
    require_text(name, "Project name")?;
    match status.trim() {
        "ACTIVE" | "ARCHIVED" => Ok(()),
        _ => Err("Project status must be ACTIVE or ARCHIVED".to_string()),
    }
}

fn require_text(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field_name} is required"));
    }

    Ok(())
}
