import { useEffect, useState } from "react";
import {
  createMediaTag,
  deleteMediaLibraryEntry,
  setMediaEntryTags,
  updateMediaLibraryEntry
} from "../../../services/media";
import {
  addBookToReadNext,
  clearBookSeriesOverride,
  createBookLink,
  createManualBookEdition,
  deleteBookLink,
  deleteManualBookEdition,
  getBookLibraryDetail,
  markBookRead,
  moveBookReadNextItem,
  openBookLink,
  refreshBookMetadata,
  removeBookFromReadNext,
  resetBookProgress,
  setBookPreferredEdition,
  setBookProgress,
  setBookSeriesOverride,
  setPreferredBookLink,
  updateBookReaderState,
  updateManualBookEdition
} from "../../../services/mediaBooks";
import type {
  BookFormat,
  BookLibraryDetail,
  BookLinkInput,
  BookOwnershipStatus,
  BookProgressUnit,
  ManualBookEditionInput
} from "../../../services/mediaBooks";
import type { MediaLibraryStatus, MediaTag } from "../../../services/media";
import { formatUnknownError } from "../../../utils/errors";
import { BookCover } from "./BookCover";
import { bookProgressLabel } from "./bookPresentation";

const emptyEdition: ManualBookEditionInput = {
  title: "",
  subtitle: "",
  format: "UNKNOWN",
  isbn10: "",
  isbn13: "",
  publisher: "",
  publishedDate: "",
  language: "",
  pageCount: null,
  audioDurationMinutes: null,
  coverUrl: ""
};

const emptyLink: BookLinkInput = {
  editionId: null,
  linkType: "INFO",
  url: "",
  regionCode: "",
  isPreferred: false
};

export function BookDetail({
  detail,
  allTags,
  onBack,
  onChanged,
  onDeleted,
  onStatus,
  onTagsChanged
}: {
  detail: BookLibraryDetail;
  allTags: MediaTag[];
  onBack: () => void;
  onChanged: (detail: BookLibraryDetail) => void;
  onDeleted: (entryId: number) => void;
  onStatus: (status: string, isError?: boolean) => void;
  onTagsChanged: (tags: MediaTag[]) => void;
}) {
  const { item } = detail;
  const [busy, setBusy] = useState(false);
  const [shared, setShared] = useState({
    libraryStatus: item.libraryStatus,
    isFavorite: item.isFavorite,
    personalRating: item.personalRating,
    notes: item.notes,
    priority: item.priority
  });
  const [progress, setProgress] = useState({
    progressUnit: item.progress.progressUnit,
    progressValue: item.progress.progressValue,
    progressTotalOverride: item.progress.progressTotalOverride,
    currentChapter: item.progress.currentChapter
  });
  const [reader, setReader] = useState({
    ownershipStatus: item.progress.ownershipStatus,
    preferredFormat: item.progress.preferredFormat
  });
  const [showEdition, setShowEdition] = useState(false);
  const [editingEditionId, setEditingEditionId] = useState<number | null>(null);
  const [edition, setEdition] = useState(emptyEdition);
  const [link, setLink] = useState(emptyLink);
  const [series, setSeries] = useState({ name: "", positionText: "", positionSort: null as number | null, isPrimary: true });
  const [tagName, setTagName] = useState("");
  const [selectedTagIds, setSelectedTagIds] = useState(detail.tags.map((tag) => tag.id));

  useEffect(() => {
    setShared({
      libraryStatus: item.libraryStatus,
      isFavorite: item.isFavorite,
      personalRating: item.personalRating,
      notes: item.notes,
      priority: item.priority
    });
    setProgress({
      progressUnit: item.progress.progressUnit,
      progressValue: item.progress.progressValue,
      progressTotalOverride: item.progress.progressTotalOverride,
      currentChapter: item.progress.currentChapter
    });
    setReader({
      ownershipStatus: item.progress.ownershipStatus,
      preferredFormat: item.progress.preferredFormat
    });
    setSelectedTagIds(detail.tags.map((tag) => tag.id));
  }, [detail.tags, item]);

  async function run(message: string, action: () => Promise<BookLibraryDetail>) {
    setBusy(true);
    try {
      const next = await action();
      onChanged(next);
      onStatus(message);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setBusy(false);
    }
  }

  async function saveShared() {
    setBusy(true);
    try {
      await updateMediaLibraryEntry(item.entryId, shared);
      const next = await getBookLibraryDetail(item.entryId);
      onChanged(next);
      onStatus("Book library fields saved.");
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setBusy(false);
    }
  }

  async function saveTags() {
    setBusy(true);
    try {
      await setMediaEntryTags(item.entryId, selectedTagIds);
      const next = await getBookLibraryDetail(item.entryId);
      onChanged(next);
      onStatus("Book tags saved.");
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setBusy(false);
    }
  }

  async function addTag() {
    if (!tagName.trim()) return;
    try {
      const tag = await createMediaTag(tagName);
      const nextTags = allTags.some((value) => value.id === tag.id)
        ? allTags
        : [...allTags, tag].sort((left, right) => left.name.localeCompare(right.name));
      onTagsChanged(nextTags);
      setSelectedTagIds((current) => current.includes(tag.id) ? current : [...current, tag.id]);
      setTagName("");
      onStatus(`Tag "${tag.name}" is ready to assign.`);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    }
  }

  function editEdition(value: BookLibraryDetail["editions"][number]) {
    setEdition({
      title: value.title,
      subtitle: value.subtitle,
      format: value.format,
      isbn10: value.isbn10,
      isbn13: value.isbn13,
      publisher: value.publisher,
      publishedDate: value.publishedDate,
      language: value.language,
      pageCount: value.pageCount,
      audioDurationMinutes: value.audioDurationMinutes,
      coverUrl: value.coverUrl
    });
    setEditingEditionId(value.id);
    setShowEdition(true);
  }

  function closeEditionForm() {
    setEdition(emptyEdition);
    setEditingEditionId(null);
    setShowEdition(false);
  }

  async function saveEdition() {
    const next = editingEditionId === null
      ? await createManualBookEdition(item.entryId, edition)
      : await updateManualBookEdition(item.entryId, editingEditionId, edition);
    closeEditionForm();
    return next;
  }

  async function removeBook() {
    if (!window.confirm(`Delete "${item.title}" and its local reading state?`)) return;
    setBusy(true);
    try {
      await deleteMediaLibraryEntry(item.entryId);
      onDeleted(item.entryId);
      onStatus(`${item.title} deleted.`);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
      setBusy(false);
    }
  }

  return (
    <div className="media-detail book-detail">
      <div className="media-detail-toolbar">
        <button className="ghost-button" onClick={onBack} type="button">Back to Books</button>
        <div>
          <button className="ghost-button" disabled={busy || detail.sources.every((source) => source.sourceKey === "manual")} onClick={() => void run("Book metadata refreshed.", () => refreshBookMetadata(item.entryId))} type="button">Refresh metadata</button>
          <button className="media-danger-button" disabled={busy} onClick={() => void removeBook()} type="button">Delete</button>
        </div>
      </div>

      <section className="media-detail-hero">
        <BookCover title={item.title} url={item.coverUrl} />
        <div>
          <p className="media-eyebrow">Book · {item.statusLabel}</p>
          <h3>{item.title}</h3>
          {item.subtitle && <p>{item.subtitle}</p>}
          <p className="media-card-meta">{[item.primaryAuthorText, item.firstPublishYear].filter(Boolean).join(" · ")}</p>
          <p>{item.description || "No description is available."}</p>
          {detail.subjects.length > 0 && <p className="media-hint">Subjects: {detail.subjects.join(", ")}</p>}
          <p className="media-hint">{bookProgressLabel(item.progress)}</p>
          {detail.communityRating !== null && <p className="media-hint">Community rating {detail.communityRating.toFixed(1)}{detail.communityRatingCount ? ` (${detail.communityRatingCount})` : ""}</p>}
        </div>
      </section>

      {detail.providerWarnings.map((warning) => <p className="media-inline-warning" key={warning}>{warning}</p>)}

      <section className="media-form-card">
        <h4>Library and reading status</h4>
        <div className="media-form-grid">
          <label><span>Status</span><select className="text-input" value={shared.libraryStatus} onChange={(event) => setShared((current) => ({ ...current, libraryStatus: event.target.value as MediaLibraryStatus }))}><BookStatusOptions /></select></label>
          <label><span>Personal rating (1–10)</span><input className="text-input" min="1" max="10" step="0.5" type="number" value={shared.personalRating ?? ""} onChange={(event) => setShared((current) => ({ ...current, personalRating: event.target.value ? Number(event.target.value) : null }))} /></label>
          <label><span>Priority</span><input className="text-input" type="number" value={shared.priority} onChange={(event) => setShared((current) => ({ ...current, priority: Number(event.target.value) }))} /></label>
          <label className="media-check-label"><input checked={shared.isFavorite} type="checkbox" onChange={(event) => setShared((current) => ({ ...current, isFavorite: event.target.checked }))} />Favourite</label>
        </div>
        <label><span>Personal notes</span><textarea className="body-input compact" value={shared.notes} onChange={(event) => setShared((current) => ({ ...current, notes: event.target.value }))} /></label>
        <button className="primary-button" disabled={busy} onClick={() => void saveShared()} type="button">Save library fields</button>
      </section>

      <section className="media-form-card">
        <h4>Reading progress</h4>
        <div className="media-form-grid">
          <label><span>Unit</span><select className="text-input" value={progress.progressUnit} onChange={(event) => setProgress((current) => ({ ...current, progressUnit: event.target.value as BookProgressUnit }))}><ProgressUnitOptions /></select></label>
          <label><span>Current value</span><input className="text-input" min="0" max={progress.progressUnit === "PERCENT" ? 100 : undefined} type="number" value={progress.progressValue} onChange={(event) => setProgress((current) => ({ ...current, progressValue: Number(event.target.value) }))} /></label>
          <label><span>Total override</span><input className="text-input" min="0" type="number" value={progress.progressTotalOverride ?? ""} onChange={(event) => setProgress((current) => ({ ...current, progressTotalOverride: event.target.value ? Number(event.target.value) : null }))} /></label>
          <label><span>Current chapter</span><input className="text-input" value={progress.currentChapter} onChange={(event) => setProgress((current) => ({ ...current, currentChapter: event.target.value }))} /></label>
        </div>
        <div className="media-action-row">
          <button className="primary-button" disabled={busy} onClick={() => void run("Reading progress saved.", () => setBookProgress(item.entryId, progress))} type="button">Save progress</button>
          <button className="ghost-button" disabled={busy} onClick={() => void run("Book marked read.", () => markBookRead(item.entryId))} type="button">Mark Read</button>
          <button className="ghost-button" disabled={busy} onClick={() => void run("Reading progress reset.", () => resetBookProgress(item.entryId))} type="button">Reset</button>
        </div>
      </section>

      <section className="media-form-card">
        <h4>Edition and ownership</h4>
        <div className="media-form-grid">
          <label><span>Preferred edition</span><select className="text-input" value={item.progress.preferredEditionId ?? ""} onChange={(event) => event.target.value && void run("Preferred edition saved.", () => setBookPreferredEdition(item.entryId, Number(event.target.value)))}>{detail.editions.map((value) => <option key={value.id} value={value.id}>{[value.format, value.publisher, value.publishedDate, value.isbn13 || value.isbn10].filter(Boolean).join(" · ") || value.title}</option>)}</select></label>
          <label><span>Ownership</span><select className="text-input" value={reader.ownershipStatus} onChange={(event) => setReader((current) => ({ ...current, ownershipStatus: event.target.value as BookOwnershipStatus }))}><option value="NONE">None</option><option value="OWNED">Owned</option><option value="BORROWED">Borrowed</option><option value="WISHLIST">Wishlist</option></select></label>
          <label><span>Preferred format</span><select className="text-input" value={reader.preferredFormat} onChange={(event) => setReader((current) => ({ ...current, preferredFormat: event.target.value as BookFormat }))}><FormatOptions /></select></label>
        </div>
        <button className="primary-button" disabled={busy} onClick={() => void run("Reader preferences saved.", () => updateBookReaderState(item.entryId, reader))} type="button">Save reader preferences</button>
        <div className="media-action-row">
          {item.progress.readingQueuePosition === null ? <button className="ghost-button" onClick={() => void run("Added to Read Next.", () => addBookToReadNext(item.entryId))} type="button">Add to Read Next</button> : <><button className="ghost-button" onClick={() => void run("Read Next order updated.", () => moveBookReadNextItem(item.entryId, "UP"))} type="button">Move up</button><button className="ghost-button" onClick={() => void run("Read Next order updated.", () => moveBookReadNextItem(item.entryId, "DOWN"))} type="button">Move down</button><button className="ghost-button" onClick={() => void run("Removed from Read Next.", () => removeBookFromReadNext(item.entryId))} type="button">Remove from Read Next</button></>}
        </div>
      </section>

      <section className="media-section-card">
        <div className="media-section-heading"><h4>Editions</h4><button className="ghost-button compact-button" onClick={() => showEdition ? closeEditionForm() : setShowEdition(true)} type="button">{showEdition ? "Close" : "Add manual edition"}</button></div>
        {showEdition && <ManualEditionForm actionLabel={editingEditionId === null ? "Add edition" : "Save edition"} value={edition} onChange={setEdition} onSave={() => void run(editingEditionId === null ? "Manual edition added." : "Manual edition updated.", saveEdition)} />}
        <div className="book-edition-list">{detail.editions.map((value) => <article key={value.id}><BookCover title={value.title} url={value.coverUrl} /><div><strong>{value.title}</strong><p>{[value.format, value.publisher, value.publishedDate, value.language].filter(Boolean).join(" · ")}</p><p className="media-hint">{value.isbn13 || value.isbn10 || "No ISBN"}{value.pageCount ? ` · ${value.pageCount} pages` : ""}{value.audioDurationMinutes ? ` · ${value.audioDurationMinutes} minutes` : ""}{!value.isPresentInSource ? " · no longer in provider source" : ""}</p><div className="media-row-badges">{value.sourceKeys.map((source) => <span key={source}>{providerName(source)}</span>)}</div></div>{value.isUserOwned && <div className="media-action-row"><button className="ghost-button compact-button" onClick={() => editEdition(value)} type="button">Edit</button>{item.progress.preferredEditionId !== value.id && <button className="ghost-button compact-button" onClick={() => void run("Manual edition deleted.", () => deleteManualBookEdition(item.entryId, value.id))} type="button">Delete</button>}</div>}</article>)}</div>
      </section>

      <section className="media-section-card">
        <h4>Book links</h4>
        <div className="book-link-list">{detail.links.map((value) => <div key={value.id}><button className="ghost-button" onClick={() => void openBookLink(item.entryId, value.id)} type="button">{value.linkType} · {providerName(value.sourceKey)}</button>{value.isPreferred && <span>Preferred</span>}{value.isUserOwned && <><button className="ghost-button compact-button" onClick={() => void run("Preferred link saved.", () => setPreferredBookLink(item.entryId, value.id))} type="button">Prefer</button><button className="ghost-button compact-button" onClick={() => void run("Book link deleted.", () => deleteBookLink(item.entryId, value.id))} type="button">Delete</button></>}</div>)}</div>
        <div className="media-form-grid"><label><span>Link type</span><select className="text-input" value={link.linkType} onChange={(event) => setLink((current) => ({ ...current, linkType: event.target.value as BookLinkInput["linkType"] }))}>{["INFO","PREVIEW","READ","BORROW","BUY","OTHER"].map((value) => <option key={value}>{value}</option>)}</select></label><label><span>URL</span><input className="text-input" placeholder="https://" value={link.url} onChange={(event) => setLink((current) => ({ ...current, url: event.target.value }))} /></label><label><span>Edition</span><select className="text-input" value={link.editionId ?? ""} onChange={(event) => setLink((current) => ({ ...current, editionId: event.target.value ? Number(event.target.value) : null }))}><option value="">Work-level link</option>{detail.editions.map((value) => <option key={value.id} value={value.id}>{value.title} · {value.format}</option>)}</select></label></div>
        <button className="primary-button" onClick={() => void run("Book link added.", async () => { const next = await createBookLink(item.entryId, link); setLink(emptyLink); return next; })} type="button">Add link</button>
      </section>

      <section className="media-section-card">
        <h4>Series</h4>
        {detail.series.length === 0 ? <p className="media-hint">No series metadata.</p> : detail.series.map((value) => <div className="book-series-row" key={value.id}><span>{value.name}{value.positionText ? ` #${value.positionText}` : ""} · {providerName(value.sourceKey)}</span>{value.isUserOverride && <button className="ghost-button compact-button" onClick={() => void run("Series override cleared.", () => clearBookSeriesOverride(item.entryId, value.id))} type="button">Clear override</button>}</div>)}
        <div className="media-form-grid"><label><span>Series name</span><input className="text-input" value={series.name} onChange={(event) => setSeries((current) => ({ ...current, name: event.target.value }))} /></label><label><span>Position</span><input className="text-input" value={series.positionText} onChange={(event) => setSeries((current) => ({ ...current, positionText: event.target.value, positionSort: Number.isFinite(Number(event.target.value)) ? Number(event.target.value) : null }))} /></label></div>
        <button className="primary-button" onClick={() => void run("Series override saved.", () => setBookSeriesOverride(item.entryId, series))} type="button">Save series override</button>
      </section>

      <section className="media-section-card">
        <h4>Tags</h4>
        <div className="media-tag-picker">{allTags.map((tag) => <label key={tag.id}><input checked={selectedTagIds.includes(tag.id)} onChange={(event) => setSelectedTagIds((current) => event.target.checked ? [...current, tag.id] : current.filter((id) => id !== tag.id))} type="checkbox" />{tag.name}</label>)}</div>
        <div className="form-actions"><input aria-label="New book tag" className="text-input" placeholder="New tag" value={tagName} onChange={(event) => setTagName(event.target.value)} /><button className="ghost-button" onClick={() => void addTag()} type="button">Create tag</button><button className="primary-button" disabled={busy} onClick={() => void saveTags()} type="button">Save assigned tags</button></div>
      </section>

      <section className="media-section-card"><h4>Provider records</h4><div className="media-row-badges">{detail.sources.map((source) => <span key={`${source.sourceKey}:${source.entityType}:${source.externalId}`} title={source.lastRefreshError}>{providerName(source.sourceKey)} · {source.entityType} · {source.lastRefreshStatus || "Local"}</span>)}</div></section>
    </div>
  );
}

function ManualEditionForm({ actionLabel, value, onChange, onSave }: { actionLabel: string; value: ManualBookEditionInput; onChange: (value: ManualBookEditionInput) => void; onSave: () => void }) {
  return <div className="media-form-card nested"><div className="media-form-grid"><label><span>Title</span><input className="text-input" value={value.title} onChange={(event) => onChange({ ...value, title: event.target.value })} /></label><label><span>Subtitle</span><input className="text-input" value={value.subtitle} onChange={(event) => onChange({ ...value, subtitle: event.target.value })} /></label><label><span>Format</span><select className="text-input" value={value.format} onChange={(event) => onChange({ ...value, format: event.target.value as BookFormat })}><FormatOptions /></select></label><label><span>ISBN-13</span><input className="text-input" value={value.isbn13} onChange={(event) => onChange({ ...value, isbn13: event.target.value })} /></label><label><span>ISBN-10</span><input className="text-input" value={value.isbn10} onChange={(event) => onChange({ ...value, isbn10: event.target.value })} /></label><label><span>Publisher</span><input className="text-input" value={value.publisher} onChange={(event) => onChange({ ...value, publisher: event.target.value })} /></label><label><span>Published date</span><input className="text-input" value={value.publishedDate} onChange={(event) => onChange({ ...value, publishedDate: event.target.value })} /></label><label><span>Language</span><input className="text-input" value={value.language} onChange={(event) => onChange({ ...value, language: event.target.value })} /></label><label><span>Page count</span><input className="text-input" min="1" type="number" value={value.pageCount ?? ""} onChange={(event) => onChange({ ...value, pageCount: event.target.value ? Number(event.target.value) : null })} /></label><label><span>Audio minutes</span><input className="text-input" min="1" type="number" value={value.audioDurationMinutes ?? ""} onChange={(event) => onChange({ ...value, audioDurationMinutes: event.target.value ? Number(event.target.value) : null })} /></label><label><span>Cover URL</span><input className="text-input" value={value.coverUrl} onChange={(event) => onChange({ ...value, coverUrl: event.target.value })} /></label></div><button className="primary-button" onClick={onSave} type="button">{actionLabel}</button></div>;
}

export function BookStatusOptions() {
  return <><option value="PLANNED">Plan to Read</option><option value="WATCHING">Reading</option><option value="COMPLETED">Read</option><option value="ON_HOLD">On Hold</option><option value="DROPPED">Did Not Finish</option></>;
}

function ProgressUnitOptions() {
  return <><option value="PAGE">Page</option><option value="PERCENT">Percent</option><option value="MINUTE">Audiobook minute</option><option value="CHAPTER">Chapter</option></>;
}

function FormatOptions() {
  return <>{["UNKNOWN","HARDCOVER","PAPERBACK","EBOOK","AUDIOBOOK","OTHER"].map((value) => <option key={value} value={value}>{value.charAt(0) + value.slice(1).toLowerCase()}</option>)}</>;
}

function providerName(source: string) {
  return { google_books: "Google Books", open_library: "Open Library", hardcover: "Hardcover", manual: "Manual" }[source] ?? source;
}
