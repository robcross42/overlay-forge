import { useCallback, useEffect, useMemo, useState } from "react";
import {
  defaultBookFilter,
  getBookLibraryDetail,
  listBookLibrary
} from "../../../services/mediaBooks";
import type {
  BookLibraryDetail,
  BookLibraryFilter,
  BookLibraryItem,
  BookOwnershipStatus
} from "../../../services/mediaBooks";
import type { MediaLibraryStatus, MediaTag } from "../../../services/media";
import { formatUnknownError } from "../../../utils/errors";
import { BookCover } from "./BookCover";
import { BookDetail, BookStatusOptions } from "./BookDetail";
import { bookProgressLabel, bookSubtitle } from "./bookPresentation";

export function BooksView({
  initialEntryId,
  allTags,
  onSearch,
  onTagsChanged,
  onStatus
}: {
  initialEntryId: number | null;
  allTags: MediaTag[];
  onSearch: () => void;
  onTagsChanged: (tags: MediaTag[]) => void;
  onStatus: (status: string, isError?: boolean) => void;
}) {
  const [items, setItems] = useState<BookLibraryItem[]>([]);
  const [filter, setFilter] = useState<BookLibraryFilter>(defaultBookFilter);
  const [selected, setSelected] = useState<BookLibraryDetail | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const next = await listBookLibrary(filter);
      setItems(next);
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setLoading(false);
    }
  }, [filter, onStatus]);

  useEffect(() => {
    const timeout = window.setTimeout(() => void load(), 200);
    return () => window.clearTimeout(timeout);
  }, [load]);

  useEffect(() => {
    if (initialEntryId === null) return;
    setLoading(true);
    getBookLibraryDetail(initialEntryId)
      .then(setSelected)
      .catch((error) => onStatus(formatUnknownError(error), true))
      .finally(() => setLoading(false));
  }, [initialEntryId, onStatus]);

  const shelves = useMemo(() => ({
    readNext: [...items]
      .filter((item) => item.progress.readingQueuePosition !== null)
      .sort((left, right) => (left.progress.readingQueuePosition ?? 0) - (right.progress.readingQueuePosition ?? 0)),
    reading: items.filter((item) => item.libraryStatus === "WATCHING"),
    planned: items.filter((item) => item.libraryStatus === "PLANNED"),
    recentlyRead: [...items]
      .filter((item) => item.libraryStatus === "COMPLETED")
      .sort((left, right) => (right.completedAt || right.progress.lastProgressAt).localeCompare(left.completedAt || left.progress.lastProgressAt))
      .slice(0, 8),
    recentlyAdded: [...items].sort((left, right) => right.addedAt.localeCompare(left.addedAt)).slice(0, 8)
  }), [items]);

  async function open(entryId: number) {
    setLoading(true);
    try {
      setSelected(await getBookLibraryDetail(entryId));
    } catch (error) {
      onStatus(formatUnknownError(error), true);
    } finally {
      setLoading(false);
    }
  }

  function changed(detail: BookLibraryDetail) {
    setSelected(detail);
    void load();
  }

  if (selected) {
    return <BookDetail allTags={allTags} detail={selected} onBack={() => setSelected(null)} onChanged={changed} onDeleted={(entryId) => { setSelected(null); setItems((current) => current.filter((item) => item.entryId !== entryId)); }} onStatus={onStatus} onTagsChanged={onTagsChanged} />;
  }

  return (
    <div className="media-library-view books-view">
      <div className="media-filter-grid book-filter-grid">
        <input aria-label="Search local books" className="text-input media-filter-search" placeholder="Search titles, authors, and notes" value={filter.query} onChange={(event) => setFilter((current) => ({ ...current, query: event.target.value }))} />
        <select aria-label="Book status filter" className="text-input" value={filter.libraryStatus ?? ""} onChange={(event) => setFilter((current) => ({ ...current, libraryStatus: (event.target.value || null) as MediaLibraryStatus | null }))}><option value="">All reading statuses</option><BookStatusOptions /></select>
        <select aria-label="Book ownership filter" className="text-input" value={filter.ownershipStatus ?? ""} onChange={(event) => setFilter((current) => ({ ...current, ownershipStatus: (event.target.value || null) as BookOwnershipStatus | null }))}><option value="">All ownership</option><option value="OWNED">Owned</option><option value="BORROWED">Borrowed</option><option value="WISHLIST">Wishlist</option><option value="NONE">No ownership</option></select>
        <select aria-label="Preferred book format filter" className="text-input" value={filter.preferredFormat ?? ""} onChange={(event) => setFilter((current) => ({ ...current, preferredFormat: (event.target.value || null) as BookLibraryFilter["preferredFormat"] }))}><option value="">All formats</option><option value="HARDCOVER">Hardcover</option><option value="PAPERBACK">Paperback</option><option value="EBOOK">Ebook</option><option value="AUDIOBOOK">Audiobook</option><option value="OTHER">Other</option><option value="UNKNOWN">Unknown</option></select>
        <input aria-label="Book author filter" className="text-input" placeholder="Filter by author" value={filter.authorQuery} onChange={(event) => setFilter((current) => ({ ...current, authorQuery: event.target.value }))} />
        <select aria-label="Book tag filter" className="text-input" value={filter.tagId ?? ""} onChange={(event) => setFilter((current) => ({ ...current, tagId: event.target.value ? Number(event.target.value) : null }))}><option value="">All tags</option>{allTags.map((tag) => <option key={tag.id} value={tag.id}>{tag.name}</option>)}</select>
        <select aria-label="Read Next filter" className="text-input" value={filter.isReadNext === null ? "" : String(filter.isReadNext)} onChange={(event) => setFilter((current) => ({ ...current, isReadNext: event.target.value === "" ? null : event.target.value === "true" }))}><option value="">Queued or not queued</option><option value="true">Read Next only</option><option value="false">Not in Read Next</option></select>
        <select aria-label="Book sort" className="text-input" value={filter.sort} onChange={(event) => setFilter((current) => ({ ...current, sort: event.target.value as BookLibraryFilter["sort"] }))}><option value="RECENTLY_ADDED">Recently added</option><option value="RECENTLY_READ">Recently read</option><option value="TITLE">Title</option><option value="AUTHOR">Author</option><option value="PROGRESS">Recent progress</option><option value="PUBLICATION_DATE">Publication date</option><option value="READ_NEXT">Read Next order</option><option value="PRIORITY">Priority</option></select>
        <button className="ghost-button" onClick={() => setFilter(defaultBookFilter)} type="button">Reset filters</button>
        <button className="primary-button" onClick={onSearch} type="button">Search Books</button>
      </div>

      {items.length === 0 && !loading ? (
        <div className="empty-editor-state media-empty-state"><p>Your local book library is ready.</p><button className="primary-button" onClick={onSearch} type="button">Search book catalogues</button></div>
      ) : (
        <div className="media-home book-home">
          <BookShelf title="Read Next" empty="Add a book to Read Next from its detail page." items={shelves.readNext} onOpen={open} />
          <BookShelf title="Currently Reading" empty="No books are currently marked Reading." items={shelves.reading} onOpen={open} />
          <BookShelf title="Plan to Read" empty="No books are currently planned." items={shelves.planned} onOpen={open} />
          <BookShelf title="Recently Read" empty="No books have been marked Read." items={shelves.recentlyRead} onOpen={open} />
          <BookShelf title="Recently Added" items={shelves.recentlyAdded} onOpen={open} />
          {(filter.query || filter.libraryStatus || filter.ownershipStatus || filter.preferredFormat || filter.authorQuery || filter.tagId || filter.isReadNext !== null) && <BookShelf title="Filtered Books" items={items} onOpen={open} />}
        </div>
      )}
    </div>
  );
}

function BookShelf({ title, items, empty, onOpen }: { title: string; items: BookLibraryItem[]; empty?: string; onOpen: (entryId: number) => void }) {
  return <section className="media-shelf"><h4>{title}</h4>{items.length === 0 ? <p className="media-hint">{empty}</p> : <div className="media-row-list">{items.map((item) => <button className="media-library-row" key={item.entryId} onClick={() => onOpen(item.entryId)} type="button"><BookCover title={item.title} url={item.coverUrl} /><div className="media-library-row-main"><div className="media-card-title-row"><strong>{item.isFavorite && "★ "}{item.title}</strong><span>Book</span></div><span>{bookSubtitle(item) || "Publication details unavailable"} · {item.statusLabel}</span><span>{bookProgressLabel(item.progress)}</span></div><div className="media-row-badges">{item.progress.readingQueuePosition !== null && <span>Read Next #{item.progress.readingQueuePosition}</span>}{item.progress.ownershipStatus !== "NONE" && <span>{item.progress.ownershipStatus}</span>}</div></button>)}</div>}</section>;
}
