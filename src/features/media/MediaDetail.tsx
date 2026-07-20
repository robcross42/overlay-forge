import { useMemo, useState } from "react";
import {
  addMediaToWatchNext,
  createMediaStreamingLink,
  createMediaTag,
  deleteMediaLibraryEntry,
  deleteMediaStreamingLink,
  getMediaLibraryDetail,
  markEpisodesWatchedThrough,
  moveMediaWatchNextItem,
  openMediaStreamingTarget,
  refreshMediaMetadata,
  removeMediaFromWatchNext,
  setEpisodeWatched,
  setMediaEntryTags,
  setMovieWatched,
  setPreferredMediaStreamingLink,
  setSeasonWatched,
  setSeriesWatched,
  updateMediaLibraryEntry,
  updateMediaStreamingLink
} from "../../services/media";
import type {
  MediaLibraryDetail,
  MediaLibraryStatus,
  MediaStreamingLink,
  MediaStreamingLinkInput,
  MediaTag,
  UpdateMediaEntryInput
} from "../../services/media";
import { formatUnknownError } from "../../utils/errors";
import { Poster, StatusOptions } from "./MediaCatalogSearch";

type MediaDetailProps = {
  detail: MediaLibraryDetail;
  allTags: MediaTag[];
  onBack: () => void;
  onChanged: (detail: MediaLibraryDetail) => void;
  onDeleted: (entryId: number) => void;
  onTagsChanged: (tags: MediaTag[]) => void;
  onStatus: (status: string, isError?: boolean) => void;
};

const emptyLink: MediaStreamingLinkInput = {
  providerName: "",
  url: "",
  linkType: "STREAM",
  isPreferred: false,
  notes: ""
};

export function MediaDetail({
  detail,
  allTags,
  onBack,
  onChanged,
  onDeleted,
  onTagsChanged,
  onStatus
}: MediaDetailProps) {
  const { entry } = detail;
  const [isEditing, setIsEditing] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [entryForm, setEntryForm] = useState<UpdateMediaEntryInput>(() => entryToForm(detail));
  const [watchedAt, setWatchedAt] = useState(toDateTimeLocal(entry.completedAt));
  const [tagName, setTagName] = useState("");
  const [selectedTagIds, setSelectedTagIds] = useState<number[]>(
    detail.tags.map((tag) => tag.id)
  );
  const [linkForm, setLinkForm] = useState<MediaStreamingLinkInput>(emptyLink);
  const [editingLinkId, setEditingLinkId] = useState<number | null>(null);

  const preferredLink = detail.streamingLinks.find((link) => link.isPreferred) ?? null;
  const providerGroups = useMemo(
    () =>
      (["FLATRATE", "FREE", "ADS", "RENT", "BUY"] as const).map((type) => ({
        type,
        providers: detail.providers.filter((provider) => provider.monetizationType === type)
      })),
    [detail.providers]
  );

  async function runMutation(
    action: () => Promise<MediaLibraryDetail>,
    successMessage: string
  ) {
    setIsBusy(true);
    try {
      const nextDetail = await action();
      onChanged(nextDetail);
      setEntryForm(entryToForm(nextDetail));
      setWatchedAt(toDateTimeLocal(nextDetail.entry.completedAt));
      setSelectedTagIds(nextDetail.tags.map((tag) => tag.id));
      onStatus(successMessage);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setIsBusy(false);
    }
  }

  async function saveEntry() {
    await runMutation(
      () => updateMediaLibraryEntry(entry.id, entryForm),
      "Media details saved."
    );
    setIsEditing(false);
  }

  async function deleteEntry() {
    if (!window.confirm(`Delete "${entry.title}" and all of its local progress?`)) {
      return;
    }
    setIsBusy(true);
    try {
      await deleteMediaLibraryEntry(entry.id);
      onDeleted(entry.id);
      onStatus(`${entry.title} deleted from the local library.`);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
      setIsBusy(false);
    }
  }

  async function saveTags() {
    await runMutation(
      () => setMediaEntryTags(entry.id, selectedTagIds),
      "Media tags updated."
    );
  }

  async function addTag() {
    if (!tagName.trim()) {
      return;
    }
    try {
      const tag = await createMediaTag(tagName);
      const nextTags = allTags.some((item) => item.id === tag.id) ? allTags : [...allTags, tag];
      onTagsChanged(nextTags.sort((left, right) => left.name.localeCompare(right.name)));
      setSelectedTagIds((current) =>
        current.includes(tag.id) ? current : [...current, tag.id]
      );
      setTagName("");
      onStatus(`Tag "${tag.name}" is ready to assign.`);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    }
  }

  function editLink(link: MediaStreamingLink) {
    setEditingLinkId(link.id);
    setLinkForm({
      providerName: link.providerName,
      url: link.url,
      linkType: link.linkType,
      isPreferred: link.isPreferred,
      notes: link.notes
    });
  }

  async function saveLink() {
    setIsBusy(true);
    try {
      if (editingLinkId) {
        await updateMediaStreamingLink(editingLinkId, linkForm);
      } else {
        await createMediaStreamingLink(entry.id, linkForm);
      }
      const nextDetail = await getMediaLibraryDetail(entry.id);
      onChanged(nextDetail);
      setLinkForm(emptyLink);
      setEditingLinkId(null);
      onStatus(editingLinkId ? "Manual link updated." : "Manual link added.");
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setIsBusy(false);
    }
  }

  async function removeLink(linkId: number) {
    if (!window.confirm("Delete this manual streaming link?")) {
      return;
    }
    setIsBusy(true);
    try {
      await deleteMediaStreamingLink(linkId);
      const nextDetail = await getMediaLibraryDetail(entry.id);
      onChanged(nextDetail);
      onStatus("Manual link deleted.");
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <div className="media-detail">
      <div className="media-detail-nav">
        <button className="ghost-button" onClick={onBack} type="button">
          Back
        </button>
        <div className="media-detail-actions">
          {entry.sourceKey === "tmdb" && (
            <button
              className="ghost-button"
              disabled={isBusy}
              onClick={() =>
                void runMutation(
                  () => refreshMediaMetadata(entry.id),
                  "Metadata and availability refreshed."
                )
              }
              type="button"
            >
              Refresh metadata
            </button>
          )}
          <button
            className="primary-button"
            disabled={isBusy}
            onClick={() => setIsEditing((current) => !current)}
            type="button"
          >
            {isEditing ? "Cancel Edit" : "Edit"}
          </button>
        </div>
      </div>

      <section className="media-detail-hero">
        <Poster path={entry.posterPath} title={entry.title} />
        <div>
          <p className="media-eyebrow">
            {entry.contentType === "MOVIE" ? "Movie" : entry.seriesType || "Series"}
          </p>
          <h3>
            {entry.isFavorite && <span aria-label="Favourite">★ </span>}
            {entry.title}
          </h3>
          <p className="media-card-meta">
            {entry.releaseDate || entry.firstAirDate || "Release date unavailable"}
            {entry.runtimeMinutes ? ` · ${entry.runtimeMinutes} min` : ""}
          </p>
          <p>{entry.overview || "No overview is stored for this title."}</p>
          <div className="media-chip-row">
            <span>{statusLabel(entry.libraryStatus)}</span>
            {entry.personalRating && <span>{entry.personalRating.toFixed(1)} / 10</span>}
            {entry.queuePosition && <span>Watch Next #{entry.queuePosition}</span>}
            {entry.newEpisodesCount > 0 && <span>New episodes: {entry.newEpisodesCount}</span>}
          </div>
          <ProgressLabel detail={detail} />
          <div className="media-detail-actions wrap">
            {(preferredLink || detail.providerSnapshot?.sourceLinkUrl) && (
              <button
                className="primary-button"
                onClick={() =>
                  void openMediaStreamingTarget(entry.id)
                    .then(() => onStatus(preferredLink ? "Opened preferred Watch link." : "Opened streaming options."))
                    .catch((error) => onStatus(formatUnknownError(error), true))
                }
                type="button"
              >
                {preferredLink ? "Watch" : "View streaming options"}
              </button>
            )}
            {entry.queuePosition ? (
              <>
                <button
                  className="ghost-button"
                  disabled={isBusy}
                  onClick={() =>
                    void runMutation(
                      () => moveMediaWatchNextItem(entry.id, "UP"),
                      "Moved up in Watch Next."
                    )
                  }
                  type="button"
                >
                  Queue up
                </button>
                <button
                  className="ghost-button"
                  disabled={isBusy}
                  onClick={() =>
                    void runMutation(
                      () => moveMediaWatchNextItem(entry.id, "DOWN"),
                      "Moved down in Watch Next."
                    )
                  }
                  type="button"
                >
                  Queue down
                </button>
                <button
                  className="ghost-button"
                  disabled={isBusy}
                  onClick={() =>
                    void runMutation(
                      () => removeMediaFromWatchNext(entry.id),
                      "Removed from Watch Next."
                    )
                  }
                  type="button"
                >
                  Remove from queue
                </button>
              </>
            ) : (
              <button
                className="ghost-button"
                disabled={isBusy}
                onClick={() =>
                  void runMutation(
                    () => addMediaToWatchNext(entry.id),
                    "Added to Watch Next."
                  )
                }
                type="button"
              >
                Add to Watch Next
              </button>
            )}
          </div>
        </div>
      </section>

      {isEditing ? (
        <section className="media-form-card">
          <h4>Edit local details</h4>
          <div className="media-form-grid">
            <label>
              <span>Status</span>
              <select
                className="text-input"
                onChange={(event) =>
                  setEntryForm((current) => ({
                    ...current,
                    libraryStatus: event.target.value as MediaLibraryStatus
                  }))
                }
                value={entryForm.libraryStatus}
              >
                <StatusOptions />
              </select>
            </label>
            <label>
              <span>Personal rating</span>
              <input
                className="text-input"
                max="10"
                min="1"
                onChange={(event) =>
                  setEntryForm((current) => ({
                    ...current,
                    personalRating: event.target.value ? Number(event.target.value) : null
                  }))
                }
                step="0.5"
                type="number"
                value={entryForm.personalRating ?? ""}
              />
            </label>
            <label>
              <span>Priority</span>
              <input
                className="text-input"
                onChange={(event) =>
                  setEntryForm((current) => ({
                    ...current,
                    priority: Number(event.target.value)
                  }))
                }
                type="number"
                value={entryForm.priority}
              />
            </label>
            <label className="media-check-label">
              <input
                checked={entryForm.isFavorite}
                onChange={(event) =>
                  setEntryForm((current) => ({
                    ...current,
                    isFavorite: event.target.checked
                  }))
                }
                type="checkbox"
              />
              Favourite
            </label>
          </div>
          <label>
            <span>Personal notes</span>
            <textarea
              className="body-input compact"
              onChange={(event) =>
                setEntryForm((current) => ({ ...current, notes: event.target.value }))
              }
              value={entryForm.notes}
            />
          </label>
          <div className="form-actions">
            <button
              className="primary-button"
              disabled={isBusy}
              onClick={() => void saveEntry()}
              type="button"
            >
              Save
            </button>
            <button
              className="primary-button media-danger-button"
              disabled={isBusy}
              onClick={() => void deleteEntry()}
              type="button"
            >
              Delete
            </button>
          </div>
        </section>
      ) : (
        entry.notes && (
          <section className="media-section-card">
            <h4>Personal notes</h4>
            <p className="media-pre-wrap">{entry.notes}</p>
          </section>
        )
      )}

      <section className="media-section-card">
        <div className="media-section-heading">
          <div>
            <p>Local viewing state</p>
            <h4>{entry.contentType === "MOVIE" ? "Movie progress" : "Series progress"}</h4>
          </div>
          {entry.contentType === "MOVIE" ? (
            <button
              className="primary-button"
              disabled={isBusy}
              onClick={() =>
                void runMutation(
                  () =>
                    setMovieWatched(
                      entry.id,
                      !entry.movieWatched,
                      entry.movieWatched ? "" : fromDateTimeLocal(watchedAt)
                    ),
                  entry.movieWatched ? "Movie marked unwatched." : "Movie marked watched."
                )
              }
              type="button"
            >
              {entry.movieWatched ? "Mark unwatched" : "Mark watched"}
            </button>
          ) : (
            <div className="form-actions">
              <button
                className="primary-button"
                disabled={isBusy || detail.seasons.length === 0}
                onClick={() =>
                  void runMutation(
                    () => setSeriesWatched(entry.id, true),
                    "Series marked watched."
                  )
                }
                type="button"
              >
                Mark all watched
              </button>
              <button
                className="ghost-button"
                disabled={isBusy || detail.seasons.length === 0}
                onClick={() =>
                  void runMutation(
                    () => setSeriesWatched(entry.id, false),
                    "Series marked unwatched."
                  )
                }
                type="button"
              >
                Mark all unwatched
              </button>
            </div>
          )}
        </div>
        {entry.contentType === "MOVIE" && (
          <div className="form-actions wrap">
            <label>
              <span>Watched date</span>
              <input
                className="text-input media-date-input"
                onChange={(event) => setWatchedAt(event.target.value)}
                type="datetime-local"
                value={watchedAt}
              />
            </label>
            {entry.movieWatched && (
              <button
                className="ghost-button"
                disabled={isBusy}
                onClick={() =>
                  void runMutation(
                    () => setMovieWatched(entry.id, true, fromDateTimeLocal(watchedAt)),
                    "Movie watched date updated."
                  )
                }
                type="button"
              >
                Save watched date
              </button>
            )}
          </div>
        )}
        {entry.contentType === "SERIES" &&
          (detail.seasons.length === 0 ? (
            <p className="media-hint">
              Manual series currently use series-level status only. Manual season and episode
              editing is deferred.
            </p>
          ) : (
            <div className="media-seasons">
              {detail.seasons.map((season) => (
                <details className="media-season" key={season.id} open={season.seasonNumber !== 0}>
                  <summary>
                    <span>
                      {season.seasonNumber === 0 ? "Specials" : `Season ${season.seasonNumber}`}
                      {season.name && season.name !== `Season ${season.seasonNumber}`
                        ? ` · ${season.name}`
                        : ""}
                    </span>
                    <span>
                      {season.watchedCount}/{season.progressTotal || season.episodes.length}
                    </span>
                  </summary>
                  <div className="media-season-actions">
                    <button
                      className="ghost-button"
                      disabled={isBusy}
                      onClick={() =>
                        void runMutation(
                          () => setSeasonWatched(entry.id, season.seasonNumber, true),
                          `Season ${season.seasonNumber} marked watched.`
                        )
                      }
                      type="button"
                    >
                      Mark season watched
                    </button>
                    <button
                      className="ghost-button"
                      disabled={isBusy}
                      onClick={() =>
                        void runMutation(
                          () => setSeasonWatched(entry.id, season.seasonNumber, false),
                          `Season ${season.seasonNumber} marked unwatched.`
                        )
                      }
                      type="button"
                    >
                      Mark season unwatched
                    </button>
                  </div>
                  <div className="media-episodes">
                    {season.episodes.map((episode) => (
                      <article
                        className={episode.isPresentInSource ? "media-episode" : "media-episode stale"}
                        key={episode.id}
                      >
                        <button
                          aria-label={`${episode.isWatched ? "Mark unwatched" : "Mark watched"}: ${episode.name}`}
                          className={episode.isWatched ? "media-watch-toggle watched" : "media-watch-toggle"}
                          disabled={isBusy}
                          onClick={() =>
                            void runMutation(
                              () =>
                                setEpisodeWatched(
                                  entry.id,
                                  episode.id,
                                  !episode.isWatched
                                ),
                              episode.isWatched
                                ? "Episode marked unwatched."
                                : "Episode marked watched."
                            )
                          }
                          type="button"
                        >
                          {episode.isWatched ? "✓" : "○"}
                        </button>
                        <div>
                          <strong>
                            E{episode.episodeNumber} · {episode.name || "Untitled episode"}
                          </strong>
                          <span>
                            {episode.airDate || "Air date unavailable"}
                            {episode.runtimeMinutes ? ` · ${episode.runtimeMinutes} min` : ""}
                            {episode.watchedAt ? ` · Watched ${episode.watchedAt}` : ""}
                          </span>
                        </div>
                        <button
                          className="ghost-button compact-button"
                          disabled={isBusy}
                          onClick={() =>
                            void runMutation(
                              () => markEpisodesWatchedThrough(entry.id, episode.id),
                              "Episodes marked watched through this episode."
                            )
                          }
                          type="button"
                        >
                          Through here
                        </button>
                      </article>
                    ))}
                  </div>
                </details>
              ))}
            </div>
          ))}
      </section>

      <section className="media-section-card">
        <div className="media-section-heading">
          <div>
            <p>Current cached region: {detail.settings.regionCode}</p>
            <h4>Streaming availability</h4>
          </div>
          {detail.providerSnapshot && (
            <span className={entry.availabilityIsStale ? "media-stale-badge" : "media-chip"}>
              {entry.availabilityIsStale ? "Cached / stale" : "Current cache"}
            </span>
          )}
        </div>
        {detail.providerSnapshot ? (
          <>
            <p className="media-hint">
              Refreshed {detail.providerSnapshot.refreshedAt || "never"}.
              {detail.providerSnapshot.lastRefreshError
                ? ` Last refresh failed: ${detail.providerSnapshot.lastRefreshError}`
                : ""}
            </p>
            {providerGroups.map(
              (group) =>
                group.providers.length > 0 && (
                  <div className="media-provider-group" key={group.type}>
                    <strong>{providerGroupLabel(group.type)}</strong>
                    <div className="media-provider-list">
                      {group.providers.map((provider) => (
                        <span className="media-provider-chip" key={provider.id}>
                          {provider.providerLogoPath && (
                            <img
                              alt=""
                              loading="lazy"
                              src={`https://image.tmdb.org/t/p/original${provider.providerLogoPath}`}
                            />
                          )}
                          {provider.providerName}
                        </span>
                      ))}
                    </div>
                  </div>
                )
            )}
            {detail.providers.length === 0 && (
              <p>No provider rows are cached for this title and region.</p>
            )}
          </>
        ) : (
          <p>No streaming availability has been cached for this title.</p>
        )}
        <p className="media-attribution">
          Streaming availability data provided by JustWatch through TMDB.
        </p>
      </section>

      <section className="media-section-card">
        <h4>Manual streaming links</h4>
        <div className="media-link-list">
          {detail.streamingLinks.map((link) => (
            <article className="media-link-row" key={link.id}>
              <div>
                <strong>
                  {link.providerName || "Manual link"} {link.isPreferred && "· Preferred"}
                </strong>
                <span>{link.url}</span>
              </div>
              <div className="form-actions">
                {!link.isPreferred && (
                  <button
                    className="ghost-button compact-button"
                    onClick={() =>
                      void setPreferredMediaStreamingLink(entry.id, link.id)
                        .then(() => getMediaLibraryDetail(entry.id))
                        .then(onChanged)
                        .catch((error) => onStatus(formatUnknownError(error), true))
                    }
                    type="button"
                  >
                    Prefer
                  </button>
                )}
                <button
                  className="ghost-button compact-button"
                  onClick={() => editLink(link)}
                  type="button"
                >
                  Edit
                </button>
                <button
                  className="ghost-button compact-button"
                  onClick={() => void removeLink(link.id)}
                  type="button"
                >
                  Delete
                </button>
              </div>
            </article>
          ))}
        </div>
        <div className="media-form-grid">
          <label>
            <span>Provider label</span>
            <input
              className="text-input"
              onChange={(event) =>
                setLinkForm((current) => ({ ...current, providerName: event.target.value }))
              }
              value={linkForm.providerName}
            />
          </label>
          <label>
            <span>Link type</span>
            <select
              className="text-input"
              onChange={(event) =>
                setLinkForm((current) => ({
                  ...current,
                  linkType: event.target.value as MediaStreamingLinkInput["linkType"]
                }))
              }
              value={linkForm.linkType}
            >
              <option value="STREAM">Stream</option>
              <option value="RENT">Rent</option>
              <option value="BUY">Buy</option>
              <option value="OTHER">Other</option>
            </select>
          </label>
        </div>
        <label>
          <span>Direct URL</span>
          <input
            className="text-input"
            onChange={(event) =>
              setLinkForm((current) => ({ ...current, url: event.target.value }))
            }
            placeholder="https://..."
            value={linkForm.url}
          />
        </label>
        <div className="form-actions">
          <label className="media-check-label">
            <input
              checked={linkForm.isPreferred}
              onChange={(event) =>
                setLinkForm((current) => ({ ...current, isPreferred: event.target.checked }))
              }
              type="checkbox"
            />
            Preferred Watch action
          </label>
          <button
            className="primary-button"
            disabled={isBusy || !linkForm.url.trim()}
            onClick={() => void saveLink()}
            type="button"
          >
            {editingLinkId ? "Save link" : "Add link"}
          </button>
          {editingLinkId && (
            <button
              className="ghost-button"
              onClick={() => {
                setEditingLinkId(null);
                setLinkForm(emptyLink);
              }}
              type="button"
            >
              Cancel
            </button>
          )}
        </div>
      </section>

      <section className="media-section-card">
        <h4>Tags</h4>
        <div className="media-tag-picker">
          {allTags.map((tag) => (
            <label key={tag.id}>
              <input
                checked={selectedTagIds.includes(tag.id)}
                onChange={(event) =>
                  setSelectedTagIds((current) =>
                    event.target.checked
                      ? [...current, tag.id]
                      : current.filter((id) => id !== tag.id)
                  )
                }
                type="checkbox"
              />
              {tag.name}
            </label>
          ))}
        </div>
        <div className="form-actions">
          <input
            aria-label="New media tag"
            className="text-input"
            onChange={(event) => setTagName(event.target.value)}
            placeholder="New tag"
            value={tagName}
          />
          <button className="ghost-button" onClick={() => void addTag()} type="button">
            Create tag
          </button>
          <button className="primary-button" onClick={() => void saveTags()} type="button">
            Save assigned tags
          </button>
        </div>
      </section>
    </div>
  );
}

function entryToForm(detail: MediaLibraryDetail): UpdateMediaEntryInput {
  return {
    libraryStatus: detail.entry.libraryStatus,
    isFavorite: detail.entry.isFavorite,
    personalRating: detail.entry.personalRating,
    notes: detail.entry.notes,
    priority: detail.entry.priority
  };
}

function toDateTimeLocal(value: string) {
  return value ? value.replace(" ", "T").slice(0, 16) : "";
}

function fromDateTimeLocal(value: string) {
  return value ? value.replace("T", " ") + (value.length === 16 ? ":00" : "") : "";
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

function providerGroupLabel(type: string) {
  return {
    FLATRATE: "Subscription",
    FREE: "Free",
    ADS: "With ads",
    RENT: "Rent",
    BUY: "Buy"
  }[type] ?? type;
}

function ProgressLabel({ detail }: { detail: MediaLibraryDetail }) {
  const { entry } = detail;
  if (entry.contentType === "MOVIE") {
    return <p className="media-progress-label">{entry.movieWatched ? "Watched" : "Unwatched"}</p>;
  }
  return (
    <p className="media-progress-label">
      {entry.watchedCount}/{entry.progressTotal} watched · {entry.progressPercent.toFixed(0)}%
      {entry.nextEpisode
        ? ` · Next S${entry.nextEpisode.seasonNumber}E${entry.nextEpisode.episodeNumber}`
        : ""}
    </p>
  );
}
