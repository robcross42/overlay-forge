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
use std::time::Duration;

const GOOGLE_BOOKS_ROOT: &str = "https://www.googleapis.com/books/v1";

pub struct GoogleBooksProvider {
    client: Client,
    api_key: String,
}

impl GoogleBooksProvider {
    pub fn from_environment_optional() -> Result<Option<Self>, MediaError> {
        let api_key = std::env::var("GOOGLE_BOOKS_API_KEY")
            .unwrap_or_default()
            .trim()
            .to_string();
        if api_key.is_empty() {
            return Ok(None);
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Overlay Forge/0.11 book-catalogue")
            .build()
            .map_err(|error| MediaError::new(MediaErrorKind::Network, error.to_string()))?;
        Ok(Some(Self { client, api_key }))
    }

    fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, MediaError> {
        let mut parameters = query.to_vec();
        parameters.push(("key", self.api_key.clone()));
        let response = self
            .client
            .get(format!("{GOOGLE_BOOKS_ROOT}{path}"))
            .query(&parameters)
            .send()
            .map_err(|error| provider_request_error("Google Books", error))?;
        decode_response("Google Books", response)
    }

    fn normalize_volume(volume: GoogleVolume) -> BookCatalogResult {
        let info = &volume.volume_info;
        let (isbn_10, isbn_13) = identifiers(&info.industry_identifiers);
        let cover_url = secure_google_url(
            info.image_links
                .as_ref()
                .and_then(|links| {
                    links
                        .thumbnail
                        .clone()
                        .or_else(|| links.small_thumbnail.clone())
                })
                .unwrap_or_default(),
        );
        BookCatalogResult {
            source_key: "google_books".to_string(),
            external_id: volume.id.clone(),
            title: info.title.clone().unwrap_or_default(),
            subtitle: info.subtitle.clone().unwrap_or_default(),
            authors: info.authors.clone(),
            published_date: info.published_date.clone().unwrap_or_default(),
            publisher: info.publisher.clone().unwrap_or_default(),
            isbn_10,
            isbn_13,
            page_count: info.page_count.filter(|value| *value > 0),
            language: info.language.clone().unwrap_or_default(),
            format: google_format(&volume),
            cover_url,
            description: plain_text_description(info.description.as_deref().unwrap_or_default()),
            work_key: String::new(),
            edition_key: String::new(),
            info_url: info.info_link.clone().unwrap_or_default(),
            preview_url: info.preview_link.clone().unwrap_or_default(),
            already_in_library: false,
            existing_entry_id: None,
            match_basis: "PROVIDER_ID".to_string(),
            provider_badges: vec!["Google Books".to_string()],
            source_identities: vec![BookSourceIdentity {
                source_key: "google_books".to_string(),
                entity_type: "EDITION".to_string(),
                external_id: volume.id,
                source_url: info.info_link.clone().unwrap_or_default(),
            }],
        }
    }

    fn volume_import(volume: GoogleVolume) -> Result<NormalizedBookImport, MediaError> {
        let candidate = Self::normalize_volume(volume.clone());
        if candidate.title.trim().is_empty() {
            return Err(MediaError::new(
                MediaErrorKind::ProviderUnavailable,
                "Google Books returned a volume without a title.",
            ));
        }
        let mut links = Vec::new();
        push_link(&mut links, "INFO", candidate.info_url.clone(), None);
        push_link(&mut links, "PREVIEW", candidate.preview_url.clone(), None);
        if let Some(buy_link) = volume
            .sale_info
            .as_ref()
            .and_then(|sale| sale.buy_link.clone())
        {
            push_link(&mut links, "BUY", buy_link, None);
        }
        if let Some(access) = &volume.access_info {
            if let Some(web_reader_link) = &access.web_reader_link {
                push_link(&mut links, "READ", web_reader_link.clone(), None);
            }
        }
        let metadata_json = serde_json::to_string(&volume).unwrap_or_else(|_| "{}".to_string());
        let published_year = candidate
            .published_date
            .get(0..4)
            .and_then(|value| value.parse::<i64>().ok());
        Ok(NormalizedBookImport {
            title: candidate.title.clone(),
            subtitle: candidate.subtitle.clone(),
            description: candidate.description,
            authors: candidate.authors,
            first_publish_year: published_year,
            subjects: volume.volume_info.categories,
            canonical_cover_url: candidate.cover_url.clone(),
            community_rating: volume.volume_info.average_rating,
            community_rating_count: volume.volume_info.ratings_count,
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
                is_ebook: volume.sale_info.as_ref().is_some_and(|sale| sale.is_ebook),
                access_viewability: volume
                    .access_info
                    .as_ref()
                    .and_then(|access| access.viewability.clone())
                    .unwrap_or_default(),
            }],
            sources: vec![NormalizedBookSource {
                edition_index: Some(0),
                source_key: "google_books".to_string(),
                entity_type: "EDITION".to_string(),
                external_id: volume.id,
                source_url: volume.volume_info.info_link.unwrap_or_default(),
                metadata_json,
            }],
            links,
            series: Vec::new(),
        })
    }
}

impl BookMetadataProvider for GoogleBooksProvider {
    fn search_books(&self, input: &BookSearchInput) -> Result<Vec<BookCatalogResult>, MediaError> {
        let query = input.validated_query()?;
        let query = match input.mode {
            BookSearchMode::General => query,
            BookSearchMode::Title => format!("intitle:{query}"),
            BookSearchMode::Author => format!("inauthor:{query}"),
            BookSearchMode::Isbn => format!("isbn:{query}"),
        };
        let mut parameters = vec![
            ("q", query),
            ("maxResults", input.bounded_limit().to_string()),
            ("printType", "books".to_string()),
            ("projection", "full".to_string()),
        ];
        if let Some(language) = input
            .language
            .split(['-', '_'])
            .next()
            .filter(|language| language.len() == 2)
        {
            parameters.push(("langRestrict", language.to_ascii_lowercase()));
        }
        let response: GoogleVolumeList = self.get("/volumes", &parameters)?;
        Ok(response
            .items
            .into_iter()
            .map(Self::normalize_volume)
            .filter(|candidate| !candidate.title.trim().is_empty())
            .collect())
    }

    fn fetch_book_candidate(&self, external_id: &str) -> Result<NormalizedBookImport, MediaError> {
        let volume: GoogleVolume = self.get(&format!("/volumes/{external_id}"), &[])?;
        Self::volume_import(volume)
    }

    fn enrich_by_isbn(&self, isbn: &str) -> Result<Option<NormalizedBookImport>, MediaError> {
        let input = BookSearchInput {
            query: isbn.to_string(),
            mode: BookSearchMode::Isbn,
            limit: 1,
            language: String::new(),
        };
        let Some(candidate) = self.search_books(&input)?.into_iter().next() else {
            return Ok(None);
        };
        self.fetch_book_candidate(&candidate.external_id).map(Some)
    }
}

fn identifiers(values: &[GoogleIndustryIdentifier]) -> (String, String) {
    let mut isbn_10 = String::new();
    let mut isbn_13 = String::new();
    for value in values {
        let Some(normalized) = normalize_isbn(&value.identifier) else {
            continue;
        };
        match normalized.len() {
            10 => isbn_10 = normalized,
            13 => isbn_13 = normalized,
            _ => {}
        }
    }
    (isbn_10, isbn_13)
}

fn google_format(volume: &GoogleVolume) -> BookFormat {
    if volume.sale_info.as_ref().is_some_and(|sale| sale.is_ebook) {
        BookFormat::Ebook
    } else {
        BookFormat::Unknown
    }
}

fn secure_google_url(url: String) -> String {
    for host in ["books.google.com", "books.googleusercontent.com"] {
        if let Some(rest) = url.strip_prefix(&format!("http://{host}")) {
            return format!("https://{host}{rest}");
        }
    }
    url
}

fn push_link(
    links: &mut Vec<NormalizedBookLink>,
    link_type: &str,
    url: String,
    edition_index: Option<usize>,
) {
    if url.starts_with("https://") || url.starts_with("http://") {
        links.push(NormalizedBookLink {
            edition_index,
            source_key: "google_books".to_string(),
            link_type: link_type.to_string(),
            url,
            region_code: String::new(),
        });
    }
}

fn provider_request_error(provider: &str, error: reqwest::Error) -> MediaError {
    let message = if error.is_timeout() {
        format!("{provider} request timed out.")
    } else if error.is_connect() {
        format!("{provider} could not be reached.")
    } else {
        format!("{provider} request failed.")
    };
    MediaError::new(
        if error.is_timeout() || error.is_connect() {
            MediaErrorKind::Network
        } else {
            MediaErrorKind::ProviderUnavailable
        },
        message,
    )
}

fn decode_response<T: DeserializeOwned>(
    provider: &str,
    response: Response,
) -> Result<T, MediaError> {
    let status = response.status();
    if !status.is_success() {
        let kind = match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => MediaErrorKind::Unauthorized,
            StatusCode::NOT_FOUND => MediaErrorKind::NotFound,
            StatusCode::TOO_MANY_REQUESTS => MediaErrorKind::RateLimited,
            status if status.is_server_error() => MediaErrorKind::ProviderUnavailable,
            _ => MediaErrorKind::ProviderUnavailable,
        };
        return Err(MediaError::new(
            kind,
            format!("{provider} returned HTTP {status}."),
        ));
    }
    response.json().map_err(|error| {
        MediaError::new(
            MediaErrorKind::ProviderUnavailable,
            format!("{provider} returned an unreadable response: {error}"),
        )
    })
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct GoogleVolumeList {
    #[serde(default)]
    items: Vec<GoogleVolume>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleVolume {
    id: String,
    #[serde(default)]
    volume_info: GoogleVolumeInfo,
    sale_info: Option<GoogleSaleInfo>,
    access_info: Option<GoogleAccessInfo>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleVolumeInfo {
    title: Option<String>,
    subtitle: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    publisher: Option<String>,
    published_date: Option<String>,
    description: Option<String>,
    #[serde(default)]
    industry_identifiers: Vec<GoogleIndustryIdentifier>,
    page_count: Option<i64>,
    #[serde(default)]
    categories: Vec<String>,
    average_rating: Option<f64>,
    ratings_count: Option<i64>,
    language: Option<String>,
    image_links: Option<GoogleImageLinks>,
    preview_link: Option<String>,
    info_link: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleIndustryIdentifier {
    #[serde(rename = "type")]
    identifier_type: String,
    identifier: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleImageLinks {
    small_thumbnail: Option<String>,
    thumbnail: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleSaleInfo {
    #[serde(default)]
    is_ebook: bool,
    buy_link: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleAccessInfo {
    viewability: Option<String>,
    web_reader_link: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_google_fixture_without_network() {
        let fixture = r#"{
          "id":"abc123",
          "volumeInfo":{"title":"Example","authors":["A. Author"],"publishedDate":"2020","industryIdentifiers":[{"type":"ISBN_13","identifier":"9780306406157"}],"pageCount":320,"description":"A <b>book</b>."}
        }"#;
        let volume: GoogleVolume = serde_json::from_str(fixture).expect("fixture should parse");
        let candidate = GoogleBooksProvider::normalize_volume(volume.clone());
        assert_eq!(candidate.external_id, "abc123");
        assert_eq!(candidate.isbn_13, "9780306406157");
        assert_eq!(candidate.description, "A book.");
        let import = GoogleBooksProvider::volume_import(volume).expect("volume should normalize");
        assert_eq!(import.editions[0].page_count, Some(320));
    }
}
