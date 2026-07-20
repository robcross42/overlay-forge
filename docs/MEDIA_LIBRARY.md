# Media Library

## Purpose And Status

Media Library is a top-level, local-first module for indexing movies and episodic series, tracking viewing progress, maintaining a Watch Next queue, and caching Canadian streaming availability.

The initial implementation supports:

- TMDB movie and TV multi-search.
- TMDB movie, series, season, and episode metadata import.
- Manual movie and series entries.
- Local library status, rating, notes, favourite, priority, tags, and queue order.
- Movie watched state and editable watched date.
- Episode, season, and series watched operations.
- Mark watched through an episode.
- Specials-aware progress and automatic status transitions.
- Canadian watch-provider availability from TMDB's JustWatch partnership.
- User-owned direct streaming links.

SQLite is authoritative for all user-owned state. Catalogue outages or a missing credential do not prevent local library browsing and editing.

## Provider And Credential Boundary

The initial metadata provider is TMDB API v3. Rust owns all provider requests through a provider trait and normalized catalogue service. React never calls TMDB and never receives the credential.

Set the backend environment variable:

```text
TMDB_API_READ_ACCESS_TOKEN
```

The token is not stored in SQLite, returned to React, committed in configuration, or written to logs.

Search excludes people and maps TMDB `movie` to `MOVIE` and `tv` to `SERIES`. Module settings default to region `CA`, language `en-CA`, and specials excluded from completion.

## Local Library States

Persisted states are:

```text
PLANNED
WATCHING
COMPLETED
ON_HOLD
DROPPED
```

Automatic progress transitions are centralized in the Rust media progress path:

- Watching a planned series episode moves it to `WATCHING`.
- Watching all included episodes moves a series to `COMPLETED`.
- Unwatching an included episode moves a completed series to `WATCHING`.
- New unwatched episodes found by refresh move a completed series to `WATCHING`.
- Automatic transitions do not override `ON_HOLD` or `DROPPED`.
- An explicit status edit wins at the time it is saved.

Season `0` remains visible. It is excluded from completion unless the setting is enabled.

## Metadata And Availability Refresh

Catalogue additions fetch remote data before starting the SQLite write transaction. A series import then upserts its title, seasons, and episodes in one transaction. Existing episode progress is preserved. Source rows missing from a later response are marked absent instead of deleted.

Refresh is explicit; there is no background scheduler. Provider paths and normalized query fields are cached in SQLite. Poster, backdrop, still, and provider-logo paths are rendered only through the official `image.tmdb.org` host. Artwork failure does not block local data.

Watch-provider child rows are replaced transactionally only after a successful response. A failed availability refresh retains the previous provider rows, records the readable error, and marks the cache stale.

TMDB availability links are not direct streaming deep links. A preferred user-supplied link powers the primary **Watch** action; otherwise the module opens TMDB's availability page. Only `http` and `https` links are accepted.

Required attribution:

```text
This product uses the TMDB API but is not endorsed or certified by TMDB.
```

```text
Streaming availability data provided by JustWatch through TMDB.
```

## Persistence

Media Library uses:

- `obj_media_title`
- `obj_media_library_entry`
- `obj_media_season`
- `obj_media_episode`
- `obj_media_episode_progress`
- `obj_media_provider_snapshot`
- `obj_media_provider_availability`
- `obj_media_streaming_link`
- `obj_media_tag`
- `n2n_media_library_entry_tag`
- `obj_media_setting`

Migrations are idempotent and non-destructive. Deleting a library entry removes its locally owned title metadata, seasons, episodes, progress, provider cache, links, and tag mappings. Shared tag definitions remain.

## Current Limitations

Manual series use series-level status only; a manual season and episode editor is deferred. Partial playback, rewatch history, imports, exports, recommendations, account sync, background refresh, deep provider integrations, notifications, and service scraping are also deferred.
