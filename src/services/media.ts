import { invoke } from "@tauri-apps/api/core";

export type MediaContentType = "MOVIE" | "SERIES";
export type MediaLibraryStatus = "PLANNED" | "WATCHING" | "COMPLETED" | "ON_HOLD" | "DROPPED";
export type MediaSort =
  | "TITLE"
  | "RECENTLY_ADDED"
  | "RECENTLY_WATCHED"
  | "PROGRESS"
  | "RELEASE_DATE"
  | "WATCH_NEXT"
  | "PRIORITY";

export type MediaCatalogSearchResult = {
  externalId: number;
  externalMediaType: "movie" | "tv";
  contentType: MediaContentType;
  title: string;
  originalTitle: string;
  overview: string;
  originalLanguage: string;
  releaseDate: string;
  posterPath: string;
  backdropPath: string;
  alreadyInLibrary: boolean;
  existingEntryId: number | null;
};

export type MediaNextEpisode = {
  id: number;
  seasonNumber: number;
  episodeNumber: number;
  name: string;
  airDate: string;
};

export type MediaProviderSummary = {
  providerName: string;
  providerLogoPath: string;
};

export type MediaLibrarySummary = {
  id: number;
  mediaTitleId: number;
  sourceKey: string;
  externalId: number | null;
  externalMediaType: string | null;
  contentType: MediaContentType;
  title: string;
  originalTitle: string;
  overview: string;
  originalLanguage: string;
  releaseDate: string;
  firstAirDate: string;
  lastAirDate: string;
  runtimeMinutes: number | null;
  episodeRuntimeMinutes: number | null;
  externalStatus: string;
  seriesType: string;
  posterPath: string;
  backdropPath: string;
  homepageUrl: string;
  totalSeasons: number | null;
  totalEpisodes: number | null;
  metadataRefreshedAt: string;
  libraryStatus: MediaLibraryStatus;
  isFavorite: boolean;
  personalRating: number | null;
  notes: string;
  priority: number;
  queuePosition: number | null;
  addedAt: string;
  startedAt: string;
  completedAt: string;
  lastWatchedAt: string;
  movieWatched: boolean;
  watchedCount: number;
  progressTotal: number;
  progressPercent: number;
  nextEpisode: MediaNextEpisode | null;
  newEpisodesCount: number;
  subscriptionProviders: MediaProviderSummary[];
  availabilityIsStale: boolean;
};

export type MediaEpisode = {
  id: number;
  externalId: number | null;
  seasonNumber: number;
  episodeNumber: number;
  name: string;
  overview: string;
  airDate: string;
  runtimeMinutes: number | null;
  stillPath: string;
  isPresentInSource: boolean;
  isWatched: boolean;
  watchedAt: string;
  notes: string;
};

export type MediaSeason = {
  id: number;
  externalId: number | null;
  seasonNumber: number;
  name: string;
  overview: string;
  airDate: string;
  posterPath: string;
  episodeCount: number;
  isPresentInSource: boolean;
  watchedCount: number;
  progressTotal: number;
  episodes: MediaEpisode[];
};

export type MediaTag = {
  id: number;
  name: string;
  createdAt: string;
  updatedAt: string;
};

export type MediaStreamingLink = {
  id: number;
  mediaLibraryEntryId: number;
  providerName: string;
  url: string;
  linkType: "STREAM" | "RENT" | "BUY" | "OTHER";
  isPreferred: boolean;
  notes: string;
  createdAt: string;
  updatedAt: string;
};

export type MediaProviderSnapshot = {
  id: number;
  regionCode: string;
  sourceLinkUrl: string;
  refreshedAt: string;
  lastRefreshStatus: string;
  lastRefreshError: string;
};

export type MediaProviderAvailability = {
  id: number;
  providerExternalId: number;
  providerName: string;
  providerLogoPath: string;
  monetizationType: "FLATRATE" | "FREE" | "ADS" | "RENT" | "BUY";
  displayPriority: number;
};

export type MediaSettings = {
  regionCode: string;
  metadataLanguage: string;
  includeSpecialsInCompletion: boolean;
  updatedAt: string;
};

export type MediaLibraryDetail = {
  entry: MediaLibrarySummary;
  seasons: MediaSeason[];
  tags: MediaTag[];
  streamingLinks: MediaStreamingLink[];
  providerSnapshot: MediaProviderSnapshot | null;
  providers: MediaProviderAvailability[];
  settings: MediaSettings;
};

export type MediaLibraryFilter = {
  query: string;
  contentType: MediaContentType | null;
  libraryStatus: MediaLibraryStatus | null;
  isFavorite: boolean | null;
  tagId: number | null;
  hasUnwatched: boolean | null;
  providerName: string;
  isWatchNext: boolean | null;
  sort: MediaSort;
  sortDirection: "ASC" | "DESC";
  limit: number;
  offset: number;
};

export type ManualMediaInput = {
  contentType: MediaContentType;
  title: string;
  releaseDate: string;
  overview: string;
  runtimeMinutes: number | null;
  libraryStatus: MediaLibraryStatus;
  notes: string;
  personalRating: number | null;
  isFavorite: boolean;
  priority: number;
  addToWatchNext: boolean;
};

export type UpdateMediaEntryInput = {
  libraryStatus: MediaLibraryStatus;
  isFavorite: boolean;
  personalRating: number | null;
  notes: string;
  priority: number;
};

export type MediaStreamingLinkInput = {
  providerName: string;
  url: string;
  linkType: MediaStreamingLink["linkType"];
  isPreferred: boolean;
  notes: string;
};

export const defaultMediaFilter: MediaLibraryFilter = {
  query: "",
  contentType: null,
  libraryStatus: null,
  isFavorite: null,
  tagId: null,
  hasUnwatched: null,
  providerName: "",
  isWatchNext: null,
  sort: "RECENTLY_ADDED",
  sortDirection: "DESC",
  limit: 100,
  offset: 0
};

export function searchMediaCatalog(query: string) {
  return invoke<MediaCatalogSearchResult[]>("search_media_catalog", { query });
}

export function addCatalogMediaToLibrary(
  result: Pick<MediaCatalogSearchResult, "externalId" | "externalMediaType">,
  libraryStatus: MediaLibraryStatus = "PLANNED"
) {
  return invoke<MediaLibraryDetail>("add_catalog_media_to_library", {
    input: { ...result, libraryStatus }
  });
}

export function createManualMediaEntry(input: ManualMediaInput) {
  return invoke<MediaLibraryDetail>("create_manual_media_entry", { input });
}

export function listMediaLibrary(filter: MediaLibraryFilter = defaultMediaFilter) {
  return invoke<MediaLibrarySummary[]>("list_media_library", { filter });
}

export function getMediaLibraryDetail(entryId: number) {
  return invoke<MediaLibraryDetail>("get_media_library_detail", { entryId });
}

export function updateMediaLibraryEntry(entryId: number, input: UpdateMediaEntryInput) {
  return invoke<MediaLibraryDetail>("update_media_library_entry", { entryId, input });
}

export function deleteMediaLibraryEntry(entryId: number) {
  return invoke<void>("delete_media_library_entry", { entryId });
}

export function refreshMediaMetadata(entryId: number) {
  return invoke<MediaLibraryDetail>("refresh_media_metadata", { entryId });
}

export function setMovieWatched(entryId: number, isWatched: boolean, watchedAt = "") {
  return invoke<MediaLibraryDetail>("set_movie_watched", { entryId, isWatched, watchedAt });
}

export function setEpisodeWatched(
  entryId: number,
  episodeId: number,
  isWatched: boolean,
  watchedAt = ""
) {
  return invoke<MediaLibraryDetail>("set_episode_watched", {
    entryId,
    episodeId,
    isWatched,
    watchedAt
  });
}

export function setSeasonWatched(entryId: number, seasonNumber: number, isWatched: boolean) {
  return invoke<MediaLibraryDetail>("set_season_watched", {
    entryId,
    seasonNumber,
    isWatched
  });
}

export function setSeriesWatched(entryId: number, isWatched: boolean) {
  return invoke<MediaLibraryDetail>("set_series_watched", { entryId, isWatched });
}

export function markEpisodesWatchedThrough(entryId: number, episodeId: number) {
  return invoke<MediaLibraryDetail>("mark_episodes_watched_through", { entryId, episodeId });
}

export function addMediaToWatchNext(entryId: number) {
  return invoke<MediaLibraryDetail>("add_media_to_watch_next", { entryId });
}

export function removeMediaFromWatchNext(entryId: number) {
  return invoke<MediaLibraryDetail>("remove_media_from_watch_next", { entryId });
}

export function moveMediaWatchNextItem(entryId: number, direction: "UP" | "DOWN") {
  return invoke<MediaLibraryDetail>("move_media_watch_next_item", { entryId, direction });
}

export function listMediaTags() {
  return invoke<MediaTag[]>("list_media_tags");
}

export function createMediaTag(name: string) {
  return invoke<MediaTag>("create_media_tag", { name });
}

export function deleteMediaTag(tagId: number) {
  return invoke<void>("delete_media_tag", { tagId });
}

export function setMediaEntryTags(entryId: number, tagIds: number[]) {
  return invoke<MediaLibraryDetail>("set_media_entry_tags", { entryId, tagIds });
}

export function createMediaStreamingLink(entryId: number, input: MediaStreamingLinkInput) {
  return invoke<MediaStreamingLink>("create_media_streaming_link", { entryId, input });
}

export function updateMediaStreamingLink(linkId: number, input: MediaStreamingLinkInput) {
  return invoke<MediaStreamingLink>("update_media_streaming_link", { linkId, input });
}

export function deleteMediaStreamingLink(linkId: number) {
  return invoke<void>("delete_media_streaming_link", { linkId });
}

export function setPreferredMediaStreamingLink(entryId: number, linkId: number) {
  return invoke<MediaStreamingLink[]>("set_preferred_media_streaming_link", { entryId, linkId });
}

export function openMediaStreamingTarget(entryId: number) {
  return invoke<void>("open_media_streaming_target", { entryId });
}

export function getMediaSettings() {
  return invoke<MediaSettings>("get_media_settings");
}

export function updateMediaSettings(input: Omit<MediaSettings, "updatedAt">) {
  return invoke<MediaSettings>("update_media_settings", { input });
}
