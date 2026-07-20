import { useCallback, useEffect, useMemo, useState } from "react";
import {
  defaultMediaFilter,
  deleteMediaTag,
  getMediaLibraryDetail,
  getMediaSettings,
  listMediaLibrary,
  listMediaTags,
  updateMediaSettings
} from "../../services/media";
import type {
  MediaContentType,
  MediaLibraryDetail,
  MediaLibraryFilter,
  MediaLibraryStatus,
  MediaLibrarySummary,
  MediaSettings,
  MediaSort,
  MediaTag
} from "../../services/media";
import { formatUnknownError } from "../../utils/errors";
import { MediaCatalogSearch, Poster, StatusOptions } from "./MediaCatalogSearch";
import { MediaDetail } from "./MediaDetail";

type MediaView = "home" | "library" | "catalogue" | "settings";

export function MediaLibrary() {
  const [view, setView] = useState<MediaView>("home");
  const [status, setStatus] = useState("Loading local media library...");
  const [statusIsError, setStatusIsError] = useState(false);
  const [homeItems, setHomeItems] = useState<MediaLibrarySummary[]>([]);
  const [libraryItems, setLibraryItems] = useState<MediaLibrarySummary[]>([]);
  const [filter, setFilter] = useState<MediaLibraryFilter>(defaultMediaFilter);
  const [tags, setTags] = useState<MediaTag[]>([]);
  const [settings, setSettings] = useState<MediaSettings | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<MediaLibraryDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const reportStatus = useCallback((message: string, isError = false) => {
    setStatus(message);
    setStatusIsError(isError);
  }, []);

  const loadHome = useCallback(async () => {
    const items = await listMediaLibrary(defaultMediaFilter);
    setHomeItems(items);
    return items;
  }, []);

  const loadTagsAndSettings = useCallback(async () => {
    const [nextTags, nextSettings] = await Promise.all([listMediaTags(), getMediaSettings()]);
    setTags(nextTags);
    setSettings(nextSettings);
  }, []);

  useEffect(() => {
    Promise.all([loadHome(), loadTagsAndSettings()])
      .then(([items]) => {
        setLibraryItems(items);
        reportStatus(
          items.length === 0 ? "Your local media library is empty." : `${items.length} local title(s)`
        );
      })
      .catch((error) => reportStatus(formatUnknownError(error), true))
      .finally(() => setIsLoading(false));
  }, [loadHome, loadTagsAndSettings, reportStatus]);

  useEffect(() => {
    if (view !== "library") {
      return;
    }
    const timeout = window.setTimeout(() => {
      setIsLoading(true);
      listMediaLibrary(filter)
        .then((items) => {
          setLibraryItems(items);
          reportStatus(
            items.length === 0 ? "No local titles match these filters." : `${items.length} match(es)`
          );
        })
        .catch((error) => reportStatus(formatUnknownError(error), true))
        .finally(() => setIsLoading(false));
    }, 250);
    return () => window.clearTimeout(timeout);
  }, [filter, reportStatus, view]);

  async function openDetail(entryId: number) {
    setIsLoading(true);
    try {
      const detail = await getMediaLibraryDetail(entryId);
      setSelectedDetail(detail);
      reportStatus(`${detail.entry.title} selected.`);
    } catch (error) {
      reportStatus(formatUnknownError(error), true);
    } finally {
      setIsLoading(false);
    }
  }

  async function refreshLists() {
    try {
      const [nextHome, nextLibrary] = await Promise.all([
        listMediaLibrary(defaultMediaFilter),
        listMediaLibrary(filter)
      ]);
      setHomeItems(nextHome);
      setLibraryItems(nextLibrary);
    } catch (error) {
      reportStatus(formatUnknownError(error), true);
    }
  }

  function handleDetailChanged(detail: MediaLibraryDetail) {
    setSelectedDetail(detail);
    void refreshLists();
  }

  function handleAdded(detail: MediaLibraryDetail) {
    setSelectedDetail(detail);
    void refreshLists();
  }

  function handleDeleted(entryId: number) {
    setSelectedDetail(null);
    setHomeItems((current) => current.filter((item) => item.id !== entryId));
    setLibraryItems((current) => current.filter((item) => item.id !== entryId));
  }

  return (
    <section className="feature-panel media-library-panel">
      <div className="panel-heading media-panel-heading">
        <div>
          <p>Local-first watch tracking</p>
          <h3>Media Library</h3>
        </div>
        <span className={statusIsError ? "save-pill save-pill-error" : "save-pill"}>
          {isLoading ? "Loading..." : status}
        </span>
      </div>

      {selectedDetail ? (
        <MediaDetail
          allTags={tags}
          detail={selectedDetail}
          onBack={() => setSelectedDetail(null)}
          onChanged={handleDetailChanged}
          onDeleted={handleDeleted}
          onStatus={reportStatus}
          onTagsChanged={setTags}
        />
      ) : (
        <div className="media-module-body">
          <nav className="workspace-tabs media-tabs" aria-label="Media Library sections">
            {(
              [
                ["home", "Home"],
                ["library", "Library"],
                ["catalogue", "Catalogue Search"],
                ["settings", "Settings"]
              ] as Array<[MediaView, string]>
            ).map(([id, label]) => (
              <button
                className={view === id ? "workspace-tab active" : "workspace-tab"}
                key={id}
                onClick={() => setView(id)}
                type="button"
              >
                {label}
              </button>
            ))}
          </nav>

          <div className="media-view-scroll">
            {view === "home" && (
              <MediaHome
                items={homeItems}
                onOpen={openDetail}
                onSearch={() => setView("catalogue")}
              />
            )}
            {view === "library" && (
              <MediaLibraryView
                filter={filter}
                isLoading={isLoading}
                items={libraryItems}
                onFilter={setFilter}
                onOpen={openDetail}
                onSearch={() => setView("catalogue")}
                tags={tags}
              />
            )}
            {view === "catalogue" && (
              <MediaCatalogSearch onAdded={handleAdded} onStatus={reportStatus} />
            )}
            {view === "settings" && settings && (
              <MediaSettingsView
                onSettings={setSettings}
                onStatus={reportStatus}
                onTags={setTags}
                settings={settings}
                tags={tags}
              />
            )}
          </div>
        </div>
      )}
    </section>
  );
}

function MediaHome({
  items,
  onOpen,
  onSearch
}: {
  items: MediaLibrarySummary[];
  onOpen: (entryId: number) => void;
  onSearch: () => void;
}) {
  const continueWatching = items
    .filter((item) => item.contentType === "SERIES" && item.libraryStatus === "WATCHING")
    .slice(0, 6);
  const watchNext = [...items]
    .filter((item) => item.queuePosition !== null)
    .sort((left, right) => (left.queuePosition ?? 0) - (right.queuePosition ?? 0))
    .slice(0, 8);
  const recentlyWatched = [...items]
    .filter((item) => item.lastWatchedAt)
    .sort((left, right) => right.lastWatchedAt.localeCompare(left.lastWatchedAt))
    .slice(0, 6);
  const recentlyAdded = [...items]
    .sort((left, right) => right.addedAt.localeCompare(left.addedAt))
    .slice(0, 6);

  if (items.length === 0) {
    return (
      <div className="empty-editor-state media-empty-state">
        <p>Your local media library is ready.</p>
        <button className="primary-button" onClick={onSearch} type="button">
          Search the catalogue
        </button>
      </div>
    );
  }

  return (
    <div className="media-home">
      <MediaShelf
        empty="No series are currently in Watching."
        items={continueWatching}
        onOpen={onOpen}
        title="Continue Watching"
      />
      <MediaShelf
        empty="Add a title to Watch Next from its detail page."
        items={watchNext}
        onOpen={onOpen}
        title="Watch Next"
      />
      <MediaShelf
        empty="Nothing has been watched yet."
        items={recentlyWatched}
        onOpen={onOpen}
        title="Recently Watched"
      />
      <MediaShelf items={recentlyAdded} onOpen={onOpen} title="Recently Added" />
    </div>
  );
}

function MediaShelf({
  title,
  items,
  empty,
  onOpen
}: {
  title: string;
  items: MediaLibrarySummary[];
  empty?: string;
  onOpen: (entryId: number) => void;
}) {
  return (
    <section className="media-shelf">
      <h4>{title}</h4>
      {items.length === 0 ? (
        <p className="media-hint">{empty}</p>
      ) : (
        <div className="media-row-list">
          {items.map((item) => (
            <MediaRow item={item} key={item.id} onOpen={onOpen} />
          ))}
        </div>
      )}
    </section>
  );
}

function MediaLibraryView({
  filter,
  items,
  tags,
  isLoading,
  onFilter,
  onOpen,
  onSearch
}: {
  filter: MediaLibraryFilter;
  items: MediaLibrarySummary[];
  tags: MediaTag[];
  isLoading: boolean;
  onFilter: (filter: MediaLibraryFilter) => void;
  onOpen: (entryId: number) => void;
  onSearch: () => void;
}) {
  function update<K extends keyof MediaLibraryFilter>(
    field: K,
    value: MediaLibraryFilter[K]
  ) {
    onFilter({ ...filter, [field]: value, offset: 0 });
  }

  return (
    <div className="media-library-view">
      <div className="media-filter-grid">
        <input
          aria-label="Search local media library"
          className="text-input media-filter-search"
          onChange={(event) => update("query", event.target.value)}
          placeholder="Search local titles, notes, and overviews"
          value={filter.query}
        />
        <select
          aria-label="Media type filter"
          className="text-input"
          onChange={(event) =>
            update(
              "contentType",
              (event.target.value || null) as MediaContentType | null
            )
          }
          value={filter.contentType ?? ""}
        >
          <option value="">All types</option>
          <option value="MOVIE">Movies</option>
          <option value="SERIES">Series</option>
        </select>
        <select
          aria-label="Library status filter"
          className="text-input"
          onChange={(event) =>
            update(
              "libraryStatus",
              (event.target.value || null) as MediaLibraryStatus | null
            )
          }
          value={filter.libraryStatus ?? ""}
        >
          <option value="">All statuses</option>
          <StatusOptions />
        </select>
        <select
          aria-label="Favourite filter"
          className="text-input"
          onChange={(event) =>
            update(
              "isFavorite",
              event.target.value === "" ? null : event.target.value === "true"
            )
          }
          value={filter.isFavorite === null ? "" : String(filter.isFavorite)}
        >
          <option value="">All favourites</option>
          <option value="true">Favourites only</option>
          <option value="false">Not favourite</option>
        </select>
        <select
          aria-label="Tag filter"
          className="text-input"
          onChange={(event) => update("tagId", event.target.value ? Number(event.target.value) : null)}
          value={filter.tagId ?? ""}
        >
          <option value="">All tags</option>
          {tags.map((tag) => (
            <option key={tag.id} value={tag.id}>
              {tag.name}
            </option>
          ))}
        </select>
        <select
          aria-label="Unwatched filter"
          className="text-input"
          onChange={(event) =>
            update(
              "hasUnwatched",
              event.target.value === "" ? null : event.target.value === "true"
            )
          }
          value={filter.hasUnwatched === null ? "" : String(filter.hasUnwatched)}
        >
          <option value="">Any progress</option>
          <option value="true">Has unwatched</option>
          <option value="false">Fully watched</option>
        </select>
        <input
          aria-label="Streaming provider filter"
          className="text-input"
          onChange={(event) => update("providerName", event.target.value)}
          placeholder="Provider name"
          value={filter.providerName}
        />
        <select
          aria-label="Watch Next filter"
          className="text-input"
          onChange={(event) =>
            update(
              "isWatchNext",
              event.target.value === "" ? null : event.target.value === "true"
            )
          }
          value={filter.isWatchNext === null ? "" : String(filter.isWatchNext)}
        >
          <option value="">Queue or not queued</option>
          <option value="true">Watch Next only</option>
          <option value="false">Not in Watch Next</option>
        </select>
        <select
          aria-label="Media library sort"
          className="text-input"
          onChange={(event) => update("sort", event.target.value as MediaSort)}
          value={filter.sort}
        >
          <option value="TITLE">Title</option>
          <option value="RECENTLY_ADDED">Recently added</option>
          <option value="RECENTLY_WATCHED">Recently watched</option>
          <option value="PROGRESS">Progress</option>
          <option value="RELEASE_DATE">Release / air date</option>
          <option value="WATCH_NEXT">Watch Next order</option>
          <option value="PRIORITY">Priority</option>
        </select>
        <select
          aria-label="Sort direction"
          className="text-input"
          onChange={(event) => update("sortDirection", event.target.value as "ASC" | "DESC")}
          value={filter.sortDirection}
        >
          <option value="ASC">Ascending</option>
          <option value="DESC">Descending</option>
        </select>
        <button
          className="ghost-button"
          onClick={() => onFilter(defaultMediaFilter)}
          type="button"
        >
          Reset filters
        </button>
      </div>
      {items.length === 0 && !isLoading ? (
        <div className="empty-editor-state media-empty-state">
          <p>No local titles match. Search the catalogue or reset filters.</p>
          <button className="primary-button" onClick={onSearch} type="button">
            Search catalogue
          </button>
        </div>
      ) : (
        <div className="media-row-list">
          {items.map((item) => (
            <MediaRow item={item} key={item.id} onOpen={onOpen} />
          ))}
        </div>
      )}
    </div>
  );
}

function MediaRow({
  item,
  onOpen
}: {
  item: MediaLibrarySummary;
  onOpen: (entryId: number) => void;
}) {
  const date = item.releaseDate || item.firstAirDate;
  return (
    <button className="media-library-row" onClick={() => onOpen(item.id)} type="button">
      <Poster path={item.posterPath} title={item.title} />
      <div className="media-library-row-main">
        <div className="media-card-title-row">
          <strong>
            {item.isFavorite && "★ "}
            {item.title}
          </strong>
          <span>{item.contentType === "MOVIE" ? "Movie" : "Series"}</span>
        </div>
        <span>
          {date ? date.slice(0, 4) : "Year unknown"} · {statusLabel(item.libraryStatus)}
        </span>
        <span>
          {item.contentType === "MOVIE"
            ? item.movieWatched
              ? "Watched"
              : "Unwatched"
            : `${item.watchedCount}/${item.progressTotal} · ${item.progressPercent.toFixed(0)}%`}
          {item.nextEpisode
            ? ` · Next S${item.nextEpisode.seasonNumber}E${item.nextEpisode.episodeNumber}`
            : ""}
        </span>
      </div>
      <div className="media-row-badges">
        {item.queuePosition && <span>Queue #{item.queuePosition}</span>}
        {item.newEpisodesCount > 0 && <span>New {item.newEpisodesCount}</span>}
        {item.availabilityIsStale && <span>Stale</span>}
        {item.subscriptionProviders.slice(0, 3).map((provider) => (
          <span title={provider.providerName} key={provider.providerName}>
            {provider.providerLogoPath ? (
              <img
                alt={provider.providerName}
                loading="lazy"
                src={`https://image.tmdb.org/t/p/original${provider.providerLogoPath}`}
              />
            ) : (
              provider.providerName
            )}
          </span>
        ))}
      </div>
    </button>
  );
}

function MediaSettingsView({
  settings,
  tags,
  onSettings,
  onTags,
  onStatus
}: {
  settings: MediaSettings;
  tags: MediaTag[];
  onSettings: (settings: MediaSettings) => void;
  onTags: (tags: MediaTag[]) => void;
  onStatus: (status: string, isError?: boolean) => void;
}) {
  const [form, setForm] = useState(settings);

  async function saveSettings() {
    try {
      const saved = await updateMediaSettings({
        regionCode: form.regionCode,
        metadataLanguage: form.metadataLanguage,
        includeSpecialsInCompletion: form.includeSpecialsInCompletion
      });
      setForm(saved);
      onSettings(saved);
      onStatus("Media Library settings saved and progress recalculated.");
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    }
  }

  async function removeTag(tag: MediaTag) {
    if (!window.confirm(`Delete the tag "${tag.name}"? Entry mappings will be removed.`)) {
      return;
    }
    try {
      await deleteMediaTag(tag.id);
      onTags(tags.filter((item) => item.id !== tag.id));
      onStatus(`Tag "${tag.name}" deleted.`);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    }
  }

  return (
    <div className="media-settings-view">
      <section className="media-form-card">
        <h4>Metadata and progress</h4>
        <div className="media-form-grid">
          <label>
            <span>Provider region</span>
            <input
              className="text-input"
              maxLength={2}
              onChange={(event) =>
                setForm((current) => ({ ...current, regionCode: event.target.value.toUpperCase() }))
              }
              value={form.regionCode}
            />
          </label>
          <label>
            <span>Metadata language</span>
            <input
              className="text-input"
              onChange={(event) =>
                setForm((current) => ({ ...current, metadataLanguage: event.target.value }))
              }
              value={form.metadataLanguage}
            />
          </label>
        </div>
        <label className="media-check-label">
          <input
            checked={form.includeSpecialsInCompletion}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                includeSpecialsInCompletion: event.target.checked
              }))
            }
            type="checkbox"
          />
          Include season 0 specials in series completion
        </label>
        <button className="primary-button" onClick={() => void saveSettings()} type="button">
          Save settings
        </button>
      </section>

      <section className="media-section-card">
        <h4>Tag definitions</h4>
        {tags.length === 0 ? (
          <p className="media-hint">Tags can be created from a selected media detail.</p>
        ) : (
          <div className="media-tag-definition-list">
            {tags.map((tag) => (
              <div key={tag.id}>
                <span>{tag.name}</span>
                <button
                  className="ghost-button compact-button"
                  onClick={() => void removeTag(tag)}
                  type="button"
                >
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="media-section-card">
        <h4>Credits and data boundaries</h4>
        <p>This product uses the TMDB API but is not endorsed or certified by TMDB.</p>
        <p>Streaming availability data provided by JustWatch through TMDB.</p>
        <p className="media-hint">
          TMDB supplies catalogue metadata and availability references. Overlay Forge stores your
          notes, ratings, tags, queue, links, and viewing history only in local SQLite.
        </p>
      </section>
    </div>
  );
}

function statusLabel(status: MediaLibraryStatus) {
  return {
    PLANNED: "Plan to Watch",
    WATCHING: "Watching",
    COMPLETED: "Completed",
    ON_HOLD: "On Hold",
    DROPPED: "Dropped"
  }[status];
}
