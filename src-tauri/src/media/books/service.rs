use super::domain::{
    AddCatalogBookInput, BookCatalogSearchResponse, BookLibraryDetail, BookLibraryFilter,
    BookLibraryItem, BookLinkInput, BookProviderStatus, BookSearchInput, BookSeriesOverrideInput,
    CreateManualBookInput, ManualBookEditionInput, SetBookProgressInput,
    UpdateBookReaderStateInput,
};
use super::provider::BookProviderCoordinator;
use super::repository::BookRepository;
use crate::db::AppDatabase;
use crate::media::domain::{MediaError, MediaErrorKind};
use crate::media::repository::MediaRepository;

pub struct BookCatalogService<'a> {
    repository: BookRepository<'a>,
}

impl<'a> BookCatalogService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: BookRepository::new(database),
        }
    }

    pub fn search(&self, input: &BookSearchInput) -> Result<BookCatalogSearchResponse, MediaError> {
        let mut resolved_input = input.clone();
        if resolved_input.language.trim().is_empty() {
            resolved_input.language = MediaRepository::new(self.repository.database())
                .get_settings()?
                .metadata_language;
        }
        let response = BookProviderCoordinator::from_environment()?.search(&resolved_input)?;
        let mut results = response.value;
        for result in &mut results {
            let mut existing = None;
            for identity in &result.source_identities {
                existing = self.repository.find_existing_candidate(
                    &identity.source_key,
                    &identity.external_id,
                    &result.isbn_10,
                    &result.isbn_13,
                )?;
                if existing.is_some() {
                    result.match_basis = "EXACT_IDENTITY_OR_ISBN".to_string();
                    break;
                }
            }
            if result.source_identities.is_empty() {
                existing = self.repository.find_existing_candidate(
                    &result.source_key,
                    &result.external_id,
                    &result.isbn_10,
                    &result.isbn_13,
                )?;
            }
            result.already_in_library = existing.is_some();
            result.existing_entry_id = existing;
        }
        Ok(BookCatalogSearchResponse {
            results,
            warnings: response
                .failures
                .iter()
                .map(|failure| failure.warning_message())
                .collect(),
        })
    }

    pub fn add(&self, input: &AddCatalogBookInput) -> Result<BookLibraryDetail, MediaError> {
        let response = BookProviderCoordinator::from_environment()?.fetch_import(input)?;
        self.repository
            .add_import(&response.value, input.library_status, &response.failures)
    }

    pub fn refresh(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        let (source_key, external_id) = self.repository.source_identity_for_entry(entry_id)?;
        let input = AddCatalogBookInput {
            source_key: source_key.clone(),
            external_id,
            library_status: crate::media::domain::MediaLibraryStatus::Planned,
        };
        match BookProviderCoordinator::from_environment()?.fetch_import(&input) {
            Ok(response) => {
                self.repository
                    .refresh_import(entry_id, &response.value, &response.failures)
            }
            Err(error) => {
                let _ = self.repository.mark_provider_failure(
                    entry_id,
                    &source_key,
                    &error.to_string(),
                );
                Err(error)
            }
        }
    }

    pub fn provider_status(&self) -> BookProviderStatus {
        BookProviderStatus {
            google_books_configured: environment_present("GOOGLE_BOOKS_API_KEY"),
            open_library_contact_configured: environment_present("OPEN_LIBRARY_CONTACT_EMAIL"),
            hardcover_configured: environment_present("HARDCOVER_API_TOKEN"),
        }
    }
}

pub struct BookLibraryService<'a> {
    repository: BookRepository<'a>,
}

impl<'a> BookLibraryService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: BookRepository::new(database),
        }
    }

    pub fn list(&self, filter: &BookLibraryFilter) -> Result<Vec<BookLibraryItem>, MediaError> {
        self.repository.list(filter)
    }

    pub fn get(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        self.repository.get_detail(entry_id)
    }

    pub fn create_manual(
        &self,
        input: &CreateManualBookInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.create_manual(input)
    }

    pub fn update_reader_state(
        &self,
        entry_id: i64,
        input: &UpdateBookReaderStateInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.update_reader_state(entry_id, input)
    }

    pub fn set_preferred_edition(
        &self,
        entry_id: i64,
        edition_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.set_preferred_edition(entry_id, edition_id)
    }

    pub fn add_to_read_next(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        self.repository.add_to_read_next(entry_id)
    }

    pub fn remove_from_read_next(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        self.repository.remove_from_read_next(entry_id)
    }

    pub fn move_read_next(
        &self,
        entry_id: i64,
        direction: &str,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.move_read_next(entry_id, direction)
    }

    pub fn create_edition(
        &self,
        entry_id: i64,
        input: &ManualBookEditionInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.create_edition(entry_id, input)
    }

    pub fn update_edition(
        &self,
        entry_id: i64,
        edition_id: i64,
        input: &ManualBookEditionInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.update_edition(entry_id, edition_id, input)
    }

    pub fn delete_edition(
        &self,
        entry_id: i64,
        edition_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.delete_edition(entry_id, edition_id)
    }

    pub fn create_link(
        &self,
        entry_id: i64,
        input: &BookLinkInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.create_link(entry_id, input)
    }

    pub fn update_link(
        &self,
        entry_id: i64,
        link_id: i64,
        input: &BookLinkInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.update_link(entry_id, link_id, input)
    }

    pub fn delete_link(
        &self,
        entry_id: i64,
        link_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.delete_link(entry_id, link_id)
    }

    pub fn set_preferred_link(
        &self,
        entry_id: i64,
        link_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.set_preferred_link(entry_id, link_id)
    }

    pub fn resolve_link(&self, entry_id: i64, link_id: i64) -> Result<String, MediaError> {
        self.repository.resolve_link(entry_id, link_id)
    }

    pub fn set_series_override(
        &self,
        entry_id: i64,
        input: &BookSeriesOverrideInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.set_series_override(entry_id, input)
    }

    pub fn clear_series_override(
        &self,
        entry_id: i64,
        member_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.clear_series_override(entry_id, member_id)
    }
}

pub struct BookProgressService<'a> {
    repository: BookRepository<'a>,
}

impl<'a> BookProgressService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self {
            repository: BookRepository::new(database),
        }
    }

    pub fn set_progress(
        &self,
        entry_id: i64,
        input: &SetBookProgressInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        self.repository.set_progress(entry_id, input)
    }

    pub fn mark_read(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        self.repository.mark_read(entry_id)
    }

    pub fn reset(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        self.repository.reset_progress(entry_id)
    }
}

fn environment_present(name: &str) -> bool {
    !std::env::var(name).unwrap_or_default().trim().is_empty()
}

pub fn external_open_error(error: String) -> String {
    MediaError::new(MediaErrorKind::ExternalOpen, error).to_string()
}

pub fn book_data_source_url(source_key: &str) -> Result<&'static str, MediaError> {
    match source_key.trim() {
        "google_books" => Ok("https://books.google.com/"),
        "open_library" => Ok("https://openlibrary.org/"),
        "hardcover" => Ok("https://hardcover.app/"),
        _ => Err(MediaError::validation("Unknown book data source.")),
    }
}
