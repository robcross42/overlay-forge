use super::domain::{
    validate_http_url, validate_rating, AddCatalogMediaInput, CreateManualMediaInput,
    MediaCatalogSearchResult, MediaError, MediaLibraryDetail, MediaLibraryFilter,
    MediaLibrarySummary, MediaSettingsRecord, MediaStreamingLinkInput, MediaStreamingLinkRecord,
    MediaTagRecord, UpdateMediaEntryInput, UpdateMediaSettingsInput,
};
use super::provider::{MediaMetadataProvider, TmdbMediaMetadataProvider};
use super::repository::MediaRepository;
use crate::db::AppDatabase;

pub struct MediaCatalogService<'a> {
    repository: MediaRepository<'a>,
}

impl<'a> MediaCatalogService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: MediaRepository::new(database),
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<MediaCatalogSearchResult>, MediaError> {
        let settings = self.repository.get_settings()?;
        let provider = TmdbMediaMetadataProvider::from_environment()?;
        let mut results = provider.search(query, &settings.metadata_language)?;
        for result in &mut results {
            result.existing_entry_id = self
                .repository
                .find_external_entry(result.external_id, &result.external_media_type)?;
            result.already_in_library = result.existing_entry_id.is_some();
        }
        Ok(results)
    }

    pub fn add_catalog_title(
        &self,
        input: &AddCatalogMediaInput,
    ) -> Result<MediaLibraryDetail, MediaError> {
        if let Some(existing_id) = self
            .repository
            .find_external_entry(input.external_id, &input.external_media_type)?
        {
            return Err(MediaError::new(
                super::domain::MediaErrorKind::Duplicate,
                format!("This title is already in the library (entry {existing_id})."),
            ));
        }
        let settings = self.repository.get_settings()?;
        let provider = TmdbMediaMetadataProvider::from_environment()?;
        let import = provider.fetch_import(
            input.external_id,
            &input.external_media_type,
            &settings.metadata_language,
            &settings.region_code,
        )?;
        self.repository
            .add_catalog_import(&import, input.library_status)
    }

    pub fn refresh(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        let (source_key, external_id, external_media_type) =
            self.repository.source_identity_for_entry(entry_id)?;
        if source_key != "tmdb" {
            return Err(MediaError::validation(
                "Manual entries do not have TMDB metadata to refresh.",
            ));
        }
        let external_id = external_id.ok_or_else(|| {
            MediaError::validation("The saved TMDB title has no external identifier.")
        })?;
        let external_media_type = external_media_type.ok_or_else(|| {
            MediaError::validation("The saved TMDB title has no external media type.")
        })?;
        let settings = self.repository.get_settings()?;
        let provider = TmdbMediaMetadataProvider::from_environment()?;
        match provider.fetch_import(
            external_id,
            &external_media_type,
            &settings.metadata_language,
            &settings.region_code,
        ) {
            Ok(import) => self.repository.refresh_import(entry_id, &import),
            Err(error) => {
                let _ = self
                    .repository
                    .mark_refresh_failure(entry_id, &error.to_string());
                Err(error)
            }
        }
    }
}

pub struct MediaLibraryService<'a> {
    repository: MediaRepository<'a>,
}

impl<'a> MediaLibraryService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: MediaRepository::new(database),
        }
    }

    pub fn list(
        &self,
        filter: &MediaLibraryFilter,
    ) -> Result<Vec<MediaLibrarySummary>, MediaError> {
        self.repository.list_library(filter)
    }

    pub fn get(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.get_detail(entry_id)
    }

    pub fn create_manual(
        &self,
        input: &CreateManualMediaInput,
    ) -> Result<MediaLibraryDetail, MediaError> {
        if input.title.trim().is_empty() {
            return Err(MediaError::validation("A title is required."));
        }
        if input.runtime_minutes.is_some_and(|runtime| runtime <= 0) {
            return Err(MediaError::validation(
                "Runtime must be a positive number of minutes.",
            ));
        }
        validate_rating(input.personal_rating)?;
        self.repository.create_manual_entry(input)
    }

    pub fn update(
        &self,
        entry_id: i64,
        input: &UpdateMediaEntryInput,
    ) -> Result<MediaLibraryDetail, MediaError> {
        validate_rating(input.personal_rating)?;
        self.repository.update_entry(entry_id, input)
    }

    pub fn delete(&self, entry_id: i64) -> Result<(), MediaError> {
        self.repository.delete_entry(entry_id)
    }

    pub fn add_to_watch_next(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.add_to_watch_next(entry_id)
    }

    pub fn remove_from_watch_next(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.remove_from_watch_next(entry_id)
    }

    pub fn move_watch_next(
        &self,
        entry_id: i64,
        direction: &str,
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.move_watch_next(entry_id, direction)
    }

    pub fn list_tags(&self) -> Result<Vec<MediaTagRecord>, MediaError> {
        self.repository.list_tags()
    }

    pub fn create_tag(&self, name: &str) -> Result<MediaTagRecord, MediaError> {
        if name.trim().is_empty() {
            return Err(MediaError::validation("Tag name is required."));
        }
        self.repository.create_tag(name)
    }

    pub fn delete_tag(&self, tag_id: i64) -> Result<(), MediaError> {
        self.repository.delete_tag(tag_id)
    }

    pub fn set_entry_tags(
        &self,
        entry_id: i64,
        tag_ids: &[i64],
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.set_entry_tags(entry_id, tag_ids)
    }

    pub fn get_settings(&self) -> Result<MediaSettingsRecord, MediaError> {
        self.repository.get_settings()
    }

    pub fn update_settings(
        &self,
        input: &UpdateMediaSettingsInput,
    ) -> Result<MediaSettingsRecord, MediaError> {
        let region = input.region_code.trim();
        if region.len() != 2 || !region.chars().all(|value| value.is_ascii_alphabetic()) {
            return Err(MediaError::validation(
                "Region must be a two-letter country code such as CA.",
            ));
        }
        let language = input.metadata_language.trim();
        if language.len() < 2 || language.len() > 12 {
            return Err(MediaError::validation(
                "Metadata language must be a language code such as en-CA.",
            ));
        }
        self.repository.update_settings(input)
    }
}

pub struct MediaProgressService<'a> {
    repository: MediaRepository<'a>,
}

impl<'a> MediaProgressService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: MediaRepository::new(database),
        }
    }

    pub fn set_movie_watched(
        &self,
        entry_id: i64,
        is_watched: bool,
        watched_at: &str,
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository
            .set_movie_watched(entry_id, is_watched, watched_at)
    }

    pub fn set_episode_watched(
        &self,
        entry_id: i64,
        episode_id: i64,
        is_watched: bool,
        watched_at: &str,
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository
            .set_episode_watched(entry_id, episode_id, is_watched, watched_at)
    }

    pub fn set_season_watched(
        &self,
        entry_id: i64,
        season_number: i64,
        is_watched: bool,
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository
            .set_season_watched(entry_id, season_number, is_watched)
    }

    pub fn set_series_watched(
        &self,
        entry_id: i64,
        is_watched: bool,
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.set_series_watched(entry_id, is_watched)
    }

    pub fn mark_watched_through(
        &self,
        entry_id: i64,
        episode_id: i64,
    ) -> Result<MediaLibraryDetail, MediaError> {
        self.repository.mark_watched_through(entry_id, episode_id)
    }
}

pub struct MediaAvailabilityService<'a> {
    repository: MediaRepository<'a>,
}

impl<'a> MediaAvailabilityService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: MediaRepository::new(database),
        }
    }

    pub fn list_links(&self, entry_id: i64) -> Result<Vec<MediaStreamingLinkRecord>, MediaError> {
        self.repository.list_streaming_links(entry_id)
    }

    pub fn create_link(
        &self,
        entry_id: i64,
        input: &MediaStreamingLinkInput,
    ) -> Result<MediaStreamingLinkRecord, MediaError> {
        Self::validate_link(input)?;
        self.repository.create_streaming_link(entry_id, input)
    }

    pub fn update_link(
        &self,
        link_id: i64,
        input: &MediaStreamingLinkInput,
    ) -> Result<MediaStreamingLinkRecord, MediaError> {
        Self::validate_link(input)?;
        self.repository.update_streaming_link(link_id, input)
    }

    pub fn delete_link(&self, link_id: i64) -> Result<(), MediaError> {
        self.repository.delete_streaming_link(link_id)
    }

    pub fn set_preferred(
        &self,
        entry_id: i64,
        link_id: i64,
    ) -> Result<Vec<MediaStreamingLinkRecord>, MediaError> {
        self.repository
            .set_preferred_streaming_link(entry_id, link_id)
    }

    pub fn resolve_open_target(&self, entry_id: i64) -> Result<String, MediaError> {
        let url = self.repository.resolve_streaming_target(entry_id)?;
        validate_http_url(&url)
    }

    fn validate_link(input: &MediaStreamingLinkInput) -> Result<(), MediaError> {
        validate_http_url(&input.url)?;
        match input.link_type.trim().to_ascii_uppercase().as_str() {
            "STREAM" | "RENT" | "BUY" | "OTHER" => Ok(()),
            _ => Err(MediaError::validation(
                "Manual link type must be STREAM, RENT, BUY, or OTHER.",
            )),
        }
    }
}
