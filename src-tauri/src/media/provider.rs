use super::domain::{
    MediaCatalogSearchResult, MediaContentType, MediaError, MediaErrorKind, NormalizedMediaEpisode,
    NormalizedMediaImport, NormalizedMediaSeason, NormalizedMediaTitle,
    NormalizedProviderAvailability, NormalizedProviderSnapshot,
};
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

const TMDB_API_ROOT: &str = "https://api.themoviedb.org/3";

pub trait MediaMetadataProvider {
    fn search(
        &self,
        query: &str,
        language: &str,
    ) -> Result<Vec<MediaCatalogSearchResult>, MediaError>;

    fn fetch_import(
        &self,
        external_id: i64,
        external_media_type: &str,
        language: &str,
        region: &str,
    ) -> Result<NormalizedMediaImport, MediaError>;
}

pub struct TmdbMediaMetadataProvider {
    client: Client,
    bearer_token: String,
}

impl TmdbMediaMetadataProvider {
    pub fn from_environment() -> Result<Self, MediaError> {
        let bearer_token = std::env::var("TMDB_API_READ_ACCESS_TOKEN")
            .unwrap_or_default()
            .trim()
            .to_string();
        if bearer_token.is_empty() {
            return Err(MediaError::new(
                MediaErrorKind::MissingCredential,
                "Set TMDB_API_READ_ACCESS_TOKEN in the Rust backend environment to search or refresh catalogue metadata. Your local library remains available.",
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Overlay Forge/0.9 media-library")
            .build()
            .map_err(|error| {
                MediaError::new(
                    MediaErrorKind::Network,
                    format!("Could not initialize the TMDB client: {error}"),
                )
            })?;

        Ok(Self {
            client,
            bearer_token,
        })
    }

    fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, MediaError> {
        let response = self
            .client
            .get(format!("{TMDB_API_ROOT}{path}"))
            .bearer_auth(&self.bearer_token)
            .query(query)
            .send()
            .map_err(|error| {
                let kind = if error.is_timeout() || error.is_connect() {
                    MediaErrorKind::Network
                } else {
                    MediaErrorKind::ProviderUnavailable
                };
                MediaError::new(kind, format!("TMDB request failed: {error}"))
            })?;
        Self::decode_response(response)
    }

    fn decode_response<T: DeserializeOwned>(response: Response) -> Result<T, MediaError> {
        let status = response.status();
        if !status.is_success() {
            let kind = match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => MediaErrorKind::Unauthorized,
                StatusCode::NOT_FOUND => MediaErrorKind::NotFound,
                StatusCode::TOO_MANY_REQUESTS => MediaErrorKind::RateLimited,
                status if status.is_server_error() => MediaErrorKind::ProviderUnavailable,
                _ => MediaErrorKind::Unknown,
            };
            let message = match kind {
                MediaErrorKind::Unauthorized => {
                    "TMDB rejected the configured API read access token.".to_string()
                }
                MediaErrorKind::NotFound => "TMDB could not find that title.".to_string(),
                MediaErrorKind::RateLimited => {
                    "TMDB rate-limited the request. Try again later.".to_string()
                }
                MediaErrorKind::ProviderUnavailable => {
                    "TMDB is temporarily unavailable. Cached local data was kept.".to_string()
                }
                _ => format!("TMDB returned HTTP status {status}."),
            };
            return Err(MediaError::new(kind, message));
        }

        response.json::<T>().map_err(|error| {
            MediaError::new(
                MediaErrorKind::ProviderUnavailable,
                format!("TMDB returned an unreadable response: {error}"),
            )
        })
    }

    fn fetch_availability(
        &self,
        external_id: i64,
        external_media_type: &str,
        region: &str,
    ) -> Result<NormalizedProviderSnapshot, MediaError> {
        let path = format!(
            "/{}/{external_id}/watch/providers",
            normalize_external_media_type(external_media_type)?
        );
        let response: TmdbWatchProviderResponse = self.get(&path, &[])?;
        let region_code = region.trim().to_ascii_uppercase();
        let availability = response.results.get(&region_code);
        let mut providers = Vec::new();
        if let Some(availability) = availability {
            append_provider_rows(&mut providers, "FLATRATE", &availability.flatrate);
            append_provider_rows(&mut providers, "FREE", &availability.free);
            append_provider_rows(&mut providers, "ADS", &availability.ads);
            append_provider_rows(&mut providers, "RENT", &availability.rent);
            append_provider_rows(&mut providers, "BUY", &availability.buy);
        }

        Ok(NormalizedProviderSnapshot {
            region_code,
            source_link_url: availability
                .and_then(|item| item.link.clone())
                .unwrap_or_default(),
            providers,
        })
    }

    fn fetch_movie_import(
        &self,
        external_id: i64,
        language: &str,
        region: &str,
    ) -> Result<NormalizedMediaImport, MediaError> {
        let response: TmdbMovieDetail = self.get(
            &format!("/movie/{external_id}"),
            &[("language", language.to_string())],
        )?;
        let metadata_json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
        let title = response.title.trim();
        let original_title = response.original_title.trim();
        let normalized = NormalizedMediaTitle {
            source_key: "tmdb".to_string(),
            external_id: Some(response.id),
            external_media_type: Some("movie".to_string()),
            content_type: MediaContentType::Movie,
            title: preferred_title(title, original_title),
            original_title: original_title.to_string(),
            overview: response.overview.unwrap_or_default(),
            original_language: response.original_language.unwrap_or_default(),
            release_date: response.release_date.unwrap_or_default(),
            first_air_date: String::new(),
            last_air_date: String::new(),
            runtime_minutes: response.runtime,
            episode_runtime_minutes: None,
            external_status: response.status.unwrap_or_default(),
            series_type: String::new(),
            poster_path: response.poster_path.unwrap_or_default(),
            backdrop_path: response.backdrop_path.unwrap_or_default(),
            homepage_url: response.homepage.unwrap_or_default(),
            total_seasons: None,
            total_episodes: None,
            metadata_json,
        };
        let (provider_snapshot, provider_error) =
            match self.fetch_availability(external_id, "movie", region) {
                Ok(snapshot) => (Some(snapshot), None),
                Err(error) => (None, Some(error.to_string())),
            };
        Ok(NormalizedMediaImport {
            title: normalized,
            seasons: Vec::new(),
            provider_snapshot,
            provider_error,
        })
    }

    fn fetch_series_import(
        &self,
        external_id: i64,
        language: &str,
        region: &str,
    ) -> Result<NormalizedMediaImport, MediaError> {
        let response: TmdbSeriesDetail = self.get(
            &format!("/tv/{external_id}"),
            &[("language", language.to_string())],
        )?;
        let metadata_json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
        let name = response.name.trim();
        let original_name = response.original_name.trim();
        let episode_runtime_minutes = response
            .episode_run_time
            .iter()
            .copied()
            .find(|runtime| *runtime > 0);
        let normalized_title = NormalizedMediaTitle {
            source_key: "tmdb".to_string(),
            external_id: Some(response.id),
            external_media_type: Some("tv".to_string()),
            content_type: MediaContentType::Series,
            title: preferred_title(name, original_name),
            original_title: original_name.to_string(),
            overview: response.overview.unwrap_or_default(),
            original_language: response.original_language.unwrap_or_default(),
            release_date: String::new(),
            first_air_date: response.first_air_date.unwrap_or_default(),
            last_air_date: response.last_air_date.unwrap_or_default(),
            runtime_minutes: None,
            episode_runtime_minutes,
            external_status: response.status.unwrap_or_default(),
            series_type: response.series_type.unwrap_or_default(),
            poster_path: response.poster_path.unwrap_or_default(),
            backdrop_path: response.backdrop_path.unwrap_or_default(),
            homepage_url: response.homepage.unwrap_or_default(),
            total_seasons: response.number_of_seasons,
            total_episodes: response.number_of_episodes,
            metadata_json,
        };

        let mut seasons = Vec::with_capacity(response.seasons.len());
        for season_summary in response.seasons {
            let season: TmdbSeasonDetail = self.get(
                &format!("/tv/{external_id}/season/{}", season_summary.season_number),
                &[("language", language.to_string())],
            )?;
            let season_json = serde_json::to_string(&season).unwrap_or_else(|_| "{}".to_string());
            let episodes = season
                .episodes
                .into_iter()
                .map(|episode| {
                    let metadata_json =
                        serde_json::to_string(&episode).unwrap_or_else(|_| "{}".to_string());
                    NormalizedMediaEpisode {
                        external_id: Some(episode.id),
                        season_number: episode.season_number,
                        episode_number: episode.episode_number,
                        name: episode.name.unwrap_or_default(),
                        overview: episode.overview.unwrap_or_default(),
                        air_date: episode.air_date.unwrap_or_default(),
                        runtime_minutes: episode.runtime,
                        still_path: episode.still_path.unwrap_or_default(),
                        metadata_json,
                    }
                })
                .collect::<Vec<_>>();
            seasons.push(NormalizedMediaSeason {
                external_id: Some(season.id),
                season_number: season.season_number,
                name: season.name.unwrap_or(season_summary.name),
                overview: season.overview.unwrap_or(season_summary.overview),
                air_date: season
                    .air_date
                    .or(season_summary.air_date)
                    .unwrap_or_default(),
                poster_path: season
                    .poster_path
                    .or(season_summary.poster_path)
                    .unwrap_or_default(),
                episode_count: episodes.len() as i64,
                metadata_json: season_json,
                episodes,
            });
        }
        seasons.sort_by_key(|season| season.season_number);

        let (provider_snapshot, provider_error) =
            match self.fetch_availability(external_id, "tv", region) {
                Ok(snapshot) => (Some(snapshot), None),
                Err(error) => (None, Some(error.to_string())),
            };
        Ok(NormalizedMediaImport {
            title: normalized_title,
            seasons,
            provider_snapshot,
            provider_error,
        })
    }
}

impl MediaMetadataProvider for TmdbMediaMetadataProvider {
    fn search(
        &self,
        query: &str,
        language: &str,
    ) -> Result<Vec<MediaCatalogSearchResult>, MediaError> {
        let query = query.trim();
        if query.len() < 2 {
            return Err(MediaError::validation(
                "Enter at least two characters to search TMDB.",
            ));
        }
        let response: TmdbMultiSearchResponse = self.get(
            "/search/multi",
            &[
                ("query", query.to_string()),
                ("include_adult", "false".to_string()),
                ("language", language.to_string()),
                ("page", "1".to_string()),
            ],
        )?;

        Ok(response
            .results
            .into_iter()
            .filter_map(|item| {
                let media_type = item.media_type?;
                let content_type = match media_type.as_str() {
                    "movie" => MediaContentType::Movie,
                    "tv" => MediaContentType::Series,
                    _ => return None,
                };
                let title = item
                    .title
                    .as_deref()
                    .or(item.name.as_deref())
                    .unwrap_or_default();
                let original_title = item
                    .original_title
                    .as_deref()
                    .or(item.original_name.as_deref())
                    .unwrap_or_default();
                Some(MediaCatalogSearchResult {
                    external_id: item.id,
                    external_media_type: media_type,
                    content_type,
                    title: preferred_title(title, original_title),
                    original_title: original_title.to_string(),
                    overview: item.overview.unwrap_or_default(),
                    original_language: item.original_language.unwrap_or_default(),
                    release_date: item
                        .release_date
                        .or(item.first_air_date)
                        .unwrap_or_default(),
                    poster_path: item.poster_path.unwrap_or_default(),
                    backdrop_path: item.backdrop_path.unwrap_or_default(),
                    already_in_library: false,
                    existing_entry_id: None,
                })
            })
            .collect())
    }

    fn fetch_import(
        &self,
        external_id: i64,
        external_media_type: &str,
        language: &str,
        region: &str,
    ) -> Result<NormalizedMediaImport, MediaError> {
        if external_id <= 0 {
            return Err(MediaError::validation(
                "TMDB title identifiers must be positive.",
            ));
        }
        match normalize_external_media_type(external_media_type)? {
            "movie" => self.fetch_movie_import(external_id, language, region),
            "tv" => self.fetch_series_import(external_id, language, region),
            _ => unreachable!(),
        }
    }
}

fn normalize_external_media_type(value: &str) -> Result<&'static str, MediaError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "movie" => Ok("movie"),
        "tv" => Ok("tv"),
        _ => Err(MediaError::validation(
            "TMDB media type must be movie or tv.",
        )),
    }
}

fn preferred_title(localized: &str, original: &str) -> String {
    if localized.trim().is_empty() {
        original.trim().to_string()
    } else {
        localized.trim().to_string()
    }
}

fn append_provider_rows(
    rows: &mut Vec<NormalizedProviderAvailability>,
    monetization_type: &str,
    providers: &[TmdbWatchProvider],
) {
    rows.extend(
        providers
            .iter()
            .map(|provider| NormalizedProviderAvailability {
                provider_external_id: provider.provider_id,
                provider_name: provider.provider_name.clone(),
                provider_logo_path: provider.logo_path.clone().unwrap_or_default(),
                monetization_type: monetization_type.to_string(),
                display_priority: provider.display_priority.unwrap_or(0),
            }),
    );
}

#[derive(Debug, Deserialize)]
struct TmdbMultiSearchResponse {
    #[serde(default)]
    results: Vec<TmdbSearchItem>,
}

#[derive(Debug, Deserialize)]
struct TmdbSearchItem {
    id: i64,
    media_type: Option<String>,
    title: Option<String>,
    original_title: Option<String>,
    name: Option<String>,
    original_name: Option<String>,
    overview: Option<String>,
    original_language: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TmdbMovieDetail {
    id: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: String,
    overview: Option<String>,
    original_language: Option<String>,
    release_date: Option<String>,
    runtime: Option<i64>,
    status: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    homepage: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TmdbSeriesDetail {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    original_name: String,
    overview: Option<String>,
    original_language: Option<String>,
    first_air_date: Option<String>,
    last_air_date: Option<String>,
    #[serde(default)]
    episode_run_time: Vec<i64>,
    status: Option<String>,
    #[serde(rename = "type")]
    series_type: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    homepage: Option<String>,
    number_of_seasons: Option<i64>,
    number_of_episodes: Option<i64>,
    #[serde(default)]
    seasons: Vec<TmdbSeasonSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TmdbSeasonSummary {
    id: i64,
    season_number: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    overview: String,
    air_date: Option<String>,
    poster_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TmdbSeasonDetail {
    id: i64,
    season_number: i64,
    name: Option<String>,
    overview: Option<String>,
    air_date: Option<String>,
    poster_path: Option<String>,
    #[serde(default)]
    episodes: Vec<TmdbEpisodeDetail>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TmdbEpisodeDetail {
    id: i64,
    season_number: i64,
    episode_number: i64,
    name: Option<String>,
    overview: Option<String>,
    air_date: Option<String>,
    runtime: Option<i64>,
    still_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbWatchProviderResponse {
    #[serde(default)]
    results: HashMap<String, TmdbRegionAvailability>,
}

#[derive(Debug, Deserialize)]
struct TmdbRegionAvailability {
    link: Option<String>,
    #[serde(default)]
    flatrate: Vec<TmdbWatchProvider>,
    #[serde(default)]
    free: Vec<TmdbWatchProvider>,
    #[serde(default)]
    ads: Vec<TmdbWatchProvider>,
    #[serde(default)]
    rent: Vec<TmdbWatchProvider>,
    #[serde(default)]
    buy: Vec<TmdbWatchProvider>,
}

#[derive(Debug, Deserialize)]
struct TmdbWatchProvider {
    provider_id: i64,
    provider_name: String,
    logo_path: Option<String>,
    display_priority: Option<i64>,
}
