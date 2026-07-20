use crate::gearblocks_api;
use crate::gearblocks_api_scraper::{GearBlocksApiImportResult, GearBlocksApiScrape};
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

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

#[derive(Clone, Serialize)]
pub struct SmokingEventRecord {
    pub id: i64,
    #[serde(rename = "smokedAt")]
    pub smoked_at: String,
    pub source: String,
    pub notes: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Serialize)]
pub struct SmokingCessationSettingsRecord {
    pub id: i64,
    #[serde(rename = "patchLabel")]
    pub patch_label: String,
    #[serde(rename = "patchStartedAt")]
    pub patch_started_at: String,
    #[serde(rename = "patchTimezone")]
    pub patch_timezone: String,
    #[serde(rename = "currentCigaretteCount")]
    pub current_cigarette_count: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct SchedulerRecord {
    pub id: i64,
    #[serde(rename = "typeId")]
    pub type_id: i64,
    #[serde(rename = "typeKey")]
    pub type_key: String,
    #[serde(rename = "typeLabel")]
    pub type_label: String,
    #[serde(rename = "ownerModule")]
    pub owner_module: String,
    pub name: String,
    #[serde(rename = "isEnabled")]
    pub is_enabled: bool,
    #[serde(rename = "intervalSeconds")]
    pub interval_seconds: i64,
    #[serde(rename = "runOnStartup")]
    pub run_on_startup: bool,
    #[serde(rename = "coalesceMissedRuns")]
    pub coalesce_missed_runs: bool,
    #[serde(rename = "payloadJson")]
    pub payload_json: String,
    #[serde(rename = "nextRunAt")]
    pub next_run_at: String,
    #[serde(rename = "lastRunAt")]
    pub last_run_at: String,
    #[serde(rename = "lastStatus")]
    pub last_status: String,
    #[serde(rename = "lastError")]
    pub last_error: String,
    #[serde(rename = "leaseUntil")]
    pub lease_until: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "modifiedAt")]
    pub modified_at: String,
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
pub struct GameDataLocationRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "locationType")]
    pub location_type: String,
    pub label: String,
    #[serde(rename = "directoryPath")]
    pub directory_path: String,
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

#[derive(Clone, Serialize)]
pub struct GameRuntimePartRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "partKey")]
    pub part_key: String,
    #[serde(rename = "assetGuid")]
    pub asset_guid: String,
    #[serde(rename = "assetName")]
    pub asset_name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "fullDisplayName")]
    pub full_display_name: String,
    pub category: String,
    pub mass: f64,
    #[serde(rename = "worldX")]
    pub world_x: Option<f64>,
    #[serde(rename = "worldY")]
    pub world_y: Option<f64>,
    #[serde(rename = "worldZ")]
    pub world_z: Option<f64>,
    #[serde(rename = "localX")]
    pub local_x: Option<f64>,
    #[serde(rename = "localY")]
    pub local_y: Option<f64>,
    #[serde(rename = "localZ")]
    pub local_z: Option<f64>,
    #[serde(rename = "worldPositionJson")]
    pub world_position_json: String,
    #[serde(rename = "localPositionJson")]
    pub local_position_json: String,
    #[serde(rename = "propertiesJson")]
    pub properties_json: String,
    #[serde(rename = "sourceExportId")]
    pub source_export_id: String,
    #[serde(rename = "sourceConstructionId")]
    pub source_construction_id: String,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    #[serde(rename = "displayImagePath")]
    pub display_image_path: String,
    #[serde(rename = "sourceImagePath")]
    pub source_image_path: String,
    pub notes: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameRuntimePartAliasRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "partInstanceKey")]
    pub part_instance_key: String,
    #[serde(rename = "friendlyName")]
    pub friendly_name: String,
    #[serde(rename = "assetGuid")]
    pub asset_guid: String,
    #[serde(rename = "assetName")]
    pub asset_name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "fullDisplayName")]
    pub full_display_name: String,
    pub category: String,
    #[serde(rename = "sourceLogPath")]
    pub source_log_path: String,
    #[serde(rename = "sourceConstructionId")]
    pub source_construction_id: String,
    #[serde(rename = "worldPositionJson")]
    pub world_position_json: String,
    #[serde(rename = "localPositionJson")]
    pub local_position_json: String,
    #[serde(rename = "currentUnitSizeJson")]
    pub current_unit_size_json: String,
    #[serde(rename = "payloadJson")]
    pub payload_json: String,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameRuntimeConstructionExportRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "exportId")]
    pub export_id: String,
    pub name: String,
    #[serde(rename = "exportKind")]
    pub export_kind: String,
    #[serde(rename = "intendedPath")]
    pub intended_path: String,
    #[serde(rename = "sourceLogPath")]
    pub source_log_path: String,
    #[serde(rename = "byteSize")]
    pub byte_size: i64,
    #[serde(rename = "constructionId")]
    pub construction_id: String,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    #[serde(rename = "partCount")]
    pub part_count: i64,
    pub mass: f64,
    #[serde(rename = "isFrozen")]
    pub is_frozen: Option<bool>,
    #[serde(rename = "isInvulnerable")]
    pub is_invulnerable: Option<bool>,
    #[serde(rename = "isPlayerCharacter")]
    pub is_player_character: Option<bool>,
    #[serde(rename = "documentJson")]
    pub document_json: String,
    #[serde(rename = "lastIndexedAt")]
    pub last_indexed_at: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GearBlocksApiTypeRecord {
    pub id: i64,
    pub namespace: String,
    #[serde(rename = "typeName")]
    pub type_name: String,
    #[serde(rename = "typeKind")]
    pub type_kind: String,
    #[serde(rename = "docsUrl")]
    pub docs_url: String,
    pub source: String,
    #[serde(rename = "sourceVersion")]
    pub source_version: String,
    pub notes: String,
    #[serde(rename = "memberCount")]
    pub member_count: i64,
    #[serde(rename = "enumValueCount")]
    pub enum_value_count: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GearBlocksApiMemberRecord {
    pub id: i64,
    #[serde(rename = "typeId")]
    pub type_id: i64,
    #[serde(rename = "typeName")]
    pub type_name: String,
    #[serde(rename = "memberKey")]
    pub member_key: String,
    #[serde(rename = "memberName")]
    pub member_name: String,
    pub signature: String,
    #[serde(rename = "memberKind")]
    pub member_kind: String,
    #[serde(rename = "returnType")]
    pub return_type: String,
    #[serde(rename = "isReadable")]
    pub is_readable: bool,
    #[serde(rename = "isWritable")]
    pub is_writable: bool,
    #[serde(rename = "isInvokable")]
    pub is_invokable: bool,
    #[serde(rename = "isMutating")]
    pub is_mutating: bool,
    #[serde(rename = "docsUrl")]
    pub docs_url: String,
    pub source: String,
    #[serde(rename = "sourceVersion")]
    pub source_version: String,
    pub notes: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GearBlocksApiParameterRecord {
    pub id: i64,
    #[serde(rename = "memberId")]
    pub member_id: i64,
    pub position: i64,
    #[serde(rename = "parameterName")]
    pub parameter_name: String,
    #[serde(rename = "parameterType")]
    pub parameter_type: String,
    #[serde(rename = "defaultValue")]
    pub default_value: String,
    #[serde(rename = "isOptional")]
    pub is_optional: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GearBlocksApiEnumValueRecord {
    pub id: i64,
    #[serde(rename = "typeId")]
    pub type_id: i64,
    pub position: i64,
    #[serde(rename = "valueName")]
    pub value_name: String,
    #[serde(rename = "numericValue")]
    pub numeric_value: String,
    #[serde(rename = "luaName")]
    pub lua_name: String,
    pub description: String,
    pub source: String,
    #[serde(rename = "sourceVersion")]
    pub source_version: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksApiCatalogRecord {
    pub types: Vec<GearBlocksApiTypeRecord>,
    pub members: Vec<GearBlocksApiMemberRecord>,
    pub parameters: Vec<GearBlocksApiParameterRecord>,
    pub enum_values: Vec<GearBlocksApiEnumValueRecord>,
}

#[derive(Clone, Serialize)]
pub struct GameRuntimePartApiMemberRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "partKey")]
    pub part_key: String,
    #[serde(rename = "apiMemberId")]
    pub api_member_id: i64,
    pub availability: String,
    #[serde(rename = "sourceExportId")]
    pub source_export_id: String,
    #[serde(rename = "sourceConstructionId")]
    pub source_construction_id: String,
    #[serde(rename = "firstSeenAt")]
    pub first_seen_at: String,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    pub namespace: String,
    #[serde(rename = "typeName")]
    pub type_name: String,
    #[serde(rename = "typeKind")]
    pub type_kind: String,
    #[serde(rename = "memberKey")]
    pub member_key: String,
    #[serde(rename = "memberName")]
    pub member_name: String,
    pub signature: String,
    #[serde(rename = "memberKind")]
    pub member_kind: String,
    #[serde(rename = "isReadable")]
    pub is_readable: bool,
    #[serde(rename = "isWritable")]
    pub is_writable: bool,
    #[serde(rename = "isInvokable")]
    pub is_invokable: bool,
    #[serde(rename = "isMutating")]
    pub is_mutating: bool,
    #[serde(rename = "docsUrl")]
    pub docs_url: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameConstructionRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    pub name: String,
    #[serde(rename = "folderPath")]
    pub folder_path: String,
    #[serde(rename = "constructionPath")]
    pub construction_path: String,
    #[serde(rename = "byteSize")]
    pub byte_size: i64,
    #[serde(rename = "decodedByteSize")]
    pub decoded_byte_size: i64,
    #[serde(rename = "compositeCount")]
    pub composite_count: i64,
    #[serde(rename = "partCount")]
    pub part_count: i64,
    #[serde(rename = "uniqueAssetGuidCount")]
    pub unique_asset_guid_count: i64,
    #[serde(rename = "attachmentCount")]
    pub attachment_count: i64,
    #[serde(rename = "linkCount")]
    pub link_count: i64,
    #[serde(rename = "intersectionCount")]
    pub intersection_count: i64,
    #[serde(rename = "isFrozen")]
    pub is_frozen: Option<bool>,
    #[serde(rename = "isInvulnerable")]
    pub is_invulnerable: Option<bool>,
    #[serde(rename = "summaryJson")]
    pub summary_json: String,
    #[serde(rename = "documentJson")]
    pub document_json: String,
    #[serde(rename = "lastIndexedAt")]
    pub last_indexed_at: String,
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
    #[serde(rename = "overlayX")]
    pub overlay_x: Option<i32>,
    #[serde(rename = "overlayY")]
    pub overlay_y: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameBuildGuideRecord {
    pub id: i64,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    pub title: String,
    #[serde(rename = "sourcePath")]
    pub source_path: String,
    #[serde(rename = "rawMarkdown")]
    pub raw_markdown: String,
    #[serde(rename = "buildGoal")]
    pub build_goal: String,
    #[serde(rename = "scaleReference")]
    pub scale_reference: String,
    #[serde(rename = "geometryNotes")]
    pub geometry_notes: String,
    #[serde(rename = "glossaryText")]
    pub glossary_text: String,
    #[serde(rename = "checklistJson")]
    pub checklist_json: String,
    #[serde(rename = "overlayX")]
    pub overlay_x: Option<i32>,
    #[serde(rename = "overlayY")]
    pub overlay_y: Option<i32>,
    #[serde(rename = "overlayWidth")]
    pub overlay_width: Option<i32>,
    #[serde(rename = "overlayHeight")]
    pub overlay_height: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameBuildGuidePartRecord {
    pub id: i64,
    #[serde(rename = "guideId")]
    pub guide_id: i64,
    pub section: String,
    pub quantity: String,
    #[serde(rename = "partName")]
    pub part_name: String,
    pub purpose: String,
    #[serde(rename = "rowOrder")]
    pub row_order: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Serialize)]
pub struct GameBuildGuideStepRecord {
    pub id: i64,
    #[serde(rename = "guideId")]
    pub guide_id: i64,
    #[serde(rename = "stepNumber")]
    pub step_number: i64,
    pub title: String,
    pub body: String,
    #[serde(rename = "rowOrder")]
    pub row_order: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

pub struct GameBuildGuidePartDraft {
    pub section: String,
    pub quantity: String,
    pub part_name: String,
    pub purpose: String,
    pub row_order: i64,
}

pub struct GameBuildGuideStepDraft {
    pub step_number: i64,
    pub title: String,
    pub body: String,
    pub row_order: i64,
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
        Self::migrate_legacy_table_names(&connection)?;
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS obj_scratchpad (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            INSERT OR IGNORE INTO obj_scratchpad (id, content) VALUES (1, '');

            CREATE TABLE IF NOT EXISTS obj_setting (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_task (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                deadline TEXT NOT NULL DEFAULT '',
                is_completed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_note (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_calendar_event (
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

            CREATE TABLE IF NOT EXISTS obj_smoking_event (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                smoked_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                source TEXT NOT NULL DEFAULT 'manual',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_smoking_cessation_setting (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                patch_label TEXT NOT NULL DEFAULT '',
                patch_started_at TEXT NOT NULL DEFAULT '',
                patch_timezone TEXT NOT NULL DEFAULT '',
                current_cigarette_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            INSERT OR IGNORE INTO obj_smoking_cessation_setting (
                id,
                patch_label,
                patch_started_at,
                patch_timezone
            )
            VALUES (1, 'Nicoderm Step 1', '2026-06-21 15:00:00', 'EDT');

            CREATE TABLE IF NOT EXISTS def_scheduler_type (
                id INTEGER PRIMARY KEY,
                scheduler_key TEXT NOT NULL UNIQUE,
                label TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                modified_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_scheduler (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                id_type INTEGER NOT NULL,
                owner_module TEXT NOT NULL,
                name TEXT NOT NULL,
                is_enabled INTEGER NOT NULL DEFAULT 1,
                interval_seconds INTEGER NOT NULL DEFAULT 60,
                run_on_startup INTEGER NOT NULL DEFAULT 1,
                coalesce_missed_runs INTEGER NOT NULL DEFAULT 1,
                payload_json TEXT NOT NULL DEFAULT '{}',
                next_run_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                last_run_at TEXT NOT NULL DEFAULT '',
                last_status TEXT NOT NULL DEFAULT '',
                last_error TEXT NOT NULL DEFAULT '',
                lease_until TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                modified_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (id_type) REFERENCES def_scheduler_type(id)
            );

            CREATE TABLE IF NOT EXISTS obj_scheduler_run (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                id_scheduler INTEGER NOT NULL,
                id_type INTEGER NOT NULL,
                started_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                finished_at TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'running',
                message TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                modified_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (id_scheduler) REFERENCES obj_scheduler(id),
                FOREIGN KEY (id_type) REFERENCES def_scheduler_type(id)
            );

            INSERT OR IGNORE INTO def_scheduler_type (
                id,
                scheduler_key,
                label,
                description
            )
            VALUES (
                1,
                'smoking_cessation_export',
                'Smoking Cessation Export',
                'Refreshes the Smoking Cessation ChatGPT Markdown export and derived time-sensitive context.'
            );

            INSERT OR IGNORE INTO obj_scheduler (
                id_type,
                owner_module,
                name,
                interval_seconds,
                run_on_startup,
                coalesce_missed_runs,
                payload_json,
                next_run_at
            )
            VALUES (
                1,
                'smoking_cessation',
                'Refresh Smoking Cessation ChatGPT export',
                60,
                1,
                1,
                '{}',
                datetime('now', 'localtime')
            );

            CREATE TABLE IF NOT EXISTS obj_project (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'ACTIVE',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_planning_conversation (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT 'Planning conversation',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_planning_message (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS n2n_planning_conversation_context (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                context_type TEXT NOT NULL,
                source_id INTEGER,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_project_github_repository (
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

            CREATE TABLE IF NOT EXISTS obj_project_markdown_context (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL UNIQUE,
                root_path TEXT NOT NULL,
                readme_path TEXT NOT NULL DEFAULT 'README.md',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_youtube_reference (
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

            CREATE TABLE IF NOT EXISTS obj_bridge_file_draft (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                conversation_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS def_game (
                id_game INTEGER PRIMARY KEY,
                game_key TEXT NOT NULL UNIQUE COLLATE NOCASE,
                ui_name TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                schema_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                modified_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                id_game INTEGER NOT NULL DEFAULT 1,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                slug TEXT NOT NULL UNIQUE COLLATE NOCASE,
                summary TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (id_game) REFERENCES def_game(id_game)
            );

            CREATE TABLE IF NOT EXISTS obj_game_setting (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                id_game INTEGER NOT NULL DEFAULT 1,
                setting_key TEXT NOT NULL,
                setting_value_json TEXT NOT NULL DEFAULT 'null',
                schema_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                modified_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (game_id) REFERENCES obj_game(id),
                FOREIGN KEY (id_game) REFERENCES def_game(id_game)
            );

            CREATE TABLE IF NOT EXISTS obj_game_catalog_object (
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

            CREATE TABLE IF NOT EXISTS obj_game_catalog_reference (
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

            CREATE TABLE IF NOT EXISTS obj_game_data_location (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                location_type TEXT NOT NULL,
                label TEXT NOT NULL DEFAULT '',
                directory_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_catalog_screenshot (
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

            CREATE TABLE IF NOT EXISTS obj_game_runtime_part (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_key TEXT NOT NULL,
                asset_guid TEXT NOT NULL DEFAULT '',
                asset_name TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                full_display_name TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                mass REAL NOT NULL DEFAULT 0,
                world_x REAL,
                world_y REAL,
                world_z REAL,
                local_x REAL,
                local_y REAL,
                local_z REAL,
                world_position_json TEXT NOT NULL DEFAULT '{}',
                local_position_json TEXT NOT NULL DEFAULT '{}',
                properties_json TEXT NOT NULL DEFAULT '{}',
                source_export_id TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                last_seen_at TEXT NOT NULL DEFAULT '',
                display_image_path TEXT NOT NULL DEFAULT '',
                source_image_path TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_runtime_part_alias (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_instance_key TEXT NOT NULL,
                friendly_name TEXT NOT NULL,
                asset_guid TEXT NOT NULL DEFAULT '',
                asset_name TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                full_display_name TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                source_log_path TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                world_position_json TEXT NOT NULL DEFAULT '{}',
                local_position_json TEXT NOT NULL DEFAULT '{}',
                current_unit_size_json TEXT NOT NULL DEFAULT '{}',
                payload_json TEXT NOT NULL DEFAULT '{}',
                last_seen_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_runtime_part_api_attribute (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_key TEXT NOT NULL,
                asset_guid TEXT NOT NULL DEFAULT '',
                asset_name TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                full_display_name TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                interface_name TEXT NOT NULL DEFAULT '',
                attribute_name TEXT NOT NULL DEFAULT '',
                value_type TEXT NOT NULL DEFAULT '',
                availability TEXT NOT NULL DEFAULT '',
                source_export_id TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                first_seen_at TEXT NOT NULL DEFAULT '',
                last_seen_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_runtime_part_value (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_key TEXT NOT NULL,
                asset_guid TEXT NOT NULL DEFAULT '',
                asset_name TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                full_display_name TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                field_path TEXT NOT NULL DEFAULT '',
                value_type TEXT NOT NULL DEFAULT '',
                value_json TEXT NOT NULL DEFAULT 'null',
                source_export_id TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                first_seen_at TEXT NOT NULL DEFAULT '',
                last_seen_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_runtime_part_property (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_key TEXT NOT NULL,
                asset_guid TEXT NOT NULL DEFAULT '',
                asset_name TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                full_display_name TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                property_path TEXT NOT NULL DEFAULT '',
                value_type TEXT NOT NULL DEFAULT '',
                value_json TEXT NOT NULL DEFAULT 'null',
                source_export_id TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                first_seen_at TEXT NOT NULL DEFAULT '',
                last_seen_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_runtime_part_attachment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_key TEXT NOT NULL,
                asset_guid TEXT NOT NULL DEFAULT '',
                asset_name TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                full_display_name TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                attachment_path TEXT NOT NULL DEFAULT '',
                value_type TEXT NOT NULL DEFAULT '',
                attachment_json TEXT NOT NULL DEFAULT 'null',
                source_export_id TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                first_seen_at TEXT NOT NULL DEFAULT '',
                last_seen_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS def_gearblocks_api_type (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                namespace TEXT NOT NULL,
                type_name TEXT NOT NULL,
                type_kind TEXT NOT NULL DEFAULT 'interface',
                docs_url TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                source_version TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS def_gearblocks_api_member (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type_id INTEGER NOT NULL,
                member_key TEXT NOT NULL,
                member_name TEXT NOT NULL,
                signature TEXT NOT NULL DEFAULT '',
                member_kind TEXT NOT NULL DEFAULT '',
                return_type TEXT NOT NULL DEFAULT '',
                is_readable INTEGER NOT NULL DEFAULT 0,
                is_writable INTEGER NOT NULL DEFAULT 0,
                is_invokable INTEGER NOT NULL DEFAULT 0,
                is_mutating INTEGER NOT NULL DEFAULT 0,
                docs_url TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                source_version TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS def_gearblocks_api_parameter (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                member_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                parameter_name TEXT NOT NULL DEFAULT '',
                parameter_type TEXT NOT NULL DEFAULT '',
                default_value TEXT NOT NULL DEFAULT '',
                is_optional INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS def_gearblocks_api_enum_value (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                value_name TEXT NOT NULL,
                numeric_value TEXT NOT NULL DEFAULT '',
                lua_name TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                source_version TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS n2n_game_runtime_part_api_member (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                part_key TEXT NOT NULL,
                api_member_id INTEGER NOT NULL,
                availability TEXT NOT NULL DEFAULT '',
                source_export_id TEXT NOT NULL DEFAULT '',
                source_construction_id TEXT NOT NULL DEFAULT '',
                first_seen_at TEXT NOT NULL DEFAULT '',
                last_seen_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_construction (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                folder_path TEXT NOT NULL,
                construction_path TEXT NOT NULL,
                byte_size INTEGER NOT NULL DEFAULT 0,
                decoded_byte_size INTEGER NOT NULL DEFAULT 0,
                composite_count INTEGER NOT NULL DEFAULT 0,
                part_count INTEGER NOT NULL DEFAULT 0,
                unique_asset_guid_count INTEGER NOT NULL DEFAULT 0,
                attachment_count INTEGER NOT NULL DEFAULT 0,
                link_count INTEGER NOT NULL DEFAULT 0,
                intersection_count INTEGER NOT NULL DEFAULT 0,
                is_frozen INTEGER,
                is_invulnerable INTEGER,
                summary_json TEXT NOT NULL DEFAULT '{}',
                document_json TEXT NOT NULL DEFAULT '{}',
                last_indexed_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_runtime_construction_export (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                export_id TEXT NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                export_kind TEXT NOT NULL DEFAULT '',
                intended_path TEXT NOT NULL DEFAULT '',
                source_log_path TEXT NOT NULL DEFAULT '',
                byte_size INTEGER NOT NULL DEFAULT 0,
                construction_id TEXT NOT NULL DEFAULT '',
                exported_at TEXT NOT NULL DEFAULT '',
                part_count INTEGER NOT NULL DEFAULT 0,
                mass REAL NOT NULL DEFAULT 0,
                is_frozen INTEGER,
                is_invulnerable INTEGER,
                is_player_character INTEGER,
                document_json TEXT NOT NULL DEFAULT '{}',
                last_indexed_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_chat_conversation (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT 'Game chat',
                overlay_x INTEGER,
                overlay_y INTEGER,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_chat_message (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_build_guide (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT 'Build guide',
                source_path TEXT NOT NULL DEFAULT '',
                raw_markdown TEXT NOT NULL DEFAULT '',
                build_goal TEXT NOT NULL DEFAULT '',
                scale_reference TEXT NOT NULL DEFAULT '',
                geometry_notes TEXT NOT NULL DEFAULT '',
                glossary_text TEXT NOT NULL DEFAULT '',
                checklist_json TEXT NOT NULL DEFAULT '[]',
                overlay_x INTEGER,
                overlay_y INTEGER,
                overlay_width INTEGER,
                overlay_height INTEGER,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_build_guide_part (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                guide_id INTEGER NOT NULL,
                section TEXT NOT NULL DEFAULT '',
                quantity TEXT NOT NULL DEFAULT '',
                part_name TEXT NOT NULL DEFAULT '',
                purpose TEXT NOT NULL DEFAULT '',
                row_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS obj_game_build_guide_step (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                guide_id INTEGER NOT NULL,
                step_number INTEGER NOT NULL DEFAULT 0,
                title TEXT NOT NULL DEFAULT '',
                body TEXT NOT NULL DEFAULT '',
                row_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_game_catalog_objects_game_id
                ON obj_game_catalog_object (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_objects_game_name
                ON obj_game_catalog_object (game_id, name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_references_game_id
                ON obj_game_catalog_reference (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_references_object_id
                ON obj_game_catalog_reference (object_id);
            CREATE INDEX IF NOT EXISTS idx_game_data_locations_game_id
                ON obj_game_data_location (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_screenshots_game_id
                ON obj_game_catalog_screenshot (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_catalog_screenshots_object_id
                ON obj_game_catalog_screenshot (object_id);
            CREATE INDEX IF NOT EXISTS idx_game_chat_conversations_game_id
                ON obj_game_chat_conversation (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_chat_messages_conversation_id
                ON obj_game_chat_message (conversation_id);
            CREATE INDEX IF NOT EXISTS idx_game_build_guides_game_id
                ON obj_game_build_guide (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_build_guide_parts_guide_id
                ON obj_game_build_guide_part (guide_id);
            CREATE INDEX IF NOT EXISTS idx_game_build_guide_steps_guide_id
                ON obj_game_build_guide_step (guide_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_parts_game_id
                ON obj_game_runtime_part (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_aliases_game_id
                ON obj_game_runtime_part_alias (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_api_attributes_game_id
                ON obj_game_runtime_part_api_attribute (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_api_attributes_interface
                ON obj_game_runtime_part_api_attribute (game_id, interface_name, attribute_name);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_values_game_id
                ON obj_game_runtime_part_value (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_values_field_path
                ON obj_game_runtime_part_value (game_id, field_path);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_properties_game_id
                ON obj_game_runtime_part_property (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_properties_property_path
                ON obj_game_runtime_part_property (game_id, property_path);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_attachments_game_id
                ON obj_game_runtime_part_attachment (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_attachments_attachment_path
                ON obj_game_runtime_part_attachment (game_id, attachment_path);
            CREATE INDEX IF NOT EXISTS idx_gearblocks_api_types_namespace
                ON def_gearblocks_api_type (namespace, type_name);
            CREATE INDEX IF NOT EXISTS idx_gearblocks_api_members_type_id
                ON def_gearblocks_api_member (type_id);
            CREATE INDEX IF NOT EXISTS idx_gearblocks_api_members_member_name
                ON def_gearblocks_api_member (member_name);
            CREATE INDEX IF NOT EXISTS idx_gearblocks_api_parameters_member_id
                ON def_gearblocks_api_parameter (member_id);
            CREATE INDEX IF NOT EXISTS idx_gearblocks_api_enum_values_type_id
                ON def_gearblocks_api_enum_value (type_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_api_members_game_id
                ON n2n_game_runtime_part_api_member (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_part_api_members_api_member_id
                ON n2n_game_runtime_part_api_member (api_member_id);
            CREATE INDEX IF NOT EXISTS idx_game_constructions_game_id
                ON obj_game_construction (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_construction_exports_game_id
                ON obj_game_runtime_construction_export (game_id);
            CREATE INDEX IF NOT EXISTS idx_game_runtime_construction_exports_exported_at
                ON obj_game_runtime_construction_export (game_id, exported_at);
            CREATE INDEX IF NOT EXISTS idx_obj_scheduler_due
                ON obj_scheduler (is_enabled, next_run_at);
            CREATE INDEX IF NOT EXISTS idx_obj_scheduler_type
                ON obj_scheduler (id_type);
            CREATE INDEX IF NOT EXISTS idx_obj_scheduler_run_scheduler
                ON obj_scheduler_run (id_scheduler);
            CREATE INDEX IF NOT EXISTS idx_obj_scheduler_run_started
                ON obj_scheduler_run (started_at);

            INSERT OR IGNORE INTO def_game (id_game, game_key, ui_name, summary, schema_json)
            VALUES (
                1,
                'gearblocks',
                'GearBlocks',
                'Supported GearBlocks game module definition.',
                '{\"fields\":{\"id_game\":\"stable integer game definition id\",\"game_key\":\"stable lowercase game key\",\"ui_name\":\"visible game definition name\",\"summary\":\"definition summary\"}}'
            );

            INSERT OR IGNORE INTO obj_game (name, slug, summary)
            VALUES (
                'GearBlocks',
                'gearblocks',
                'Game-specific workspace section for GearBlocks planning, object cataloging, references, and screenshots.'
            );

            INSERT OR IGNORE INTO def_game (id_game, game_key, ui_name, summary, schema_json)
            VALUES (
                2,
                'path-of-exile-2',
                'Path of Exile 2',
                'Supported Path of Exile 2 game module definition.',
                '{\"fields\":{\"id_game\":\"stable integer game definition id\",\"game_key\":\"stable lowercase game key\",\"ui_name\":\"visible game definition name\",\"summary\":\"definition summary\"}}'
            );

            INSERT OR IGNORE INTO obj_game (name, slug, summary)
            VALUES (
                'Path of Exile 2',
                'path-of-exile-2',
                'Game-specific workspace section for Path of Exile 2 chats, builds, passive planning, item tracking, gems, loot filters, and trade.'
            );
            ",
        )?;
        Self::ensure_column(&connection, "obj_task", "body", "TEXT NOT NULL DEFAULT ''")?;
        Self::ensure_column(
            &connection,
            "obj_task",
            "deadline",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game",
            "id_game",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        connection.execute(
            "UPDATE obj_game SET id_game = 2 WHERE slug = 'path-of-exile-2'",
            [],
        )?;
        Self::ensure_column(
            &connection,
            "obj_smoking_cessation_setting",
            "current_cigarette_count",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_screenshot",
            "request_id",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_screenshot",
            "request_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_screenshot",
            "capture_status",
            "TEXT NOT NULL DEFAULT 'captured'",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_object",
            "category_icon",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_object",
            "category_icon_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_object",
            "thumbnail_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_catalog_object",
            "source_screenshot_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(&connection, "obj_game_runtime_part", "world_x", "REAL")?;
        Self::ensure_column(&connection, "obj_game_runtime_part", "world_y", "REAL")?;
        Self::ensure_column(&connection, "obj_game_runtime_part", "world_z", "REAL")?;
        Self::ensure_column(&connection, "obj_game_runtime_part", "local_x", "REAL")?;
        Self::ensure_column(&connection, "obj_game_runtime_part", "local_y", "REAL")?;
        Self::ensure_column(&connection, "obj_game_runtime_part", "local_z", "REAL")?;
        Self::ensure_column(
            &connection,
            "obj_game_runtime_part",
            "world_position_json",
            "TEXT NOT NULL DEFAULT '{}'",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_runtime_part",
            "local_position_json",
            "TEXT NOT NULL DEFAULT '{}'",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_runtime_part",
            "display_image_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_runtime_part",
            "source_image_path",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_runtime_part",
            "notes",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::backfill_game_runtime_part_positions(&connection)?;
        Self::ensure_column(
            &connection,
            "obj_game_chat_conversation",
            "overlay_x",
            "INTEGER",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_chat_conversation",
            "overlay_y",
            "INTEGER",
        )?;
        Self::ensure_column(
            &connection,
            "obj_game_build_guide",
            "glossary_text",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_schema_metadata_columns(&connection)?;
        Self::ensure_modified_at_triggers(&connection)?;
        Self::refresh_schema_json_metadata(&connection)?;
        connection.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_game_catalog_screenshots_request_id
                ON obj_game_catalog_screenshot (request_id)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_catalog_objects_game_name_unique
                ON obj_game_catalog_object (game_id, name COLLATE NOCASE)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_obj_game_setting_game_key_unique
                ON obj_game_setting (game_id, setting_key)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_catalog_objects_game_name_exact_unique
                ON obj_game_catalog_object (game_id, name)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_data_locations_game_type_unique
                ON obj_game_data_location (game_id, location_type)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_parts_game_part_key_unique
                ON obj_game_runtime_part (game_id, part_key)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_part_aliases_game_instance_unique
                ON obj_game_runtime_part_alias (game_id, part_instance_key)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_part_api_attributes_unique
                ON obj_game_runtime_part_api_attribute (
                    game_id,
                    part_key,
                    interface_name,
                    attribute_name
                )
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_part_values_unique
                ON obj_game_runtime_part_value (game_id, part_key, field_path)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_part_properties_unique
                ON obj_game_runtime_part_property (game_id, part_key, property_path)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_part_attachments_unique
                ON obj_game_runtime_part_attachment (game_id, part_key, attachment_path)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_gearblocks_api_types_unique
                ON def_gearblocks_api_type (namespace, type_name)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_gearblocks_api_members_unique
                ON def_gearblocks_api_member (type_id, member_key)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_gearblocks_api_parameters_unique
                ON def_gearblocks_api_parameter (member_id, position)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_gearblocks_api_enum_values_unique
                ON def_gearblocks_api_enum_value (type_id, value_name)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_part_api_members_unique
                ON n2n_game_runtime_part_api_member (game_id, part_key, api_member_id)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_constructions_game_path_unique
                ON obj_game_construction (game_id, construction_path)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_game_runtime_construction_exports_game_export_unique
                ON obj_game_runtime_construction_export (game_id, export_id)
            ",
            [],
        )?;
        connection.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_obj_scheduler_owner_name_unique
                ON obj_scheduler (owner_module, name)
            ",
            [],
        )?;
        crate::media::repository::migrate_schema(&connection)?;
        Self::seed_gearblocks_api_catalog(&connection)?;

        Ok(Self {
            connection: Mutex::new(connection),
            ready: true,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub(crate) fn connection(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)
    }

    fn migrate_legacy_table_names(connection: &Connection) -> Result<()> {
        let table_migrations = [
            ("scratchpad", "obj_scratchpad"),
            ("app_settings", "obj_setting"),
            ("tasks", "obj_task"),
            ("notes", "obj_note"),
            ("calendar_events", "obj_calendar_event"),
            ("smoking_events", "obj_smoking_event"),
            (
                "smoking_cessation_settings",
                "obj_smoking_cessation_setting",
            ),
            ("projects", "obj_project"),
            ("planning_conversations", "obj_planning_conversation"),
            ("planning_messages", "obj_planning_message"),
            (
                "planning_conversation_context",
                "n2n_planning_conversation_context",
            ),
            (
                "project_github_repositories",
                "obj_project_github_repository",
            ),
            ("project_markdown_context", "obj_project_markdown_context"),
            ("youtube_references", "obj_youtube_reference"),
            ("bridge_file_drafts", "obj_bridge_file_draft"),
            ("games", "obj_game"),
            ("game_catalog_objects", "obj_game_catalog_object"),
            ("game_catalog_references", "obj_game_catalog_reference"),
            ("game_data_locations", "obj_game_data_location"),
            ("game_catalog_screenshots", "obj_game_catalog_screenshot"),
            ("game_runtime_parts", "obj_game_runtime_part"),
            (
                "game_runtime_part_api_attributes",
                "obj_game_runtime_part_api_attribute",
            ),
            ("game_runtime_part_values", "obj_game_runtime_part_value"),
            (
                "game_runtime_part_properties",
                "obj_game_runtime_part_property",
            ),
            (
                "game_runtime_part_attachments",
                "obj_game_runtime_part_attachment",
            ),
            ("gearblocks_api_types", "def_gearblocks_api_type"),
            ("gearblocks_api_members", "def_gearblocks_api_member"),
            ("gearblocks_api_parameters", "def_gearblocks_api_parameter"),
            (
                "gearblocks_api_enum_values",
                "def_gearblocks_api_enum_value",
            ),
            (
                "game_runtime_part_api_members",
                "n2n_game_runtime_part_api_member",
            ),
            ("game_constructions", "obj_game_construction"),
            (
                "game_runtime_construction_exports",
                "obj_game_runtime_construction_export",
            ),
            ("game_chat_conversations", "obj_game_chat_conversation"),
            ("game_chat_messages", "obj_game_chat_message"),
        ];

        for (legacy_name, normalized_name) in table_migrations {
            Self::rename_table_if_needed(connection, legacy_name, normalized_name)?;
        }

        Ok(())
    }

    fn rename_table_if_needed(
        connection: &Connection,
        legacy_name: &str,
        normalized_name: &str,
    ) -> Result<()> {
        let legacy_exists = Self::table_exists(connection, legacy_name)?;
        let normalized_exists = Self::table_exists(connection, normalized_name)?;

        if !legacy_exists {
            return Ok(());
        }

        if normalized_exists {
            Self::copy_legacy_rows_to_normalized_table(connection, legacy_name, normalized_name)?;
            connection.execute(&format!("DROP TABLE {legacy_name}"), [])?;
            return Ok(());
        }

        connection.execute(
            &format!("ALTER TABLE {legacy_name} RENAME TO {normalized_name}"),
            [],
        )?;
        Ok(())
    }

    fn table_exists(connection: &Connection, table_name: &str) -> Result<bool> {
        connection
            .query_row(
                "
                SELECT 1
                FROM sqlite_master
                WHERE type = 'table' AND name = ?1
                LIMIT 1
                ",
                params![table_name],
                |_| Ok(()),
            )
            .optional()
            .map(|value| value.is_some())
    }

    fn copy_legacy_rows_to_normalized_table(
        connection: &Connection,
        legacy_name: &str,
        normalized_name: &str,
    ) -> Result<()> {
        let legacy_columns = Self::table_columns(connection, legacy_name)?;
        let normalized_columns = Self::table_columns(connection, normalized_name)?;
        let normalized_column_set = normalized_columns
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let common_columns = legacy_columns
            .iter()
            .filter(|column| normalized_column_set.contains(column.as_str()))
            .map(String::as_str)
            .collect::<Vec<_>>();

        if common_columns.is_empty() {
            return Ok(());
        }

        let quoted_columns = common_columns
            .iter()
            .map(|column| Self::quote_identifier(column))
            .collect::<Vec<_>>()
            .join(", ");
        connection.execute(
            &format!(
                "
                INSERT OR IGNORE INTO {normalized_name} ({quoted_columns})
                SELECT {quoted_columns}
                FROM {legacy_name}
                "
            ),
            [],
        )?;

        Ok(())
    }

    fn table_columns(connection: &Connection, table_name: &str) -> Result<Vec<String>> {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;
        Ok(columns)
    }

    fn quote_identifier(identifier: &str) -> String {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn ensure_schema_metadata_columns(connection: &Connection) -> Result<()> {
        for table in Self::normalized_schema_tables() {
            Self::ensure_column(
                connection,
                table,
                "schema_json",
                "TEXT NOT NULL DEFAULT '{}'",
            )?;
            Self::ensure_column(connection, table, "modified_at", "TEXT NOT NULL DEFAULT ''")?;
            connection.execute(
                &format!(
                    "
                    UPDATE {table}
                    SET modified_at = CURRENT_TIMESTAMP
                    WHERE modified_at = ''
                    "
                ),
                [],
            )?;
        }

        Ok(())
    }

    fn ensure_modified_at_triggers(connection: &Connection) -> Result<()> {
        let table_primary_keys = [
            ("obj_scratchpad", "id"),
            ("obj_setting", "key"),
            ("obj_task", "id"),
            ("obj_note", "id"),
            ("obj_calendar_event", "id"),
            ("obj_smoking_event", "id"),
            ("obj_smoking_cessation_setting", "id"),
            ("def_scheduler_type", "id"),
            ("obj_scheduler", "id"),
            ("obj_scheduler_run", "id"),
            ("def_game", "id_game"),
            ("obj_game", "id"),
            ("obj_game_setting", "id"),
            ("obj_project", "id"),
            ("obj_planning_conversation", "id"),
            ("obj_planning_message", "id"),
            ("n2n_planning_conversation_context", "id"),
            ("obj_project_github_repository", "id"),
            ("obj_project_markdown_context", "id"),
            ("obj_youtube_reference", "id"),
            ("obj_bridge_file_draft", "id"),
            ("obj_game_catalog_object", "id"),
            ("obj_game_catalog_reference", "id"),
            ("obj_game_data_location", "id"),
            ("obj_game_catalog_screenshot", "id"),
            ("obj_game_runtime_part", "id"),
            ("obj_game_runtime_part_alias", "id"),
            ("obj_game_runtime_part_api_attribute", "id"),
            ("obj_game_runtime_part_value", "id"),
            ("obj_game_runtime_part_property", "id"),
            ("obj_game_runtime_part_attachment", "id"),
            ("def_gearblocks_api_type", "id"),
            ("def_gearblocks_api_member", "id"),
            ("def_gearblocks_api_parameter", "id"),
            ("def_gearblocks_api_enum_value", "id"),
            ("n2n_game_runtime_part_api_member", "id"),
            ("obj_game_construction", "id"),
            ("obj_game_runtime_construction_export", "id"),
            ("obj_game_chat_conversation", "id"),
            ("obj_game_chat_message", "id"),
        ];

        for (table, primary_key) in table_primary_keys {
            connection.execute(
                &format!(
                    "
                    CREATE TRIGGER IF NOT EXISTS trg_{table}_modified_at
                    AFTER UPDATE ON {table}
                    WHEN OLD.modified_at IS NEW.modified_at
                    BEGIN
                        UPDATE {table}
                        SET modified_at = CURRENT_TIMESTAMP
                        WHERE {primary_key} = NEW.{primary_key}
                            AND modified_at IS OLD.modified_at;
                    END
                    "
                ),
                [],
            )?;
        }

        Ok(())
    }

    fn refresh_schema_json_metadata(connection: &Connection) -> Result<()> {
        let tables = Self::normalized_schema_tables();
        for table in tables {
            let schema_json = Self::table_schema_json(connection, table)?;
            connection.execute(
                &format!(
                    "
                    UPDATE {table}
                    SET schema_json = ?1
                    WHERE schema_json = '' OR schema_json = '{{}}'
                    "
                ),
                params![schema_json],
            )?;
        }
        Ok(())
    }

    fn normalized_schema_tables() -> [&'static str; 43] {
        [
            "obj_scratchpad",
            "obj_setting",
            "obj_task",
            "obj_note",
            "obj_calendar_event",
            "obj_smoking_event",
            "obj_smoking_cessation_setting",
            "def_scheduler_type",
            "obj_scheduler",
            "obj_scheduler_run",
            "def_game",
            "obj_game",
            "obj_game_setting",
            "obj_project",
            "obj_planning_conversation",
            "obj_planning_message",
            "n2n_planning_conversation_context",
            "obj_project_github_repository",
            "obj_project_markdown_context",
            "obj_youtube_reference",
            "obj_bridge_file_draft",
            "obj_game_catalog_object",
            "obj_game_catalog_reference",
            "obj_game_data_location",
            "obj_game_catalog_screenshot",
            "obj_game_runtime_part",
            "obj_game_runtime_part_alias",
            "obj_game_runtime_part_api_attribute",
            "obj_game_runtime_part_value",
            "obj_game_runtime_part_property",
            "obj_game_runtime_part_attachment",
            "def_gearblocks_api_type",
            "def_gearblocks_api_member",
            "def_gearblocks_api_parameter",
            "def_gearblocks_api_enum_value",
            "n2n_game_runtime_part_api_member",
            "obj_game_construction",
            "obj_game_runtime_construction_export",
            "obj_game_chat_conversation",
            "obj_game_chat_message",
            "obj_game_build_guide",
            "obj_game_build_guide_part",
            "obj_game_build_guide_step",
        ]
    }

    fn table_schema_json(connection: &Connection, table: &str) -> Result<String> {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
        let columns = statement
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "name": row.get::<_, String>(1)?,
                    "storageType": row.get::<_, String>(2)?,
                    "notNull": row.get::<_, i64>(3)? == 1,
                    "defaultValue": row.get::<_, Option<String>>(4)?,
                    "isPrimaryKey": row.get::<_, i64>(5)? > 0,
                }))
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(serde_json::json!({
            "table": table,
            "columns": columns,
        })
        .to_string())
    }

    fn seed_gearblocks_api_catalog(connection: &Connection) -> Result<()> {
        let namespace = "SmashHammer.GearBlocks.Construction";
        for definition in gearblocks_api::CONSTRUCTION_CLASS_DEFINITIONS {
            Self::seed_gearblocks_api_type(
                connection,
                namespace,
                definition.name,
                definition.type_kind,
                definition.docs_url,
                "docs",
                "0.8.96622",
                definition.summary,
            )?;
        }

        for definition in gearblocks_api::CONSTRUCTION_INTERFACE_DEFINITIONS {
            let docs_url = gearblocks_api_docs_url(definition.docs_url);
            let type_id = Self::seed_gearblocks_api_type(
                connection,
                namespace,
                definition.name,
                "interface",
                &docs_url,
                "docs",
                "0.8.96622",
                "",
            )?;

            for raw_member in definition.members {
                let parsed = parse_gearblocks_api_member(raw_member, definition.name);
                connection.execute(
                    "
                    INSERT INTO def_gearblocks_api_member (
                        type_id,
                        member_key,
                        member_name,
                        signature,
                        member_kind,
                        return_type,
                        is_readable,
                        is_writable,
                        is_invokable,
                        is_mutating,
                        docs_url,
                        source,
                        source_version
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, 0, ?7, ?8, ?9, 'docs', '0.8.96622')
                    ON CONFLICT(type_id, member_key) DO UPDATE SET
                        member_name = excluded.member_name,
                        signature = excluded.signature,
                        member_kind = excluded.member_kind,
                        return_type = excluded.return_type,
                        is_readable = excluded.is_readable,
                        is_writable = excluded.is_writable,
                        is_invokable = excluded.is_invokable,
                        is_mutating = excluded.is_mutating,
                        docs_url = excluded.docs_url,
                        source = excluded.source,
                        source_version = excluded.source_version,
                        updated_at = CURRENT_TIMESTAMP
                    ",
                    params![
                        type_id,
                        parsed.member_key,
                        parsed.member_name,
                        parsed.signature,
                        parsed.member_kind,
                        parsed.is_readable,
                        parsed.is_invokable,
                        parsed.is_mutating,
                        docs_url,
                    ],
                )?;

                let member_id = connection.query_row(
                    "
                    SELECT id
                    FROM def_gearblocks_api_member
                    WHERE type_id = ?1
                        AND member_key = ?2
                    ",
                    params![type_id, parsed.member_key],
                    |row| row.get::<_, i64>(0),
                )?;

                connection.execute(
                    "
                    DELETE FROM def_gearblocks_api_parameter
                    WHERE member_id = ?1
                    ",
                    params![member_id],
                )?;

                for parameter in parsed.parameters {
                    connection.execute(
                        "
                        INSERT INTO def_gearblocks_api_parameter (
                            member_id,
                            position,
                            parameter_name,
                            parameter_type,
                            default_value,
                            is_optional
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                        ON CONFLICT(member_id, position) DO UPDATE SET
                            parameter_name = excluded.parameter_name,
                            parameter_type = excluded.parameter_type,
                            default_value = excluded.default_value,
                            is_optional = excluded.is_optional,
                            updated_at = CURRENT_TIMESTAMP
                        ",
                        params![
                            member_id,
                            parameter.position,
                            parameter.name,
                            parameter.parameter_type,
                            parameter.default_value,
                            parameter.is_optional,
                        ],
                    )?;
                }
            }
        }

        for definition in gearblocks_api::CONSTRUCTION_ENUM_DEFINITIONS {
            let type_id = Self::seed_gearblocks_api_type(
                connection,
                namespace,
                definition.name,
                "enum",
                definition.docs_url,
                "docs",
                "0.8.96622",
                &format!(
                    "{} Underlying type: {}.",
                    definition.summary, definition.underlying_type
                ),
            )?;

            connection.execute(
                "
                DELETE FROM def_gearblocks_api_enum_value
                WHERE type_id = ?1
                ",
                params![type_id],
            )?;

            for (index, value) in definition.values.iter().enumerate() {
                connection.execute(
                    "
                    INSERT INTO def_gearblocks_api_enum_value (
                        type_id,
                        position,
                        value_name,
                        numeric_value,
                        lua_name,
                        description,
                        source,
                        source_version
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'docs', '0.8.96622')
                    ON CONFLICT(type_id, value_name) DO UPDATE SET
                        position = excluded.position,
                        numeric_value = excluded.numeric_value,
                        lua_name = excluded.lua_name,
                        description = excluded.description,
                        source = excluded.source,
                        source_version = excluded.source_version,
                        updated_at = CURRENT_TIMESTAMP
                    ",
                    params![
                        type_id,
                        index as i64,
                        value.name,
                        value.numeric_value,
                        value.lua_name,
                        value.description,
                    ],
                )?;
            }
        }

        Ok(())
    }

    fn seed_gearblocks_api_type(
        connection: &Connection,
        namespace: &str,
        type_name: &str,
        type_kind: &str,
        docs_url: &str,
        source: &str,
        source_version: &str,
        notes: &str,
    ) -> Result<i64> {
        connection.execute(
            "
            INSERT INTO def_gearblocks_api_type (
                namespace,
                type_name,
                type_kind,
                docs_url,
                source,
                source_version,
                notes
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(namespace, type_name) DO UPDATE SET
                type_kind = excluded.type_kind,
                docs_url = excluded.docs_url,
                source = excluded.source,
                source_version = excluded.source_version,
                notes = excluded.notes,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                namespace.trim(),
                type_name.trim(),
                type_kind.trim(),
                gearblocks_api_docs_url(docs_url),
                source.trim(),
                source_version.trim(),
                notes.trim(),
            ],
        )?;

        connection.query_row(
            "
            SELECT id
            FROM def_gearblocks_api_type
            WHERE namespace = ?1
                AND type_name = ?2
            ",
            params![namespace.trim(), type_name.trim()],
            |row| row.get::<_, i64>(0),
        )
    }

    pub fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT value FROM obj_setting WHERE key = ?1",
                params![key.trim()],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn save_app_setting(&self, key: &str, value: &str) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_setting (key, value)
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
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_setting WHERE key = ?1",
            params![key.trim()],
        )?;
        Ok(())
    }

    pub fn get_scratchpad(&self) -> Result<String> {
        let connection = self.connection()?;
        connection.query_row(
            "SELECT content FROM obj_scratchpad WHERE id = 1",
            [],
            |row| row.get(0),
        )
    }

    pub fn save_scratchpad(&self, content: &str) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_scratchpad (id, content, updated_at)
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
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, title, body, deadline, is_completed, created_at, updated_at
            FROM obj_task
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
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO obj_task (title, body, deadline) VALUES (?1, ?2, ?3)",
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
        let connection = self.connection()?;

        if let Some(next_title) = title {
            connection.execute(
                "
                UPDATE obj_task
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
                UPDATE obj_task
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
                "UPDATE obj_task SET is_completed = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![if next_state { 1 } else { 0 }, id],
            )?;
        }

        Self::get_task_by_id(&connection, id)
    }

    pub fn delete_task(&self, id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM obj_task WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, title, body, created_at, updated_at
            FROM obj_note
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
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO obj_note (title, body) VALUES (?1, ?2)",
            params![title.trim(), body],
        )?;
        let id = connection.last_insert_rowid();
        Self::get_note_by_id(&connection, id)
    }

    pub fn update_note(&self, id: i64, title: &str, body: &str) -> Result<NoteRecord> {
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_note
            SET title = ?1, body = ?2, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?3
            ",
            params![title.trim(), body, id],
        )?;

        Self::get_note_by_id(&connection, id)
    }

    pub fn delete_note(&self, id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM obj_note WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_calendar_events(&self) -> Result<Vec<CalendarEventRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, title, start_date, start_time, end_date, end_time, notes, created_at, updated_at
            FROM obj_calendar_event
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
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_calendar_event (title, start_date, start_time, end_date, end_time, notes)
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
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_calendar_event
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
        let connection = self.connection()?;
        connection.execute("DELETE FROM obj_calendar_event WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_smoking_events(&self) -> Result<Vec<SmokingEventRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, smoked_at, source, notes, created_at
            FROM obj_smoking_event
            ORDER BY smoked_at DESC, id DESC
            ",
        )?;

        let records = statement
            .query_map([], smoking_event_from_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(records)
    }

    pub fn create_smoking_event(
        &self,
        smoked_at: Option<&str>,
        source: &str,
        notes: &str,
    ) -> Result<SmokingEventRecord> {
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_smoking_event (smoked_at, source, notes)
            VALUES (COALESCE(NULLIF(?1, ''), datetime('now', 'localtime')), ?2, ?3)
            ",
            params![
                smoked_at.map(str::trim).unwrap_or_default(),
                source.trim(),
                notes
            ],
        )?;
        let id = connection.last_insert_rowid();
        connection.execute(
            "
            UPDATE obj_smoking_cessation_setting
            SET current_cigarette_count = MAX(current_cigarette_count - 1, 0),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = 1
            ",
            [],
        )?;
        Self::get_smoking_event_by_id(&connection, id)
    }

    pub fn delete_smoking_event(&self, id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM obj_smoking_event WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_smoking_cessation_settings(&self) -> Result<SmokingCessationSettingsRecord> {
        let connection = self.connection()?;
        Self::get_smoking_cessation_settings_for_connection(&connection)
    }

    pub fn update_smoking_cigarette_count(
        &self,
        current_cigarette_count: i64,
    ) -> Result<SmokingCessationSettingsRecord> {
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_smoking_cessation_setting
            SET current_cigarette_count = MAX(?1, 0),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = 1
            ",
            params![current_cigarette_count],
        )?;
        Self::get_smoking_cessation_settings_for_connection(&connection)
    }

    pub fn list_schedulers(&self) -> Result<Vec<SchedulerRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT
                scheduler.id,
                scheduler.id_type,
                scheduler_type.scheduler_key,
                scheduler_type.label,
                scheduler.owner_module,
                scheduler.name,
                scheduler.is_enabled,
                scheduler.interval_seconds,
                scheduler.run_on_startup,
                scheduler.coalesce_missed_runs,
                scheduler.payload_json,
                scheduler.next_run_at,
                scheduler.last_run_at,
                scheduler.last_status,
                scheduler.last_error,
                scheduler.lease_until,
                scheduler.created_at,
                scheduler.modified_at
            FROM obj_scheduler scheduler
            JOIN def_scheduler_type scheduler_type ON scheduler_type.id = scheduler.id_type
            ORDER BY scheduler.owner_module, scheduler.name
            ",
        )?;
        let records = statement
            .query_map([], scheduler_from_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(records)
    }

    pub fn list_due_schedulers(&self, limit: i64) -> Result<Vec<SchedulerRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT
                scheduler.id,
                scheduler.id_type,
                scheduler_type.scheduler_key,
                scheduler_type.label,
                scheduler.owner_module,
                scheduler.name,
                scheduler.is_enabled,
                scheduler.interval_seconds,
                scheduler.run_on_startup,
                scheduler.coalesce_missed_runs,
                scheduler.payload_json,
                scheduler.next_run_at,
                scheduler.last_run_at,
                scheduler.last_status,
                scheduler.last_error,
                scheduler.lease_until,
                scheduler.created_at,
                scheduler.modified_at
            FROM obj_scheduler scheduler
            JOIN def_scheduler_type scheduler_type ON scheduler_type.id = scheduler.id_type
            WHERE scheduler.is_enabled = 1
              AND (
                (scheduler.run_on_startup = 1 AND scheduler.last_run_at = '')
                OR datetime(scheduler.next_run_at) <= datetime('now', 'localtime')
              )
              AND (
                scheduler.lease_until = ''
                OR datetime(scheduler.lease_until) <= datetime('now', 'localtime')
              )
            ORDER BY scheduler.next_run_at, scheduler.id
            LIMIT ?1
            ",
        )?;
        let records = statement
            .query_map(params![limit.max(1)], scheduler_from_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(records)
    }

    pub fn try_acquire_scheduler(&self, scheduler_id: i64, lease_seconds: i64) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "
            UPDATE obj_scheduler
            SET lease_until = datetime('now', 'localtime', ?1),
                modified_at = CURRENT_TIMESTAMP
            WHERE id = ?2
              AND is_enabled = 1
              AND (
                lease_until = ''
                OR datetime(lease_until) <= datetime('now', 'localtime')
              )
            ",
            params![format!("+{} seconds", lease_seconds.max(1)), scheduler_id],
        )?;
        Ok(changed == 1)
    }

    pub fn start_scheduler_run(&self, scheduler: &SchedulerRecord) -> Result<i64> {
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_scheduler_run (
                id_scheduler,
                id_type,
                started_at,
                status,
                message
            )
            VALUES (?1, ?2, datetime('now', 'localtime'), 'running', '')
            ",
            params![scheduler.id, scheduler.type_id],
        )?;
        Ok(connection.last_insert_rowid())
    }

    pub fn complete_scheduler_run(
        &self,
        scheduler: &SchedulerRecord,
        run_id: i64,
        status: &str,
        message: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        let next_run_modifier = format!("+{} seconds", scheduler.interval_seconds.max(1));
        connection.execute(
            "
            UPDATE obj_scheduler_run
            SET finished_at = datetime('now', 'localtime'),
                status = ?1,
                message = ?2,
                modified_at = CURRENT_TIMESTAMP
            WHERE id = ?3
            ",
            params![status, message, run_id],
        )?;
        connection.execute(
            "
            UPDATE obj_scheduler
            SET next_run_at = datetime('now', 'localtime', ?1),
                last_run_at = datetime('now', 'localtime'),
                last_status = ?2,
                last_error = ?3,
                lease_until = '',
                modified_at = CURRENT_TIMESTAMP
            WHERE id = ?4
            ",
            params![
                next_run_modifier,
                status,
                if status == "success" { "" } else { message },
                scheduler.id
            ],
        )?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, name, description, status, created_at, updated_at
            FROM obj_project
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
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO obj_project (name, description, status) VALUES (?1, ?2, ?3)",
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
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_project
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
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_bridge_file_draft WHERE project_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_project_markdown_context WHERE project_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_project_github_repository WHERE project_id = ?1",
            params![id],
        )?;
        connection.execute("DELETE FROM obj_project WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_project(&self, id: i64) -> Result<ProjectRecord> {
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, id)
    }

    pub fn get_project_github_repository(
        &self,
        project_id: i64,
    ) -> Result<Option<ProjectGitHubRepositoryRecord>> {
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        Self::get_project_github_repository_by_project_id(&connection, project_id).optional()
    }

    pub fn save_project_github_repository(
        &self,
        project_id: i64,
        repository_full_name: &str,
    ) -> Result<ProjectGitHubRepositoryRecord> {
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        connection.execute(
            "
            INSERT INTO obj_project_github_repository (
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
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        connection.execute(
            "DELETE FROM obj_project_github_repository WHERE project_id = ?1",
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
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_project_github_repository
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
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_project_github_repository
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
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        Self::get_project_markdown_context_by_project_id(&connection, project_id).optional()
    }

    pub fn save_project_markdown_context(
        &self,
        project_id: i64,
        root_path: &str,
        readme_path: &str,
    ) -> Result<ProjectMarkdownContextRecord> {
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        let clean_readme_path = if readme_path.trim().is_empty() {
            "README.md"
        } else {
            readme_path.trim()
        };

        connection.execute(
            "
            INSERT INTO obj_project_markdown_context (
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
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        connection.execute(
            "DELETE FROM obj_project_markdown_context WHERE project_id = ?1",
            params![project_id],
        )?;
        Ok(())
    }

    pub fn load_project_markdown_context(
        &self,
        project_id: i64,
    ) -> Result<ProjectMarkdownContextPayload> {
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        Self::load_project_markdown_context_for_project(&connection, project_id)
    }

    pub fn list_planning_conversations(
        &self,
        project_id: Option<i64>,
    ) -> Result<Vec<PlanningConversationRecord>> {
        let connection = self.connection()?;

        if let Some(project_id) = project_id {
            let mut statement = connection.prepare(
                "
                SELECT id, project_id, title, created_at, updated_at
                FROM obj_planning_conversation
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
            FROM obj_planning_conversation
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
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        let clean_title = if title.trim().is_empty() {
            "Planning conversation"
        } else {
            title.trim()
        };

        connection.execute(
            "
            INSERT INTO obj_planning_conversation (project_id, title)
            VALUES (?1, ?2)
            ",
            params![project_id, clean_title],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_planning_conversation_by_id(&connection, id)
    }

    pub fn get_planning_conversation(&self, id: i64) -> Result<PlanningConversationRecord> {
        let connection = self.connection()?;
        Self::get_planning_conversation_by_id(&connection, id)
    }

    pub fn list_planning_messages(
        &self,
        conversation_id: i64,
    ) -> Result<Vec<PlanningMessageRecord>> {
        let connection = self.connection()?;
        Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        Self::list_planning_messages_for_connection(&connection, conversation_id)
    }

    pub fn recent_planning_messages(
        &self,
        conversation_id: i64,
        limit: i64,
    ) -> Result<Vec<PlanningMessageRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM (
                SELECT id, conversation_id, role, content, created_at
                FROM obj_planning_message
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
        let connection = self.connection()?;
        Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        connection.execute(
            "
            INSERT INTO obj_planning_message (conversation_id, role, content)
            VALUES (?1, ?2, ?3)
            ",
            params![conversation_id, role, content],
        )?;
        connection.execute(
            "
            UPDATE obj_planning_conversation
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            ",
            params![conversation_id],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_planning_message_by_id(&connection, id)
    }

    pub fn delete_planning_conversation(&self, conversation_id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_bridge_file_draft WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM n2n_planning_conversation_context WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM obj_planning_message WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM obj_planning_conversation WHERE id = ?1",
            params![conversation_id],
        )?;
        Ok(())
    }

    pub fn list_planning_conversation_context(
        &self,
        conversation_id: i64,
    ) -> Result<Vec<PlanningConversationContextRecord>> {
        let connection = self.connection()?;
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
        let connection = self.connection()?;
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
            INSERT INTO n2n_planning_conversation_context (
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
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM n2n_planning_conversation_context WHERE id = ?1",
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
        let connection = self.connection()?;
        let conversation = Self::get_planning_conversation_by_id(&connection, conversation_id)?;
        let project = Self::get_project_by_id(&connection, conversation.project_id)?;
        let contexts =
            Self::list_planning_conversation_context_for_connection(&connection, conversation_id)?;
        let markdown_context =
            Self::load_project_markdown_context_for_project(&connection, project.id)?;
        let message_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM obj_planning_message WHERE conversation_id = ?1",
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
        let connection = self.connection()?;
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
        let connection = self.connection()?;
        Self::get_project_by_id(&connection, project_id)?;
        let mut statement = connection.prepare(
            "
            SELECT id, project_id, conversation_id, title, content, status, created_at, updated_at
            FROM obj_bridge_file_draft
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
        let connection = self.connection()?;
        Self::get_bridge_file_draft_by_id(&connection, id)
    }

    pub fn create_bridge_file_draft_from_conversation(
        &self,
        conversation_id: i64,
    ) -> Result<BridgeFileDraftRecord> {
        let connection = self.connection()?;
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
            INSERT INTO obj_bridge_file_draft (
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
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_bridge_file_draft WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_youtube_references(&self) -> Result<Vec<YouTubeReferenceRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, title, url, video_id, channel_name, notes, tags, created_at, updated_at
            FROM obj_youtube_reference
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let references = statement
            .query_map([], youtube_reference_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(references)
    }

    pub fn get_youtube_reference(&self, id: i64) -> Result<YouTubeReferenceRecord> {
        let connection = self.connection()?;
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
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_youtube_reference (title, url, video_id, channel_name, notes, tags)
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
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE obj_youtube_reference
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
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_youtube_reference WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<GameRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, name, slug, summary, created_at, updated_at
            FROM obj_game
            ORDER BY name COLLATE NOCASE ASC, id ASC
            ",
        )?;

        let games = statement
            .query_map([], game_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(games)
    }

    pub fn get_game(&self, id: i64) -> Result<GameRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, id)
    }

    pub fn create_game(&self, name: &str, summary: &str) -> Result<GameRecord> {
        let connection = self.connection()?;
        let trimmed_name = name.trim();
        let slug = game_slug(trimmed_name);
        connection.execute(
            "
            INSERT INTO obj_game (name, slug, summary)
            VALUES (?1, ?2, ?3)
            ",
            params![trimmed_name, slug, summary.trim()],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_by_id(&connection, id)
    }

    pub fn delete_game(&self, id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            DELETE FROM obj_game_chat_message
            WHERE conversation_id IN (
                SELECT id FROM obj_game_chat_conversation WHERE game_id = ?1
            )
            ",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_game_chat_conversation WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_game_catalog_screenshot WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_game_catalog_reference WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_game_data_location WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute(
            "DELETE FROM obj_game_catalog_object WHERE game_id = ?1",
            params![id],
        )?;
        connection.execute("DELETE FROM obj_game WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_game_data_locations(&self, game_id: i64) -> Result<Vec<GameDataLocationRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT id, game_id, location_type, label, directory_path, created_at, updated_at
            FROM obj_game_data_location
            WHERE game_id = ?1
            ORDER BY
                CASE location_type
                    WHEN 'save' THEN 0
                    WHEN 'alternate' THEN 1
                    ELSE 2
                END,
                id ASC
            ",
        )?;

        let locations = statement
            .query_map(params![game_id], game_data_location_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(locations)
    }

    pub fn save_game_data_location(
        &self,
        game_id: i64,
        location_type: &str,
        label: &str,
        directory_path: &str,
    ) -> Result<GameDataLocationRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let trimmed_type = location_type.trim();
        connection.execute(
            "
            INSERT INTO obj_game_data_location (game_id, location_type, label, directory_path)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(game_id, location_type) DO UPDATE SET
                label = excluded.label,
                directory_path = excluded.directory_path,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![game_id, trimmed_type, label.trim(), directory_path.trim()],
        )?;

        Self::get_game_data_location_by_type(&connection, game_id, trimmed_type)
    }

    pub fn delete_game_data_location(&self, game_id: i64, location_type: &str) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "DELETE FROM obj_game_data_location WHERE game_id = ?1 AND location_type = ?2",
            params![game_id, location_type.trim()],
        )?;
        Ok(())
    }

    pub fn list_game_catalog_objects(&self, game_id: i64) -> Result<Vec<GameCatalogObjectRecord>> {
        let connection = self.connection()?;
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
            FROM obj_game_catalog_object
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
        let connection = self.connection()?;
        connection.execute(
            "
            DELETE FROM obj_game_catalog_object
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
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_game_catalog_object (
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

    pub fn list_game_runtime_parts(&self, game_id: i64) -> Result<Vec<GameRuntimePartRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                mass,
                world_x,
                world_y,
                world_z,
                local_x,
                local_y,
                local_z,
                world_position_json,
                local_position_json,
                properties_json,
                source_export_id,
                source_construction_id,
                last_seen_at,
                display_image_path,
                source_image_path,
                notes,
                created_at,
                updated_at
            FROM obj_game_runtime_part
            WHERE game_id = ?1
            ORDER BY category COLLATE NOCASE ASC, display_name COLLATE NOCASE ASC, asset_name COLLATE NOCASE ASC
            ",
        )?;

        let parts = statement
            .query_map(params![game_id], game_runtime_part_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(parts)
    }

    pub fn list_game_runtime_part_aliases(
        &self,
        game_id: i64,
    ) -> Result<Vec<GameRuntimePartAliasRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                part_instance_key,
                friendly_name,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                source_log_path,
                source_construction_id,
                world_position_json,
                local_position_json,
                current_unit_size_json,
                payload_json,
                last_seen_at,
                created_at,
                updated_at
            FROM obj_game_runtime_part_alias
            WHERE game_id = ?1
            ORDER BY friendly_name COLLATE NOCASE ASC, part_instance_key COLLATE NOCASE ASC
            ",
        )?;

        let aliases = statement
            .query_map(params![game_id], game_runtime_part_alias_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(aliases)
    }

    pub fn count_game_runtime_parts(&self, game_id: i64) -> Result<usize> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let count = connection.query_row(
            "SELECT COUNT(*) FROM obj_game_runtime_part WHERE game_id = ?1",
            params![game_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(count.max(0) as usize)
    }

    pub fn list_gearblocks_api_catalog(&self) -> Result<GearBlocksApiCatalogRecord> {
        let connection = self.connection()?;
        let mut type_statement = connection.prepare(
            "
            SELECT
                types.id,
                types.namespace,
                types.type_name,
                types.type_kind,
                types.docs_url,
                types.source,
                types.source_version,
                types.notes,
                COUNT(DISTINCT members.id) AS member_count,
                COUNT(DISTINCT enum_values.id) AS enum_value_count,
                types.created_at,
                types.updated_at
            FROM def_gearblocks_api_type types
            LEFT JOIN def_gearblocks_api_member members
                ON members.type_id = types.id
            LEFT JOIN def_gearblocks_api_enum_value enum_values
                ON enum_values.type_id = types.id
            GROUP BY types.id
            ORDER BY
                types.namespace COLLATE NOCASE ASC,
                CASE types.type_kind
                    WHEN 'class' THEN 0
                    WHEN 'interface' THEN 1
                    WHEN 'enum' THEN 2
                    ELSE 3
                END,
                types.type_name COLLATE NOCASE ASC
            ",
        )?;
        let types = type_statement
            .query_map([], gearblocks_api_type_from_row)?
            .collect::<Result<Vec<_>>>()?;

        let mut member_statement = connection.prepare(
            "
            SELECT
                members.id,
                members.type_id,
                types.type_name,
                members.member_key,
                members.member_name,
                members.signature,
                members.member_kind,
                members.return_type,
                members.is_readable,
                members.is_writable,
                members.is_invokable,
                members.is_mutating,
                members.docs_url,
                members.source,
                members.source_version,
                members.notes,
                members.created_at,
                members.updated_at
            FROM def_gearblocks_api_member members
            INNER JOIN def_gearblocks_api_type types
                ON types.id = members.type_id
            ORDER BY
                types.namespace COLLATE NOCASE ASC,
                types.type_name COLLATE NOCASE ASC,
                members.member_kind COLLATE NOCASE ASC,
                members.member_name COLLATE NOCASE ASC,
                members.signature COLLATE NOCASE ASC
            ",
        )?;
        let members = member_statement
            .query_map([], gearblocks_api_member_from_row)?
            .collect::<Result<Vec<_>>>()?;

        let mut parameter_statement = connection.prepare(
            "
            SELECT
                id,
                member_id,
                position,
                parameter_name,
                parameter_type,
                default_value,
                is_optional,
                created_at,
                updated_at
            FROM def_gearblocks_api_parameter
            ORDER BY member_id ASC, position ASC
            ",
        )?;
        let parameters = parameter_statement
            .query_map([], gearblocks_api_parameter_from_row)?
            .collect::<Result<Vec<_>>>()?;

        let mut enum_value_statement = connection.prepare(
            "
            SELECT
                id,
                type_id,
                position,
                value_name,
                numeric_value,
                lua_name,
                description,
                source,
                source_version,
                created_at,
                updated_at
            FROM def_gearblocks_api_enum_value
            ORDER BY type_id ASC, position ASC, value_name COLLATE NOCASE ASC
            ",
        )?;
        let enum_values = enum_value_statement
            .query_map([], gearblocks_api_enum_value_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(GearBlocksApiCatalogRecord {
            types,
            members,
            parameters,
            enum_values,
        })
    }

    pub fn import_gearblocks_api_catalog(
        &self,
        scrape: &GearBlocksApiScrape,
    ) -> Result<GearBlocksApiImportResult> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let mut imported_member_count = 0usize;
        let mut imported_parameter_count = 0usize;
        let mut imported_enum_value_count = 0usize;

        for api_type in &scrape.types {
            let type_id = Self::seed_gearblocks_api_type(
                &transaction,
                &api_type.namespace,
                &api_type.type_name,
                &api_type.type_kind,
                &api_type.docs_url,
                &scrape.source,
                &scrape.source_version,
                &api_type.notes,
            )?;

            if !api_type.members.is_empty() {
                transaction.execute(
                    "
                    DELETE FROM def_gearblocks_api_member
                    WHERE type_id = ?1
                        AND source = ?2
                    ",
                    params![type_id, scrape.source],
                )?;
            }

            for member in &api_type.members {
                transaction.execute(
                    "
                    INSERT INTO def_gearblocks_api_member (
                        type_id,
                        member_key,
                        member_name,
                        signature,
                        member_kind,
                        return_type,
                        is_readable,
                        is_writable,
                        is_invokable,
                        is_mutating,
                        docs_url,
                        source,
                        source_version,
                        notes
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    ON CONFLICT(type_id, member_key) DO UPDATE SET
                        member_name = excluded.member_name,
                        signature = excluded.signature,
                        member_kind = excluded.member_kind,
                        return_type = excluded.return_type,
                        is_readable = excluded.is_readable,
                        is_writable = excluded.is_writable,
                        is_invokable = excluded.is_invokable,
                        is_mutating = excluded.is_mutating,
                        docs_url = excluded.docs_url,
                        source = excluded.source,
                        source_version = excluded.source_version,
                        notes = excluded.notes,
                        updated_at = CURRENT_TIMESTAMP
                    ",
                    params![
                        type_id,
                        member.member_key,
                        member.member_name,
                        member.signature,
                        member.member_kind,
                        member.return_type,
                        member.is_readable,
                        member.is_writable,
                        member.is_invokable,
                        member.is_mutating,
                        member.docs_url,
                        scrape.source,
                        scrape.source_version,
                        member.notes,
                    ],
                )?;
                imported_member_count += 1;

                let member_id = transaction.query_row(
                    "
                    SELECT id
                    FROM def_gearblocks_api_member
                    WHERE type_id = ?1
                        AND member_key = ?2
                    ",
                    params![type_id, member.member_key],
                    |row| row.get::<_, i64>(0),
                )?;

                transaction.execute(
                    "
                    DELETE FROM def_gearblocks_api_parameter
                    WHERE member_id = ?1
                    ",
                    params![member_id],
                )?;

                for parameter in &member.parameters {
                    transaction.execute(
                        "
                        INSERT INTO def_gearblocks_api_parameter (
                            member_id,
                            position,
                            parameter_name,
                            parameter_type,
                            default_value,
                            is_optional
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                        ON CONFLICT(member_id, position) DO UPDATE SET
                            parameter_name = excluded.parameter_name,
                            parameter_type = excluded.parameter_type,
                            default_value = excluded.default_value,
                            is_optional = excluded.is_optional,
                            updated_at = CURRENT_TIMESTAMP
                        ",
                        params![
                            member_id,
                            parameter.position,
                            parameter.parameter_name,
                            parameter.parameter_type,
                            parameter.default_value,
                            parameter.is_optional,
                        ],
                    )?;
                    imported_parameter_count += 1;
                }
            }

            if !api_type.enum_values.is_empty() {
                transaction.execute(
                    "
                    DELETE FROM def_gearblocks_api_enum_value
                    WHERE type_id = ?1
                        AND source = ?2
                    ",
                    params![type_id, scrape.source],
                )?;
            }

            for value in &api_type.enum_values {
                transaction.execute(
                    "
                    INSERT INTO def_gearblocks_api_enum_value (
                        type_id,
                        position,
                        value_name,
                        numeric_value,
                        lua_name,
                        description,
                        source,
                        source_version
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    ON CONFLICT(type_id, value_name) DO UPDATE SET
                        position = excluded.position,
                        numeric_value = excluded.numeric_value,
                        lua_name = excluded.lua_name,
                        description = excluded.description,
                        source = excluded.source,
                        source_version = excluded.source_version,
                        updated_at = CURRENT_TIMESTAMP
                    ",
                    params![
                        type_id,
                        value.position,
                        value.value_name,
                        value.numeric_value,
                        value.lua_name,
                        value.description,
                        scrape.source,
                        scrape.source_version,
                    ],
                )?;
                imported_enum_value_count += 1;
            }
        }

        transaction.commit()?;

        Ok(GearBlocksApiImportResult {
            source: scrape.source.clone(),
            source_version: scrape.source_version.clone(),
            docs_root: scrape.docs_root.clone(),
            fetched_pages: scrape.fetched_pages,
            imported_type_count: scrape.types.len(),
            imported_member_count,
            imported_parameter_count,
            imported_enum_value_count,
        })
    }

    pub fn list_game_runtime_part_api_members(
        &self,
        game_id: i64,
        part_id: i64,
    ) -> Result<Vec<GameRuntimePartApiMemberRecord>> {
        let connection = self.connection()?;
        let part_key = connection
            .query_row(
                "
                SELECT part_key
                FROM obj_game_runtime_part
                WHERE id = ?1
                    AND game_id = ?2
                ",
                params![part_id, game_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        let Some(part_key) = part_key else {
            return Ok(Vec::new());
        };

        let mut statement = connection.prepare(
            "
            SELECT
                observed.id,
                observed.game_id,
                observed.part_key,
                observed.api_member_id,
                observed.availability,
                observed.source_export_id,
                observed.source_construction_id,
                observed.first_seen_at,
                observed.last_seen_at,
                types.namespace,
                types.type_name,
                types.type_kind,
                members.member_key,
                members.member_name,
                members.signature,
                members.member_kind,
                members.is_readable,
                members.is_writable,
                members.is_invokable,
                members.is_mutating,
                members.docs_url,
                observed.created_at,
                observed.updated_at
            FROM n2n_game_runtime_part_api_member observed
            INNER JOIN def_gearblocks_api_member members
                ON members.id = observed.api_member_id
            INNER JOIN def_gearblocks_api_type types
                ON types.id = members.type_id
            WHERE observed.game_id = ?1
                AND observed.part_key = ?2
            ORDER BY
                types.type_name COLLATE NOCASE ASC,
                members.member_kind COLLATE NOCASE ASC,
                members.member_name COLLATE NOCASE ASC,
                members.signature COLLATE NOCASE ASC
            ",
        )?;
        let members = statement
            .query_map(
                params![game_id, part_key.trim()],
                game_runtime_part_api_member_from_row,
            )?
            .collect::<Result<Vec<_>>>()?;

        Ok(members)
    }

    pub fn get_game_runtime_part(&self, id: i64) -> Result<Option<GameRuntimePartRecord>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "
                SELECT
                    id,
                    game_id,
                    part_key,
                    asset_guid,
                    asset_name,
                    display_name,
                    full_display_name,
                    category,
                    mass,
                    world_x,
                    world_y,
                    world_z,
                    local_x,
                    local_y,
                    local_z,
                    world_position_json,
                    local_position_json,
                    properties_json,
                    source_export_id,
                    source_construction_id,
                    last_seen_at,
                    display_image_path,
                    source_image_path,
                    notes,
                    created_at,
                    updated_at
                FROM obj_game_runtime_part
                WHERE id = ?1
                ",
                params![id],
                game_runtime_part_from_row,
            )
            .optional()
    }

    pub fn update_game_runtime_part_display_image(
        &self,
        game_id: i64,
        part_id: i64,
        display_image_path: &str,
        source_image_path: &str,
    ) -> Result<GameRuntimePartRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            UPDATE obj_game_runtime_part
            SET
                display_image_path = ?1,
                source_image_path = ?2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?3
                AND game_id = ?4
            ",
            params![
                display_image_path.trim(),
                source_image_path.trim(),
                part_id,
                game_id
            ],
        )?;

        Self::get_game_runtime_part_by_id(&connection, game_id, part_id)
    }

    pub fn clear_game_runtime_part_images_for_category(
        &self,
        game_id: i64,
        category: &str,
    ) -> Result<Vec<GameRuntimePartRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            UPDATE obj_game_runtime_part
            SET
                display_image_path = '',
                source_image_path = '',
                updated_at = CURRENT_TIMESTAMP
            WHERE game_id = ?1
                AND category = ?2
            ",
            params![game_id, category.trim()],
        )?;

        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                mass,
                world_x,
                world_y,
                world_z,
                local_x,
                local_y,
                local_z,
                world_position_json,
                local_position_json,
                properties_json,
                source_export_id,
                source_construction_id,
                last_seen_at,
                display_image_path,
                source_image_path,
                notes,
                created_at,
                updated_at
            FROM obj_game_runtime_part
            WHERE game_id = ?1
                AND category = ?2
            ORDER BY display_name COLLATE NOCASE ASC, asset_name COLLATE NOCASE ASC
            ",
        )?;

        let parts = statement
            .query_map(
                params![game_id, category.trim()],
                game_runtime_part_from_row,
            )?
            .collect::<Result<Vec<_>>>()?;

        Ok(parts)
    }

    pub fn update_game_runtime_part_notes(
        &self,
        game_id: i64,
        part_id: i64,
        notes: &str,
    ) -> Result<GameRuntimePartRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            UPDATE obj_game_runtime_part
            SET
                notes = ?1,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2
                AND game_id = ?3
            ",
            params![notes, part_id, game_id],
        )?;

        Self::get_game_runtime_part_by_id(&connection, game_id, part_id)
    }

    pub fn upsert_game_runtime_part(
        &self,
        game_id: i64,
        part_key: &str,
        asset_guid: &str,
        asset_name: &str,
        display_name: &str,
        full_display_name: &str,
        category: &str,
        mass: f64,
        world_position: Option<(f64, f64, f64)>,
        local_position: Option<(f64, f64, f64)>,
        world_position_json: &str,
        local_position_json: &str,
        properties_json: &str,
        source_export_id: &str,
        source_construction_id: &str,
        last_seen_at: &str,
    ) -> Result<GameRuntimePartRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let (world_x, world_y, world_z) = match world_position {
            Some((x, y, z)) => (Some(x), Some(y), Some(z)),
            None => (None, None, None),
        };
        let (local_x, local_y, local_z) = match local_position {
            Some((x, y, z)) => (Some(x), Some(y), Some(z)),
            None => (None, None, None),
        };
        connection.execute(
            "
            INSERT INTO obj_game_runtime_part (
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                mass,
                world_x,
                world_y,
                world_z,
                local_x,
                local_y,
                local_z,
                world_position_json,
                local_position_json,
                properties_json,
                source_export_id,
                source_construction_id,
                last_seen_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
            ON CONFLICT(game_id, part_key) DO UPDATE SET
                asset_guid = excluded.asset_guid,
                asset_name = excluded.asset_name,
                display_name = excluded.display_name,
                full_display_name = excluded.full_display_name,
                category = excluded.category,
                mass = excluded.mass,
                world_x = excluded.world_x,
                world_y = excluded.world_y,
                world_z = excluded.world_z,
                local_x = excluded.local_x,
                local_y = excluded.local_y,
                local_z = excluded.local_z,
                world_position_json = excluded.world_position_json,
                local_position_json = excluded.local_position_json,
                properties_json = excluded.properties_json,
                source_export_id = excluded.source_export_id,
                source_construction_id = excluded.source_construction_id,
                last_seen_at = excluded.last_seen_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                part_key.trim(),
                asset_guid.trim(),
                asset_name.trim(),
                display_name.trim(),
                full_display_name.trim(),
                category.trim(),
                mass,
                world_x,
                world_y,
                world_z,
                local_x,
                local_y,
                local_z,
                world_position_json.trim(),
                local_position_json.trim(),
                properties_json,
                source_export_id.trim(),
                source_construction_id.trim(),
                last_seen_at.trim()
            ],
        )?;

        Self::get_game_runtime_part_by_key(&connection, game_id, part_key)
    }

    pub fn upsert_game_runtime_part_alias(
        &self,
        game_id: i64,
        part_instance_key: &str,
        friendly_name: &str,
        asset_guid: &str,
        asset_name: &str,
        display_name: &str,
        full_display_name: &str,
        category: &str,
        source_log_path: &str,
        source_construction_id: &str,
        world_position_json: &str,
        local_position_json: &str,
        current_unit_size_json: &str,
        payload_json: &str,
        last_seen_at: &str,
    ) -> Result<GameRuntimePartAliasRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_runtime_part_alias (
                game_id,
                part_instance_key,
                friendly_name,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                source_log_path,
                source_construction_id,
                world_position_json,
                local_position_json,
                current_unit_size_json,
                payload_json,
                last_seen_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(game_id, part_instance_key) DO UPDATE SET
                friendly_name = excluded.friendly_name,
                asset_guid = excluded.asset_guid,
                asset_name = excluded.asset_name,
                display_name = excluded.display_name,
                full_display_name = excluded.full_display_name,
                category = excluded.category,
                source_log_path = excluded.source_log_path,
                source_construction_id = excluded.source_construction_id,
                world_position_json = excluded.world_position_json,
                local_position_json = excluded.local_position_json,
                current_unit_size_json = excluded.current_unit_size_json,
                payload_json = excluded.payload_json,
                last_seen_at = excluded.last_seen_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                part_instance_key.trim(),
                friendly_name.trim(),
                asset_guid.trim(),
                asset_name.trim(),
                display_name.trim(),
                full_display_name.trim(),
                category.trim(),
                source_log_path.trim(),
                source_construction_id.trim(),
                world_position_json.trim(),
                local_position_json.trim(),
                current_unit_size_json.trim(),
                payload_json.trim(),
                last_seen_at.trim()
            ],
        )?;

        connection.query_row(
            "
            SELECT
                id,
                game_id,
                part_instance_key,
                friendly_name,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                source_log_path,
                source_construction_id,
                world_position_json,
                local_position_json,
                current_unit_size_json,
                payload_json,
                last_seen_at,
                created_at,
                updated_at
            FROM obj_game_runtime_part_alias
            WHERE game_id = ?1
                AND part_instance_key = ?2
            ",
            params![game_id, part_instance_key.trim()],
            game_runtime_part_alias_from_row,
        )
    }

    pub fn upsert_game_runtime_part_api_attribute(
        &self,
        game_id: i64,
        part_key: &str,
        asset_guid: &str,
        asset_name: &str,
        display_name: &str,
        full_display_name: &str,
        category: &str,
        interface_name: &str,
        attribute_name: &str,
        value_type: &str,
        availability: &str,
        source_export_id: &str,
        source_construction_id: &str,
        seen_at: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_runtime_part_api_attribute (
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                interface_name,
                attribute_name,
                value_type,
                availability,
                source_export_id,
                source_construction_id,
                first_seen_at,
                last_seen_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)
            ON CONFLICT(game_id, part_key, interface_name, attribute_name) DO UPDATE SET
                asset_guid = excluded.asset_guid,
                asset_name = excluded.asset_name,
                display_name = excluded.display_name,
                full_display_name = excluded.full_display_name,
                category = excluded.category,
                value_type = excluded.value_type,
                availability = excluded.availability,
                source_export_id = excluded.source_export_id,
                source_construction_id = excluded.source_construction_id,
                last_seen_at = excluded.last_seen_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                part_key.trim(),
                asset_guid.trim(),
                asset_name.trim(),
                display_name.trim(),
                full_display_name.trim(),
                category.trim(),
                interface_name.trim(),
                attribute_name.trim(),
                value_type.trim(),
                availability.trim(),
                source_export_id.trim(),
                source_construction_id.trim(),
                seen_at.trim(),
            ],
        )?;

        Ok(())
    }

    pub fn upsert_game_runtime_part_api_member(
        &self,
        game_id: i64,
        part_key: &str,
        interface_name: &str,
        attribute_name: &str,
        availability: &str,
        source_export_id: &str,
        source_construction_id: &str,
        seen_at: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let observed_member_name = gearblocks_observed_member_name(attribute_name);
        let mut statement = connection.prepare(
            "
                SELECT DISTINCT members.id
                FROM def_gearblocks_api_member members
                INNER JOIN def_gearblocks_api_type types
                    ON types.id = members.type_id
                WHERE types.namespace = 'SmashHammer.GearBlocks.Construction'
                    AND types.type_name = ?1
                    AND (
                        members.member_key = ?2
                        OR members.member_name = ?3
                    )
                ",
        )?;
        let api_member_ids = statement
            .query_map(
                params![
                    interface_name.trim(),
                    attribute_name.trim(),
                    observed_member_name
                ],
                |row| row.get::<_, i64>(0),
            )?
            .collect::<Result<Vec<_>>>()?;

        for api_member_id in api_member_ids {
            connection.execute(
                "
                INSERT INTO n2n_game_runtime_part_api_member (
                    game_id,
                    part_key,
                    api_member_id,
                    availability,
                    source_export_id,
                    source_construction_id,
                    first_seen_at,
                    last_seen_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                ON CONFLICT(game_id, part_key, api_member_id) DO UPDATE SET
                    availability = excluded.availability,
                    source_export_id = excluded.source_export_id,
                    source_construction_id = excluded.source_construction_id,
                    last_seen_at = excluded.last_seen_at,
                    updated_at = CURRENT_TIMESTAMP
                ",
                params![
                    game_id,
                    part_key.trim(),
                    api_member_id,
                    availability.trim(),
                    source_export_id.trim(),
                    source_construction_id.trim(),
                    seen_at.trim(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn upsert_game_runtime_part_value(
        &self,
        game_id: i64,
        part_key: &str,
        asset_guid: &str,
        asset_name: &str,
        display_name: &str,
        full_display_name: &str,
        category: &str,
        field_path: &str,
        value_type: &str,
        value_json: &str,
        source_export_id: &str,
        source_construction_id: &str,
        seen_at: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_runtime_part_value (
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                field_path,
                value_type,
                value_json,
                source_export_id,
                source_construction_id,
                first_seen_at,
                last_seen_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
            ON CONFLICT(game_id, part_key, field_path) DO UPDATE SET
                asset_guid = excluded.asset_guid,
                asset_name = excluded.asset_name,
                display_name = excluded.display_name,
                full_display_name = excluded.full_display_name,
                category = excluded.category,
                value_type = excluded.value_type,
                value_json = excluded.value_json,
                source_export_id = excluded.source_export_id,
                source_construction_id = excluded.source_construction_id,
                last_seen_at = excluded.last_seen_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                part_key.trim(),
                asset_guid.trim(),
                asset_name.trim(),
                display_name.trim(),
                full_display_name.trim(),
                category.trim(),
                field_path.trim(),
                value_type.trim(),
                value_json,
                source_export_id.trim(),
                source_construction_id.trim(),
                seen_at.trim(),
            ],
        )?;

        Ok(())
    }

    pub fn upsert_game_runtime_part_property(
        &self,
        game_id: i64,
        part_key: &str,
        asset_guid: &str,
        asset_name: &str,
        display_name: &str,
        full_display_name: &str,
        category: &str,
        property_path: &str,
        value_type: &str,
        value_json: &str,
        source_export_id: &str,
        source_construction_id: &str,
        seen_at: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_runtime_part_property (
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                property_path,
                value_type,
                value_json,
                source_export_id,
                source_construction_id,
                first_seen_at,
                last_seen_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
            ON CONFLICT(game_id, part_key, property_path) DO UPDATE SET
                asset_guid = excluded.asset_guid,
                asset_name = excluded.asset_name,
                display_name = excluded.display_name,
                full_display_name = excluded.full_display_name,
                category = excluded.category,
                value_type = excluded.value_type,
                value_json = excluded.value_json,
                source_export_id = excluded.source_export_id,
                source_construction_id = excluded.source_construction_id,
                last_seen_at = excluded.last_seen_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                part_key.trim(),
                asset_guid.trim(),
                asset_name.trim(),
                display_name.trim(),
                full_display_name.trim(),
                category.trim(),
                property_path.trim(),
                value_type.trim(),
                value_json,
                source_export_id.trim(),
                source_construction_id.trim(),
                seen_at.trim(),
            ],
        )?;

        Ok(())
    }

    pub fn upsert_game_runtime_part_attachment(
        &self,
        game_id: i64,
        part_key: &str,
        asset_guid: &str,
        asset_name: &str,
        display_name: &str,
        full_display_name: &str,
        category: &str,
        attachment_path: &str,
        value_type: &str,
        attachment_json: &str,
        source_export_id: &str,
        source_construction_id: &str,
        seen_at: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_runtime_part_attachment (
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                attachment_path,
                value_type,
                attachment_json,
                source_export_id,
                source_construction_id,
                first_seen_at,
                last_seen_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
            ON CONFLICT(game_id, part_key, attachment_path) DO UPDATE SET
                asset_guid = excluded.asset_guid,
                asset_name = excluded.asset_name,
                display_name = excluded.display_name,
                full_display_name = excluded.full_display_name,
                category = excluded.category,
                value_type = excluded.value_type,
                attachment_json = excluded.attachment_json,
                source_export_id = excluded.source_export_id,
                source_construction_id = excluded.source_construction_id,
                last_seen_at = excluded.last_seen_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                part_key.trim(),
                asset_guid.trim(),
                asset_name.trim(),
                display_name.trim(),
                full_display_name.trim(),
                category.trim(),
                attachment_path.trim(),
                value_type.trim(),
                attachment_json,
                source_export_id.trim(),
                source_construction_id.trim(),
                seen_at.trim(),
            ],
        )?;

        Ok(())
    }

    pub fn list_game_runtime_construction_exports(
        &self,
        game_id: i64,
    ) -> Result<Vec<GameRuntimeConstructionExportRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                export_id,
                name,
                export_kind,
                intended_path,
                source_log_path,
                byte_size,
                construction_id,
                exported_at,
                part_count,
                mass,
                is_frozen,
                is_invulnerable,
                is_player_character,
                document_json,
                last_indexed_at,
                created_at,
                updated_at
            FROM obj_game_runtime_construction_export
            WHERE game_id = ?1
            ORDER BY exported_at DESC, id DESC
            ",
        )?;

        let exports = statement
            .query_map(params![game_id], game_runtime_construction_export_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(exports)
    }

    pub fn count_game_runtime_construction_exports(&self, game_id: i64) -> Result<usize> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let count = connection.query_row(
            "SELECT COUNT(*) FROM obj_game_runtime_construction_export WHERE game_id = ?1",
            params![game_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(count.max(0) as usize)
    }

    pub fn latest_game_runtime_construction_export(
        &self,
        game_id: i64,
    ) -> Result<Option<GameRuntimeConstructionExportRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection
            .query_row(
                "
                SELECT
                    id,
                    game_id,
                    export_id,
                    name,
                    export_kind,
                    intended_path,
                    source_log_path,
                    byte_size,
                    construction_id,
                    exported_at,
                    part_count,
                    mass,
                    is_frozen,
                    is_invulnerable,
                    is_player_character,
                    document_json,
                    last_indexed_at,
                    created_at,
                    updated_at
                FROM obj_game_runtime_construction_export
                WHERE game_id = ?1
                ORDER BY exported_at DESC, id DESC
                LIMIT 1
                ",
                params![game_id],
                game_runtime_construction_export_from_row,
            )
            .optional()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_game_runtime_construction_export(
        &self,
        game_id: i64,
        export_id: &str,
        name: &str,
        export_kind: &str,
        intended_path: &str,
        source_log_path: &str,
        byte_size: i64,
        construction_id: &str,
        exported_at: &str,
        part_count: i64,
        mass: f64,
        is_frozen: Option<bool>,
        is_invulnerable: Option<bool>,
        is_player_character: Option<bool>,
        document_json: &str,
        last_indexed_at: &str,
    ) -> Result<GameRuntimeConstructionExportRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_runtime_construction_export (
                game_id,
                export_id,
                name,
                export_kind,
                intended_path,
                source_log_path,
                byte_size,
                construction_id,
                exported_at,
                part_count,
                mass,
                is_frozen,
                is_invulnerable,
                is_player_character,
                document_json,
                last_indexed_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            ON CONFLICT(game_id, export_id) DO UPDATE SET
                name = excluded.name,
                export_kind = excluded.export_kind,
                intended_path = excluded.intended_path,
                source_log_path = excluded.source_log_path,
                byte_size = excluded.byte_size,
                construction_id = excluded.construction_id,
                exported_at = excluded.exported_at,
                part_count = excluded.part_count,
                mass = excluded.mass,
                is_frozen = excluded.is_frozen,
                is_invulnerable = excluded.is_invulnerable,
                is_player_character = excluded.is_player_character,
                document_json = excluded.document_json,
                last_indexed_at = excluded.last_indexed_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                export_id.trim(),
                name.trim(),
                export_kind.trim(),
                intended_path.trim(),
                source_log_path.trim(),
                byte_size,
                construction_id.trim(),
                exported_at.trim(),
                part_count,
                mass,
                is_frozen.map(i64::from),
                is_invulnerable.map(i64::from),
                is_player_character.map(i64::from),
                document_json,
                last_indexed_at.trim()
            ],
        )?;

        Self::get_game_runtime_construction_export_by_export_id(&connection, game_id, export_id)
    }

    pub fn list_game_constructions(&self, game_id: i64) -> Result<Vec<GameConstructionRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT
                id,
                game_id,
                name,
                folder_path,
                construction_path,
                byte_size,
                decoded_byte_size,
                composite_count,
                part_count,
                unique_asset_guid_count,
                attachment_count,
                link_count,
                intersection_count,
                is_frozen,
                is_invulnerable,
                summary_json,
                document_json,
                last_indexed_at,
                created_at,
                updated_at
            FROM obj_game_construction
            WHERE game_id = ?1
            ORDER BY name COLLATE NOCASE ASC
            ",
        )?;

        let constructions = statement
            .query_map(params![game_id], game_construction_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(constructions)
    }

    pub fn count_game_constructions(&self, game_id: i64) -> Result<usize> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let count = connection.query_row(
            "SELECT COUNT(*) FROM obj_game_construction WHERE game_id = ?1",
            params![game_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(count.max(0) as usize)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_game_construction(
        &self,
        game_id: i64,
        name: &str,
        folder_path: &str,
        construction_path: &str,
        byte_size: i64,
        decoded_byte_size: i64,
        composite_count: i64,
        part_count: i64,
        unique_asset_guid_count: i64,
        attachment_count: i64,
        link_count: i64,
        intersection_count: i64,
        is_frozen: Option<bool>,
        is_invulnerable: Option<bool>,
        summary_json: &str,
        document_json: &str,
        last_indexed_at: &str,
    ) -> Result<GameConstructionRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_construction (
                game_id,
                name,
                folder_path,
                construction_path,
                byte_size,
                decoded_byte_size,
                composite_count,
                part_count,
                unique_asset_guid_count,
                attachment_count,
                link_count,
                intersection_count,
                is_frozen,
                is_invulnerable,
                summary_json,
                document_json,
                last_indexed_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(game_id, construction_path) DO UPDATE SET
                name = excluded.name,
                folder_path = excluded.folder_path,
                byte_size = excluded.byte_size,
                decoded_byte_size = excluded.decoded_byte_size,
                composite_count = excluded.composite_count,
                part_count = excluded.part_count,
                unique_asset_guid_count = excluded.unique_asset_guid_count,
                attachment_count = excluded.attachment_count,
                link_count = excluded.link_count,
                intersection_count = excluded.intersection_count,
                is_frozen = excluded.is_frozen,
                is_invulnerable = excluded.is_invulnerable,
                summary_json = excluded.summary_json,
                document_json = excluded.document_json,
                last_indexed_at = excluded.last_indexed_at,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                game_id,
                name.trim(),
                folder_path.trim(),
                construction_path.trim(),
                byte_size,
                decoded_byte_size,
                composite_count,
                part_count,
                unique_asset_guid_count,
                attachment_count,
                link_count,
                intersection_count,
                is_frozen.map(i64::from),
                is_invulnerable.map(i64::from),
                summary_json,
                document_json,
                last_indexed_at.trim(),
            ],
        )?;

        Self::get_game_construction_by_path(&connection, game_id, construction_path)
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
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO obj_game_catalog_screenshot (
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
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT id, game_id, title, overlay_x, overlay_y, created_at, updated_at
            FROM obj_game_chat_conversation
            WHERE game_id = ?1
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let conversations = statement
            .query_map(params![game_id], game_chat_conversation_from_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(conversations)
    }

    pub fn list_game_build_guides(&self, game_id: i64) -> Result<Vec<GameBuildGuideRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let mut statement = connection.prepare(
            "
            SELECT id, game_id, title, source_path, raw_markdown, build_goal, scale_reference,
                geometry_notes, glossary_text, checklist_json, overlay_x, overlay_y, overlay_width,
                overlay_height, created_at, updated_at
            FROM obj_game_build_guide
            WHERE game_id = ?1
            ORDER BY updated_at DESC, id DESC
            ",
        )?;

        let guides = statement
            .query_map(params![game_id], game_build_guide_from_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(guides)
    }

    pub fn latest_game_build_guide(&self, game_id: i64) -> Result<Option<GameBuildGuideRecord>> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        connection
            .query_row(
                "
                SELECT id, game_id, title, source_path, raw_markdown, build_goal, scale_reference,
                    geometry_notes, glossary_text, checklist_json, overlay_x, overlay_y, overlay_width,
                    overlay_height, created_at, updated_at
                FROM obj_game_build_guide
                WHERE game_id = ?1
                ORDER BY updated_at DESC, id DESC
                LIMIT 1
                ",
                params![game_id],
                game_build_guide_from_row,
            )
            .optional()
    }

    pub fn get_game_build_guide(&self, id: i64) -> Result<GameBuildGuideRecord> {
        let connection = self.connection()?;
        Self::get_game_build_guide_by_id(&connection, id)
    }

    pub fn create_game_build_guide(
        &self,
        game_id: i64,
        title: &str,
        source_path: &str,
        raw_markdown: &str,
        build_goal: &str,
        scale_reference: &str,
        geometry_notes: &str,
        glossary_text: &str,
        checklist_json: &str,
    ) -> Result<GameBuildGuideRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let clean_title = if title.trim().is_empty() {
            "Build guide"
        } else {
            title.trim()
        };
        connection.execute(
            "
            INSERT INTO obj_game_build_guide (
                game_id, title, source_path, raw_markdown, build_goal, scale_reference,
                geometry_notes, glossary_text, checklist_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                game_id,
                clean_title,
                source_path.trim(),
                raw_markdown,
                build_goal.trim(),
                scale_reference.trim(),
                geometry_notes.trim(),
                glossary_text.trim(),
                checklist_json.trim()
            ],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_build_guide_by_id(&connection, id)
    }

    pub fn replace_game_build_guide_parts(
        &self,
        guide_id: i64,
        parts: &[GameBuildGuidePartDraft],
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_build_guide_by_id(&connection, guide_id)?;
        connection.execute(
            "DELETE FROM obj_game_build_guide_part WHERE guide_id = ?1",
            params![guide_id],
        )?;
        for part in parts {
            connection.execute(
                "
                INSERT INTO obj_game_build_guide_part (
                    guide_id, section, quantity, part_name, purpose, row_order
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    guide_id,
                    part.section.trim(),
                    part.quantity.trim(),
                    part.part_name.trim(),
                    part.purpose.trim(),
                    part.row_order
                ],
            )?;
        }
        connection.execute(
            "UPDATE obj_game_build_guide SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![guide_id],
        )?;
        Ok(())
    }

    pub fn replace_game_build_guide_steps(
        &self,
        guide_id: i64,
        steps: &[GameBuildGuideStepDraft],
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_build_guide_by_id(&connection, guide_id)?;
        connection.execute(
            "DELETE FROM obj_game_build_guide_step WHERE guide_id = ?1",
            params![guide_id],
        )?;
        for step in steps {
            connection.execute(
                "
                INSERT INTO obj_game_build_guide_step (
                    guide_id, step_number, title, body, row_order
                )
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![
                    guide_id,
                    step.step_number,
                    step.title.trim(),
                    step.body.trim(),
                    step.row_order
                ],
            )?;
        }
        connection.execute(
            "UPDATE obj_game_build_guide SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![guide_id],
        )?;
        Ok(())
    }

    pub fn list_game_build_guide_parts(
        &self,
        guide_id: i64,
    ) -> Result<Vec<GameBuildGuidePartRecord>> {
        let connection = self.connection()?;
        Self::get_game_build_guide_by_id(&connection, guide_id)?;
        Self::list_game_build_guide_parts_for_connection(&connection, guide_id)
    }

    pub fn list_game_build_guide_steps(
        &self,
        guide_id: i64,
    ) -> Result<Vec<GameBuildGuideStepRecord>> {
        let connection = self.connection()?;
        Self::get_game_build_guide_by_id(&connection, guide_id)?;
        Self::list_game_build_guide_steps_for_connection(&connection, guide_id)
    }

    pub fn update_game_build_guide_overlay_bounds(
        &self,
        guide_id: i64,
        overlay_x: Option<i32>,
        overlay_y: Option<i32>,
        overlay_width: Option<i32>,
        overlay_height: Option<i32>,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_build_guide_by_id(&connection, guide_id)?;
        connection.execute(
            "
            UPDATE obj_game_build_guide
            SET overlay_x = COALESCE(?2, overlay_x),
                overlay_y = COALESCE(?3, overlay_y),
                overlay_width = COALESCE(?4, overlay_width),
                overlay_height = COALESCE(?5, overlay_height),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            ",
            params![
                guide_id,
                overlay_x,
                overlay_y,
                overlay_width,
                overlay_height
            ],
        )?;
        Ok(())
    }

    pub fn create_game_chat_conversation(
        &self,
        game_id: i64,
        title: &str,
    ) -> Result<GameChatConversationRecord> {
        let connection = self.connection()?;
        Self::get_game_by_id(&connection, game_id)?;
        let clean_title = if title.trim().is_empty() {
            "Game chat"
        } else {
            title.trim()
        };

        connection.execute(
            "
            INSERT INTO obj_game_chat_conversation (game_id, title)
            VALUES (?1, ?2)
            ",
            params![game_id, clean_title],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_chat_conversation_by_id(&connection, id)
    }

    pub fn get_game_chat_conversation(&self, id: i64) -> Result<GameChatConversationRecord> {
        let connection = self.connection()?;
        Self::get_game_chat_conversation_by_id(&connection, id)
    }

    pub fn update_game_chat_overlay_position(
        &self,
        conversation_id: i64,
        overlay_x: i32,
        overlay_y: i32,
    ) -> Result<()> {
        let connection = self.connection()?;
        Self::get_game_chat_conversation_by_id(&connection, conversation_id)?;
        connection.execute(
            "
            UPDATE obj_game_chat_conversation
            SET overlay_x = ?2, overlay_y = ?3
            WHERE id = ?1
            ",
            params![conversation_id, overlay_x, overlay_y],
        )?;
        Ok(())
    }

    pub fn list_game_chat_messages(
        &self,
        conversation_id: i64,
    ) -> Result<Vec<GameChatMessageRecord>> {
        let connection = self.connection()?;
        Self::get_game_chat_conversation_by_id(&connection, conversation_id)?;
        Self::list_game_chat_messages_for_connection(&connection, conversation_id)
    }

    pub fn recent_game_chat_messages(
        &self,
        conversation_id: i64,
        limit: i64,
    ) -> Result<Vec<GameChatMessageRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM (
                SELECT id, conversation_id, role, content, created_at
                FROM obj_game_chat_message
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
        let connection = self.connection()?;
        Self::get_game_chat_conversation_by_id(&connection, conversation_id)?;
        connection.execute(
            "
            INSERT INTO obj_game_chat_message (conversation_id, role, content)
            VALUES (?1, ?2, ?3)
            ",
            params![conversation_id, role.trim(), content.trim()],
        )?;
        connection.execute(
            "
            UPDATE obj_game_chat_conversation
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            ",
            params![conversation_id],
        )?;

        let id = connection.last_insert_rowid();
        Self::get_game_chat_message_by_id(&connection, id)
    }

    pub fn delete_game_chat_conversation(&self, conversation_id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_game_chat_message WHERE conversation_id = ?1",
            params![conversation_id],
        )?;
        connection.execute(
            "DELETE FROM obj_game_chat_conversation WHERE id = ?1",
            params![conversation_id],
        )?;
        Ok(())
    }

    pub fn list_game_screenshots(
        &self,
        game_id: i64,
    ) -> Result<Vec<GameScreenshotCaptureRequestRecord>> {
        let connection = self.connection()?;
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
            FROM obj_game_catalog_screenshot
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
        let connection = self.connection()?;
        Self::get_game_screenshot_capture_request_by_id(&connection, id).optional()
    }

    pub fn delete_game_screenshot_references(
        &self,
        game_id: i64,
        file_path: &str,
        request_path: &str,
    ) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            DELETE FROM obj_game_catalog_reference
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
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM obj_game_catalog_screenshot WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn get_task_by_id(connection: &Connection, id: i64) -> Result<TaskRecord> {
        connection.query_row(
            "
            SELECT id, title, body, deadline, is_completed, created_at, updated_at
            FROM obj_task
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

    fn get_smoking_event_by_id(connection: &Connection, id: i64) -> Result<SmokingEventRecord> {
        connection.query_row(
            "
            SELECT id, smoked_at, source, notes, created_at
            FROM obj_smoking_event
            WHERE id = ?1
            ",
            params![id],
            smoking_event_from_row,
        )
    }

    fn get_smoking_cessation_settings_for_connection(
        connection: &Connection,
    ) -> Result<SmokingCessationSettingsRecord> {
        connection.query_row(
            "
            SELECT id, patch_label, patch_started_at, patch_timezone, current_cigarette_count, created_at, updated_at
            FROM obj_smoking_cessation_setting
            WHERE id = 1
            ",
            [],
            smoking_cessation_settings_from_row,
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

    fn backfill_game_runtime_part_positions(connection: &Connection) -> Result<()> {
        let rows = {
            let mut statement = connection.prepare(
                "
                SELECT id, properties_json
                FROM obj_game_runtime_part
                WHERE
                    world_x IS NULL
                    OR world_y IS NULL
                    OR world_z IS NULL
                    OR local_x IS NULL
                    OR local_y IS NULL
                    OR local_z IS NULL
                    OR world_position_json = ''
                    OR world_position_json = '{}'
                    OR local_position_json = ''
                    OR local_position_json = '{}'
                ",
            )?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>>>()?;
            rows
        };

        for (id, properties_json) in rows {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&properties_json) else {
                continue;
            };
            let world_position = value.get("position").and_then(db_json_vector3);
            let local_position = value.get("localPosition").and_then(db_json_vector3);
            if world_position.is_none() && local_position.is_none() {
                continue;
            }

            let (world_x, world_y, world_z) = match world_position {
                Some((x, y, z)) => (Some(x), Some(y), Some(z)),
                None => (None, None, None),
            };
            let (local_x, local_y, local_z) = match local_position {
                Some((x, y, z)) => (Some(x), Some(y), Some(z)),
                None => (None, None, None),
            };
            let world_position_json = value
                .get("position")
                .and_then(|position| serde_json::to_string(position).ok())
                .unwrap_or_else(|| "{}".to_string());
            let local_position_json = value
                .get("localPosition")
                .and_then(|position| serde_json::to_string(position).ok())
                .unwrap_or_else(|| "{}".to_string());

            connection.execute(
                "
                UPDATE obj_game_runtime_part
                SET
                    world_x = ?1,
                    world_y = ?2,
                    world_z = ?3,
                    local_x = ?4,
                    local_y = ?5,
                    local_z = ?6,
                    world_position_json = ?7,
                    local_position_json = ?8,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?9
                ",
                params![
                    world_x,
                    world_y,
                    world_z,
                    local_x,
                    local_y,
                    local_z,
                    world_position_json,
                    local_position_json,
                    id
                ],
            )?;
        }

        Ok(())
    }

    fn get_note_by_id(connection: &Connection, id: i64) -> Result<NoteRecord> {
        connection.query_row(
            "
            SELECT id, title, body, created_at, updated_at
            FROM obj_note
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
            FROM obj_calendar_event
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
            FROM obj_project
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
            FROM obj_project_github_repository
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
            FROM obj_project_markdown_context
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
            FROM obj_planning_conversation
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
            FROM obj_planning_message
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
            FROM obj_planning_message
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
            FROM n2n_planning_conversation_context
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
            FROM n2n_planning_conversation_context
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
            FROM n2n_planning_conversation_context
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
            FROM obj_youtube_reference
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
            FROM obj_game
            WHERE id = ?1
            ",
            params![id],
            game_from_row,
        )
    }

    fn get_game_data_location_by_type(
        connection: &Connection,
        game_id: i64,
        location_type: &str,
    ) -> Result<GameDataLocationRecord> {
        connection.query_row(
            "
            SELECT id, game_id, location_type, label, directory_path, created_at, updated_at
            FROM obj_game_data_location
            WHERE game_id = ?1 AND location_type = ?2
            ",
            params![game_id, location_type],
            game_data_location_from_row,
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
            FROM obj_game_catalog_object
            WHERE game_id = ?1
                AND name = ?2 COLLATE NOCASE
            ",
            params![game_id, name.trim()],
            game_catalog_object_from_row,
        )
    }

    fn get_game_runtime_part_by_key(
        connection: &Connection,
        game_id: i64,
        part_key: &str,
    ) -> Result<GameRuntimePartRecord> {
        connection.query_row(
            "
            SELECT
                id,
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                mass,
                world_x,
                world_y,
                world_z,
                local_x,
                local_y,
                local_z,
                world_position_json,
                local_position_json,
                properties_json,
                source_export_id,
                source_construction_id,
                last_seen_at,
                display_image_path,
                source_image_path,
                notes,
                created_at,
                updated_at
            FROM obj_game_runtime_part
            WHERE game_id = ?1
                AND part_key = ?2
            ",
            params![game_id, part_key.trim()],
            game_runtime_part_from_row,
        )
    }

    fn get_game_runtime_part_by_id(
        connection: &Connection,
        game_id: i64,
        part_id: i64,
    ) -> Result<GameRuntimePartRecord> {
        connection.query_row(
            "
            SELECT
                id,
                game_id,
                part_key,
                asset_guid,
                asset_name,
                display_name,
                full_display_name,
                category,
                mass,
                world_x,
                world_y,
                world_z,
                local_x,
                local_y,
                local_z,
                world_position_json,
                local_position_json,
                properties_json,
                source_export_id,
                source_construction_id,
                last_seen_at,
                display_image_path,
                source_image_path,
                notes,
                created_at,
                updated_at
            FROM obj_game_runtime_part
            WHERE id = ?1
                AND game_id = ?2
            ",
            params![part_id, game_id],
            game_runtime_part_from_row,
        )
    }

    fn get_game_runtime_construction_export_by_export_id(
        connection: &Connection,
        game_id: i64,
        export_id: &str,
    ) -> Result<GameRuntimeConstructionExportRecord> {
        connection.query_row(
            "
            SELECT
                id,
                game_id,
                export_id,
                name,
                export_kind,
                intended_path,
                source_log_path,
                byte_size,
                construction_id,
                exported_at,
                part_count,
                mass,
                is_frozen,
                is_invulnerable,
                is_player_character,
                document_json,
                last_indexed_at,
                created_at,
                updated_at
            FROM obj_game_runtime_construction_export
            WHERE game_id = ?1
                AND export_id = ?2
            ",
            params![game_id, export_id.trim()],
            game_runtime_construction_export_from_row,
        )
    }

    fn get_game_construction_by_path(
        connection: &Connection,
        game_id: i64,
        construction_path: &str,
    ) -> Result<GameConstructionRecord> {
        connection.query_row(
            "
            SELECT
                id,
                game_id,
                name,
                folder_path,
                construction_path,
                byte_size,
                decoded_byte_size,
                composite_count,
                part_count,
                unique_asset_guid_count,
                attachment_count,
                link_count,
                intersection_count,
                is_frozen,
                is_invulnerable,
                summary_json,
                document_json,
                last_indexed_at,
                created_at,
                updated_at
            FROM obj_game_construction
            WHERE game_id = ?1
                AND construction_path = ?2
            ",
            params![game_id, construction_path.trim()],
            game_construction_from_row,
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
            FROM obj_game_catalog_screenshot
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
            SELECT id, game_id, title, overlay_x, overlay_y, created_at, updated_at
            FROM obj_game_chat_conversation
            WHERE id = ?1
            ",
            params![id],
            game_chat_conversation_from_row,
        )
    }

    fn get_game_build_guide_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<GameBuildGuideRecord> {
        connection.query_row(
            "
            SELECT id, game_id, title, source_path, raw_markdown, build_goal, scale_reference,
                geometry_notes, glossary_text, checklist_json, overlay_x, overlay_y, overlay_width,
                overlay_height, created_at, updated_at
            FROM obj_game_build_guide
            WHERE id = ?1
            ",
            params![id],
            game_build_guide_from_row,
        )
    }

    fn list_game_build_guide_parts_for_connection(
        connection: &Connection,
        guide_id: i64,
    ) -> Result<Vec<GameBuildGuidePartRecord>> {
        let mut statement = connection.prepare(
            "
            SELECT id, guide_id, section, quantity, part_name, purpose, row_order, created_at,
                updated_at
            FROM obj_game_build_guide_part
            WHERE guide_id = ?1
            ORDER BY row_order ASC, id ASC
            ",
        )?;

        let parts = statement
            .query_map(params![guide_id], game_build_guide_part_from_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(parts)
    }

    fn list_game_build_guide_steps_for_connection(
        connection: &Connection,
        guide_id: i64,
    ) -> Result<Vec<GameBuildGuideStepRecord>> {
        let mut statement = connection.prepare(
            "
            SELECT id, guide_id, step_number, title, body, row_order, created_at, updated_at
            FROM obj_game_build_guide_step
            WHERE guide_id = ?1
            ORDER BY row_order ASC, id ASC
            ",
        )?;

        let steps = statement
            .query_map(params![guide_id], game_build_guide_step_from_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(steps)
    }

    fn get_game_chat_message_by_id(
        connection: &Connection,
        id: i64,
    ) -> Result<GameChatMessageRecord> {
        connection.query_row(
            "
            SELECT id, conversation_id, role, content, created_at
            FROM obj_game_chat_message
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
            FROM obj_game_chat_message
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
            FROM obj_bridge_file_draft
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
                    "SELECT content FROM obj_scratchpad WHERE id = 1",
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
            FROM obj_project_github_repository
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

struct ParsedGearBlocksApiMember {
    member_key: String,
    member_name: String,
    signature: String,
    member_kind: &'static str,
    is_readable: i64,
    is_invokable: i64,
    is_mutating: i64,
    parameters: Vec<ParsedGearBlocksApiParameter>,
}

struct ParsedGearBlocksApiParameter {
    position: i64,
    name: String,
    parameter_type: String,
    default_value: String,
    is_optional: i64,
}

fn parse_gearblocks_api_member(raw_member: &str, type_name: &str) -> ParsedGearBlocksApiMember {
    let signature = raw_member.trim().to_string();
    let is_method = signature.contains('(') && signature.ends_with(')');
    let member_name = if is_method {
        signature
            .split('(')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        signature.clone()
    };

    ParsedGearBlocksApiMember {
        member_key: signature.clone(),
        member_name: member_name.clone(),
        signature: signature.clone(),
        member_kind: if is_method { "method" } else { "property" },
        is_readable: if is_method { 0 } else { 1 },
        is_invokable: if is_method { 1 } else { 0 },
        is_mutating: if is_mutating_gearblocks_api_member(type_name, &member_name) {
            1
        } else {
            0
        },
        parameters: if is_method {
            parse_gearblocks_api_parameters(&signature)
        } else {
            Vec::new()
        },
    }
}

fn gearblocks_observed_member_name(attribute_name: &str) -> String {
    attribute_name
        .split('(')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_gearblocks_api_parameters(signature: &str) -> Vec<ParsedGearBlocksApiParameter> {
    let Some(start) = signature.find('(') else {
        return Vec::new();
    };
    let Some(end) = signature.rfind(')') else {
        return Vec::new();
    };
    let parameters_text = signature[start + 1..end].trim();
    if parameters_text.is_empty() {
        return Vec::new();
    }

    split_gearblocks_parameter_list(parameters_text)
        .into_iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            let mut parts = parameter.splitn(2, '=');
            let declaration = parts.next().unwrap_or_default().trim();
            let default_value = parts.next().unwrap_or_default().trim().to_string();
            let mut declaration_parts = declaration.rsplitn(2, ' ');
            let name = declaration_parts.next().unwrap_or_default().trim();
            let parameter_type = declaration_parts.next().unwrap_or_default().trim();
            if name.is_empty() {
                None
            } else {
                Some(ParsedGearBlocksApiParameter {
                    position: index as i64,
                    name: name.to_string(),
                    parameter_type: parameter_type.to_string(),
                    is_optional: if default_value.is_empty() { 0 } else { 1 },
                    default_value,
                })
            }
        })
        .collect()
}

fn split_gearblocks_parameter_list(parameters_text: &str) -> Vec<String> {
    let mut parameters = Vec::new();
    let mut current = String::new();
    let mut generic_depth = 0_i64;
    for character in parameters_text.chars() {
        match character {
            '<' => {
                generic_depth += 1;
                current.push(character);
            }
            '>' => {
                generic_depth = (generic_depth - 1).max(0);
                current.push(character);
            }
            ',' if generic_depth == 0 => {
                let parameter = current.trim();
                if !parameter.is_empty() {
                    parameters.push(parameter.to_string());
                }
                current.clear();
            }
            _ => current.push(character),
        }
    }
    let parameter = current.trim();
    if !parameter.is_empty() {
        parameters.push(parameter.to_string());
    }
    parameters
}

fn is_mutating_gearblocks_api_member(type_name: &str, member_name: &str) -> bool {
    if type_name.ends_with("Operations") {
        return true;
    }
    [
        "Assign",
        "Charge",
        "Create",
        "Delete",
        "Destroy",
        "Discharge",
        "Duplicate",
        "Freeze",
        "Replace",
        "Set",
        "Spawn",
        "Sync",
        "Toggle",
    ]
    .iter()
    .any(|prefix| member_name.starts_with(prefix))
}

fn gearblocks_api_docs_url(page: &str) -> String {
    if page.starts_with("http://") || page.starts_with("https://") {
        page.to_string()
    } else {
        format!("https://www.gearblocksgame.com/apidoc/{page}")
    }
}

fn smoking_event_from_row(row: &rusqlite::Row<'_>) -> Result<SmokingEventRecord> {
    Ok(SmokingEventRecord {
        id: row.get(0)?,
        smoked_at: row.get(1)?,
        source: row.get(2)?,
        notes: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn smoking_cessation_settings_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<SmokingCessationSettingsRecord> {
    Ok(SmokingCessationSettingsRecord {
        id: row.get(0)?,
        patch_label: row.get(1)?,
        patch_started_at: row.get(2)?,
        patch_timezone: row.get(3)?,
        current_cigarette_count: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn scheduler_from_row(row: &rusqlite::Row<'_>) -> Result<SchedulerRecord> {
    Ok(SchedulerRecord {
        id: row.get(0)?,
        type_id: row.get(1)?,
        type_key: row.get(2)?,
        type_label: row.get(3)?,
        owner_module: row.get(4)?,
        name: row.get(5)?,
        is_enabled: row.get::<_, i64>(6)? == 1,
        interval_seconds: row.get(7)?,
        run_on_startup: row.get::<_, i64>(8)? == 1,
        coalesce_missed_runs: row.get::<_, i64>(9)? == 1,
        payload_json: row.get(10)?,
        next_run_at: row.get(11)?,
        last_run_at: row.get(12)?,
        last_status: row.get(13)?,
        last_error: row.get(14)?,
        lease_until: row.get(15)?,
        created_at: row.get(16)?,
        modified_at: row.get(17)?,
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

fn game_data_location_from_row(row: &rusqlite::Row<'_>) -> Result<GameDataLocationRecord> {
    Ok(GameDataLocationRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        location_type: row.get(2)?,
        label: row.get(3)?,
        directory_path: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
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

fn gearblocks_api_type_from_row(row: &rusqlite::Row<'_>) -> Result<GearBlocksApiTypeRecord> {
    Ok(GearBlocksApiTypeRecord {
        id: row.get(0)?,
        namespace: row.get(1)?,
        type_name: row.get(2)?,
        type_kind: row.get(3)?,
        docs_url: row.get(4)?,
        source: row.get(5)?,
        source_version: row.get(6)?,
        notes: row.get(7)?,
        member_count: row.get(8)?,
        enum_value_count: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn gearblocks_api_member_from_row(row: &rusqlite::Row<'_>) -> Result<GearBlocksApiMemberRecord> {
    let is_readable: i64 = row.get(8)?;
    let is_writable: i64 = row.get(9)?;
    let is_invokable: i64 = row.get(10)?;
    let is_mutating: i64 = row.get(11)?;
    Ok(GearBlocksApiMemberRecord {
        id: row.get(0)?,
        type_id: row.get(1)?,
        type_name: row.get(2)?,
        member_key: row.get(3)?,
        member_name: row.get(4)?,
        signature: row.get(5)?,
        member_kind: row.get(6)?,
        return_type: row.get(7)?,
        is_readable: is_readable != 0,
        is_writable: is_writable != 0,
        is_invokable: is_invokable != 0,
        is_mutating: is_mutating != 0,
        docs_url: row.get(12)?,
        source: row.get(13)?,
        source_version: row.get(14)?,
        notes: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

fn gearblocks_api_parameter_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<GearBlocksApiParameterRecord> {
    let is_optional: i64 = row.get(6)?;
    Ok(GearBlocksApiParameterRecord {
        id: row.get(0)?,
        member_id: row.get(1)?,
        position: row.get(2)?,
        parameter_name: row.get(3)?,
        parameter_type: row.get(4)?,
        default_value: row.get(5)?,
        is_optional: is_optional != 0,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn gearblocks_api_enum_value_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<GearBlocksApiEnumValueRecord> {
    Ok(GearBlocksApiEnumValueRecord {
        id: row.get(0)?,
        type_id: row.get(1)?,
        position: row.get(2)?,
        value_name: row.get(3)?,
        numeric_value: row.get(4)?,
        lua_name: row.get(5)?,
        description: row.get(6)?,
        source: row.get(7)?,
        source_version: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn game_runtime_part_api_member_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<GameRuntimePartApiMemberRecord> {
    let is_readable: i64 = row.get(16)?;
    let is_writable: i64 = row.get(17)?;
    let is_invokable: i64 = row.get(18)?;
    let is_mutating: i64 = row.get(19)?;
    Ok(GameRuntimePartApiMemberRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        part_key: row.get(2)?,
        api_member_id: row.get(3)?,
        availability: row.get(4)?,
        source_export_id: row.get(5)?,
        source_construction_id: row.get(6)?,
        first_seen_at: row.get(7)?,
        last_seen_at: row.get(8)?,
        namespace: row.get(9)?,
        type_name: row.get(10)?,
        type_kind: row.get(11)?,
        member_key: row.get(12)?,
        member_name: row.get(13)?,
        signature: row.get(14)?,
        member_kind: row.get(15)?,
        is_readable: is_readable != 0,
        is_writable: is_writable != 0,
        is_invokable: is_invokable != 0,
        is_mutating: is_mutating != 0,
        docs_url: row.get(20)?,
        created_at: row.get(21)?,
        updated_at: row.get(22)?,
    })
}

fn game_runtime_part_from_row(row: &rusqlite::Row<'_>) -> Result<GameRuntimePartRecord> {
    Ok(GameRuntimePartRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        part_key: row.get(2)?,
        asset_guid: row.get(3)?,
        asset_name: row.get(4)?,
        display_name: row.get(5)?,
        full_display_name: row.get(6)?,
        category: row.get(7)?,
        mass: row.get(8)?,
        world_x: row.get(9)?,
        world_y: row.get(10)?,
        world_z: row.get(11)?,
        local_x: row.get(12)?,
        local_y: row.get(13)?,
        local_z: row.get(14)?,
        world_position_json: row.get(15)?,
        local_position_json: row.get(16)?,
        properties_json: row.get(17)?,
        source_export_id: row.get(18)?,
        source_construction_id: row.get(19)?,
        last_seen_at: row.get(20)?,
        display_image_path: row.get(21)?,
        source_image_path: row.get(22)?,
        notes: row.get(23)?,
        created_at: row.get(24)?,
        updated_at: row.get(25)?,
    })
}

fn game_runtime_part_alias_from_row(row: &rusqlite::Row<'_>) -> Result<GameRuntimePartAliasRecord> {
    Ok(GameRuntimePartAliasRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        part_instance_key: row.get(2)?,
        friendly_name: row.get(3)?,
        asset_guid: row.get(4)?,
        asset_name: row.get(5)?,
        display_name: row.get(6)?,
        full_display_name: row.get(7)?,
        category: row.get(8)?,
        source_log_path: row.get(9)?,
        source_construction_id: row.get(10)?,
        world_position_json: row.get(11)?,
        local_position_json: row.get(12)?,
        current_unit_size_json: row.get(13)?,
        payload_json: row.get(14)?,
        last_seen_at: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

fn db_json_vector3(value: &serde_json::Value) -> Option<(f64, f64, f64)> {
    Some((
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ))
}

fn game_runtime_construction_export_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<GameRuntimeConstructionExportRecord> {
    let is_frozen: Option<i64> = row.get(12)?;
    let is_invulnerable: Option<i64> = row.get(13)?;
    let is_player_character: Option<i64> = row.get(14)?;
    Ok(GameRuntimeConstructionExportRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        export_id: row.get(2)?,
        name: row.get(3)?,
        export_kind: row.get(4)?,
        intended_path: row.get(5)?,
        source_log_path: row.get(6)?,
        byte_size: row.get(7)?,
        construction_id: row.get(8)?,
        exported_at: row.get(9)?,
        part_count: row.get(10)?,
        mass: row.get(11)?,
        is_frozen: is_frozen.map(|value| value != 0),
        is_invulnerable: is_invulnerable.map(|value| value != 0),
        is_player_character: is_player_character.map(|value| value != 0),
        document_json: row.get(15)?,
        last_indexed_at: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

fn game_construction_from_row(row: &rusqlite::Row<'_>) -> Result<GameConstructionRecord> {
    let is_frozen: Option<i64> = row.get(13)?;
    let is_invulnerable: Option<i64> = row.get(14)?;
    Ok(GameConstructionRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        name: row.get(2)?,
        folder_path: row.get(3)?,
        construction_path: row.get(4)?,
        byte_size: row.get(5)?,
        decoded_byte_size: row.get(6)?,
        composite_count: row.get(7)?,
        part_count: row.get(8)?,
        unique_asset_guid_count: row.get(9)?,
        attachment_count: row.get(10)?,
        link_count: row.get(11)?,
        intersection_count: row.get(12)?,
        is_frozen: is_frozen.map(|value| value != 0),
        is_invulnerable: is_invulnerable.map(|value| value != 0),
        summary_json: row.get(15)?,
        document_json: row.get(16)?,
        last_indexed_at: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
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
        overlay_x: row.get(3)?,
        overlay_y: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn game_build_guide_from_row(row: &rusqlite::Row<'_>) -> Result<GameBuildGuideRecord> {
    Ok(GameBuildGuideRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        title: row.get(2)?,
        source_path: row.get(3)?,
        raw_markdown: row.get(4)?,
        build_goal: row.get(5)?,
        scale_reference: row.get(6)?,
        geometry_notes: row.get(7)?,
        glossary_text: row.get(8)?,
        checklist_json: row.get(9)?,
        overlay_x: row.get(10)?,
        overlay_y: row.get(11)?,
        overlay_width: row.get(12)?,
        overlay_height: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

fn game_build_guide_part_from_row(row: &rusqlite::Row<'_>) -> Result<GameBuildGuidePartRecord> {
    Ok(GameBuildGuidePartRecord {
        id: row.get(0)?,
        guide_id: row.get(1)?,
        section: row.get(2)?,
        quantity: row.get(3)?,
        part_name: row.get(4)?,
        purpose: row.get(5)?,
        row_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn game_build_guide_step_from_row(row: &rusqlite::Row<'_>) -> Result<GameBuildGuideStepRecord> {
    Ok(GameBuildGuideStepRecord {
        id: row.get(0)?,
        guide_id: row.get(1)?,
        step_number: row.get(2)?,
        title: row.get(3)?,
        body: row.get(4)?,
        row_order: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "overlay-forge-{test_name}-{}-{unique}.sqlite3",
            std::process::id()
        ))
    }

    fn remove_db_files(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
    }

    #[test]
    fn initializes_normalized_schema_on_new_database() {
        let path = temp_db_path("normalized-schema");
        remove_db_files(&path);

        let database = AppDatabase::new(path.clone()).expect("database should initialize");
        assert!(database.is_ready());

        let connection = Connection::open(&path).expect("database should reopen");
        for table in [
            "def_game",
            "obj_game",
            "obj_game_setting",
            "obj_setting",
            "obj_scheduler",
            "n2n_planning_conversation_context",
        ] {
            assert!(
                AppDatabase::table_exists(&connection, table).expect("table lookup should work"),
                "{table} should exist"
            );
        }

        let game_schema: String = connection
            .query_row(
                "SELECT schema_json FROM obj_game WHERE slug = 'gearblocks'",
                [],
                |row| row.get(0),
            )
            .expect("seeded game should have schema metadata");
        assert!(game_schema.contains("\"table\":\"obj_game\""));

        let path_of_exile_id_game: i64 = connection
            .query_row(
                "SELECT id_game FROM obj_game WHERE slug = 'path-of-exile-2'",
                [],
                |row| row.get(0),
            )
            .expect("Path of Exile 2 game row should be seeded");
        assert_eq!(path_of_exile_id_game, 2);

        drop(connection);
        drop(database);
        remove_db_files(&path);
    }

    #[test]
    fn renames_legacy_game_table_without_dropping_rows() {
        let path = temp_db_path("legacy-rename");
        remove_db_files(&path);

        {
            let connection = Connection::open(&path).expect("legacy database should open");
            connection
                .execute_batch(
                    "
                    CREATE TABLE games (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                        slug TEXT NOT NULL UNIQUE COLLATE NOCASE,
                        summary TEXT NOT NULL DEFAULT '',
                        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                    );

                    INSERT INTO games (name, slug, summary)
                    VALUES ('Legacy Game', 'legacy-game', 'legacy row');
                    ",
                )
                .expect("legacy games table should be created");
        }

        let database = AppDatabase::new(path.clone()).expect("database should migrate");
        let connection = Connection::open(&path).expect("database should reopen after migration");

        assert!(
            !AppDatabase::table_exists(&connection, "games").expect("legacy lookup should work"),
            "legacy games table should be renamed"
        );
        assert!(
            AppDatabase::table_exists(&connection, "obj_game").expect("new lookup should work"),
            "obj_game should exist after rename"
        );

        let (id_game, summary): (i64, String) = connection
            .query_row(
                "SELECT id_game, summary FROM obj_game WHERE slug = 'legacy-game'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("legacy game row should survive migration");
        assert_eq!(id_game, 1);
        assert_eq!(summary, "legacy row");

        drop(connection);
        drop(database);
        remove_db_files(&path);
    }

    #[test]
    fn repairs_partial_legacy_and_normalized_table_state() {
        let path = temp_db_path("partial-legacy-state");
        remove_db_files(&path);

        {
            let connection = Connection::open(&path).expect("partial database should open");
            connection
                .execute_batch(
                    "
                    CREATE TABLE gearblocks_api_types (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        namespace TEXT NOT NULL,
                        type_name TEXT NOT NULL,
                        type_kind TEXT NOT NULL DEFAULT '',
                        docs_url TEXT NOT NULL DEFAULT '',
                        source TEXT NOT NULL DEFAULT '',
                        source_version TEXT NOT NULL DEFAULT '',
                        summary TEXT NOT NULL DEFAULT '',
                        notes TEXT NOT NULL DEFAULT '',
                        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                    );

                    CREATE UNIQUE INDEX idx_gearblocks_api_types_unique
                        ON gearblocks_api_types (namespace, type_name);

                    INSERT INTO gearblocks_api_types (
                        namespace,
                        type_name,
                        type_kind,
                        docs_url,
                        source,
                        source_version,
                        summary,
                        notes
                    )
                    VALUES (
                        'Legacy.Namespace',
                        'LegacyType',
                        'class',
                        '',
                        'test',
                        '1',
                        'legacy row',
                        ''
                    );

                    CREATE TABLE def_gearblocks_api_type (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        namespace TEXT NOT NULL,
                        type_name TEXT NOT NULL,
                        type_kind TEXT NOT NULL DEFAULT '',
                        docs_url TEXT NOT NULL DEFAULT '',
                        source TEXT NOT NULL DEFAULT '',
                        source_version TEXT NOT NULL DEFAULT '',
                        summary TEXT NOT NULL DEFAULT '',
                        notes TEXT NOT NULL DEFAULT '',
                        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                    );
                    ",
                )
                .expect("partial legacy/normalized schema should be created");
        }

        let database = AppDatabase::new(path.clone()).expect("database should repair and migrate");
        let connection = Connection::open(&path).expect("database should reopen after repair");

        assert!(
            !AppDatabase::table_exists(&connection, "gearblocks_api_types")
                .expect("legacy table lookup should work"),
            "legacy duplicate table should be dropped after copy"
        );
        let copied_count: i64 = connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM def_gearblocks_api_type
                WHERE namespace = 'Legacy.Namespace'
                    AND type_name = 'LegacyType'
                ",
                [],
                |row| row.get(0),
            )
            .expect("copied legacy type count should be readable");
        assert_eq!(copied_count, 1);

        let unique_index_table: String = connection
            .query_row(
                "
                SELECT tbl_name
                FROM sqlite_master
                WHERE type = 'index'
                    AND name = 'idx_gearblocks_api_types_unique'
                ",
                [],
                |row| row.get(0),
            )
            .expect("unique index should be recreated");
        assert_eq!(unique_index_table, "def_gearblocks_api_type");

        drop(connection);
        drop(database);
        remove_db_files(&path);
    }
}
