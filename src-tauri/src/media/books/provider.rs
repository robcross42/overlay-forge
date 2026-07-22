use super::domain::{
    AddCatalogBookInput, BookCatalogResult, BookProviderFailure, BookSearchInput,
    NormalizedBookImport,
};
use super::google_books::GoogleBooksProvider;
use super::hardcover::HardcoverProvider;
use super::merge::{merge_catalog_results, BookMetadataMerger};
use super::open_library::OpenLibraryProvider;
use crate::media::domain::{MediaError, MediaErrorKind};

pub trait BookMetadataProvider {
    fn search_books(&self, input: &BookSearchInput) -> Result<Vec<BookCatalogResult>, MediaError>;
    fn fetch_book_candidate(&self, external_id: &str) -> Result<NormalizedBookImport, MediaError>;
    fn enrich_by_isbn(&self, isbn: &str) -> Result<Option<NormalizedBookImport>, MediaError>;
}

#[derive(Debug)]
pub struct BookProviderResponse<T> {
    pub value: T,
    pub failures: Vec<BookProviderFailure>,
}

pub struct BookProviderCoordinator {
    google: Option<GoogleBooksProvider>,
    open_library: OpenLibraryProvider,
    hardcover: Option<HardcoverProvider>,
}

impl BookProviderCoordinator {
    pub fn from_environment() -> Result<Self, MediaError> {
        Ok(Self {
            google: GoogleBooksProvider::from_environment_optional()?,
            open_library: OpenLibraryProvider::from_environment()?,
            hardcover: HardcoverProvider::from_environment_optional()?,
        })
    }

    pub fn search(
        &self,
        input: &BookSearchInput,
    ) -> Result<BookProviderResponse<Vec<BookCatalogResult>>, MediaError> {
        input.validated_query()?;
        let mut failures = Vec::new();
        let mut results = Vec::new();

        if let Some(google) = &self.google {
            match google.search_books(input) {
                Ok(items) => results.extend(items),
                Err(error) => failures.push(BookProviderFailure {
                    source_key: "google_books".to_string(),
                    message: error.message,
                }),
            }
        } else {
            failures.push(BookProviderFailure {
                source_key: "google_books".to_string(),
                message: "Not configured; Open Library fallback was used.".to_string(),
            });
        }

        match self.open_library.search_books(input) {
            Ok(items) => results.extend(items),
            Err(error) if results.is_empty() => return Err(error),
            Err(error) => failures.push(BookProviderFailure {
                source_key: "open_library".to_string(),
                message: error.message,
            }),
        }

        if results.is_empty() && !failures.is_empty() {
            return Err(MediaError::new(
                MediaErrorKind::ProviderUnavailable,
                failures
                    .iter()
                    .map(BookProviderFailure::warning_message)
                    .collect::<Vec<_>>()
                    .join(" "),
            ));
        }

        Ok(BookProviderResponse {
            value: merge_catalog_results(results),
            failures,
        })
    }

    pub fn fetch_import(
        &self,
        input: &AddCatalogBookInput,
    ) -> Result<BookProviderResponse<NormalizedBookImport>, MediaError> {
        let mut failures = Vec::new();
        let mut import = match input.source_key.trim() {
            "google_books" => self
                .google
                .as_ref()
                .ok_or_else(|| {
                    MediaError::new(
                        MediaErrorKind::MissingCredential,
                        "GOOGLE_BOOKS_API_KEY is required to import this Google Books result.",
                    )
                })?
                .fetch_book_candidate(&input.external_id)?,
            "open_library" => self.open_library.fetch_book_candidate(&input.external_id)?,
            _ => {
                return Err(MediaError::validation(
                    "Book imports support google_books or open_library sources.",
                ))
            }
        };

        if let Some(isbn) = preferred_isbn(&import) {
            if input.source_key != "open_library" {
                match self.open_library.enrich_by_isbn(&isbn) {
                    Ok(Some(enrichment)) => BookMetadataMerger::merge(&mut import, enrichment),
                    Ok(None) => {}
                    Err(error) => failures.push(BookProviderFailure {
                        source_key: "open_library".to_string(),
                        message: error.message,
                    }),
                }
            }
            if let Some(hardcover) = &self.hardcover {
                match hardcover.enrich_by_isbn(&isbn) {
                    Ok(Some(enrichment)) => BookMetadataMerger::merge(&mut import, enrichment),
                    Ok(None) => {}
                    Err(error) => failures.push(BookProviderFailure {
                        source_key: "hardcover".to_string(),
                        message: error.message,
                    }),
                }
            }
        }

        Ok(BookProviderResponse {
            value: import,
            failures,
        })
    }
}

fn preferred_isbn(import: &NormalizedBookImport) -> Option<String> {
    import
        .editions
        .iter()
        .find_map(|edition| (!edition.isbn_13.is_empty()).then(|| edition.isbn_13.clone()))
        .or_else(|| {
            import
                .editions
                .iter()
                .find_map(|edition| (!edition.isbn_10.is_empty()).then(|| edition.isbn_10.clone()))
        })
}
