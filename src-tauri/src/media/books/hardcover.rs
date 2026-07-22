use super::domain::{
    normalize_isbn, plain_text_description, BookCatalogResult, BookFormat, BookSearchInput,
    NormalizedBookEdition, NormalizedBookImport, NormalizedBookSeries, NormalizedBookSource,
};
use super::provider::BookMetadataProvider;
use crate::media::domain::{MediaError, MediaErrorKind};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

const HARDCOVER_GRAPHQL_ENDPOINT: &str = "https://api.hardcover.app/v1/graphql";
const EDITION_BY_ISBN_QUERY: &str = r#"
query EditionByIsbn($isbn: String!) {
  editions(
    where: { _or: [{ isbn_13: { _eq: $isbn } }, { isbn_10: { _eq: $isbn } }] }
    limit: 1
  ) {
    id
    title
    subtitle
    isbn_10
    isbn_13
    pages
    audio_seconds
    physical_format
    release_date
    rating
    users_count
    publisher { name }
    book {
      id
      title
      description
      rating
      users_count
      slug
      book_series {
        position
        featured
        series { id name }
      }
    }
  }
}
"#;

pub struct HardcoverProvider {
    client: Client,
    token: String,
}

impl HardcoverProvider {
    pub fn from_environment_optional() -> Result<Option<Self>, MediaError> {
        let token = std::env::var("HARDCOVER_API_TOKEN")
            .unwrap_or_default()
            .trim()
            .to_string();
        if token.is_empty() {
            return Ok(None);
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Overlay Forge/0.11 book-catalogue")
            .build()
            .map_err(|error| MediaError::new(MediaErrorKind::Network, error.to_string()))?;
        Ok(Some(Self { client, token }))
    }

    fn execute<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, MediaError> {
        let response = self
            .client
            .post(HARDCOVER_GRAPHQL_ENDPOINT)
            .bearer_auth(&self.token)
            .json(&json!({ "query": query, "variables": variables }))
            .send()
            .map_err(|error| {
                MediaError::new(
                    if error.is_timeout() || error.is_connect() {
                        MediaErrorKind::Network
                    } else {
                        MediaErrorKind::ProviderUnavailable
                    },
                    format!("Hardcover request failed: {error}"),
                )
            })?;
        let status = response.status();
        if !status.is_success() {
            let kind = match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => MediaErrorKind::Unauthorized,
                StatusCode::TOO_MANY_REQUESTS => MediaErrorKind::RateLimited,
                _ => MediaErrorKind::ProviderUnavailable,
            };
            return Err(MediaError::new(
                kind,
                format!("Hardcover returned HTTP {status}."),
            ));
        }
        let envelope: GraphQlEnvelope<T> = response.json().map_err(|error| {
            MediaError::new(
                MediaErrorKind::ProviderUnavailable,
                format!("Hardcover returned an unreadable response: {error}"),
            )
        })?;
        decode_graphql_envelope(envelope)
    }

    fn normalize_edition(edition: HardcoverEdition) -> NormalizedBookImport {
        let book = edition.book.clone();
        let title = book
            .as_ref()
            .and_then(|value| value.title.clone())
            .or_else(|| edition.title.clone())
            .unwrap_or_default();
        let isbn_10 = edition
            .isbn_10
            .as_deref()
            .and_then(normalize_isbn)
            .filter(|value| value.len() == 10)
            .unwrap_or_default();
        let isbn_13 = edition
            .isbn_13
            .as_deref()
            .and_then(normalize_isbn)
            .filter(|value| value.len() == 13)
            .unwrap_or_default();
        let book_id = book.as_ref().map(|value| value.id.to_string());
        let source_url = book
            .as_ref()
            .and_then(|value| value.slug.as_ref())
            .map(|slug| format!("https://hardcover.app/books/{slug}"))
            .unwrap_or_default();
        let mut sources = vec![NormalizedBookSource {
            edition_index: Some(0),
            source_key: "hardcover".to_string(),
            entity_type: "EDITION".to_string(),
            external_id: edition.id.to_string(),
            source_url: source_url.clone(),
            metadata_json: serde_json::to_string(&edition).unwrap_or_else(|_| "{}".to_string()),
        }];
        if let Some(book_id) = book_id {
            sources.push(NormalizedBookSource {
                edition_index: None,
                source_key: "hardcover".to_string(),
                entity_type: "BOOK".to_string(),
                external_id: book_id,
                source_url,
                metadata_json: serde_json::to_string(&book).unwrap_or_else(|_| "{}".to_string()),
            });
        }
        let series = book
            .as_ref()
            .map(|value| {
                value
                    .book_series
                    .iter()
                    .filter_map(|member| {
                        member.series.as_ref().map(|series| NormalizedBookSeries {
                            name: series.name.clone(),
                            position_text: member.position.map(format_position).unwrap_or_default(),
                            position_sort: member.position,
                            source_key: "hardcover".to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let community_rating = book
            .as_ref()
            .and_then(|value| value.rating)
            .or(edition.rating);
        let community_rating_count = book
            .as_ref()
            .and_then(|value| value.users_count)
            .or(edition.users_count);
        NormalizedBookImport {
            title: title.clone(),
            subtitle: edition.subtitle.clone().unwrap_or_default(),
            description: plain_text_description(
                book.as_ref()
                    .and_then(|value| value.description.as_deref())
                    .unwrap_or_default(),
            ),
            authors: Vec::new(),
            first_publish_year: edition
                .release_date
                .as_deref()
                .and_then(|date| date.get(0..4))
                .and_then(|year| year.parse::<i64>().ok()),
            subjects: Vec::new(),
            canonical_cover_url: String::new(),
            community_rating,
            community_rating_count,
            editions: vec![NormalizedBookEdition {
                title,
                subtitle: edition.subtitle.unwrap_or_default(),
                format: hardcover_format(edition.physical_format.as_deref(), edition.audio_seconds),
                isbn_10,
                isbn_13,
                publisher: edition
                    .publisher
                    .and_then(|publisher| publisher.name)
                    .unwrap_or_default(),
                published_date: edition.release_date.unwrap_or_default(),
                language: String::new(),
                page_count: edition.pages.filter(|value| *value > 0),
                audio_duration_minutes: edition
                    .audio_seconds
                    .filter(|value| *value > 0)
                    .map(|seconds| (seconds + 59) / 60),
                cover_url: String::new(),
                is_ebook: false,
                access_viewability: String::new(),
            }],
            sources,
            links: Vec::new(),
            series,
        }
    }
}

impl BookMetadataProvider for HardcoverProvider {
    fn search_books(&self, _input: &BookSearchInput) -> Result<Vec<BookCatalogResult>, MediaError> {
        Err(MediaError::validation(
            "Hardcover is an ISBN enrichment provider and is not used for catalogue search.",
        ))
    }

    fn fetch_book_candidate(&self, _external_id: &str) -> Result<NormalizedBookImport, MediaError> {
        Err(MediaError::validation(
            "Hardcover records are fetched only by exact ISBN.",
        ))
    }

    fn enrich_by_isbn(&self, isbn: &str) -> Result<Option<NormalizedBookImport>, MediaError> {
        let isbn = normalize_isbn(isbn)
            .ok_or_else(|| MediaError::validation("A valid ISBN is required for Hardcover."))?;
        let data: HardcoverData = self.execute(EDITION_BY_ISBN_QUERY, json!({ "isbn": isbn }))?;
        Ok(data
            .editions
            .into_iter()
            .next()
            .map(Self::normalize_edition))
    }
}

fn decode_graphql_envelope<T>(envelope: GraphQlEnvelope<T>) -> Result<T, MediaError> {
    if !envelope.errors.is_empty() {
        return Err(MediaError::new(
            MediaErrorKind::ProviderUnavailable,
            format!(
                "Hardcover GraphQL error: {}",
                envelope
                    .errors
                    .into_iter()
                    .map(|error| error.message)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }
    envelope.data.ok_or_else(|| {
        MediaError::new(
            MediaErrorKind::ProviderUnavailable,
            "Hardcover returned no GraphQL data.",
        )
    })
}

fn hardcover_format(value: Option<&str>, audio_seconds: Option<i64>) -> BookFormat {
    if audio_seconds.is_some_and(|value| value > 0) {
        return BookFormat::Audiobook;
    }
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        value if value.contains("hardcover") || value.contains("hardback") => BookFormat::Hardcover,
        value if value.contains("paperback") => BookFormat::Paperback,
        value if value.contains("ebook") || value.contains("kindle") => BookFormat::Ebook,
        "" => BookFormat::Unknown,
        _ => BookFormat::Other,
    }
}

fn format_position(position: f64) -> String {
    if position.fract() == 0.0 {
        format!("{}", position as i64)
    } else {
        position.to_string()
    }
}

#[derive(Debug, Deserialize)]
struct GraphQlEnvelope<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct HardcoverData {
    #[serde(default)]
    editions: Vec<HardcoverEdition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HardcoverEdition {
    id: i64,
    title: Option<String>,
    subtitle: Option<String>,
    isbn_10: Option<String>,
    isbn_13: Option<String>,
    pages: Option<i64>,
    audio_seconds: Option<i64>,
    physical_format: Option<String>,
    release_date: Option<String>,
    rating: Option<f64>,
    users_count: Option<i64>,
    publisher: Option<HardcoverPublisher>,
    book: Option<HardcoverBook>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HardcoverPublisher {
    name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HardcoverBook {
    id: i64,
    title: Option<String>,
    description: Option<String>,
    rating: Option<f64>,
    users_count: Option<i64>,
    slug: Option<String>,
    #[serde(default)]
    book_series: Vec<HardcoverSeriesMember>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HardcoverSeriesMember {
    position: Option<f64>,
    #[serde(default)]
    featured: bool,
    series: Option<HardcoverSeries>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HardcoverSeries {
    id: i64,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_hardcover_fixture_without_network() {
        let fixture = r#"{
          "id":12,"title":"Edition title","isbn_13":"9780306406157","pages":400,
          "physical_format":"Hardcover","rating":4.2,"users_count":50,
          "publisher":{"name":"Press"},
          "book":{"id":8,"title":"Work title","description":"A <i>story</i>","rating":4.4,"users_count":100,"slug":"work-title","book_series":[{"position":2.5,"series":{"id":3,"name":"A Series"}}]}
        }"#;
        let edition: HardcoverEdition =
            serde_json::from_str(fixture).expect("fixture should parse");
        let import = HardcoverProvider::normalize_edition(edition);
        assert_eq!(import.title, "Work title");
        assert_eq!(import.series[0].position_text, "2.5");
        assert_eq!(import.community_rating, Some(4.4));
    }

    #[test]
    fn graphql_errors_are_failures_even_with_http_success() {
        let envelope: GraphQlEnvelope<HardcoverData> =
            serde_json::from_str(r#"{"data":null,"errors":[{"message":"query failed"}]}"#)
                .expect("fixture should parse");
        let error = decode_graphql_envelope(envelope).expect_err("GraphQL error should fail");
        assert!(error.message.contains("query failed"));
    }
}
