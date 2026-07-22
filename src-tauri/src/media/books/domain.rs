use crate::media::domain::{MediaError, MediaLibraryStatus, MediaTagRecord};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookSearchMode {
    General,
    Title,
    Author,
    Isbn,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookFormat {
    Hardcover,
    Paperback,
    Ebook,
    Audiobook,
    Other,
    #[default]
    Unknown,
}

impl Display for BookFormat {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Hardcover => "HARDCOVER",
            Self::Paperback => "PAPERBACK",
            Self::Ebook => "EBOOK",
            Self::Audiobook => "AUDIOBOOK",
            Self::Other => "OTHER",
            Self::Unknown => "UNKNOWN",
        })
    }
}

impl FromStr for BookFormat {
    type Err = MediaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.trim().to_ascii_uppercase().as_str() {
            "HARDCOVER" => Self::Hardcover,
            "PAPERBACK" => Self::Paperback,
            "EBOOK" => Self::Ebook,
            "AUDIOBOOK" => Self::Audiobook,
            "OTHER" => Self::Other,
            "" | "UNKNOWN" => Self::Unknown,
            _ => return Err(MediaError::validation("Unsupported book format.")),
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookProgressUnit {
    Page,
    Percent,
    Minute,
    Chapter,
}

impl Display for BookProgressUnit {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Page => "PAGE",
            Self::Percent => "PERCENT",
            Self::Minute => "MINUTE",
            Self::Chapter => "CHAPTER",
        })
    }
}

impl FromStr for BookProgressUnit {
    type Err = MediaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "PAGE" => Ok(Self::Page),
            "PERCENT" => Ok(Self::Percent),
            "MINUTE" => Ok(Self::Minute),
            "CHAPTER" => Ok(Self::Chapter),
            _ => Err(MediaError::validation("Unsupported book progress unit.")),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookOwnershipStatus {
    #[default]
    None,
    Owned,
    Borrowed,
    Wishlist,
}

impl Display for BookOwnershipStatus {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::None => "NONE",
            Self::Owned => "OWNED",
            Self::Borrowed => "BORROWED",
            Self::Wishlist => "WISHLIST",
        })
    }
}

impl FromStr for BookOwnershipStatus {
    type Err = MediaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "NONE" => Ok(Self::None),
            "OWNED" => Ok(Self::Owned),
            "BORROWED" => Ok(Self::Borrowed),
            "WISHLIST" => Ok(Self::Wishlist),
            _ => Err(MediaError::validation("Unsupported book ownership status.")),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSearchInput {
    pub query: String,
    pub mode: BookSearchMode,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    #[serde(default)]
    pub language: String,
}

fn default_search_limit() -> usize {
    20
}

impl BookSearchInput {
    pub fn validated_query(&self) -> Result<String, MediaError> {
        let query = self.query.trim();
        if self.mode == BookSearchMode::Isbn {
            return normalize_isbn(query).ok_or_else(|| {
                MediaError::validation("Enter a valid ISBN-10 or ISBN-13 checksum.")
            });
        }
        if query.chars().count() < 2 {
            return Err(MediaError::validation(
                "Book searches require at least two characters.",
            ));
        }
        Ok(query.to_string())
    }

    pub fn bounded_limit(&self) -> usize {
        self.limit.clamp(1, 40)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSourceIdentity {
    pub source_key: String,
    pub entity_type: String,
    pub external_id: String,
    #[serde(default)]
    pub source_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCatalogResult {
    pub source_key: String,
    pub external_id: String,
    pub title: String,
    pub subtitle: String,
    pub authors: Vec<String>,
    pub published_date: String,
    pub publisher: String,
    pub isbn_10: String,
    pub isbn_13: String,
    pub page_count: Option<i64>,
    pub language: String,
    pub format: BookFormat,
    pub cover_url: String,
    pub description: String,
    pub work_key: String,
    pub edition_key: String,
    pub info_url: String,
    pub preview_url: String,
    pub already_in_library: bool,
    pub existing_entry_id: Option<i64>,
    pub match_basis: String,
    pub provider_badges: Vec<String>,
    pub source_identities: Vec<BookSourceIdentity>,
}

impl BookCatalogResult {
    pub fn dedupe_key(&self) -> String {
        if !self.isbn_13.is_empty() {
            return format!("isbn13:{}", self.isbn_13);
        }
        if !self.isbn_10.is_empty() {
            return format!("isbn10:{}", self.isbn_10);
        }
        if !self.work_key.is_empty() {
            return format!("work:{}", self.work_key);
        }
        format!("{}:{}", self.source_key, self.external_id)
    }
}

#[derive(Clone, Debug)]
pub struct NormalizedBookEdition {
    pub title: String,
    pub subtitle: String,
    pub format: BookFormat,
    pub isbn_10: String,
    pub isbn_13: String,
    pub publisher: String,
    pub published_date: String,
    pub language: String,
    pub page_count: Option<i64>,
    pub audio_duration_minutes: Option<i64>,
    pub cover_url: String,
    pub is_ebook: bool,
    pub access_viewability: String,
}

#[derive(Clone, Debug)]
pub struct NormalizedBookSource {
    pub edition_index: Option<usize>,
    pub source_key: String,
    pub entity_type: String,
    pub external_id: String,
    pub source_url: String,
    pub metadata_json: String,
}

#[derive(Clone, Debug)]
pub struct NormalizedBookLink {
    pub edition_index: Option<usize>,
    pub source_key: String,
    pub link_type: String,
    pub url: String,
    pub region_code: String,
}

#[derive(Clone, Debug)]
pub struct NormalizedBookSeries {
    pub name: String,
    pub position_text: String,
    pub position_sort: Option<f64>,
    pub source_key: String,
}

#[derive(Clone, Debug)]
pub struct NormalizedBookImport {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub authors: Vec<String>,
    pub first_publish_year: Option<i64>,
    pub subjects: Vec<String>,
    pub canonical_cover_url: String,
    pub community_rating: Option<f64>,
    pub community_rating_count: Option<i64>,
    pub editions: Vec<NormalizedBookEdition>,
    pub sources: Vec<NormalizedBookSource>,
    pub links: Vec<NormalizedBookLink>,
    pub series: Vec<NormalizedBookSeries>,
}

#[derive(Clone, Debug)]
pub struct BookProviderFailure {
    pub source_key: String,
    pub message: String,
}

impl BookProviderFailure {
    pub fn warning_message(&self) -> String {
        format!("{}: {}", self.provider_name(), self.message)
    }

    fn provider_name(&self) -> &'static str {
        match self.source_key.as_str() {
            "google_books" => "Google Books",
            "open_library" => "Open Library",
            "hardcover" => "Hardcover",
            _ => "Book provider",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCatalogBookInput {
    pub source_key: String,
    pub external_id: String,
    #[serde(default = "default_planned_status")]
    pub library_status: MediaLibraryStatus,
}

fn default_planned_status() -> MediaLibraryStatus {
    MediaLibraryStatus::Planned
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManualBookInput {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: String,
    pub first_publish_year: Option<i64>,
    #[serde(default)]
    pub isbn_10: String,
    #[serde(default)]
    pub isbn_13: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub published_date: String,
    pub page_count: Option<i64>,
    pub audio_duration_minutes: Option<i64>,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub preferred_format: BookFormat,
    #[serde(default = "default_planned_status")]
    pub library_status: MediaLibraryStatus,
    #[serde(default)]
    pub notes: String,
    pub personal_rating: Option<f64>,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub ownership_status: BookOwnershipStatus,
    #[serde(default)]
    pub is_read_next: bool,
    #[serde(default)]
    pub series_name: String,
    #[serde(default)]
    pub series_position: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLibraryFilter {
    #[serde(default)]
    pub query: String,
    pub library_status: Option<MediaLibraryStatus>,
    pub ownership_status: Option<BookOwnershipStatus>,
    pub preferred_format: Option<BookFormat>,
    #[serde(default)]
    pub author_query: String,
    pub tag_id: Option<i64>,
    pub is_favorite: Option<bool>,
    pub is_read_next: Option<bool>,
    #[serde(default = "default_book_sort")]
    pub sort: String,
    #[serde(default = "default_sort_direction")]
    pub sort_direction: String,
}

fn default_book_sort() -> String {
    "RECENTLY_ADDED".to_string()
}

fn default_sort_direction() -> String {
    "DESC".to_string()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookReaderStateInput {
    pub ownership_status: BookOwnershipStatus,
    pub preferred_format: BookFormat,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBookProgressInput {
    pub progress_unit: BookProgressUnit,
    pub progress_value: f64,
    pub progress_total_override: Option<f64>,
    #[serde(default)]
    pub current_chapter: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualBookEditionInput {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub format: BookFormat,
    #[serde(default)]
    pub isbn_10: String,
    #[serde(default)]
    pub isbn_13: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub published_date: String,
    #[serde(default)]
    pub language: String,
    pub page_count: Option<i64>,
    pub audio_duration_minutes: Option<i64>,
    #[serde(default)]
    pub cover_url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLinkInput {
    pub edition_id: Option<i64>,
    pub link_type: String,
    pub url: String,
    #[serde(default)]
    pub region_code: String,
    #[serde(default)]
    pub is_preferred: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSeriesOverrideInput {
    pub name: String,
    #[serde(default)]
    pub position_text: String,
    pub position_sort: Option<f64>,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookProgressSummary {
    pub progress_unit: BookProgressUnit,
    pub progress_value: f64,
    pub progress_total_override: Option<f64>,
    pub effective_total: Option<f64>,
    pub progress_percent: Option<f64>,
    pub current_chapter: String,
    pub preferred_edition_id: Option<i64>,
    pub ownership_status: BookOwnershipStatus,
    pub preferred_format: BookFormat,
    pub reading_queue_position: Option<i64>,
    pub last_progress_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLibraryItem {
    pub entry_id: i64,
    pub media_title_id: i64,
    pub title: String,
    pub subtitle: String,
    pub primary_author_text: String,
    pub description: String,
    pub first_publish_year: Option<i64>,
    pub cover_url: String,
    pub library_status: MediaLibraryStatus,
    pub status_label: String,
    pub is_favorite: bool,
    pub personal_rating: Option<f64>,
    pub notes: String,
    pub priority: i64,
    pub added_at: String,
    pub started_at: String,
    pub completed_at: String,
    pub metadata_refreshed_at: String,
    pub progress: BookProgressSummary,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookEditionRecord {
    pub id: i64,
    pub title: String,
    pub subtitle: String,
    pub format: BookFormat,
    pub isbn_10: String,
    pub isbn_13: String,
    pub publisher: String,
    pub published_date: String,
    pub language: String,
    pub page_count: Option<i64>,
    pub audio_duration_minutes: Option<i64>,
    pub cover_url: String,
    pub is_ebook: bool,
    pub access_viewability: String,
    pub is_present_in_source: bool,
    pub source_keys: Vec<String>,
    pub is_user_owned: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookAuthorRecord {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub position: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLinkRecord {
    pub id: i64,
    pub edition_id: Option<i64>,
    pub source_key: String,
    pub link_type: String,
    pub url: String,
    pub region_code: String,
    pub is_user_owned: bool,
    pub is_preferred: bool,
    pub refreshed_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSeriesRecord {
    pub id: i64,
    pub name: String,
    pub position_text: String,
    pub position_sort: Option<f64>,
    pub is_primary: bool,
    pub source_key: String,
    pub is_user_override: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSourceRecord {
    pub source_key: String,
    pub entity_type: String,
    pub external_id: String,
    pub source_url: String,
    pub refreshed_at: String,
    pub last_refresh_status: String,
    pub last_refresh_error: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLibraryDetail {
    pub item: BookLibraryItem,
    pub editions: Vec<BookEditionRecord>,
    pub authors: Vec<BookAuthorRecord>,
    pub links: Vec<BookLinkRecord>,
    pub series: Vec<BookSeriesRecord>,
    pub sources: Vec<BookSourceRecord>,
    pub subjects: Vec<String>,
    pub community_rating: Option<f64>,
    pub community_rating_count: Option<i64>,
    pub provider_warnings: Vec<String>,
    pub tags: Vec<MediaTagRecord>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMediaSummary {
    pub progress: BookProgressSummary,
    pub primary_author_text: String,
    pub primary_series_text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMediaDetailSummary {
    pub progress: BookProgressSummary,
    pub preferred_edition: Option<BookEditionRecord>,
    pub authors: Vec<BookAuthorRecord>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookProviderStatus {
    pub google_books_configured: bool,
    pub open_library_contact_configured: bool,
    pub hardcover_configured: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCatalogSearchResponse {
    pub results: Vec<BookCatalogResult>,
    pub warnings: Vec<String>,
}

pub fn normalize_isbn(value: &str) -> Option<String> {
    let normalized = value
        .chars()
        .filter(|character| character.is_ascii_digit() || matches!(character, 'x' | 'X'))
        .map(|character| character.to_ascii_uppercase())
        .collect::<String>();
    match normalized.len() {
        10 if valid_isbn_10(&normalized) => Some(normalized),
        13 if valid_isbn_13(&normalized) => Some(normalized),
        _ => None,
    }
}

fn valid_isbn_10(value: &str) -> bool {
    value
        .chars()
        .enumerate()
        .all(|(index, character)| character.is_ascii_digit() || (index == 9 && character == 'X'))
        && value
            .chars()
            .enumerate()
            .map(|(index, character)| {
                let digit = if character == 'X' {
                    10
                } else {
                    character.to_digit(10).unwrap_or_default()
                };
                (10 - index as u32) * digit
            })
            .sum::<u32>()
            % 11
            == 0
}

fn valid_isbn_13(value: &str) -> bool {
    value.chars().all(|character| character.is_ascii_digit())
        && value
            .chars()
            .enumerate()
            .map(|(index, character)| {
                character.to_digit(10).unwrap_or_default() * if index % 2 == 0 { 1 } else { 3 }
            })
            .sum::<u32>()
            % 10
            == 0
}

pub fn validate_progress(
    unit: BookProgressUnit,
    value: f64,
    total_override: Option<f64>,
) -> Result<(), MediaError> {
    if !value.is_finite() || value < 0.0 {
        return Err(MediaError::validation("Book progress cannot be negative."));
    }
    if total_override.is_some_and(|total| !total.is_finite() || total < 0.0) {
        return Err(MediaError::validation(
            "Book progress total cannot be negative.",
        ));
    }
    if unit == BookProgressUnit::Percent && value > 100.0 {
        return Err(MediaError::validation(
            "Percentage progress must be between 0 and 100.",
        ));
    }
    Ok(())
}

pub fn effective_progress_total(
    unit: BookProgressUnit,
    total_override: Option<f64>,
    page_count: Option<i64>,
    audio_duration_minutes: Option<i64>,
) -> Option<f64> {
    if total_override.is_some_and(|total| total > 0.0) {
        return total_override;
    }
    match unit {
        BookProgressUnit::Percent => Some(100.0),
        BookProgressUnit::Page => page_count
            .filter(|value| *value > 0)
            .map(|value| value as f64),
        BookProgressUnit::Minute => audio_duration_minutes
            .filter(|value| *value > 0)
            .map(|value| value as f64),
        BookProgressUnit::Chapter => None,
    }
}

pub fn automatic_book_status(
    current: MediaLibraryStatus,
    unit: BookProgressUnit,
    value: f64,
    effective_total: Option<f64>,
) -> MediaLibraryStatus {
    if current.protects_from_automatic_transition() {
        return current;
    }
    let completed = unit != BookProgressUnit::Chapter
        && effective_total.is_some_and(|total| total > 0.0 && value >= total);
    if completed {
        MediaLibraryStatus::Completed
    } else if value > 0.0 {
        MediaLibraryStatus::Watching
    } else {
        MediaLibraryStatus::Planned
    }
}

pub fn plain_text_description(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(character),
            _ => {}
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_normalizes_isbns() {
        assert_eq!(
            normalize_isbn("978-0-306-40615-7").as_deref(),
            Some("9780306406157")
        );
        assert_eq!(
            normalize_isbn("0-306-40615-2").as_deref(),
            Some("0306406152")
        );
        assert_eq!(normalize_isbn("978-0-306-40615-8"), None);
    }

    #[test]
    fn typed_search_rejects_invalid_input() {
        let invalid_isbn = BookSearchInput {
            query: "1234".to_string(),
            mode: BookSearchMode::Isbn,
            limit: 20,
            language: String::new(),
        };
        assert!(invalid_isbn.validated_query().is_err());
        assert_eq!(
            BookSearchInput {
                limit: 400,
                ..invalid_isbn
            }
            .bounded_limit(),
            40
        );
    }

    #[test]
    fn progress_transitions_respect_protected_statuses() {
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Planned,
                BookProgressUnit::Percent,
                1.0,
                Some(100.0),
            ),
            MediaLibraryStatus::Watching
        );
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Watching,
                BookProgressUnit::Page,
                300.0,
                Some(300.0),
            ),
            MediaLibraryStatus::Completed
        );
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::OnHold,
                BookProgressUnit::Percent,
                100.0,
                Some(100.0),
            ),
            MediaLibraryStatus::OnHold
        );
    }

    #[test]
    fn chapter_progress_never_auto_completes() {
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Watching,
                BookProgressUnit::Chapter,
                100.0,
                Some(100.0),
            ),
            MediaLibraryStatus::Watching
        );
    }

    #[test]
    fn measurable_progress_requires_and_uses_the_correct_total() {
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Watching,
                BookProgressUnit::Page,
                500.0,
                None,
            ),
            MediaLibraryStatus::Watching
        );
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Planned,
                BookProgressUnit::Percent,
                100.0,
                Some(100.0),
            ),
            MediaLibraryStatus::Completed
        );
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Watching,
                BookProgressUnit::Minute,
                420.0,
                Some(420.0),
            ),
            MediaLibraryStatus::Completed
        );
        assert_eq!(
            automatic_book_status(
                MediaLibraryStatus::Completed,
                BookProgressUnit::Page,
                25.0,
                Some(100.0),
            ),
            MediaLibraryStatus::Watching
        );
    }

    #[test]
    fn strips_provider_html_from_descriptions() {
        assert_eq!(
            plain_text_description("A <b>great</b> book &amp; story"),
            "A great book & story"
        );
    }
}
