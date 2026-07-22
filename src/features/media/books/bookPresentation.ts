import type { BookLibraryItem, BookProgressSummary } from "../../../services/mediaBooks";
import type { MediaContentType, MediaLibraryStatus } from "../../../services/media";

const allowedBookArtworkHosts = new Set([
  "books.google.com",
  "books.googleusercontent.com",
  "covers.openlibrary.org"
]);

export function resolveMediaArtwork(path: string, contentType: MediaContentType): string {
  if (!path) return "";
  if (contentType !== "BOOK") return `https://image.tmdb.org/t/p/w342${path}`;
  try {
    const url = new URL(path);
    return url.protocol === "https:" && allowedBookArtworkHosts.has(url.hostname) ? url.toString() : "";
  } catch {
    return "";
  }
}

export function mediaStatusLabel(status: MediaLibraryStatus, contentType: MediaContentType) {
  if (contentType === "BOOK") {
    return {
      PLANNED: "Plan to Read",
      WATCHING: "Reading",
      COMPLETED: "Read",
      ON_HOLD: "On Hold",
      DROPPED: "Did Not Finish"
    }[status];
  }
  return {
    PLANNED: "Plan to Watch",
    WATCHING: "Watching",
    COMPLETED: "Completed",
    ON_HOLD: "On Hold",
    DROPPED: "Dropped"
  }[status];
}

export function bookProgressLabel(progress: BookProgressSummary) {
  if (progress.progressUnit === "CHAPTER") {
    return progress.currentChapter ? `Chapter: ${progress.currentChapter}` : "No chapter recorded";
  }
  const unit = { PAGE: "pages", PERCENT: "%", MINUTE: "minutes", CHAPTER: "" }[
    progress.progressUnit
  ];
  const total = progress.effectiveTotal === null ? "" : ` / ${progress.effectiveTotal}`;
  return `${progress.progressValue}${total} ${unit}`.trim();
}

export function bookSubtitle(item: BookLibraryItem) {
  return [item.primaryAuthorText, item.firstPublishYear?.toString()].filter(Boolean).join(" · ");
}
