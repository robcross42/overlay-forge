import { useEffect, useState } from "react";
import {
  addCatalogMediaToLibrary,
  createManualMediaEntry,
  searchMediaCatalog
} from "../../services/media";
import type {
  ManualMediaInput,
  MediaCatalogSearchResult,
  MediaContentType,
  MediaLibraryDetail,
  MediaLibraryStatus
} from "../../services/media";
import { formatUnknownError } from "../../utils/errors";
import { resolveMediaArtwork } from "./books/bookPresentation";

const emptyManualEntry: ManualMediaInput = {
  contentType: "MOVIE",
  title: "",
  releaseDate: "",
  overview: "",
  runtimeMinutes: null,
  libraryStatus: "PLANNED",
  notes: "",
  personalRating: null,
  isFavorite: false,
  priority: 0,
  addToWatchNext: false
};

type MediaCatalogSearchProps = {
  onAdded: (detail: MediaLibraryDetail) => void;
  onStatus: (status: string, isError?: boolean) => void;
};

export function MediaCatalogSearch({ onAdded, onStatus }: MediaCatalogSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<MediaCatalogSearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [searchMessage, setSearchMessage] = useState(
    "Search TMDB for a movie or episodic series."
  );
  const [addingId, setAddingId] = useState<number | null>(null);
  const [showManual, setShowManual] = useState(false);
  const [manual, setManual] = useState<ManualMediaInput>(emptyManualEntry);

  useEffect(() => {
    const trimmed = query.trim();
    if (trimmed.length < 2) {
      setResults([]);
      setSearchMessage("Enter at least two characters to search the catalogue.");
      return;
    }

    const timeout = window.setTimeout(() => {
      setIsSearching(true);
      setSearchMessage("Searching TMDB...");
      searchMediaCatalog(trimmed)
        .then((nextResults) => {
          setResults(nextResults);
          setSearchMessage(
            nextResults.length === 0
              ? "No movie or series results found."
              : `${nextResults.length} catalogue result(s)`
          );
        })
        .catch((error) => {
          setResults([]);
          setSearchMessage(formatUnknownError(error));
        })
        .finally(() => setIsSearching(false));
    }, 500);

    return () => window.clearTimeout(timeout);
  }, [query]);

  async function addResult(result: MediaCatalogSearchResult) {
    if (result.alreadyInLibrary) {
      return;
    }
    setAddingId(result.externalId);
    onStatus(`Importing ${result.title}...`);
    try {
      const detail = await addCatalogMediaToLibrary(result);
      setResults((current) =>
        current.map((item) =>
          item.externalId === result.externalId &&
          item.externalMediaType === result.externalMediaType
            ? { ...item, alreadyInLibrary: true, existingEntryId: detail.entry.id }
            : item
        )
      );
      onStatus(`${result.title} added to the local library.`);
      onAdded(detail);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setAddingId(null);
    }
  }

  async function addManualEntry() {
    if (!manual.title.trim()) {
      onStatus("A manual title is required.", true);
      return;
    }
    try {
      const detail = await createManualMediaEntry(manual);
      setManual(emptyManualEntry);
      setShowManual(false);
      onStatus(`${detail.entry.title} added as a manual entry.`);
      onAdded(detail);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    }
  }

  function setManualField<K extends keyof ManualMediaInput>(
    field: K,
    value: ManualMediaInput[K]
  ) {
    setManual((current) => ({ ...current, [field]: value }));
  }

  return (
    <div className="media-catalog-view">
      <div className="media-toolbar">
        <input
          aria-label="Search the TMDB catalogue"
          className="text-input"
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search movies and series"
          value={query}
        />
        <button
          className="ghost-button"
          onClick={() => setShowManual((current) => !current)}
          type="button"
        >
          {showManual ? "Close Manual Entry" : "Add Manual Entry"}
        </button>
      </div>

      {showManual && (
        <section className="media-form-card" aria-label="Manual media entry">
          <div className="media-form-grid">
            <label>
              <span>Media type</span>
              <select
                className="text-input"
                onChange={(event) =>
                  setManualField("contentType", event.target.value as MediaContentType)
                }
                value={manual.contentType}
              >
                <option value="MOVIE">Movie</option>
                <option value="SERIES">Series</option>
              </select>
            </label>
            <label>
              <span>Title</span>
              <input
                className="text-input"
                onChange={(event) => setManualField("title", event.target.value)}
                value={manual.title}
              />
            </label>
            <label>
              <span>{manual.contentType === "MOVIE" ? "Release date" : "First air date"}</span>
              <input
                className="text-input"
                onChange={(event) => setManualField("releaseDate", event.target.value)}
                type="date"
                value={manual.releaseDate}
              />
            </label>
            <label>
              <span>Runtime minutes</span>
              <input
                className="text-input"
                min="1"
                onChange={(event) =>
                  setManualField(
                    "runtimeMinutes",
                    event.target.value ? Number(event.target.value) : null
                  )
                }
                type="number"
                value={manual.runtimeMinutes ?? ""}
              />
            </label>
            <label>
              <span>Initial status</span>
              <select
                className="text-input"
                onChange={(event) =>
                  setManualField("libraryStatus", event.target.value as MediaLibraryStatus)
                }
                value={manual.libraryStatus}
              >
                <StatusOptions />
              </select>
            </label>
            <label>
              <span>Rating (1-10)</span>
              <input
                className="text-input"
                max="10"
                min="1"
                onChange={(event) =>
                  setManualField(
                    "personalRating",
                    event.target.value ? Number(event.target.value) : null
                  )
                }
                step="0.5"
                type="number"
                value={manual.personalRating ?? ""}
              />
            </label>
          </div>
          <label>
            <span>Overview</span>
            <textarea
              className="body-input compact"
              onChange={(event) => setManualField("overview", event.target.value)}
              value={manual.overview}
            />
          </label>
          <label>
            <span>Personal notes</span>
            <textarea
              className="body-input compact"
              onChange={(event) => setManualField("notes", event.target.value)}
              value={manual.notes}
            />
          </label>
          <div className="media-check-row">
            <label>
              <input
                checked={manual.isFavorite}
                onChange={(event) => setManualField("isFavorite", event.target.checked)}
                type="checkbox"
              />
              Favourite
            </label>
            <label>
              <input
                checked={manual.addToWatchNext}
                onChange={(event) => setManualField("addToWatchNext", event.target.checked)}
                type="checkbox"
              />
              Add to Watch Next
            </label>
          </div>
          <button className="primary-button" onClick={() => void addManualEntry()} type="button">
            Add Manual Entry
          </button>
        </section>
      )}

      <p className={searchMessage.includes("Credential") ? "media-inline-error" : "media-hint"}>
        {isSearching ? "Searching..." : searchMessage}
      </p>

      <div className="media-catalog-results">
        {results.map((result) => (
          <article
            className="media-catalog-card"
            key={`${result.externalMediaType}-${result.externalId}`}
          >
            <Poster path={result.posterPath} title={result.title} />
            <div>
              <div className="media-card-title-row">
                <h4>{result.title}</h4>
                <span>{result.contentType === "MOVIE" ? "Movie" : "Series"}</span>
              </div>
              <p className="media-card-meta">{result.releaseDate || "Date unavailable"}</p>
              <p className="media-card-overview">
                {result.overview || "No catalogue overview is available."}
              </p>
            </div>
            <button
              className={result.alreadyInLibrary ? "ghost-button" : "primary-button"}
              disabled={result.alreadyInLibrary || addingId !== null}
              onClick={() => void addResult(result)}
              type="button"
            >
              {result.alreadyInLibrary
                ? "Already in library"
                : addingId === result.externalId
                  ? "Adding..."
                  : "Add"}
            </button>
          </article>
        ))}
      </div>
    </div>
  );
}

export function Poster({
  path,
  title,
  contentType = "MOVIE"
}: {
  path: string;
  title: string;
  contentType?: MediaContentType;
}) {
  const source = resolveMediaArtwork(path, contentType);
  const [failedSource, setFailedSource] = useState("");
  if (!source || failedSource === source) {
    return <div className="media-poster-placeholder">{title.slice(0, 1).toUpperCase()}</div>;
  }
  return (
    <img
      alt={`${title} poster`}
      className="media-poster"
      loading="lazy"
      onError={() => setFailedSource(source)}
      src={source}
    />
  );
}

export function StatusOptions() {
  return (
    <>
      <option value="PLANNED">Plan to Watch</option>
      <option value="WATCHING">Watching</option>
      <option value="COMPLETED">Completed</option>
      <option value="ON_HOLD">On Hold</option>
      <option value="DROPPED">Dropped</option>
    </>
  );
}
