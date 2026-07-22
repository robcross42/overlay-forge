import { invoke } from "@tauri-apps/api/core";
import type { MediaLibraryStatus, MediaTag } from "./media";

export type BookSearchMode = "GENERAL" | "TITLE" | "AUTHOR" | "ISBN";
export type BookFormat = "HARDCOVER" | "PAPERBACK" | "EBOOK" | "AUDIOBOOK" | "OTHER" | "UNKNOWN";
export type BookProgressUnit = "PAGE" | "PERCENT" | "MINUTE" | "CHAPTER";
export type BookOwnershipStatus = "NONE" | "OWNED" | "BORROWED" | "WISHLIST";

export type BookSourceIdentity = {
  sourceKey: string;
  entityType: string;
  externalId: string;
  sourceUrl: string;
};

export type BookCatalogResult = {
  sourceKey: string;
  externalId: string;
  title: string;
  subtitle: string;
  authors: string[];
  publishedDate: string;
  publisher: string;
  isbn10: string;
  isbn13: string;
  pageCount: number | null;
  language: string;
  format: BookFormat;
  coverUrl: string;
  description: string;
  workKey: string;
  editionKey: string;
  infoUrl: string;
  previewUrl: string;
  alreadyInLibrary: boolean;
  existingEntryId: number | null;
  matchBasis: string;
  providerBadges: string[];
  sourceIdentities: BookSourceIdentity[];
};

export type BookCatalogSearchResponse = {
  results: BookCatalogResult[];
  warnings: string[];
};

export type BookProgressSummary = {
  progressUnit: BookProgressUnit;
  progressValue: number;
  progressTotalOverride: number | null;
  effectiveTotal: number | null;
  progressPercent: number | null;
  currentChapter: string;
  preferredEditionId: number | null;
  ownershipStatus: BookOwnershipStatus;
  preferredFormat: BookFormat;
  readingQueuePosition: number | null;
  lastProgressAt: string;
};

export type BookMediaSummary = {
  progress: BookProgressSummary;
  primaryAuthorText: string;
  primarySeriesText: string;
};

export type BookLibraryItem = {
  entryId: number;
  mediaTitleId: number;
  title: string;
  subtitle: string;
  primaryAuthorText: string;
  description: string;
  firstPublishYear: number | null;
  coverUrl: string;
  libraryStatus: MediaLibraryStatus;
  statusLabel: string;
  isFavorite: boolean;
  personalRating: number | null;
  notes: string;
  priority: number;
  addedAt: string;
  startedAt: string;
  completedAt: string;
  metadataRefreshedAt: string;
  progress: BookProgressSummary;
};

export type BookEdition = {
  id: number;
  title: string;
  subtitle: string;
  format: BookFormat;
  isbn10: string;
  isbn13: string;
  publisher: string;
  publishedDate: string;
  language: string;
  pageCount: number | null;
  audioDurationMinutes: number | null;
  coverUrl: string;
  isEbook: boolean;
  accessViewability: string;
  isPresentInSource: boolean;
  sourceKeys: string[];
  isUserOwned: boolean;
};

export type BookAuthor = { id: number; name: string; role: string; position: number };
export type BookLink = {
  id: number;
  editionId: number | null;
  sourceKey: string;
  linkType: "INFO" | "PREVIEW" | "READ" | "BORROW" | "BUY" | "OTHER";
  url: string;
  regionCode: string;
  isUserOwned: boolean;
  isPreferred: boolean;
  refreshedAt: string;
};
export type BookSeries = {
  id: number;
  name: string;
  positionText: string;
  positionSort: number | null;
  isPrimary: boolean;
  sourceKey: string;
  isUserOverride: boolean;
};
export type BookSource = {
  sourceKey: string;
  entityType: string;
  externalId: string;
  sourceUrl: string;
  refreshedAt: string;
  lastRefreshStatus: string;
  lastRefreshError: string;
};

export type BookLibraryDetail = {
  item: BookLibraryItem;
  editions: BookEdition[];
  authors: BookAuthor[];
  links: BookLink[];
  series: BookSeries[];
  sources: BookSource[];
  subjects: string[];
  communityRating: number | null;
  communityRatingCount: number | null;
  providerWarnings: string[];
  tags: MediaTag[];
};

export type BookLibraryFilter = {
  query: string;
  libraryStatus: MediaLibraryStatus | null;
  ownershipStatus: BookOwnershipStatus | null;
  preferredFormat: BookFormat | null;
  authorQuery: string;
  tagId: number | null;
  isFavorite: boolean | null;
  isReadNext: boolean | null;
  sort:
    | "TITLE"
    | "AUTHOR"
    | "RECENTLY_ADDED"
    | "RECENTLY_READ"
    | "PROGRESS"
    | "PUBLICATION_DATE"
    | "READ_NEXT"
    | "PRIORITY";
  sortDirection: "ASC" | "DESC";
};

export type CreateManualBookInput = {
  title: string;
  subtitle: string;
  authors: string[];
  description: string;
  firstPublishYear: number | null;
  isbn10: string;
  isbn13: string;
  publisher: string;
  publishedDate: string;
  pageCount: number | null;
  audioDurationMinutes: number | null;
  language: string;
  coverUrl: string;
  preferredFormat: BookFormat;
  libraryStatus: MediaLibraryStatus;
  notes: string;
  personalRating: number | null;
  isFavorite: boolean;
  priority: number;
  ownershipStatus: BookOwnershipStatus;
  isReadNext: boolean;
  seriesName: string;
  seriesPosition: string;
};

export type ManualBookEditionInput = {
  title: string;
  subtitle: string;
  format: BookFormat;
  isbn10: string;
  isbn13: string;
  publisher: string;
  publishedDate: string;
  language: string;
  pageCount: number | null;
  audioDurationMinutes: number | null;
  coverUrl: string;
};

export type BookLinkInput = {
  editionId: number | null;
  linkType: BookLink["linkType"];
  url: string;
  regionCode: string;
  isPreferred: boolean;
};

export type BookProviderStatus = {
  googleBooksConfigured: boolean;
  openLibraryContactConfigured: boolean;
  hardcoverConfigured: boolean;
};

export const defaultBookFilter: BookLibraryFilter = {
  query: "",
  libraryStatus: null,
  ownershipStatus: null,
  preferredFormat: null,
  authorQuery: "",
  tagId: null,
  isFavorite: null,
  isReadNext: null,
  sort: "RECENTLY_ADDED",
  sortDirection: "DESC"
};

export function searchBookCatalog(query: string, mode: BookSearchMode, limit = 20) {
  return invoke<BookCatalogSearchResponse>("search_book_catalog", { input: { query, mode, limit } });
}

export function addCatalogBookToLibrary(result: Pick<BookCatalogResult, "sourceKey" | "externalId">) {
  return invoke<BookLibraryDetail>("add_catalog_book_to_library", {
    input: { ...result, libraryStatus: "PLANNED" }
  });
}

export function createManualBookEntry(input: CreateManualBookInput) {
  return invoke<BookLibraryDetail>("create_manual_book_entry", { input });
}

export function listBookLibrary(filter: BookLibraryFilter = defaultBookFilter) {
  return invoke<BookLibraryItem[]>("list_book_library", { filter });
}

export function getBookLibraryDetail(entryId: number) {
  return invoke<BookLibraryDetail>("get_book_library_detail", { entryId });
}

export function refreshBookMetadata(entryId: number) {
  return invoke<BookLibraryDetail>("refresh_book_metadata", { entryId });
}

export function updateBookReaderState(
  entryId: number,
  input: {
    ownershipStatus: BookOwnershipStatus;
    preferredFormat: BookFormat;
  }
) {
  return invoke<BookLibraryDetail>("update_book_reader_state", { entryId, input });
}

export function setBookPreferredEdition(entryId: number, editionId: number) {
  return invoke<BookLibraryDetail>("set_book_preferred_edition", { entryId, editionId });
}

export function setBookProgress(
  entryId: number,
  input: {
    progressUnit: BookProgressUnit;
    progressValue: number;
    progressTotalOverride: number | null;
    currentChapter: string;
  }
) {
  return invoke<BookLibraryDetail>("set_book_progress", { entryId, input });
}

export const markBookRead = (entryId: number) =>
  invoke<BookLibraryDetail>("mark_book_read", { entryId });
export const resetBookProgress = (entryId: number) =>
  invoke<BookLibraryDetail>("reset_book_progress", { entryId });
export const addBookToReadNext = (entryId: number) =>
  invoke<BookLibraryDetail>("add_book_to_read_next", { entryId });
export const removeBookFromReadNext = (entryId: number) =>
  invoke<BookLibraryDetail>("remove_book_from_read_next", { entryId });
export const moveBookReadNextItem = (entryId: number, direction: "UP" | "DOWN") =>
  invoke<BookLibraryDetail>("move_book_read_next_item", { entryId, direction });

export const createManualBookEdition = (entryId: number, input: ManualBookEditionInput) =>
  invoke<BookLibraryDetail>("create_manual_book_edition", { entryId, input });
export const updateManualBookEdition = (
  entryId: number,
  editionId: number,
  input: ManualBookEditionInput
) => invoke<BookLibraryDetail>("update_manual_book_edition", { entryId, editionId, input });
export const deleteManualBookEdition = (entryId: number, editionId: number) =>
  invoke<BookLibraryDetail>("delete_manual_book_edition", { entryId, editionId });

export const createBookLink = (entryId: number, input: BookLinkInput) =>
  invoke<BookLibraryDetail>("create_book_link", { entryId, input });
export const updateBookLink = (entryId: number, linkId: number, input: BookLinkInput) =>
  invoke<BookLibraryDetail>("update_book_link", { entryId, linkId, input });
export const deleteBookLink = (entryId: number, linkId: number) =>
  invoke<BookLibraryDetail>("delete_book_link", { entryId, linkId });
export const setPreferredBookLink = (entryId: number, linkId: number) =>
  invoke<BookLibraryDetail>("set_preferred_book_link", { entryId, linkId });
export const openBookLink = (entryId: number, linkId: number) =>
  invoke<void>("open_book_link", { entryId, linkId });

export const setBookSeriesOverride = (
  entryId: number,
  input: { name: string; positionText: string; positionSort: number | null; isPrimary: boolean }
) => invoke<BookLibraryDetail>("set_book_series_override", { entryId, input });
export const clearBookSeriesOverride = (entryId: number, memberId: number) =>
  invoke<BookLibraryDetail>("clear_book_series_override", { entryId, memberId });

export function getBookProviderStatus() {
  return invoke<BookProviderStatus>("get_book_provider_status");
}

export function openBookDataSource(sourceKey: "google_books" | "open_library" | "hardcover") {
  return invoke<void>("open_book_data_source", { sourceKey });
}
