# Media Library

## Purpose And Status

Media Library is a shell-hosted, local-first module for movies, episodic series, and books. SQLite is authoritative for personal status, progress, notes, ratings, favourites, tags, ownership, queues, preferred editions, and user-owned links. A missing credential or provider outage must not block local browsing or editing.

The module has five internal sections: Home, Library, Books, Catalogue Search, and Settings. Catalogue Search explicitly switches between Movies & Series and Books.

## Movies And Series

The video subdomain supports:

- Backend-owned TMDB movie and TV search, import, and explicit refresh.
- Manual movie and series entries.
- Movie, episode, season, series, and watched-through progress.
- Specials-aware completion and automatic status transitions.
- Watch Next ordering.
- Canadian watch-provider availability from TMDB's JustWatch partnership.
- User-owned direct streaming links.

Set `TMDB_API_READ_ACCESS_TOKEN` in the Rust backend environment. TMDB image paths are rendered only through `image.tmdb.org`. TMDB availability links are provider information pages, not guaranteed deep links.

Required attribution:

```text
This product uses the TMDB API but is not endorsed or certified by TMDB.
Streaming availability data provided by JustWatch through TMDB.
```

## Books

The Books subdomain models a work separately from its editions. It supports:

- Dedicated Books shelves for Currently Reading, Read Next, Plan to Read, Recently Read, and Recently Added, plus local title/author search and status, format, ownership, author, tag, queue, and favourite filters.
- Continue Reading, Read Next, and Recently Read shelves on the Media Library Home view without mixing books into the video Watch Next queue.
- Google Books primary search and selected-volume metadata.
- Open Library supplemental search, fallback search, and exact-ISBN enrichment.
- Optional read-only Hardcover exact-ISBN enrichment for community rating and series data.
- Manual books and editable manual editions without provider credentials, including format, ownership, rating, favourite, priority, Read Next, language, duration, and series fields.
- Exact provider-identity and ISBN duplicate detection.
- Preferred editions, ownership, preferred format, series overrides, and user-owned links.
- Page, percent, audiobook-minute, and chapter progress.
- Content-aware status labels: Plan to Read, Reading, Read, On Hold, and Did Not Finish.
- A book-only Read Next queue that never uses the video `queue_position` column.

Backend environment variables:

```text
GOOGLE_BOOKS_API_KEY
OPEN_LIBRARY_CONTACT_EMAIL
HARDCOVER_API_TOKEN
```

`OPEN_LIBRARY_CONTACT_EMAIL` identifies Overlay Forge in the Open Library user agent and is not a secret. Google Books and Hardcover credentials remain backend-only and are never stored in SQLite or returned to React. Hardcover access is read-only: Overlay Forge sends queries, never mutations or user reading state.

Google Books public volume requests are bounded to at most 40 results. Open Library requests use documented API endpoints, an identifying user agent, and a one-request-per-second limit without contact information or three requests per second with it. The module does not scrape provider pages.

Book search reuses the Media Library metadata-language setting (default `en-CA`); Google Books receives its compatible two-letter language restriction while locally cached metadata remains usable regardless of provider configuration.

## Book Matching And Refresh

Only exact identities merge automatically, in this order:

1. Stored provider identity.
2. ISBN-13.
3. ISBN-10.
4. Open Library work key.
5. Hardcover book ID.

Title, author, and publication year alone do not auto-merge. Google Books remains the preferred selected-edition source when Google Books and Open Library share an exact ISBN.

Remote fetches finish before SQLite transactions begin. A book import writes the shared title/entry, work, editions, source identities, authors, links, and series rows in one transaction. Refresh may update provider-owned metadata and mark missing editions absent, but it never replaces reading progress, status, dates, ownership, preferred edition, queue order, notes, rating, favourite, tags, series overrides, or user-owned links. Empty provider values do not replace useful cached values. A provider failure updates only that provider's status/error and retains its last successful payload.

## Reading Progress

Book progress rules live in `BookProgressService` and the book domain, not React:

- Positive progress moves Planned to Reading.
- Page and audiobook-minute progress complete only when an effective total is known and reached.
- Percent completes at 100.
- Chapter text never completes automatically.
- Mark Read explicitly completes the book.
- Lowering measurable progress below completion moves Read back to Reading.
- Reset moves Reading or Read to Planned and clears progress.
- Automatic transitions never override On Hold or Did Not Finish.
- An explicitly saved status wins at save time.

The effective total uses a user override first, then the preferred edition's page count or audio duration. Changing a preferred edition does not discard progress.

## Artwork And Links

Video artwork is restricted to TMDB's official image host. Initial book artwork is restricted to HTTPS URLs on `books.google.com`, `books.googleusercontent.com`, and `covers.openlibrary.org`; unapproved, malformed, or failed images fall back to a local placeholder. Hardcover artwork is not allowlisted until a stable official image host is verified.

Book link types are Info, Preview, Read, Borrow, Buy, and Other. Only HTTP(S) URLs are accepted and all opening routes through the centralized external-URL command. Provider-reported availability is informational and may be regional or stale. Overlay Forge does not download ebooks or audiobooks, bypass DRM/authentication/lending controls, automate provider login, or claim that availability is guaranteed in Canada.

## Persistence And Migration

Shared tables:

- `obj_media_title`
- `obj_media_library_entry`
- `obj_media_tag`
- `n2n_media_library_entry_tag`
- `obj_media_setting`

Book tables:

- `obj_media_book_work`
- `obj_media_book_edition`
- `obj_media_book_source_record`
- `obj_media_book_author`
- `n2n_media_book_work_author`
- `obj_media_book_reader_state`
- `obj_media_book_link`
- `obj_media_book_series`
- `obj_media_book_series_member`

The `obj_media_title` migration explicitly rebuilds its SQLite check constraint from `MOVIE`/`SERIES` to `MOVIE`/`SERIES`/`BOOK`. It is idempotent, preserves IDs and all columns, checks foreign keys before commit, and rolls back on failure. Provider-backed books use `source_key='book'`, `external_media_type='book'`, and a null numeric `external_id`; string provider IDs live only in `obj_media_book_source_record`.

Deleting a media entry cascades its locally owned normalized metadata and state. Shared tag definitions remain.

## Current Limitations

There is no Google Bookshelf sync, provider-account sync, background refresh, recommendation engine, book import/export, reading-history log, lending reminders, ebook/audiobook download, embedded reader/player, OCR, or arbitrary provider scraping. The full deferred list is in `docs/DEFERRED_ITEMS.md`.

## Provider References

- Google Books API: <https://developers.google.com/books/docs/v1/using>
- Google Books Volume resource: <https://developers.google.com/books/docs/v1/reference/volumes>
- Open Library APIs and rate limits: <https://openlibrary.org/developers/api>
- Open Library Search API: <https://openlibrary.org/dev/docs/api/search>
- Hardcover API getting started: <https://docs.hardcover.app/api/getting-started/>
- Hardcover current GraphQL schema: <https://github.com/hardcoverapp/hardcover-docs/blob/main/schema.graphql>
