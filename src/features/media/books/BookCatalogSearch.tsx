import { useState } from "react";
import {
  addCatalogBookToLibrary,
  createManualBookEntry,
  searchBookCatalog
} from "../../../services/mediaBooks";
import type {
  BookCatalogResult,
  BookLibraryDetail,
  BookSearchMode,
  CreateManualBookInput
} from "../../../services/mediaBooks";
import { formatUnknownError } from "../../../utils/errors";
import { BookCover } from "./BookCover";

const emptyManualBook: CreateManualBookInput = {
  title: "",
  subtitle: "",
  authors: [],
  description: "",
  firstPublishYear: null,
  isbn10: "",
  isbn13: "",
  publisher: "",
  publishedDate: "",
  pageCount: null,
  audioDurationMinutes: null,
  language: "",
  coverUrl: "",
  preferredFormat: "UNKNOWN",
  libraryStatus: "PLANNED",
  notes: "",
  personalRating: null,
  isFavorite: false,
  priority: 0,
  ownershipStatus: "NONE",
  isReadNext: false,
  seriesName: "",
  seriesPosition: ""
};

export function BookCatalogSearch({
  onAdded,
  onStatus
}: {
  onAdded: (detail: BookLibraryDetail) => void;
  onStatus: (status: string, isError?: boolean) => void;
}) {
  const [query, setQuery] = useState("");
  const [mode, setMode] = useState<BookSearchMode>("GENERAL");
  const [results, setResults] = useState<BookCatalogResult[]>([]);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [addingKey, setAddingKey] = useState("");
  const [showManual, setShowManual] = useState(false);
  const [manual, setManual] = useState<CreateManualBookInput>(emptyManualBook);
  const [authorText, setAuthorText] = useState("");

  async function runSearch() {
    setIsSearching(true);
    setWarnings([]);
    try {
      const response = await searchBookCatalog(query, mode);
      setResults(response.results);
      setWarnings(response.warnings);
      onStatus(
        response.results.length === 0
          ? "No book catalogue results found."
          : `${response.results.length} book result(s).`
      );
    } catch (error) {
      setResults([]);
      onStatus(formatUnknownError(error), true);
    } finally {
      setIsSearching(false);
    }
  }

  async function addResult(result: BookCatalogResult) {
    if (result.alreadyInLibrary) return;
    const key = `${result.sourceKey}:${result.externalId}`;
    setAddingKey(key);
    try {
      const detail = await addCatalogBookToLibrary(result);
      setResults((current) =>
        current.map((item) =>
          item.sourceKey === result.sourceKey && item.externalId === result.externalId
            ? { ...item, alreadyInLibrary: true, existingEntryId: detail.item.entryId }
            : item
        )
      );
      onStatus(`${detail.item.title} added to the local book library.`);
      onAdded(detail);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setAddingKey("");
    }
  }

  async function addManualBook() {
    try {
      const detail = await createManualBookEntry({
        ...manual,
        authors: authorText
          .split(",")
          .map((author) => author.trim())
          .filter(Boolean)
      });
      setManual(emptyManualBook);
      setAuthorText("");
      setShowManual(false);
      onStatus(`${detail.item.title} added as a manual book.`);
      onAdded(detail);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    }
  }

  function setManualField<K extends keyof CreateManualBookInput>(
    field: K,
    value: CreateManualBookInput[K]
  ) {
    setManual((current) => ({ ...current, [field]: value }));
  }

  return (
    <div className="media-catalog-view book-catalog-view">
      <div className="media-toolbar book-search-toolbar">
        <select
          aria-label="Book search mode"
          className="text-input"
          onChange={(event) => setMode(event.target.value as BookSearchMode)}
          value={mode}
        >
          <option value="GENERAL">General</option>
          <option value="TITLE">Title</option>
          <option value="AUTHOR">Author</option>
          <option value="ISBN">ISBN</option>
        </select>
        <input
          aria-label="Search book catalogues"
          className="text-input"
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") void runSearch();
          }}
          placeholder={mode === "ISBN" ? "ISBN-10 or ISBN-13" : "Search books"}
          value={query}
        />
        <button className="primary-button" disabled={isSearching} onClick={() => void runSearch()} type="button">
          {isSearching ? "Searching..." : "Search"}
        </button>
        <button className="ghost-button" onClick={() => setShowManual((value) => !value)} type="button">
          {showManual ? "Close Manual Entry" : "Add Manual Book"}
        </button>
      </div>

      {showManual && (
        <section className="media-form-card" aria-label="Manual book entry">
          <div className="media-form-grid">
            <label><span>Title</span><input className="text-input" value={manual.title} onChange={(event) => setManualField("title", event.target.value)} /></label>
            <label><span>Subtitle</span><input className="text-input" value={manual.subtitle} onChange={(event) => setManualField("subtitle", event.target.value)} /></label>
            <label><span>Authors (comma-separated)</span><input className="text-input" value={authorText} onChange={(event) => setAuthorText(event.target.value)} /></label>
            <label><span>First publish year</span><input className="text-input" min="0" type="number" value={manual.firstPublishYear ?? ""} onChange={(event) => setManualField("firstPublishYear", event.target.value ? Number(event.target.value) : null)} /></label>
            <label><span>ISBN-13</span><input className="text-input" value={manual.isbn13} onChange={(event) => setManualField("isbn13", event.target.value)} /></label>
            <label><span>ISBN-10</span><input className="text-input" value={manual.isbn10} onChange={(event) => setManualField("isbn10", event.target.value)} /></label>
            <label><span>Publisher</span><input className="text-input" value={manual.publisher} onChange={(event) => setManualField("publisher", event.target.value)} /></label>
            <label><span>Published date</span><input className="text-input" value={manual.publishedDate} onChange={(event) => setManualField("publishedDate", event.target.value)} /></label>
            <label><span>Page count</span><input className="text-input" min="1" type="number" value={manual.pageCount ?? ""} onChange={(event) => setManualField("pageCount", event.target.value ? Number(event.target.value) : null)} /></label>
            <label><span>Audio minutes</span><input className="text-input" min="1" type="number" value={manual.audioDurationMinutes ?? ""} onChange={(event) => setManualField("audioDurationMinutes", event.target.value ? Number(event.target.value) : null)} /></label>
            <label><span>Language</span><input className="text-input" value={manual.language} onChange={(event) => setManualField("language", event.target.value)} /></label>
            <label><span>Preferred format</span><select className="text-input" value={manual.preferredFormat} onChange={(event) => setManualField("preferredFormat", event.target.value as CreateManualBookInput["preferredFormat"])}><BookFormatOptions /></select></label>
            <label><span>Status</span><select className="text-input" value={manual.libraryStatus} onChange={(event) => setManualField("libraryStatus", event.target.value as CreateManualBookInput["libraryStatus"])}><option value="PLANNED">Plan to Read</option><option value="WATCHING">Reading</option><option value="COMPLETED">Read</option><option value="ON_HOLD">On Hold</option><option value="DROPPED">Did Not Finish</option></select></label>
            <label><span>Ownership</span><select className="text-input" value={manual.ownershipStatus} onChange={(event) => setManualField("ownershipStatus", event.target.value as CreateManualBookInput["ownershipStatus"])}><option value="NONE">None</option><option value="OWNED">Owned</option><option value="BORROWED">Borrowed</option><option value="WISHLIST">Wishlist</option></select></label>
            <label><span>Personal rating (1–10)</span><input className="text-input" min="1" max="10" step="0.5" type="number" value={manual.personalRating ?? ""} onChange={(event) => setManualField("personalRating", event.target.value ? Number(event.target.value) : null)} /></label>
            <label><span>Priority</span><input className="text-input" type="number" value={manual.priority} onChange={(event) => setManualField("priority", Number(event.target.value))} /></label>
            <label><span>Series name</span><input className="text-input" value={manual.seriesName} onChange={(event) => setManualField("seriesName", event.target.value)} /></label>
            <label><span>Series position</span><input className="text-input" value={manual.seriesPosition} onChange={(event) => setManualField("seriesPosition", event.target.value)} /></label>
            <label><span>Cover URL</span><input className="text-input" placeholder="https://" value={manual.coverUrl} onChange={(event) => setManualField("coverUrl", event.target.value)} /></label>
            <label className="media-check-label"><input checked={manual.isFavorite} type="checkbox" onChange={(event) => setManualField("isFavorite", event.target.checked)} />Favourite</label>
            <label className="media-check-label"><input checked={manual.isReadNext} type="checkbox" onChange={(event) => setManualField("isReadNext", event.target.checked)} />Add to Read Next</label>
          </div>
          <label><span>Description</span><textarea className="body-input compact" value={manual.description} onChange={(event) => setManualField("description", event.target.value)} /></label>
          <label><span>Personal notes</span><textarea className="body-input compact" value={manual.notes} onChange={(event) => setManualField("notes", event.target.value)} /></label>
          <button className="primary-button" onClick={() => void addManualBook()} type="button">Add Manual Book</button>
        </section>
      )}

      {warnings.map((warning) => <p className="media-inline-warning" key={warning}>{warning}</p>)}
      <div className="media-catalog-results">
        {results.map((result) => {
          const key = `${result.sourceKey}:${result.externalId}`;
          return (
            <article className="media-catalog-card" key={key}>
              <BookCover title={result.title} url={result.coverUrl} />
              <div>
                <div className="media-card-title-row"><h4>{result.title}</h4><span>Book</span></div>
                <p className="media-card-meta">{[result.subtitle, result.authors.join(", "), result.publishedDate, result.publisher, result.format].filter(Boolean).join(" · ")}</p>
                <p className="media-hint">{result.isbn13 || result.isbn10 || "No ISBN reported"}</p>
                <p className="media-card-overview">{result.description || "No catalogue description is available."}</p>
                <div className="media-row-badges">{result.providerBadges.map((badge) => <span key={badge}>{badge}</span>)}</div>
              </div>
              <button
                className={result.alreadyInLibrary ? "ghost-button" : "primary-button"}
                disabled={result.alreadyInLibrary || Boolean(addingKey)}
                onClick={() => void addResult(result)}
                type="button"
              >
                {result.alreadyInLibrary ? "Already in library" : addingKey === key ? "Adding..." : "Add"}
              </button>
            </article>
          );
        })}
      </div>
    </div>
  );
}

function BookFormatOptions() {
  return <>{["UNKNOWN", "HARDCOVER", "PAPERBACK", "EBOOK", "AUDIOBOOK", "OTHER"].map((value) => <option key={value} value={value}>{value.charAt(0) + value.slice(1).toLowerCase()}</option>)}</>;
}
