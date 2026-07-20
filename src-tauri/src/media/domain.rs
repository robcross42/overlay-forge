use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const DEFAULT_MEDIA_REGION: &str = "CA";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaContentType {
    Movie,
    Series,
}

impl Display for MediaContentType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Movie => "MOVIE",
            Self::Series => "SERIES",
        })
    }
}

impl FromStr for MediaContentType {
    type Err = MediaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "MOVIE" => Ok(Self::Movie),
            "SERIES" => Ok(Self::Series),
            _ => Err(MediaError::validation(
                "Media type must be MOVIE or SERIES.",
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaLibraryStatus {
    Planned,
    Watching,
    Completed,
    OnHold,
    Dropped,
}

impl MediaLibraryStatus {
    pub fn protects_from_automatic_transition(self) -> bool {
        matches!(self, Self::OnHold | Self::Dropped)
    }
}

impl Display for MediaLibraryStatus {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Planned => "PLANNED",
            Self::Watching => "WATCHING",
            Self::Completed => "COMPLETED",
            Self::OnHold => "ON_HOLD",
            Self::Dropped => "DROPPED",
        })
    }
}

impl FromStr for MediaLibraryStatus {
    type Err = MediaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "PLANNED" => Ok(Self::Planned),
            "WATCHING" => Ok(Self::Watching),
            "COMPLETED" => Ok(Self::Completed),
            "ON_HOLD" => Ok(Self::OnHold),
            "DROPPED" => Ok(Self::Dropped),
            _ => Err(MediaError::validation(
                "Library status must be PLANNED, WATCHING, COMPLETED, ON_HOLD, or DROPPED.",
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaErrorKind {
    MissingCredential,
    Network,
    RateLimited,
    Unauthorized,
    ProviderUnavailable,
    NotFound,
    Validation,
    Duplicate,
    Database,
    ExternalOpen,
    Unknown,
}

impl Display for MediaErrorKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::MissingCredential => "MissingCredential",
            Self::Network => "Network",
            Self::RateLimited => "RateLimited",
            Self::Unauthorized => "Unauthorized",
            Self::ProviderUnavailable => "ProviderUnavailable",
            Self::NotFound => "NotFound",
            Self::Validation => "Validation",
            Self::Duplicate => "Duplicate",
            Self::Database => "Database",
            Self::ExternalOpen => "ExternalOpen",
            Self::Unknown => "Unknown",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaError {
    pub kind: MediaErrorKind,
    pub message: String,
}

impl MediaError {
    pub fn new(kind: MediaErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::Validation, message)
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::Database, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::NotFound, message)
    }
}

impl Display for MediaError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for MediaError {}

impl From<rusqlite::Error> for MediaError {
    fn from(error: rusqlite::Error) -> Self {
        Self::database(format!("Media Library database operation failed: {error}"))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCatalogSearchResult {
    pub external_id: i64,
    pub external_media_type: String,
    pub content_type: MediaContentType,
    pub title: String,
    pub original_title: String,
    pub overview: String,
    pub original_language: String,
    pub release_date: String,
    pub poster_path: String,
    pub backdrop_path: String,
    pub already_in_library: bool,
    pub existing_entry_id: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct NormalizedMediaTitle {
    pub source_key: String,
    pub external_id: Option<i64>,
    pub external_media_type: Option<String>,
    pub content_type: MediaContentType,
    pub title: String,
    pub original_title: String,
    pub overview: String,
    pub original_language: String,
    pub release_date: String,
    pub first_air_date: String,
    pub last_air_date: String,
    pub runtime_minutes: Option<i64>,
    pub episode_runtime_minutes: Option<i64>,
    pub external_status: String,
    pub series_type: String,
    pub poster_path: String,
    pub backdrop_path: String,
    pub homepage_url: String,
    pub total_seasons: Option<i64>,
    pub total_episodes: Option<i64>,
    pub metadata_json: String,
}

#[derive(Clone, Debug)]
pub struct NormalizedMediaSeason {
    pub external_id: Option<i64>,
    pub season_number: i64,
    pub name: String,
    pub overview: String,
    pub air_date: String,
    pub poster_path: String,
    pub episode_count: i64,
    pub metadata_json: String,
    pub episodes: Vec<NormalizedMediaEpisode>,
}

#[derive(Clone, Debug)]
pub struct NormalizedMediaEpisode {
    pub external_id: Option<i64>,
    pub season_number: i64,
    pub episode_number: i64,
    pub name: String,
    pub overview: String,
    pub air_date: String,
    pub runtime_minutes: Option<i64>,
    pub still_path: String,
    pub metadata_json: String,
}

#[derive(Clone, Debug)]
pub struct NormalizedProviderAvailability {
    pub provider_external_id: i64,
    pub provider_name: String,
    pub provider_logo_path: String,
    pub monetization_type: String,
    pub display_priority: i64,
}

#[derive(Clone, Debug)]
pub struct NormalizedProviderSnapshot {
    pub region_code: String,
    pub source_link_url: String,
    pub providers: Vec<NormalizedProviderAvailability>,
}

#[derive(Clone, Debug)]
pub struct NormalizedMediaImport {
    pub title: NormalizedMediaTitle,
    pub seasons: Vec<NormalizedMediaSeason>,
    pub provider_snapshot: Option<NormalizedProviderSnapshot>,
    pub provider_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCatalogMediaInput {
    pub external_id: i64,
    pub external_media_type: String,
    pub library_status: MediaLibraryStatus,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManualMediaInput {
    pub content_type: MediaContentType,
    pub title: String,
    #[serde(default)]
    pub release_date: String,
    #[serde(default)]
    pub overview: String,
    pub runtime_minutes: Option<i64>,
    pub library_status: MediaLibraryStatus,
    #[serde(default)]
    pub notes: String,
    pub personal_rating: Option<f64>,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub add_to_watch_next: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMediaEntryInput {
    pub library_status: MediaLibraryStatus,
    pub is_favorite: bool,
    pub personal_rating: Option<f64>,
    #[serde(default)]
    pub notes: String,
    pub priority: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryFilter {
    #[serde(default)]
    pub query: String,
    pub content_type: Option<MediaContentType>,
    pub library_status: Option<MediaLibraryStatus>,
    pub is_favorite: Option<bool>,
    pub tag_id: Option<i64>,
    pub has_unwatched: Option<bool>,
    #[serde(default)]
    pub provider_name: String,
    pub is_watch_next: Option<bool>,
    #[serde(default = "default_media_sort")]
    pub sort: String,
    #[serde(default = "default_sort_direction")]
    pub sort_direction: String,
    #[serde(default = "default_page_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_media_sort() -> String {
    "RECENTLY_ADDED".to_string()
}

fn default_sort_direction() -> String {
    "DESC".to_string()
}

fn default_page_limit() -> i64 {
    100
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaNextEpisode {
    pub id: i64,
    pub season_number: i64,
    pub episode_number: i64,
    pub name: String,
    pub air_date: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaProviderSummary {
    pub provider_name: String,
    pub provider_logo_path: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibrarySummary {
    pub id: i64,
    pub media_title_id: i64,
    pub source_key: String,
    pub external_id: Option<i64>,
    pub external_media_type: Option<String>,
    pub content_type: MediaContentType,
    pub title: String,
    pub original_title: String,
    pub overview: String,
    pub original_language: String,
    pub release_date: String,
    pub first_air_date: String,
    pub last_air_date: String,
    pub runtime_minutes: Option<i64>,
    pub episode_runtime_minutes: Option<i64>,
    pub external_status: String,
    pub series_type: String,
    pub poster_path: String,
    pub backdrop_path: String,
    pub homepage_url: String,
    pub total_seasons: Option<i64>,
    pub total_episodes: Option<i64>,
    pub metadata_refreshed_at: String,
    pub library_status: MediaLibraryStatus,
    pub is_favorite: bool,
    pub personal_rating: Option<f64>,
    pub notes: String,
    pub priority: i64,
    pub queue_position: Option<i64>,
    pub added_at: String,
    pub started_at: String,
    pub completed_at: String,
    pub last_watched_at: String,
    pub movie_watched: bool,
    pub watched_count: i64,
    pub progress_total: i64,
    pub progress_percent: f64,
    pub next_episode: Option<MediaNextEpisode>,
    pub new_episodes_count: i64,
    pub subscription_providers: Vec<MediaProviderSummary>,
    pub availability_is_stale: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEpisodeRecord {
    pub id: i64,
    pub external_id: Option<i64>,
    pub season_number: i64,
    pub episode_number: i64,
    pub name: String,
    pub overview: String,
    pub air_date: String,
    pub runtime_minutes: Option<i64>,
    pub still_path: String,
    pub is_present_in_source: bool,
    pub is_watched: bool,
    pub watched_at: String,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSeasonRecord {
    pub id: i64,
    pub external_id: Option<i64>,
    pub season_number: i64,
    pub name: String,
    pub overview: String,
    pub air_date: String,
    pub poster_path: String,
    pub episode_count: i64,
    pub is_present_in_source: bool,
    pub watched_count: i64,
    pub progress_total: i64,
    pub episodes: Vec<MediaEpisodeRecord>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTagRecord {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStreamingLinkRecord {
    pub id: i64,
    pub media_library_entry_id: i64,
    pub provider_name: String,
    pub url: String,
    pub link_type: String,
    pub is_preferred: bool,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStreamingLinkInput {
    #[serde(default)]
    pub provider_name: String,
    pub url: String,
    pub link_type: String,
    #[serde(default)]
    pub is_preferred: bool,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaProviderSnapshotRecord {
    pub id: i64,
    pub region_code: String,
    pub source_link_url: String,
    pub refreshed_at: String,
    pub last_refresh_status: String,
    pub last_refresh_error: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaProviderAvailabilityRecord {
    pub id: i64,
    pub provider_external_id: i64,
    pub provider_name: String,
    pub provider_logo_path: String,
    pub monetization_type: String,
    pub display_priority: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSettingsRecord {
    pub region_code: String,
    pub metadata_language: String,
    pub include_specials_in_completion: bool,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMediaSettingsInput {
    pub region_code: String,
    pub metadata_language: String,
    pub include_specials_in_completion: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryDetail {
    pub entry: MediaLibrarySummary,
    pub seasons: Vec<MediaSeasonRecord>,
    pub tags: Vec<MediaTagRecord>,
    pub streaming_links: Vec<MediaStreamingLinkRecord>,
    pub provider_snapshot: Option<MediaProviderSnapshotRecord>,
    pub providers: Vec<MediaProviderAvailabilityRecord>,
    pub settings: MediaSettingsRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgressFacts {
    pub watched: i64,
    pub total: i64,
}

pub fn automatic_series_status(
    current: MediaLibraryStatus,
    facts: ProgressFacts,
) -> MediaLibraryStatus {
    if current.protects_from_automatic_transition() {
        return current;
    }
    if facts.total > 0 && facts.watched >= facts.total {
        MediaLibraryStatus::Completed
    } else if facts.watched > 0 || current == MediaLibraryStatus::Completed {
        MediaLibraryStatus::Watching
    } else {
        current
    }
}

pub fn validate_rating(rating: Option<f64>) -> Result<(), MediaError> {
    if rating.is_some_and(|value| !(1.0..=10.0).contains(&value)) {
        return Err(MediaError::validation(
            "Personal rating must be between 1.0 and 10.0.",
        ));
    }
    Ok(())
}

pub fn validate_http_url(url: &str) -> Result<String, MediaError> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if trimmed.is_empty() {
        return Err(MediaError::validation("A URL is required."));
    }
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(MediaError::validation(
            "Only http and https URLs can be opened.",
        ));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(MediaError::validation("The URL cannot contain whitespace."));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_library_statuses() {
        assert_eq!(
            MediaLibraryStatus::from_str("ON_HOLD").expect("status should parse"),
            MediaLibraryStatus::OnHold
        );
        assert!(MediaLibraryStatus::from_str("NOT_INTERESTED").is_err());
    }

    #[test]
    fn series_progress_transitions_are_centralized() {
        assert_eq!(
            automatic_series_status(
                MediaLibraryStatus::Planned,
                ProgressFacts {
                    watched: 1,
                    total: 10
                }
            ),
            MediaLibraryStatus::Watching
        );
        assert_eq!(
            automatic_series_status(
                MediaLibraryStatus::Watching,
                ProgressFacts {
                    watched: 10,
                    total: 10
                }
            ),
            MediaLibraryStatus::Completed
        );
        assert_eq!(
            automatic_series_status(
                MediaLibraryStatus::Completed,
                ProgressFacts {
                    watched: 9,
                    total: 10
                }
            ),
            MediaLibraryStatus::Watching
        );
    }

    #[test]
    fn automatic_progress_does_not_override_hold_or_dropped() {
        for status in [MediaLibraryStatus::OnHold, MediaLibraryStatus::Dropped] {
            assert_eq!(
                automatic_series_status(
                    status,
                    ProgressFacts {
                        watched: 10,
                        total: 10
                    }
                ),
                status
            );
        }
    }

    #[test]
    fn external_urls_accept_only_http_and_https() {
        assert!(validate_http_url("https://example.com/watch").is_ok());
        assert!(validate_http_url("http://example.com/watch").is_ok());
        assert!(validate_http_url("file:///tmp/media").is_err());
        assert!(validate_http_url("javascript:alert(1)").is_err());
    }
}
