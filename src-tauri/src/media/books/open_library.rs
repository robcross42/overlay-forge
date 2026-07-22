use super::domain::{
    normalize_isbn, plain_text_description, BookCatalogResult, BookFormat, BookSearchInput,
    BookSearchMode, BookSourceIdentity, NormalizedBookEdition, NormalizedBookImport,
    NormalizedBookLink, NormalizedBookSource,
};
use super::provider::BookMetadataProvider;
use crate::media::domain::{MediaError, MediaErrorKind};
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const OPEN_LIBRARY_ROOT: &str = "https://openlibrary.org";
static LAST_OPEN_LIBRARY_REQUEST: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

pub struct OpenLibraryProvider {
    client: Client,
    contact_configured: bool,
}

impl OpenLibraryProvider {
    pub fn from_environment() -> Result<Self, MediaError> {
        let contact = std::env::var("OPEN_LIBRARY_CONTACT_EMAIL")
            .unwrap_or_default()
            .trim()
            .to_string();
        let user_agent = if contact.is_empty() {
            "Overlay Forge/0.11 book-catalogue".to_string()
        } else {
            format!("Overlay Forge/0.11 book-catalogue ({contact})")
        };
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(user_agent)
            .build()
            .map_err(|error| MediaError::new(MediaErrorKind::Network, error.to_string()))?;
        Ok(Self {
            client,
            contact_configured: !contact.is_empty(),
        })
    }

    fn minimum_interval(&self) -> Duration {
        if self.contact_configured {
            Duration::from_millis(334)
        } else {
            Duration::from_secs(1)
        }
    }

    fn throttle(&self) {
        let lock = LAST_OPEN_LIBRARY_REQUEST.get_or_init(|| Mutex::new(None));
        if let Ok(mut last_request) = lock.lock() {
            if let Some(previous) = *last_request {
                let elapsed = previous.elapsed();
                let interval = self.minimum_interval();
                if elapsed < interval {
                    thread::sleep(interval - elapsed);
                }
            }
            *last_request = Some(Instant::now());
        }
    }

    fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, MediaError> {
        self.throttle();
        let response = self
            .client
            .get(format!("{OPEN_LIBRARY_ROOT}{path}"))
            .query(query)
            .send()
            .map_err(request_error)?;
        decode_response(response)
    }

    fn normalize_doc(doc: OpenLibrarySearchDoc) -> Option<BookCatalogResult> {
        let title = doc.title.trim().to_string();
        if title.is_empty() {
            return None;
        }
        let (isbn_10, isbn_13) = normalized_isbns(&doc.isbn);
        let work_key = key_tail(&doc.key);
        let edition_key = doc.edition_key.first().cloned().unwrap_or_default();
        let external_id = if edition_key.is_empty() {
            work_key.clone()
        } else {
            edition_key.clone()
        };
        let cover_url = doc
            .cover_i
            .map(|cover| format!("https://covers.openlibrary.org/b/id/{cover}-L.jpg"))
            .unwrap_or_default();
        let info_url = if !work_key.is_empty() {
            format!("https://openlibrary.org/works/{work_key}")
        } else if !edition_key.is_empty() {
            format!("https://openlibrary.org/books/{edition_key}")
        } else {
            String::new()
        };
        let source_identities = [
            (!work_key.is_empty()).then(|| BookSourceIdentity {
                source_key: "open_library".to_string(),
                entity_type: "WORK".to_string(),
                external_id: work_key.clone(),
                source_url: info_url.clone(),
            }),
            (!edition_key.is_empty()).then(|| BookSourceIdentity {
                source_key: "open_library".to_string(),
                entity_type: "EDITION".to_string(),
                external_id: edition_key.clone(),
                source_url: format!("https://openlibrary.org/books/{edition_key}"),
            }),
        ]
        .into_iter()
        .flatten()
        .collect();
        Some(BookCatalogResult {
            source_key: "open_library".to_string(),
            external_id,
            title,
            subtitle: doc.subtitle.unwrap_or_default(),
            authors: doc.author_name,
            published_date: doc
                .first_publish_year
                .map(|year| year.to_string())
                .unwrap_or_default(),
            publisher: doc.publisher.first().cloned().unwrap_or_default(),
            isbn_10,
            isbn_13,
            page_count: doc.number_of_pages_median.filter(|value| *value > 0),
            language: doc.language.first().cloned().unwrap_or_default(),
            format: BookFormat::Unknown,
            cover_url,
            description: String::new(),
            work_key,
            edition_key,
            info_url,
            preview_url: String::new(),
            already_in_library: false,
            existing_entry_id: None,
            match_basis: "PROVIDER_ID".to_string(),
            provider_badges: vec!["Open Library".to_string()],
            source_identities,
        })
    }

    fn import_from_doc(doc: OpenLibrarySearchDoc) -> Result<NormalizedBookImport, MediaError> {
        let raw_json = serde_json::to_string(&doc).unwrap_or_else(|_| "{}".to_string());
        let candidate = Self::normalize_doc(doc.clone()).ok_or_else(|| {
            MediaError::new(
                MediaErrorKind::ProviderUnavailable,
                "Open Library returned a record without a title.",
            )
        })?;
        let mut sources = Vec::new();
        for identity in &candidate.source_identities {
            sources.push(NormalizedBookSource {
                edition_index: (identity.entity_type == "EDITION").then_some(0),
                source_key: identity.source_key.clone(),
                entity_type: identity.entity_type.clone(),
                external_id: identity.external_id.clone(),
                source_url: identity.source_url.clone(),
                metadata_json: raw_json.clone(),
            });
        }
        let mut links = Vec::new();
        if !candidate.info_url.is_empty() {
            let link_type = if doc.public_scan_b {
                "READ"
            } else if doc.ebook_access.as_deref() == Some("borrowable") {
                "BORROW"
            } else {
                "INFO"
            };
            links.push(NormalizedBookLink {
                edition_index: None,
                source_key: "open_library".to_string(),
                link_type: link_type.to_string(),
                url: candidate.info_url.clone(),
                region_code: String::new(),
            });
        }
        Ok(NormalizedBookImport {
            title: candidate.title.clone(),
            subtitle: candidate.subtitle.clone(),
            description: plain_text_description(&candidate.description),
            authors: candidate.authors,
            first_publish_year: doc.first_publish_year,
            subjects: doc.subject,
            canonical_cover_url: candidate.cover_url.clone(),
            community_rating: None,
            community_rating_count: None,
            editions: vec![NormalizedBookEdition {
                title: candidate.title,
                subtitle: candidate.subtitle,
                format: candidate.format,
                isbn_10: candidate.isbn_10,
                isbn_13: candidate.isbn_13,
                publisher: candidate.publisher,
                published_date: candidate.published_date,
                language: candidate.language,
                page_count: candidate.page_count,
                audio_duration_minutes: None,
                cover_url: candidate.cover_url,
                is_ebook: doc
                    .ebook_access
                    .as_deref()
                    .is_some_and(|value| value != "no_ebook"),
                access_viewability: doc.ebook_access.unwrap_or_default(),
            }],
            sources,
            links,
            series: Vec::new(),
        })
    }

    fn lookup_doc(&self, external_id: &str) -> Result<Option<OpenLibrarySearchDoc>, MediaError> {
        let response: OpenLibrarySearchResponse = self.get(
            "/search.json",
            &[
                ("q", external_id.to_string()),
                ("limit", "10".to_string()),
                ("fields", search_fields()),
            ],
        )?;
        let needle = external_id.trim_matches('/');
        Ok(response.docs.into_iter().find(|doc| {
            key_tail(&doc.key) == needle || doc.edition_key.iter().any(|key| key == needle)
        }))
    }

    fn import_with_detail(
        &self,
        doc: OpenLibrarySearchDoc,
    ) -> Result<NormalizedBookImport, MediaError> {
        let mut import = Self::import_from_doc(doc.clone())?;
        if let Some(edition_key) = doc.edition_key.first() {
            let edition: OpenLibraryEditionRecord =
                self.get(&format!("/books/{edition_key}.json"), &[])?;
            apply_edition_record(&mut import, edition_key, edition)?;
        }
        let work_key = key_tail(&doc.key);
        if !work_key.is_empty() {
            let work: OpenLibraryWorkRecord = self.get(&format!("/works/{work_key}.json"), &[])?;
            apply_work_record(&mut import, &work_key, work)?;
        }
        Ok(import)
    }
}

impl BookMetadataProvider for OpenLibraryProvider {
    fn search_books(&self, input: &BookSearchInput) -> Result<Vec<BookCatalogResult>, MediaError> {
        let query = input.validated_query()?;
        let mut parameters = vec![
            ("limit", input.bounded_limit().to_string()),
            ("fields", search_fields()),
        ];
        match input.mode {
            BookSearchMode::General => parameters.push(("q", query)),
            BookSearchMode::Title => parameters.push(("title", query)),
            BookSearchMode::Author => parameters.push(("author", query)),
            BookSearchMode::Isbn => parameters.push(("isbn", query)),
        }
        let response: OpenLibrarySearchResponse = self.get("/search.json", &parameters)?;
        Ok(response
            .docs
            .into_iter()
            .filter_map(Self::normalize_doc)
            .collect())
    }

    fn fetch_book_candidate(&self, external_id: &str) -> Result<NormalizedBookImport, MediaError> {
        let doc = self.lookup_doc(external_id)?.ok_or_else(|| {
            MediaError::not_found("Open Library could not find the selected work or edition.")
        })?;
        self.import_with_detail(doc)
    }

    fn enrich_by_isbn(&self, isbn: &str) -> Result<Option<NormalizedBookImport>, MediaError> {
        let input = BookSearchInput {
            query: isbn.to_string(),
            mode: BookSearchMode::Isbn,
            limit: 1,
            language: String::new(),
        };
        let query = input.validated_query()?;
        let response: OpenLibrarySearchResponse = self.get(
            "/search.json",
            &[
                ("isbn", query),
                ("limit", "1".to_string()),
                ("fields", search_fields()),
            ],
        )?;
        response
            .docs
            .into_iter()
            .next()
            .map(|doc| self.import_with_detail(doc))
            .transpose()
    }
}

fn apply_edition_record(
    import: &mut NormalizedBookImport,
    edition_key: &str,
    record: OpenLibraryEditionRecord,
) -> Result<(), MediaError> {
    let payload = serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string());
    let source_url = format!("https://openlibrary.org/books/{edition_key}");
    upsert_source_payload(import, "EDITION", edition_key, source_url, payload, Some(0));
    let Some(edition) = import.editions.first_mut() else {
        return Ok(());
    };
    fill_if_present(&mut edition.title, record.title);
    fill_if_present(&mut edition.subtitle, record.subtitle.unwrap_or_default());
    fill_if_present(
        &mut edition.publisher,
        record.publishers.first().cloned().unwrap_or_default(),
    );
    fill_if_present(
        &mut edition.published_date,
        record.publish_date.unwrap_or_default(),
    );
    if let Some(page_count) = record.number_of_pages.filter(|value| *value > 0) {
        edition.page_count = Some(page_count);
    }
    if let Some(language) = record.languages.first() {
        fill_if_present(&mut edition.language, key_tail(&language.key));
    }
    let (isbn_10, _) = normalized_isbns(&record.isbn_10);
    let (_, isbn_13) = normalized_isbns(&record.isbn_13);
    fill_if_present(&mut edition.isbn_10, isbn_10);
    fill_if_present(&mut edition.isbn_13, isbn_13);
    if let Some(cover_id) = record.covers.first() {
        let cover_url = format!("https://covers.openlibrary.org/b/id/{cover_id}-L.jpg");
        edition.cover_url = cover_url.clone();
        if import.canonical_cover_url.is_empty() {
            import.canonical_cover_url = cover_url;
        }
    }
    Ok(())
}

fn apply_work_record(
    import: &mut NormalizedBookImport,
    work_key: &str,
    record: OpenLibraryWorkRecord,
) -> Result<(), MediaError> {
    let payload = serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string());
    upsert_source_payload(
        import,
        "WORK",
        work_key,
        format!("https://openlibrary.org/works/{work_key}"),
        payload,
        None,
    );
    if import.description.is_empty() {
        import.description = record
            .description
            .map(OpenLibraryText::into_text)
            .map(|value| plain_text_description(&value))
            .unwrap_or_default();
    }
    for subject in record.subjects {
        if !import
            .subjects
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&subject))
        {
            import.subjects.push(subject);
        }
    }
    if import.first_publish_year.is_none() {
        import.first_publish_year = record
            .first_publish_date
            .as_deref()
            .and_then(|date| date.get(0..4))
            .and_then(|year| year.parse::<i64>().ok());
    }
    if import.canonical_cover_url.is_empty() {
        if let Some(cover_id) = record.covers.first() {
            import.canonical_cover_url =
                format!("https://covers.openlibrary.org/b/id/{cover_id}-L.jpg");
        }
    }
    Ok(())
}

fn upsert_source_payload(
    import: &mut NormalizedBookImport,
    entity_type: &str,
    external_id: &str,
    source_url: String,
    metadata_json: String,
    edition_index: Option<usize>,
) {
    if let Some(source) = import.sources.iter_mut().find(|source| {
        source.source_key == "open_library"
            && source.entity_type == entity_type
            && source.external_id == external_id
    }) {
        source.source_url = source_url;
        source.metadata_json = metadata_json;
        source.edition_index = edition_index;
    } else {
        import.sources.push(NormalizedBookSource {
            edition_index,
            source_key: "open_library".to_string(),
            entity_type: entity_type.to_string(),
            external_id: external_id.to_string(),
            source_url,
            metadata_json,
        });
    }
}

fn fill_if_present(target: &mut String, value: String) {
    if !value.trim().is_empty() {
        *target = value;
    }
}

fn search_fields() -> String {
    "key,title,subtitle,author_name,first_publish_year,isbn,edition_key,cover_i,publisher,language,number_of_pages_median,subject,ebook_access,public_scan_b".to_string()
}

fn normalized_isbns(values: &[String]) -> (String, String) {
    let mut isbn_10 = String::new();
    let mut isbn_13 = String::new();
    for value in values {
        if let Some(isbn) = normalize_isbn(value) {
            if isbn.len() == 10 && isbn_10.is_empty() {
                isbn_10 = isbn;
            } else if isbn.len() == 13 && isbn_13.is_empty() {
                isbn_13 = isbn;
            }
        }
    }
    (isbn_10, isbn_13)
}

fn key_tail(value: &str) -> String {
    value.rsplit('/').next().unwrap_or(value).to_string()
}

fn request_error(error: reqwest::Error) -> MediaError {
    MediaError::new(
        if error.is_timeout() || error.is_connect() {
            MediaErrorKind::Network
        } else {
            MediaErrorKind::ProviderUnavailable
        },
        format!("Open Library request failed: {error}"),
    )
}

fn decode_response<T: DeserializeOwned>(response: Response) -> Result<T, MediaError> {
    let status = response.status();
    if !status.is_success() {
        let kind = match status {
            StatusCode::NOT_FOUND => MediaErrorKind::NotFound,
            StatusCode::TOO_MANY_REQUESTS => MediaErrorKind::RateLimited,
            status if status.is_server_error() => MediaErrorKind::ProviderUnavailable,
            _ => MediaErrorKind::ProviderUnavailable,
        };
        return Err(MediaError::new(
            kind,
            format!("Open Library returned HTTP {status}."),
        ));
    }
    response.json().map_err(|error| {
        MediaError::new(
            MediaErrorKind::ProviderUnavailable,
            format!("Open Library returned an unreadable response: {error}"),
        )
    })
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OpenLibrarySearchResponse {
    #[serde(default)]
    docs: Vec<OpenLibrarySearchDoc>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OpenLibrarySearchDoc {
    #[serde(default)]
    key: String,
    #[serde(default)]
    title: String,
    subtitle: Option<String>,
    #[serde(default)]
    author_name: Vec<String>,
    first_publish_year: Option<i64>,
    #[serde(default)]
    isbn: Vec<String>,
    #[serde(default)]
    edition_key: Vec<String>,
    cover_i: Option<i64>,
    #[serde(default)]
    publisher: Vec<String>,
    #[serde(default)]
    language: Vec<String>,
    number_of_pages_median: Option<i64>,
    #[serde(default)]
    subject: Vec<String>,
    ebook_access: Option<String>,
    #[serde(default)]
    public_scan_b: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OpenLibraryEditionRecord {
    #[serde(default)]
    title: String,
    subtitle: Option<String>,
    #[serde(default)]
    publishers: Vec<String>,
    publish_date: Option<String>,
    #[serde(default)]
    isbn_10: Vec<String>,
    #[serde(default)]
    isbn_13: Vec<String>,
    number_of_pages: Option<i64>,
    #[serde(default)]
    covers: Vec<i64>,
    #[serde(default)]
    languages: Vec<OpenLibraryKey>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OpenLibraryWorkRecord {
    description: Option<OpenLibraryText>,
    #[serde(default)]
    subjects: Vec<String>,
    first_publish_date: Option<String>,
    #[serde(default)]
    covers: Vec<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OpenLibraryKey {
    #[serde(default)]
    key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum OpenLibraryText {
    Plain(String),
    Object { value: String },
}

impl OpenLibraryText {
    fn into_text(self) -> String {
        match self {
            Self::Plain(value) | Self::Object { value } => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_open_library_fixture_without_network() {
        let fixture = r#"{
          "key":"/works/OL1W","title":"Example","author_name":["A. Author"],"first_publish_year":2021,
          "isbn":["9780306406157"],"edition_key":["OL2M"],"cover_i":42,"publisher":["Press"],
          "language":["eng"],"number_of_pages_median":250,"subject":["History"],"public_scan_b":true
        }"#;
        let doc: OpenLibrarySearchDoc =
            serde_json::from_str(fixture).expect("fixture should parse");
        let candidate = OpenLibraryProvider::normalize_doc(doc.clone()).expect("candidate");
        assert_eq!(candidate.work_key, "OL1W");
        assert_eq!(candidate.edition_key, "OL2M");
        assert_eq!(candidate.isbn_13, "9780306406157");
        let import = OpenLibraryProvider::import_from_doc(doc).expect("import");
        assert_eq!(import.subjects, vec!["History"]);
    }

    #[test]
    fn identified_clients_use_the_documented_three_per_second_limit() {
        let provider = OpenLibraryProvider {
            client: Client::new(),
            contact_configured: true,
        };
        assert_eq!(provider.minimum_interval(), Duration::from_millis(334));
    }

    #[test]
    fn maps_official_work_and_edition_json_fixtures_without_network() {
        let doc: OpenLibrarySearchDoc = serde_json::from_str(
            r#"{"key":"/works/OL1W","title":"Example","edition_key":["OL2M"]}"#,
        )
        .expect("search fixture");
        let mut import = OpenLibraryProvider::import_from_doc(doc).expect("search import");
        let edition: OpenLibraryEditionRecord = serde_json::from_str(
            r#"{
              "title":"Example edition","publishers":["Example Press"],
              "publish_date":"2024-05-01","isbn_13":["9780306406157"],
              "number_of_pages":321,"covers":[55],"languages":[{"key":"/languages/eng"}]
            }"#,
        )
        .expect("edition fixture");
        let work: OpenLibraryWorkRecord = serde_json::from_str(
            r#"{
              "description":{"value":"A <b>work</b> description."},
              "subjects":["History"],"first_publish_date":"2020-01-01","covers":[66]
            }"#,
        )
        .expect("work fixture");

        apply_edition_record(&mut import, "OL2M", edition).expect("edition mapping");
        apply_work_record(&mut import, "OL1W", work).expect("work mapping");

        assert_eq!(import.description, "A work description.");
        assert_eq!(import.subjects, vec!["History"]);
        assert_eq!(import.editions[0].page_count, Some(321));
        assert_eq!(import.editions[0].language, "eng");
        assert!(import.sources.iter().any(|source| {
            source.entity_type == "EDITION" && source.metadata_json.contains("Example Press")
        }));
    }
}
