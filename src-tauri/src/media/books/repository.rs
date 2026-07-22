use super::domain::{
    automatic_book_status, effective_progress_total, normalize_isbn, validate_progress,
    BookAuthorRecord, BookEditionRecord, BookFormat, BookLibraryDetail, BookLibraryFilter,
    BookLibraryItem, BookLinkInput, BookLinkRecord, BookMediaDetailSummary, BookMediaSummary,
    BookOwnershipStatus, BookProgressSummary, BookProgressUnit, BookProviderFailure,
    BookSeriesOverrideInput, BookSeriesRecord, BookSourceRecord, CreateManualBookInput,
    ManualBookEditionInput, NormalizedBookEdition, NormalizedBookImport, NormalizedBookSeries,
    SetBookProgressInput, UpdateBookReaderStateInput,
};
use crate::db::AppDatabase;
use crate::media::domain::{
    validate_http_url, validate_rating, MediaError, MediaLibraryStatus, MediaTagRecord,
};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult, Transaction};
use std::str::FromStr;

pub(crate) fn migrate_schema(connection: &Connection) -> SqlResult<()> {
    migrate_media_title_book_constraint(connection)?;
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS obj_media_book_work (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL UNIQUE,
            subtitle TEXT NOT NULL DEFAULT '',
            primary_author_text TEXT NOT NULL DEFAULT '',
            first_publish_year INTEGER,
            subjects_json TEXT NOT NULL DEFAULT '[]',
            canonical_cover_url TEXT NOT NULL DEFAULT '',
            community_rating REAL,
            community_rating_count INTEGER,
            metadata_refreshed_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_book_edition (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            subtitle TEXT NOT NULL DEFAULT '',
            format TEXT NOT NULL DEFAULT 'UNKNOWN'
                CHECK (format IN ('HARDCOVER','PAPERBACK','EBOOK','AUDIOBOOK','OTHER','UNKNOWN')),
            isbn_10 TEXT NOT NULL DEFAULT '',
            isbn_13 TEXT NOT NULL DEFAULT '',
            publisher TEXT NOT NULL DEFAULT '',
            published_date TEXT NOT NULL DEFAULT '',
            language TEXT NOT NULL DEFAULT '',
            page_count INTEGER CHECK (page_count IS NULL OR page_count > 0),
            audio_duration_minutes INTEGER
                CHECK (audio_duration_minutes IS NULL OR audio_duration_minutes > 0),
            cover_url TEXT NOT NULL DEFAULT '',
            is_ebook INTEGER NOT NULL DEFAULT 0,
            access_viewability TEXT NOT NULL DEFAULT '',
            is_present_in_source INTEGER NOT NULL DEFAULT 1,
            metadata_refreshed_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_book_edition_isbn13_unique
            ON obj_media_book_edition (isbn_13) WHERE isbn_13 <> '';
        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_book_edition_isbn10_unique
            ON obj_media_book_edition (isbn_10) WHERE isbn_10 <> '';

        CREATE TABLE IF NOT EXISTS obj_media_book_source_record (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            edition_id INTEGER,
            source_key TEXT NOT NULL
                CHECK (source_key IN ('google_books','open_library','hardcover','manual')),
            entity_type TEXT NOT NULL CHECK (entity_type IN ('WORK','EDITION','BOOK')),
            external_id TEXT NOT NULL,
            source_url TEXT NOT NULL DEFAULT '',
            metadata_json TEXT NOT NULL DEFAULT '{}',
            refreshed_at TEXT NOT NULL DEFAULT '',
            last_refresh_status TEXT NOT NULL DEFAULT '',
            last_refresh_error TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (source_key, entity_type, external_id),
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE,
            FOREIGN KEY (edition_id) REFERENCES obj_media_book_edition(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS obj_media_book_author (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort_name TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS n2n_media_book_work_author (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            author_id INTEGER NOT NULL,
            role TEXT NOT NULL DEFAULT 'AUTHOR'
                CHECK (role IN ('AUTHOR','EDITOR','NARRATOR','TRANSLATOR','OTHER')),
            position INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (media_title_id, author_id, role),
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE,
            FOREIGN KEY (author_id) REFERENCES obj_media_book_author(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_book_reader_state (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_library_entry_id INTEGER NOT NULL UNIQUE,
            preferred_edition_id INTEGER,
            ownership_status TEXT NOT NULL DEFAULT 'NONE'
                CHECK (ownership_status IN ('NONE','OWNED','BORROWED','WISHLIST')),
            preferred_format TEXT NOT NULL DEFAULT 'UNKNOWN'
                CHECK (preferred_format IN ('HARDCOVER','PAPERBACK','EBOOK','AUDIOBOOK','OTHER','UNKNOWN')),
            progress_unit TEXT NOT NULL DEFAULT 'PAGE'
                CHECK (progress_unit IN ('PAGE','PERCENT','MINUTE','CHAPTER')),
            progress_value REAL NOT NULL DEFAULT 0 CHECK (progress_value >= 0),
            progress_total_override REAL CHECK (
                progress_total_override IS NULL OR progress_total_override >= 0
            ),
            current_chapter TEXT NOT NULL DEFAULT '',
            reading_queue_position INTEGER UNIQUE,
            last_progress_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (media_library_entry_id)
                REFERENCES obj_media_library_entry(id) ON DELETE CASCADE,
            FOREIGN KEY (preferred_edition_id)
                REFERENCES obj_media_book_edition(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS obj_media_book_link (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            edition_id INTEGER,
            source_key TEXT NOT NULL DEFAULT 'manual',
            link_type TEXT NOT NULL CHECK (link_type IN ('INFO','PREVIEW','READ','BORROW','BUY','OTHER')),
            url TEXT NOT NULL,
            region_code TEXT NOT NULL DEFAULT '',
            is_user_owned INTEGER NOT NULL DEFAULT 0,
            is_preferred INTEGER NOT NULL DEFAULT 0,
            refreshed_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE,
            FOREIGN KEY (edition_id) REFERENCES obj_media_book_edition(id) ON DELETE SET NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_book_link_preferred_user
            ON obj_media_book_link (media_title_id)
            WHERE is_user_owned = 1 AND is_preferred = 1;

        CREATE TABLE IF NOT EXISTS obj_media_book_series (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL COLLATE NOCASE UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS obj_media_book_series_member (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            series_id INTEGER NOT NULL,
            media_title_id INTEGER NOT NULL,
            position_text TEXT NOT NULL DEFAULT '',
            position_sort REAL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            source_key TEXT NOT NULL DEFAULT 'manual',
            is_user_override INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (series_id, media_title_id, source_key),
            FOREIGN KEY (series_id) REFERENCES obj_media_book_series(id) ON DELETE CASCADE,
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_media_book_title_type
            ON obj_media_title (content_type, title COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_media_book_author_name
            ON obj_media_book_author (name COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_media_book_source_identity
            ON obj_media_book_source_record (source_key, entity_type, external_id);
        CREATE INDEX IF NOT EXISTS idx_media_book_reader_preferred
            ON obj_media_book_reader_state (preferred_edition_id);
        CREATE INDEX IF NOT EXISTS idx_media_book_reading_queue
            ON obj_media_book_reader_state (reading_queue_position);
        CREATE INDEX IF NOT EXISTS idx_media_book_progress_timestamp
            ON obj_media_book_reader_state (last_progress_at);
        CREATE INDEX IF NOT EXISTS idx_media_book_series_order
            ON obj_media_book_series_member (series_id, position_sort, position_text);
        CREATE INDEX IF NOT EXISTS idx_media_book_series_title
            ON obj_media_book_series_member (media_title_id);
        CREATE INDEX IF NOT EXISTS idx_media_book_link_lookup
            ON obj_media_book_link (media_title_id, link_type);
        ",
    )?;
    Ok(())
}

pub(crate) fn load_media_book_summary(
    connection: &Connection,
    entry_id: i64,
) -> Result<Option<BookMediaSummary>, MediaError> {
    let raw = connection
        .query_row(
            &format!("{} AND entry.id=?1", book_item_select()),
            [entry_id],
            raw_book_item_from_row,
        )
        .optional()?;
    let Some(item) = raw.map(book_item_from_raw).transpose()? else {
        return Ok(None);
    };
    let primary_series_text = connection
        .query_row(
            "
            SELECT series.name || CASE
                WHEN member.position_text <> '' THEN ' #' || member.position_text ELSE '' END
            FROM obj_media_book_series_member member
            JOIN obj_media_book_series series ON series.id=member.series_id
            WHERE member.media_title_id=?1
            ORDER BY member.is_user_override DESC, member.is_primary DESC, member.id
            LIMIT 1
            ",
            [item.media_title_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_default();
    Ok(Some(BookMediaSummary {
        progress: item.progress,
        primary_author_text: item.primary_author_text,
        primary_series_text,
    }))
}

pub(crate) fn load_media_book_detail(
    connection: &Connection,
    entry_id: i64,
    title_id: i64,
) -> Result<Option<BookMediaDetailSummary>, MediaError> {
    let Some(summary) = load_media_book_summary(connection, entry_id)? else {
        return Ok(None);
    };
    let progress = summary.progress;
    let preferred_edition = if let Some(edition_id) = progress.preferred_edition_id {
        load_editions(connection, title_id)?
            .into_iter()
            .find(|edition| edition.id == edition_id)
    } else {
        None
    };
    Ok(Some(BookMediaDetailSummary {
        progress,
        preferred_edition,
        authors: load_authors(connection, title_id)?,
    }))
}

fn migrate_media_title_book_constraint(connection: &Connection) -> SqlResult<()> {
    let table_sql: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'obj_media_title'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let Some(table_sql) = table_sql else {
        return Ok(());
    };
    if table_sql.to_ascii_uppercase().contains("'BOOK'") {
        return Ok(());
    }

    connection.execute_batch("PRAGMA foreign_keys = OFF;")?;
    if let Err(error) = connection.execute_batch("BEGIN IMMEDIATE;") {
        let _ = connection.execute_batch("PRAGMA foreign_keys = ON;");
        return Err(error);
    }
    let migration_result = connection.execute_batch(
        "
        CREATE TABLE obj_media_title_book_migration (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_key TEXT NOT NULL,
            external_id INTEGER,
            external_media_type TEXT,
            content_type TEXT NOT NULL CHECK (content_type IN ('MOVIE', 'SERIES', 'BOOK')),
            title TEXT NOT NULL,
            original_title TEXT NOT NULL DEFAULT '',
            overview TEXT NOT NULL DEFAULT '',
            original_language TEXT NOT NULL DEFAULT '',
            release_date TEXT NOT NULL DEFAULT '',
            first_air_date TEXT NOT NULL DEFAULT '',
            last_air_date TEXT NOT NULL DEFAULT '',
            runtime_minutes INTEGER,
            episode_runtime_minutes INTEGER,
            external_status TEXT NOT NULL DEFAULT '',
            series_type TEXT NOT NULL DEFAULT '',
            poster_path TEXT NOT NULL DEFAULT '',
            backdrop_path TEXT NOT NULL DEFAULT '',
            homepage_url TEXT NOT NULL DEFAULT '',
            total_seasons INTEGER,
            total_episodes INTEGER,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            metadata_refreshed_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (source_key, external_media_type, external_id)
        );
        INSERT INTO obj_media_title_book_migration (
            id, source_key, external_id, external_media_type, content_type, title,
            original_title, overview, original_language, release_date, first_air_date,
            last_air_date, runtime_minutes, episode_runtime_minutes, external_status,
            series_type, poster_path, backdrop_path, homepage_url, total_seasons,
            total_episodes, metadata_json, metadata_refreshed_at, created_at, updated_at
        )
        SELECT
            id, source_key, external_id, external_media_type, content_type, title,
            original_title, overview, original_language, release_date, first_air_date,
            last_air_date, runtime_minutes, episode_runtime_minutes, external_status,
            series_type, poster_path, backdrop_path, homepage_url, total_seasons,
            total_episodes, metadata_json, metadata_refreshed_at, created_at, updated_at
        FROM obj_media_title;
        DROP TABLE obj_media_title;
        ALTER TABLE obj_media_title_book_migration RENAME TO obj_media_title;
        CREATE INDEX IF NOT EXISTS idx_media_title_search
            ON obj_media_title (title COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_media_title_source
            ON obj_media_title (source_key, external_media_type, external_id);
        ",
    );
    if let Err(error) = migration_result {
        let _ = connection.execute_batch("ROLLBACK; PRAGMA foreign_keys = ON;");
        return Err(error);
    }
    let violation: Option<i64> = match connection
        .query_row(
            "SELECT 1 FROM pragma_foreign_key_check LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(violation) => violation,
        Err(error) => {
            let _ = connection.execute_batch("ROLLBACK; PRAGMA foreign_keys = ON;");
            return Err(error);
        }
    };
    if violation.is_some() {
        let _ = connection.execute_batch("ROLLBACK; PRAGMA foreign_keys = ON;");
        return Err(rusqlite::Error::InvalidQuery);
    }
    if let Err(error) = connection.execute_batch("COMMIT;") {
        let _ = connection.execute_batch("ROLLBACK; PRAGMA foreign_keys = ON;");
        return Err(error);
    }
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

pub struct BookRepository<'a> {
    database: &'a AppDatabase,
}

impl<'a> BookRepository<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self { database }
    }

    pub(crate) fn database(&self) -> &'a AppDatabase {
        self.database
    }

    pub fn find_existing_candidate(
        &self,
        source_key: &str,
        external_id: &str,
        isbn_10: &str,
        isbn_13: &str,
    ) -> Result<Option<i64>, MediaError> {
        let connection = self.database.connection()?;
        find_existing(&connection, source_key, external_id, isbn_10, isbn_13)
    }

    pub fn add_import(
        &self,
        import: &NormalizedBookImport,
        status: MediaLibraryStatus,
        provider_failures: &[BookProviderFailure],
    ) -> Result<BookLibraryDetail, MediaError> {
        validate_import(import)?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        if let Some(entry_id) = find_import_duplicate(&transaction, import)? {
            let title_id = book_title_id(&transaction, entry_id)?;
            record_provider_failures(&transaction, title_id, provider_failures)?;
            transaction.commit()?;
            drop(connection);
            return self.get_detail_with_warnings(
                entry_id,
                provider_failures
                    .iter()
                    .map(BookProviderFailure::warning_message)
                    .collect(),
            );
        }

        transaction.execute(
            "
            INSERT INTO obj_media_title (
                source_key, external_media_type, content_type, title, original_title,
                overview, release_date, poster_path, homepage_url, metadata_refreshed_at
            ) VALUES ('book', 'book', 'BOOK', ?1, ?1, ?2, ?3, ?4, ?5, datetime('now','localtime'))
            ",
            params![
                import.title.trim(),
                import.description,
                import
                    .first_publish_year
                    .map(|year| year.to_string())
                    .unwrap_or_default(),
                import.canonical_cover_url,
                import
                    .links
                    .first()
                    .map(|link| link.url.as_str())
                    .unwrap_or_default(),
            ],
        )?;
        let title_id = transaction.last_insert_rowid();
        transaction.execute(
            "INSERT INTO obj_media_library_entry (media_title_id, library_status) VALUES (?1, ?2)",
            params![title_id, status.to_string()],
        )?;
        let entry_id = transaction.last_insert_rowid();
        insert_work(&transaction, title_id, import)?;
        let edition_ids = insert_editions(&transaction, title_id, &import.editions)?;
        transaction.execute(
            "
            INSERT INTO obj_media_book_reader_state (
                media_library_entry_id, preferred_edition_id, preferred_format
            ) VALUES (?1, ?2, ?3)
            ",
            params![
                entry_id,
                edition_ids.first(),
                import
                    .editions
                    .first()
                    .map(|edition| edition.format.to_string())
                    .unwrap_or_else(|| BookFormat::Unknown.to_string()),
            ],
        )?;
        insert_sources(&transaction, title_id, &edition_ids, import)?;
        record_provider_failures(&transaction, title_id, provider_failures)?;
        insert_authors(&transaction, title_id, &import.authors)?;
        insert_provider_links(&transaction, title_id, &edition_ids, import)?;
        insert_series(&transaction, title_id, import)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail_with_warnings(
            entry_id,
            provider_failures
                .iter()
                .map(BookProviderFailure::warning_message)
                .collect(),
        )
    }

    pub fn create_manual(
        &self,
        input: &CreateManualBookInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        if input.title.trim().is_empty() {
            return Err(MediaError::validation("A book title is required."));
        }
        if input.page_count.is_some_and(|value| value <= 0) {
            return Err(MediaError::validation("Page count must be positive."));
        }
        if input.audio_duration_minutes.is_some_and(|value| value <= 0) {
            return Err(MediaError::validation(
                "Audiobook duration must be positive.",
            ));
        }
        validate_rating(input.personal_rating)?;
        let isbn_10 = normalize_optional_isbn(&input.isbn_10, 10)?;
        let isbn_13 = normalize_optional_isbn(&input.isbn_13, 13)?;
        let series_position_sort = input.series_position.trim().parse::<f64>().ok();
        let import = NormalizedBookImport {
            title: input.title.trim().to_string(),
            subtitle: input.subtitle.trim().to_string(),
            description: input.description.trim().to_string(),
            authors: input.authors.clone(),
            first_publish_year: input.first_publish_year,
            subjects: Vec::new(),
            canonical_cover_url: validate_optional_url(&input.cover_url)?,
            community_rating: None,
            community_rating_count: None,
            editions: vec![NormalizedBookEdition {
                title: input.title.trim().to_string(),
                subtitle: input.subtitle.trim().to_string(),
                format: input.preferred_format,
                isbn_10,
                isbn_13,
                publisher: input.publisher.trim().to_string(),
                published_date: input.published_date.trim().to_string(),
                language: input.language.trim().to_string(),
                page_count: input.page_count,
                audio_duration_minutes: input.audio_duration_minutes,
                cover_url: validate_optional_url(&input.cover_url)?,
                is_ebook: input.preferred_format == BookFormat::Ebook,
                access_viewability: String::new(),
            }],
            sources: Vec::new(),
            links: Vec::new(),
            series: (!input.series_name.trim().is_empty())
                .then(|| NormalizedBookSeries {
                    name: input.series_name.trim().to_string(),
                    position_text: input.series_position.trim().to_string(),
                    position_sort: series_position_sort,
                    source_key: "manual".to_string(),
                })
                .into_iter()
                .collect(),
        };
        let detail = self.add_manual_import(&import, input)?;
        Ok(detail)
    }

    fn add_manual_import(
        &self,
        import: &NormalizedBookImport,
        input: &CreateManualBookInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        validate_import(import)?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        if let Some(entry_id) = find_import_duplicate(&transaction, import)? {
            transaction.commit()?;
            drop(connection);
            return self.get_detail(entry_id);
        }
        transaction.execute(
            "
            INSERT INTO obj_media_title (
                source_key, external_media_type, content_type, title, original_title,
                overview, release_date, poster_path
            ) VALUES ('manual', 'book', 'BOOK', ?1, ?1, ?2, ?3, ?4)
            ",
            params![
                import.title,
                import.description,
                import
                    .first_publish_year
                    .map(|year| year.to_string())
                    .unwrap_or_default(),
                import.canonical_cover_url,
            ],
        )?;
        let title_id = transaction.last_insert_rowid();
        transaction.execute(
            "
            INSERT INTO obj_media_library_entry (
                media_title_id, library_status, is_favorite, personal_rating, notes, priority,
                started_at, completed_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                CASE WHEN ?2 IN ('WATCHING','COMPLETED') THEN datetime('now','localtime') ELSE '' END,
                CASE WHEN ?2 = 'COMPLETED' THEN datetime('now','localtime') ELSE '' END
            )
            ",
            params![
                title_id,
                input.library_status.to_string(),
                input.is_favorite as i64,
                input.personal_rating,
                input.notes.trim(),
                input.priority,
            ],
        )?;
        let entry_id = transaction.last_insert_rowid();
        insert_work(&transaction, title_id, import)?;
        let edition_ids = insert_editions(&transaction, title_id, &import.editions)?;
        transaction.execute(
            "
            INSERT INTO obj_media_book_reader_state (
                media_library_entry_id, preferred_edition_id, ownership_status,
                preferred_format, reading_queue_position
            ) VALUES (
                ?1, ?2, ?3, ?4,
                CASE WHEN ?5 = 1 THEN (
                    SELECT COALESCE(MAX(reading_queue_position), 0) + 1
                    FROM obj_media_book_reader_state
                ) ELSE NULL END
            )
            ",
            params![
                entry_id,
                edition_ids.first(),
                input.ownership_status.to_string(),
                input.preferred_format.to_string(),
                input.is_read_next as i64,
            ],
        )?;
        insert_authors(&transaction, title_id, &import.authors)?;
        insert_series(&transaction, title_id, import)?;
        transaction.execute(
            "
            INSERT INTO obj_media_book_source_record (
                media_title_id, edition_id, source_key, entity_type, external_id,
                refreshed_at, last_refresh_status
            ) VALUES (?1, ?2, 'manual', 'WORK', ?3, datetime('now','localtime'), 'SUCCESS')
            ",
            params![title_id, edition_ids.first(), format!("local-{title_id}")],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn list(&self, filter: &BookLibraryFilter) -> Result<Vec<BookLibraryItem>, MediaError> {
        let connection = self.database.connection()?;
        let direction = if filter.sort_direction.eq_ignore_ascii_case("ASC") {
            "ASC"
        } else {
            "DESC"
        };
        let order = match filter.sort.trim().to_ascii_uppercase().as_str() {
            "TITLE" => format!("title.title COLLATE NOCASE {direction}"),
            "AUTHOR" => format!("work.primary_author_text COLLATE NOCASE {direction}"),
            "PROGRESS" => format!("reader.last_progress_at {direction}"),
            "RECENTLY_READ" => format!("entry.completed_at {direction}"),
            "PUBLICATION_DATE" => format!("work.first_publish_year {direction}"),
            "READ_NEXT" => {
                "reader.reading_queue_position IS NULL, reader.reading_queue_position".to_string()
            }
            "PRIORITY" => format!("entry.priority {direction}"),
            _ => format!("entry.added_at {direction}"),
        };
        let sql = format!("{} ORDER BY {order}", book_item_select());
        let mut statement = connection.prepare(&sql)?;
        let raw = statement
            .query_map([], raw_book_item_from_row)?
            .collect::<SqlResult<Vec<_>>>()?;
        let query = filter.query.trim().to_ascii_lowercase();
        let author_query = filter.author_query.trim().to_ascii_lowercase();
        let mut items = Vec::new();
        for raw_item in raw {
            let item = book_item_from_raw(raw_item)?;
            let matches_fields = (query.is_empty()
                || item.title.to_ascii_lowercase().contains(&query)
                || item.subtitle.to_ascii_lowercase().contains(&query)
                || item
                    .primary_author_text
                    .to_ascii_lowercase()
                    .contains(&query)
                || item.notes.to_ascii_lowercase().contains(&query))
                && (author_query.is_empty()
                    || item
                        .primary_author_text
                        .to_ascii_lowercase()
                        .contains(&author_query))
                && filter
                    .library_status
                    .is_none_or(|status| item.library_status == status)
                && filter
                    .ownership_status
                    .is_none_or(|ownership| item.progress.ownership_status == ownership)
                && filter
                    .preferred_format
                    .is_none_or(|format| item.progress.preferred_format == format)
                && filter
                    .is_favorite
                    .is_none_or(|favorite| item.is_favorite == favorite)
                && filter
                    .is_read_next
                    .is_none_or(|queued| item.progress.reading_queue_position.is_some() == queued);
            if !matches_fields {
                continue;
            }
            let matches_tag = if let Some(tag_id) = filter.tag_id {
                connection.query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM n2n_media_library_entry_tag
                        WHERE media_library_entry_id=?1 AND media_tag_id=?2
                    )",
                    params![item.entry_id, tag_id],
                    |row| Ok(row.get::<_, i64>(0)? != 0),
                )?
            } else {
                true
            };
            if matches_tag {
                items.push(item);
            }
        }
        Ok(items)
    }

    pub fn get_detail(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        self.get_detail_with_warnings(entry_id, Vec::new())
    }

    fn get_detail_with_warnings(
        &self,
        entry_id: i64,
        provider_warnings: Vec<String>,
    ) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let raw = connection
            .query_row(
                &format!("{} AND entry.id = ?1", book_item_select()),
                [entry_id],
                raw_book_item_from_row,
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Book library entry was not found."))?;
        let item = book_item_from_raw(raw)?;
        let editions = load_editions(&connection, item.media_title_id)?;
        let authors = load_authors(&connection, item.media_title_id)?;
        let links = load_links(&connection, item.media_title_id)?;
        let series = load_series(&connection, item.media_title_id)?;
        let sources = load_sources(&connection, item.media_title_id)?;
        let tags = load_tags(&connection, item.entry_id)?;
        let (subjects_json, community_rating, community_rating_count): (String, Option<f64>, Option<i64>) =
            connection.query_row(
                "SELECT subjects_json, community_rating, community_rating_count FROM obj_media_book_work WHERE media_title_id = ?1",
                [item.media_title_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
        let subjects = serde_json::from_str(&subjects_json).unwrap_or_default();
        Ok(BookLibraryDetail {
            item,
            editions,
            authors,
            links,
            series,
            sources,
            subjects,
            community_rating,
            community_rating_count,
            provider_warnings,
            tags,
        })
    }
}

impl<'a> BookRepository<'a> {
    pub fn refresh_import(
        &self,
        entry_id: i64,
        import: &NormalizedBookImport,
        provider_failures: &[BookProviderFailure],
    ) -> Result<BookLibraryDetail, MediaError> {
        validate_import(import)?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = book_title_id(&transaction, entry_id)?;
        transaction.execute(
            "
            UPDATE obj_media_title SET
                title = CASE WHEN ?2 <> '' THEN ?2 ELSE title END,
                original_title = CASE WHEN ?2 <> '' THEN ?2 ELSE original_title END,
                overview = CASE WHEN ?3 <> '' THEN ?3 ELSE overview END,
                poster_path = CASE WHEN ?4 <> '' THEN ?4 ELSE poster_path END,
                metadata_refreshed_at = datetime('now','localtime'),
                updated_at = datetime('now','localtime')
            WHERE id = ?1 AND content_type = 'BOOK'
            ",
            params![
                title_id,
                import.title,
                import.description,
                import.canonical_cover_url
            ],
        )?;
        transaction.execute(
            "
            UPDATE obj_media_book_work SET
                subtitle = CASE WHEN ?2 <> '' THEN ?2 ELSE subtitle END,
                primary_author_text = CASE WHEN ?3 <> '' THEN ?3 ELSE primary_author_text END,
                first_publish_year = COALESCE(?4, first_publish_year),
                subjects_json = CASE WHEN ?5 <> '[]' THEN ?5 ELSE subjects_json END,
                canonical_cover_url = CASE WHEN ?6 <> '' THEN ?6 ELSE canonical_cover_url END,
                community_rating = COALESCE(?7, community_rating),
                community_rating_count = COALESCE(?8, community_rating_count),
                metadata_refreshed_at = datetime('now','localtime'),
                updated_at = datetime('now','localtime')
            WHERE media_title_id = ?1
            ",
            params![
                title_id,
                import.subtitle,
                import.authors.join(", "),
                import.first_publish_year,
                serde_json::to_string(&import.subjects).unwrap_or_else(|_| "[]".to_string()),
                import.canonical_cover_url,
                import.community_rating,
                import.community_rating_count,
            ],
        )?;

        transaction.execute(
            "
            UPDATE obj_media_book_edition SET is_present_in_source = 0
            WHERE media_title_id = ?1
              AND EXISTS (
                SELECT 1 FROM obj_media_book_source_record source
                WHERE source.edition_id=obj_media_book_edition.id
                  AND source.source_key <> 'manual'
              )
            ",
            [title_id],
        )?;
        let edition_ids = upsert_editions(&transaction, title_id, &import.editions)?;
        insert_sources(&transaction, title_id, &edition_ids, import)?;
        record_provider_failures(&transaction, title_id, provider_failures)?;
        insert_authors(&transaction, title_id, &import.authors)?;
        for source in import
            .sources
            .iter()
            .map(|source| source.source_key.as_str())
            .collect::<std::collections::HashSet<_>>()
        {
            transaction.execute(
                "DELETE FROM obj_media_book_link WHERE media_title_id = ?1 AND source_key = ?2 AND is_user_owned = 0",
                params![title_id, source],
            )?;
        }
        insert_provider_links(&transaction, title_id, &edition_ids, import)?;
        insert_series(&transaction, title_id, import)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail_with_warnings(
            entry_id,
            provider_failures
                .iter()
                .map(BookProviderFailure::warning_message)
                .collect(),
        )
    }

    pub fn update_reader_state(
        &self,
        entry_id: i64,
        input: &UpdateBookReaderStateInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let changed = connection.execute(
            "
            UPDATE obj_media_book_reader_state SET
                ownership_status = ?2,
                preferred_format = ?3,
                updated_at = datetime('now','localtime')
            WHERE media_library_entry_id = ?1
            ",
            params![
                entry_id,
                input.ownership_status.to_string(),
                input.preferred_format.to_string(),
            ],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Book library entry was not found."));
        }
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn set_preferred_edition(
        &self,
        entry_id: i64,
        edition_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let context: Option<(String, f64, Option<f64>, Option<i64>)> = connection
            .query_row(
                "
                SELECT reader.progress_unit,reader.progress_value,
                       reader.progress_total_override,edition.page_count
                FROM obj_media_book_reader_state reader
                JOIN obj_media_library_entry entry
                    ON entry.id=reader.media_library_entry_id
                JOIN obj_media_book_edition edition
                    ON edition.id=?2 AND edition.media_title_id=entry.media_title_id
                WHERE reader.media_library_entry_id=?1
                ",
                params![entry_id, edition_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let Some((progress_unit, progress_value, total_override, page_count)) = context else {
            return Err(MediaError::validation(
                "The preferred edition must belong to this book.",
            ));
        };
        let changed = connection.execute(
            "
            UPDATE obj_media_book_reader_state
            SET preferred_edition_id = ?2, updated_at = datetime('now','localtime')
            WHERE media_library_entry_id = ?1
              AND EXISTS (
                SELECT 1
                FROM obj_media_book_edition edition
                JOIN obj_media_library_entry entry ON entry.media_title_id = edition.media_title_id
                WHERE edition.id = ?2 AND entry.id = ?1
              )
            ",
            params![entry_id, edition_id],
        )?;
        if changed == 0 {
            return Err(MediaError::validation(
                "The preferred edition must belong to this book.",
            ));
        }
        let warnings = if progress_unit == "PAGE"
            && total_override.is_none()
            && page_count.is_some_and(|total| progress_value > total as f64)
        {
            vec![format!(
                "Current page {progress_value} exceeds this edition's {page_count} pages.",
                page_count = page_count.unwrap_or_default()
            )]
        } else {
            Vec::new()
        };
        drop(connection);
        self.get_detail_with_warnings(entry_id, warnings)
    }

    pub fn set_progress(
        &self,
        entry_id: i64,
        input: &SetBookProgressInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        validate_progress(
            input.progress_unit,
            input.progress_value,
            input.progress_total_override,
        )?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let (status_text, page_count, audio_minutes): (String, Option<i64>, Option<i64>) =
            transaction
                .query_row(
                    "
                    SELECT entry.library_status, edition.page_count, edition.audio_duration_minutes
                    FROM obj_media_library_entry entry
                    JOIN obj_media_book_reader_state reader ON reader.media_library_entry_id = entry.id
                    LEFT JOIN obj_media_book_edition edition ON edition.id = reader.preferred_edition_id
                    WHERE entry.id = ?1
                    ",
                    [entry_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?
                .ok_or_else(|| MediaError::not_found("Book library entry was not found."))?;
        let current_status = MediaLibraryStatus::from_str(&status_text)?;
        let effective_total = effective_progress_total(
            input.progress_unit,
            input.progress_total_override,
            page_count,
            audio_minutes,
        );
        let status = automatic_book_status(
            current_status,
            input.progress_unit,
            input.progress_value,
            effective_total,
        );
        transaction.execute(
            "
            UPDATE obj_media_book_reader_state SET
                progress_unit = ?2,
                progress_value = ?3,
                progress_total_override = ?4,
                current_chapter = ?5,
                last_progress_at = datetime('now','localtime'),
                updated_at = datetime('now','localtime')
            WHERE media_library_entry_id = ?1
            ",
            params![
                entry_id,
                input.progress_unit.to_string(),
                input.progress_value,
                input.progress_total_override,
                input.current_chapter.trim(),
            ],
        )?;
        update_entry_for_progress(&transaction, entry_id, status, input.progress_value)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn mark_read(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let (unit_text, total_override, pages, minutes): (
            String,
            Option<f64>,
            Option<i64>,
            Option<i64>,
        ) = transaction
            .query_row(
                "
                SELECT reader.progress_unit, reader.progress_total_override,
                       edition.page_count, edition.audio_duration_minutes
                FROM obj_media_book_reader_state reader
                LEFT JOIN obj_media_book_edition edition ON edition.id = reader.preferred_edition_id
                WHERE reader.media_library_entry_id = ?1
                ",
                [entry_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Book library entry was not found."))?;
        let unit = BookProgressUnit::from_str(&unit_text)?;
        let value = effective_progress_total(unit, total_override, pages, minutes)
            .filter(|_| unit != BookProgressUnit::Chapter)
            .unwrap_or_else(|| {
                if unit == BookProgressUnit::Percent {
                    100.0
                } else {
                    0.0
                }
            });
        transaction.execute(
            "
            UPDATE obj_media_book_reader_state SET progress_value = ?2,
                last_progress_at = datetime('now','localtime'),
                updated_at = datetime('now','localtime')
            WHERE media_library_entry_id = ?1
            ",
            params![entry_id, value],
        )?;
        transaction.execute(
            "
            UPDATE obj_media_library_entry SET library_status = 'COMPLETED',
                started_at = CASE WHEN started_at = '' THEN datetime('now','localtime') ELSE started_at END,
                completed_at = datetime('now','localtime'),
                updated_at = datetime('now','localtime')
            WHERE id = ?1
            ",
            [entry_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn reset_progress(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "
            UPDATE obj_media_book_reader_state SET progress_value = 0,
                current_chapter = '', last_progress_at = '',
                updated_at = datetime('now','localtime')
            WHERE media_library_entry_id = ?1
            ",
            [entry_id],
        )?;
        transaction.execute(
            "
            UPDATE obj_media_library_entry SET
                library_status = CASE
                    WHEN library_status IN ('ON_HOLD','DROPPED') THEN library_status ELSE 'PLANNED' END,
                started_at = CASE
                    WHEN library_status IN ('ON_HOLD','DROPPED') THEN started_at ELSE '' END,
                completed_at = CASE
                    WHEN library_status IN ('ON_HOLD','DROPPED') THEN completed_at ELSE '' END,
                updated_at = datetime('now','localtime')
            WHERE id = ?1
            ",
            [entry_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn add_to_read_next(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let changed = connection.execute(
            "
            UPDATE obj_media_book_reader_state SET
                reading_queue_position = COALESCE(reading_queue_position, (
                    SELECT COALESCE(MAX(reading_queue_position), 0) + 1
                    FROM obj_media_book_reader_state
                )), updated_at = datetime('now','localtime')
            WHERE media_library_entry_id = ?1
            ",
            [entry_id],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Book library entry was not found."));
        }
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn remove_from_read_next(&self, entry_id: i64) -> Result<BookLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE obj_media_book_reader_state SET reading_queue_position = NULL WHERE media_library_entry_id = ?1",
            [entry_id],
        )?;
        normalize_reading_queue(&transaction)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn move_read_next(
        &self,
        entry_id: i64,
        direction: &str,
    ) -> Result<BookLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let current: i64 = transaction
            .query_row(
                "SELECT reading_queue_position FROM obj_media_book_reader_state WHERE media_library_entry_id = ?1",
                [entry_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| MediaError::validation("This book is not in Read Next."))?;
        let target = match direction.trim().to_ascii_uppercase().as_str() {
            "UP" => current - 1,
            "DOWN" => current + 1,
            _ => {
                return Err(MediaError::validation(
                    "Read Next direction must be UP or DOWN.",
                ))
            }
        };
        if target > 0 {
            if let Some(target_entry) = transaction
                .query_row(
                    "SELECT media_library_entry_id FROM obj_media_book_reader_state WHERE reading_queue_position = ?1",
                    [target],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
            {
                transaction.execute(
                    "UPDATE obj_media_book_reader_state SET reading_queue_position = -1 WHERE media_library_entry_id = ?1",
                    [entry_id],
                )?;
                transaction.execute(
                    "UPDATE obj_media_book_reader_state SET reading_queue_position = ?2 WHERE media_library_entry_id = ?1",
                    params![target_entry, current],
                )?;
                transaction.execute(
                    "UPDATE obj_media_book_reader_state SET reading_queue_position = ?2 WHERE media_library_entry_id = ?1",
                    params![entry_id, target],
                )?;
            }
        }
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn create_edition(
        &self,
        entry_id: i64,
        input: &ManualBookEditionInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        let edition = normalized_manual_edition(input)?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = book_title_id(&transaction, entry_id)?;
        let edition_id = insert_edition(&transaction, title_id, &edition)?;
        transaction.execute(
            "
            INSERT INTO obj_media_book_source_record (
                media_title_id,edition_id,source_key,entity_type,external_id,
                refreshed_at,last_refresh_status
            ) VALUES (?1,?2,'manual','EDITION',?3,datetime('now','localtime'),'SUCCESS')
            ",
            params![title_id, edition_id, format!("local-edition-{edition_id}")],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn update_edition(
        &self,
        entry_id: i64,
        edition_id: i64,
        input: &ManualBookEditionInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        let edition = normalized_manual_edition(input)?;
        let connection = self.database.connection()?;
        let title_id = book_title_id(&connection, entry_id)?;
        let provider_owned: bool = connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM obj_media_book_source_record
                WHERE edition_id=?1 AND source_key <> 'manual'
            )",
            [edition_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if provider_owned {
            return Err(MediaError::validation(
                "Provider editions cannot be edited manually.",
            ));
        }
        let changed = connection.execute(
            "
            UPDATE obj_media_book_edition SET title=?3, subtitle=?4, format=?5,
                isbn_10=?6, isbn_13=?7, publisher=?8, published_date=?9, language=?10,
                page_count=?11, audio_duration_minutes=?12, cover_url=?13,
                is_ebook=?14, updated_at=datetime('now','localtime')
            WHERE id=?1 AND media_title_id=?2
            ",
            params![
                edition_id,
                title_id,
                edition.title,
                edition.subtitle,
                edition.format.to_string(),
                edition.isbn_10,
                edition.isbn_13,
                edition.publisher,
                edition.published_date,
                edition.language,
                edition.page_count,
                edition.audio_duration_minutes,
                edition.cover_url,
                edition.is_ebook as i64
            ],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Book edition was not found."));
        }
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn delete_edition(
        &self,
        entry_id: i64,
        edition_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let title_id = book_title_id(&connection, entry_id)?;
        let preferred: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM obj_media_book_reader_state WHERE media_library_entry_id=?1 AND preferred_edition_id=?2)",
            params![entry_id, edition_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if preferred {
            return Err(MediaError::validation(
                "Choose another preferred edition before deleting this edition.",
            ));
        }
        let provider_owned: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM obj_media_book_source_record WHERE edition_id=?1 AND source_key <> 'manual')",
            [edition_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if provider_owned {
            return Err(MediaError::validation(
                "Provider editions are retained for refresh history and cannot be deleted manually.",
            ));
        }
        let changed = connection.execute(
            "DELETE FROM obj_media_book_edition WHERE id=?1 AND media_title_id=?2",
            params![edition_id, title_id],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Book edition was not found."));
        }
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn create_link(
        &self,
        entry_id: i64,
        input: &BookLinkInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        validate_book_link(input)?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = book_title_id(&transaction, entry_id)?;
        validate_edition_ownership(&transaction, title_id, input.edition_id)?;
        if input.is_preferred {
            transaction.execute(
                "UPDATE obj_media_book_link SET is_preferred=0 WHERE media_title_id=?1 AND is_user_owned=1",
                [title_id],
            )?;
        }
        transaction.execute(
            "INSERT INTO obj_media_book_link (media_title_id, edition_id, source_key, link_type, url, region_code, is_user_owned, is_preferred) VALUES (?1,?2,'manual',?3,?4,?5,1,?6)",
            params![title_id, input.edition_id, input.link_type.to_ascii_uppercase(), validate_http_url(&input.url)?, input.region_code.trim().to_ascii_uppercase(), input.is_preferred as i64],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn update_link(
        &self,
        entry_id: i64,
        link_id: i64,
        input: &BookLinkInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        validate_book_link(input)?;
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = book_title_id(&transaction, entry_id)?;
        validate_edition_ownership(&transaction, title_id, input.edition_id)?;
        if input.is_preferred {
            transaction.execute(
                "UPDATE obj_media_book_link SET is_preferred=0 WHERE media_title_id=?1 AND is_user_owned=1",
                [title_id],
            )?;
        }
        let changed = transaction.execute(
            "UPDATE obj_media_book_link SET edition_id=?3, link_type=?4, url=?5, region_code=?6, is_preferred=?7, updated_at=datetime('now','localtime') WHERE id=?1 AND media_title_id=?2 AND is_user_owned=1",
            params![link_id, title_id, input.edition_id, input.link_type.to_ascii_uppercase(), validate_http_url(&input.url)?, input.region_code.trim().to_ascii_uppercase(), input.is_preferred as i64],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("User-owned book link was not found."));
        }
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn delete_link(
        &self,
        entry_id: i64,
        link_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let title_id = book_title_id(&connection, entry_id)?;
        let changed = connection.execute(
            "DELETE FROM obj_media_book_link WHERE id=?1 AND media_title_id=?2 AND is_user_owned=1",
            params![link_id, title_id],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("User-owned book link was not found."));
        }
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn set_preferred_link(
        &self,
        entry_id: i64,
        link_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = book_title_id(&transaction, entry_id)?;
        transaction.execute(
            "UPDATE obj_media_book_link SET is_preferred=0 WHERE media_title_id=?1 AND is_user_owned=1",
            [title_id],
        )?;
        let changed = transaction.execute(
            "UPDATE obj_media_book_link SET is_preferred=1 WHERE id=?1 AND media_title_id=?2 AND is_user_owned=1",
            params![link_id, title_id],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("User-owned book link was not found."));
        }
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn resolve_link(&self, entry_id: i64, link_id: i64) -> Result<String, MediaError> {
        let connection = self.database.connection()?;
        let title_id = book_title_id(&connection, entry_id)?;
        let url = connection
            .query_row(
                "SELECT url FROM obj_media_book_link WHERE id=?1 AND media_title_id=?2",
                params![link_id, title_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Book link was not found."))?;
        validate_http_url(&url)
    }

    pub fn set_series_override(
        &self,
        entry_id: i64,
        input: &BookSeriesOverrideInput,
    ) -> Result<BookLibraryDetail, MediaError> {
        if input.name.trim().is_empty() {
            return Err(MediaError::validation("A series name is required."));
        }
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = book_title_id(&transaction, entry_id)?;
        transaction.execute(
            "INSERT OR IGNORE INTO obj_media_book_series (name) VALUES (?1)",
            [input.name.trim()],
        )?;
        let series_id: i64 = transaction.query_row(
            "SELECT id FROM obj_media_book_series WHERE name=?1 COLLATE NOCASE",
            [input.name.trim()],
            |row| row.get(0),
        )?;
        if input.is_primary {
            transaction.execute(
                "UPDATE obj_media_book_series_member SET is_primary=0 WHERE media_title_id=?1",
                [title_id],
            )?;
        }
        transaction.execute(
            "
            INSERT INTO obj_media_book_series_member (series_id, media_title_id, position_text, position_sort, is_primary, source_key, is_user_override)
            VALUES (?1,?2,?3,?4,?5,'manual',1)
            ON CONFLICT(series_id,media_title_id,source_key) DO UPDATE SET
                position_text=excluded.position_text, position_sort=excluded.position_sort,
                is_primary=excluded.is_primary, is_user_override=1, updated_at=datetime('now','localtime')
            ",
            params![series_id,title_id,input.position_text.trim(),input.position_sort,input.is_primary as i64],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn clear_series_override(
        &self,
        entry_id: i64,
        series_member_id: i64,
    ) -> Result<BookLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let title_id = book_title_id(&connection, entry_id)?;
        connection.execute(
            "DELETE FROM obj_media_book_series_member WHERE id=?1 AND media_title_id=?2 AND is_user_override=1",
            params![series_member_id, title_id],
        )?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn source_identity_for_entry(&self, entry_id: i64) -> Result<(String, String), MediaError> {
        let connection = self.database.connection()?;
        connection
            .query_row(
                "
                SELECT source.source_key, source.external_id
                FROM obj_media_book_source_record source
                JOIN obj_media_library_entry entry ON entry.media_title_id=source.media_title_id
                WHERE entry.id=?1 AND source.source_key IN ('google_books','open_library')
                ORDER BY CASE source.source_key WHEN 'google_books' THEN 1 ELSE 2 END,
                         CASE source.entity_type WHEN 'EDITION' THEN 1 ELSE 2 END
                LIMIT 1
                ",
                [entry_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| {
                MediaError::validation("This manual book has no provider identity to refresh.")
            })
    }

    pub fn mark_provider_failure(
        &self,
        entry_id: i64,
        source_key: &str,
        message: &str,
    ) -> Result<(), MediaError> {
        let connection = self.database.connection()?;
        let title_id = book_title_id(&connection, entry_id)?;
        connection.execute(
            "
            UPDATE obj_media_book_source_record SET last_refresh_status='FAILED',
                last_refresh_error=?3, updated_at=datetime('now','localtime')
            WHERE media_title_id=?1 AND source_key=?2
            ",
            params![title_id, source_key, message],
        )?;
        Ok(())
    }
}

fn validate_import(import: &NormalizedBookImport) -> Result<(), MediaError> {
    if import.title.trim().is_empty() {
        return Err(MediaError::validation(
            "A normalized book title is required.",
        ));
    }
    if import.editions.is_empty() {
        return Err(MediaError::validation(
            "A book import requires at least one edition.",
        ));
    }
    for edition in &import.editions {
        if edition.page_count.is_some_and(|value| value <= 0)
            || edition
                .audio_duration_minutes
                .is_some_and(|value| value <= 0)
        {
            return Err(MediaError::validation(
                "Book page counts and audio durations must be positive.",
            ));
        }
        if !edition.isbn_10.is_empty() && normalize_isbn(&edition.isbn_10).is_none() {
            return Err(MediaError::validation(
                "A provider returned an invalid ISBN-10.",
            ));
        }
        if !edition.isbn_13.is_empty() && normalize_isbn(&edition.isbn_13).is_none() {
            return Err(MediaError::validation(
                "A provider returned an invalid ISBN-13.",
            ));
        }
        validate_optional_url(&edition.cover_url)?;
    }
    for link in &import.links {
        validate_http_url(&link.url)?;
        validate_link_type(&link.link_type)?;
    }
    Ok(())
}

fn normalize_optional_isbn(value: &str, expected_length: usize) -> Result<String, MediaError> {
    if value.trim().is_empty() {
        return Ok(String::new());
    }
    let isbn = normalize_isbn(value)
        .ok_or_else(|| MediaError::validation(format!("ISBN-{expected_length} is invalid.")))?;
    if isbn.len() != expected_length {
        return Err(MediaError::validation(format!(
            "Expected an ISBN-{expected_length}."
        )));
    }
    Ok(isbn)
}

fn validate_optional_url(value: &str) -> Result<String, MediaError> {
    if value.trim().is_empty() {
        Ok(String::new())
    } else {
        validate_http_url(value)
    }
}

fn validate_link_type(value: &str) -> Result<(), MediaError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "INFO" | "PREVIEW" | "READ" | "BORROW" | "BUY" | "OTHER" => Ok(()),
        _ => Err(MediaError::validation(
            "Book link type must be INFO, PREVIEW, READ, BORROW, BUY, or OTHER.",
        )),
    }
}

fn validate_book_link(input: &BookLinkInput) -> Result<(), MediaError> {
    validate_http_url(&input.url)?;
    validate_link_type(&input.link_type)
}

fn find_existing(
    connection: &Connection,
    source_key: &str,
    external_id: &str,
    isbn_10: &str,
    isbn_13: &str,
) -> Result<Option<i64>, MediaError> {
    let source_match = connection
        .query_row(
            "
            SELECT entry.id
            FROM obj_media_book_source_record source
            JOIN obj_media_library_entry entry ON entry.media_title_id=source.media_title_id
            WHERE source.source_key=?1 AND source.external_id=?2
            LIMIT 1
            ",
            params![source_key, external_id],
            |row| row.get(0),
        )
        .optional()?;
    if source_match.is_some() {
        return Ok(source_match);
    }
    let normalized_13 = normalize_isbn(isbn_13).filter(|value| value.len() == 13);
    let normalized_10 = normalize_isbn(isbn_10).filter(|value| value.len() == 10);
    connection
        .query_row(
            "
            SELECT entry.id
            FROM obj_media_book_edition edition
            JOIN obj_media_library_entry entry ON entry.media_title_id=edition.media_title_id
            WHERE (?1 <> '' AND edition.isbn_13=?1) OR (?2 <> '' AND edition.isbn_10=?2)
            LIMIT 1
            ",
            params![
                normalized_13.unwrap_or_default(),
                normalized_10.unwrap_or_default()
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(MediaError::from)
}

fn find_import_duplicate(
    connection: &Connection,
    import: &NormalizedBookImport,
) -> Result<Option<i64>, MediaError> {
    for source in &import.sources {
        if let Some(entry_id) =
            find_existing(connection, &source.source_key, &source.external_id, "", "")?
        {
            return Ok(Some(entry_id));
        }
    }
    for edition in &import.editions {
        if let Some(entry_id) =
            find_existing(connection, "", "", &edition.isbn_10, &edition.isbn_13)?
        {
            return Ok(Some(entry_id));
        }
    }
    Ok(None)
}

fn insert_work(
    transaction: &Transaction<'_>,
    title_id: i64,
    import: &NormalizedBookImport,
) -> Result<(), MediaError> {
    transaction.execute(
        "
        INSERT INTO obj_media_book_work (
            media_title_id, subtitle, primary_author_text, first_publish_year,
            subjects_json, canonical_cover_url, community_rating, community_rating_count,
            metadata_refreshed_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,datetime('now','localtime'))
        ",
        params![
            title_id,
            import.subtitle,
            import.authors.join(", "),
            import.first_publish_year,
            serde_json::to_string(&import.subjects).unwrap_or_else(|_| "[]".to_string()),
            import.canonical_cover_url,
            import.community_rating,
            import.community_rating_count,
        ],
    )?;
    Ok(())
}

fn insert_editions(
    transaction: &Transaction<'_>,
    title_id: i64,
    editions: &[NormalizedBookEdition],
) -> Result<Vec<i64>, MediaError> {
    editions
        .iter()
        .map(|edition| insert_edition(transaction, title_id, edition))
        .collect()
}

fn insert_edition(
    connection: &Connection,
    title_id: i64,
    edition: &NormalizedBookEdition,
) -> Result<i64, MediaError> {
    connection.execute(
        "
        INSERT INTO obj_media_book_edition (
            media_title_id,title,subtitle,format,isbn_10,isbn_13,publisher,published_date,
            language,page_count,audio_duration_minutes,cover_url,is_ebook,access_viewability,
            metadata_refreshed_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,datetime('now','localtime'))
        ",
        params![
            title_id,
            edition.title,
            edition.subtitle,
            edition.format.to_string(),
            edition.isbn_10,
            edition.isbn_13,
            edition.publisher,
            edition.published_date,
            edition.language,
            edition.page_count,
            edition.audio_duration_minutes,
            edition.cover_url,
            edition.is_ebook as i64,
            edition.access_viewability,
        ],
    )?;
    Ok(connection.last_insert_rowid())
}

fn upsert_editions(
    transaction: &Transaction<'_>,
    title_id: i64,
    editions: &[NormalizedBookEdition],
) -> Result<Vec<i64>, MediaError> {
    let mut ids = Vec::new();
    for edition in editions {
        let existing = transaction
            .query_row(
                "
                SELECT id FROM obj_media_book_edition
                WHERE media_title_id=?1 AND (
                    (?2 <> '' AND isbn_13=?2) OR (?3 <> '' AND isbn_10=?3) OR
                    (?2 = '' AND ?3 = '' AND title=?4 COLLATE NOCASE AND publisher=?5 COLLATE NOCASE
                        AND published_date=?6 AND format=?7)
                ) LIMIT 1
                ",
                params![
                    title_id,
                    edition.isbn_13,
                    edition.isbn_10,
                    edition.title,
                    edition.publisher,
                    edition.published_date,
                    edition.format.to_string()
                ],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if let Some(id) = existing {
            transaction.execute(
                "
                UPDATE obj_media_book_edition SET
                    title=CASE WHEN ?2<>'' THEN ?2 ELSE title END,
                    subtitle=CASE WHEN ?3<>'' THEN ?3 ELSE subtitle END,
                    format=CASE WHEN ?4<>'UNKNOWN' THEN ?4 ELSE format END,
                    isbn_10=CASE WHEN ?5<>'' THEN ?5 ELSE isbn_10 END,
                    isbn_13=CASE WHEN ?6<>'' THEN ?6 ELSE isbn_13 END,
                    publisher=CASE WHEN ?7<>'' THEN ?7 ELSE publisher END,
                    published_date=CASE WHEN ?8<>'' THEN ?8 ELSE published_date END,
                    language=CASE WHEN ?9<>'' THEN ?9 ELSE language END,
                    page_count=COALESCE(?10,page_count), audio_duration_minutes=COALESCE(?11,audio_duration_minutes),
                    cover_url=CASE WHEN ?12<>'' THEN ?12 ELSE cover_url END,
                    is_ebook=CASE WHEN ?13=1 THEN 1 ELSE is_ebook END,
                    access_viewability=CASE WHEN ?14<>'' THEN ?14 ELSE access_viewability END,
                    is_present_in_source=1, metadata_refreshed_at=datetime('now','localtime'),
                    updated_at=datetime('now','localtime') WHERE id=?1
                ",
                params![id,edition.title,edition.subtitle,edition.format.to_string(),edition.isbn_10,edition.isbn_13,edition.publisher,edition.published_date,edition.language,edition.page_count,edition.audio_duration_minutes,edition.cover_url,edition.is_ebook as i64,edition.access_viewability],
            )?;
            ids.push(id);
        } else {
            ids.push(insert_edition(transaction, title_id, edition)?);
        }
    }
    Ok(ids)
}

fn insert_sources(
    transaction: &Transaction<'_>,
    title_id: i64,
    edition_ids: &[i64],
    import: &NormalizedBookImport,
) -> Result<(), MediaError> {
    for source in &import.sources {
        transaction.execute(
            "DELETE FROM obj_media_book_source_record
             WHERE media_title_id=?1 AND source_key=?2 AND entity_type='BOOK'
               AND external_id=?3",
            params![
                title_id,
                source.source_key,
                format!("status-{title_id}-{}", source.source_key)
            ],
        )?;
        let edition_id = source
            .edition_index
            .and_then(|index| edition_ids.get(index))
            .copied();
        transaction.execute(
            "
            INSERT INTO obj_media_book_source_record (
                media_title_id,edition_id,source_key,entity_type,external_id,source_url,
                metadata_json,refreshed_at,last_refresh_status,last_refresh_error
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,datetime('now','localtime'),'SUCCESS','')
            ON CONFLICT(source_key,entity_type,external_id) DO UPDATE SET
                edition_id=COALESCE(excluded.edition_id,obj_media_book_source_record.edition_id),
                source_url=CASE WHEN excluded.source_url<>'' THEN excluded.source_url ELSE obj_media_book_source_record.source_url END,
                metadata_json=CASE WHEN excluded.metadata_json<>'{}' THEN excluded.metadata_json ELSE obj_media_book_source_record.metadata_json END,
                refreshed_at=excluded.refreshed_at,last_refresh_status='SUCCESS',last_refresh_error='',
                updated_at=datetime('now','localtime')
            ",
            params![title_id,edition_id,source.source_key,source.entity_type,source.external_id,source.source_url,source.metadata_json],
        )?;
    }
    Ok(())
}

fn record_provider_failures(
    transaction: &Transaction<'_>,
    title_id: i64,
    failures: &[BookProviderFailure],
) -> Result<(), MediaError> {
    for failure in failures {
        let changed = transaction.execute(
            "
            UPDATE obj_media_book_source_record
            SET last_refresh_status='FAILED',last_refresh_error=?3,
                updated_at=datetime('now','localtime')
            WHERE media_title_id=?1 AND source_key=?2
            ",
            params![title_id, failure.source_key, failure.message],
        )?;
        if changed == 0 {
            transaction.execute(
                "
                INSERT INTO obj_media_book_source_record (
                    media_title_id,source_key,entity_type,external_id,
                    last_refresh_status,last_refresh_error
                ) VALUES (?1,?2,'BOOK',?3,'FAILED',?4)
                ",
                params![
                    title_id,
                    failure.source_key,
                    format!("status-{title_id}-{}", failure.source_key),
                    failure.message,
                ],
            )?;
        }
    }
    Ok(())
}

fn insert_authors(
    transaction: &Transaction<'_>,
    title_id: i64,
    authors: &[String],
) -> Result<(), MediaError> {
    for (position, author) in authors.iter().enumerate() {
        let name = author.split_whitespace().collect::<Vec<_>>().join(" ");
        if name.is_empty() {
            continue;
        }
        let existing = transaction
            .query_row(
                "
                SELECT author.id FROM obj_media_book_author author
                JOIN n2n_media_book_work_author mapping ON mapping.author_id=author.id
                WHERE mapping.media_title_id=?1 AND author.name=?2 COLLATE NOCASE AND mapping.role='AUTHOR'
                ",
                params![title_id, name],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        let author_id = if let Some(id) = existing {
            id
        } else {
            transaction.execute(
                "INSERT INTO obj_media_book_author (name,sort_name) VALUES (?1,?1)",
                [name.as_str()],
            )?;
            transaction.last_insert_rowid()
        };
        transaction.execute(
            "INSERT OR IGNORE INTO n2n_media_book_work_author (media_title_id,author_id,role,position) VALUES (?1,?2,'AUTHOR',?3)",
            params![title_id, author_id, position as i64],
        )?;
    }
    Ok(())
}

fn insert_provider_links(
    transaction: &Transaction<'_>,
    title_id: i64,
    edition_ids: &[i64],
    import: &NormalizedBookImport,
) -> Result<(), MediaError> {
    for link in &import.links {
        validate_http_url(&link.url)?;
        validate_link_type(&link.link_type)?;
        let edition_id = link
            .edition_index
            .and_then(|index| edition_ids.get(index))
            .copied();
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM obj_media_book_link WHERE media_title_id=?1 AND source_key=?2 AND link_type=?3 AND url=?4 AND is_user_owned=0)",
            params![title_id,link.source_key,link.link_type.to_ascii_uppercase(),link.url],
            |row| Ok(row.get::<_,i64>(0)? != 0),
        )?;
        if !exists {
            transaction.execute(
                "INSERT INTO obj_media_book_link (media_title_id,edition_id,source_key,link_type,url,region_code,is_user_owned,is_preferred,refreshed_at) VALUES (?1,?2,?3,?4,?5,?6,0,0,datetime('now','localtime'))",
                params![title_id,edition_id,link.source_key,link.link_type.to_ascii_uppercase(),link.url,link.region_code],
            )?;
        }
    }
    Ok(())
}

fn insert_series(
    transaction: &Transaction<'_>,
    title_id: i64,
    import: &NormalizedBookImport,
) -> Result<(), MediaError> {
    for (index, series) in import.series.iter().enumerate() {
        if series.name.trim().is_empty() {
            continue;
        }
        transaction.execute(
            "INSERT OR IGNORE INTO obj_media_book_series (name) VALUES (?1)",
            [series.name.trim()],
        )?;
        let series_id: i64 = transaction.query_row(
            "SELECT id FROM obj_media_book_series WHERE name=?1 COLLATE NOCASE",
            [series.name.trim()],
            |row| row.get(0),
        )?;
        transaction.execute(
            "
            INSERT INTO obj_media_book_series_member (
                series_id,media_title_id,position_text,position_sort,is_primary,source_key,is_user_override
            ) VALUES (?1,?2,?3,?4,?5,?6,?7)
            ON CONFLICT(series_id,media_title_id,source_key) DO UPDATE SET
                position_text=CASE WHEN excluded.position_text<>'' THEN excluded.position_text ELSE obj_media_book_series_member.position_text END,
                position_sort=COALESCE(excluded.position_sort,obj_media_book_series_member.position_sort),
                is_user_override=MAX(obj_media_book_series_member.is_user_override, excluded.is_user_override),
                updated_at=datetime('now','localtime')
            ",
            params![series_id,title_id,series.position_text,series.position_sort,(index==0) as i64,series.source_key,(series.source_key == "manual") as i64],
        )?;
    }
    Ok(())
}

fn book_item_select() -> &'static str {
    "
    SELECT entry.id,entry.media_title_id,title.title,work.subtitle,work.primary_author_text,
           title.overview,work.first_publish_year,work.canonical_cover_url,
           entry.library_status,entry.is_favorite,entry.personal_rating,entry.notes,entry.priority,
           entry.added_at,entry.started_at,entry.completed_at,work.metadata_refreshed_at,
           reader.progress_unit,reader.progress_value,reader.progress_total_override,
           reader.current_chapter,reader.preferred_edition_id,reader.ownership_status,
           reader.preferred_format,reader.reading_queue_position,reader.last_progress_at,
           edition.page_count,edition.audio_duration_minutes
    FROM obj_media_library_entry entry
    JOIN obj_media_title title ON title.id=entry.media_title_id AND title.content_type='BOOK'
    JOIN obj_media_book_work work ON work.media_title_id=title.id
    JOIN obj_media_book_reader_state reader ON reader.media_library_entry_id=entry.id
    LEFT JOIN obj_media_book_edition edition ON edition.id=reader.preferred_edition_id
    WHERE 1=1
    "
}

struct RawBookItem {
    entry_id: i64,
    media_title_id: i64,
    title: String,
    subtitle: String,
    primary_author_text: String,
    description: String,
    first_publish_year: Option<i64>,
    cover_url: String,
    library_status: String,
    is_favorite: bool,
    personal_rating: Option<f64>,
    notes: String,
    priority: i64,
    added_at: String,
    started_at: String,
    completed_at: String,
    metadata_refreshed_at: String,
    progress_unit: String,
    progress_value: f64,
    progress_total_override: Option<f64>,
    current_chapter: String,
    preferred_edition_id: Option<i64>,
    ownership_status: String,
    preferred_format: String,
    reading_queue_position: Option<i64>,
    last_progress_at: String,
    page_count: Option<i64>,
    audio_duration_minutes: Option<i64>,
}

fn raw_book_item_from_row(row: &rusqlite::Row<'_>) -> SqlResult<RawBookItem> {
    Ok(RawBookItem {
        entry_id: row.get(0)?,
        media_title_id: row.get(1)?,
        title: row.get(2)?,
        subtitle: row.get(3)?,
        primary_author_text: row.get(4)?,
        description: row.get(5)?,
        first_publish_year: row.get(6)?,
        cover_url: row.get(7)?,
        library_status: row.get(8)?,
        is_favorite: row.get::<_, i64>(9)? != 0,
        personal_rating: row.get(10)?,
        notes: row.get(11)?,
        priority: row.get(12)?,
        added_at: row.get(13)?,
        started_at: row.get(14)?,
        completed_at: row.get(15)?,
        metadata_refreshed_at: row.get(16)?,
        progress_unit: row.get(17)?,
        progress_value: row.get(18)?,
        progress_total_override: row.get(19)?,
        current_chapter: row.get(20)?,
        preferred_edition_id: row.get(21)?,
        ownership_status: row.get(22)?,
        preferred_format: row.get(23)?,
        reading_queue_position: row.get(24)?,
        last_progress_at: row.get(25)?,
        page_count: row.get(26)?,
        audio_duration_minutes: row.get(27)?,
    })
}

fn book_item_from_raw(raw: RawBookItem) -> Result<BookLibraryItem, MediaError> {
    let progress_unit = BookProgressUnit::from_str(&raw.progress_unit)?;
    let library_status = MediaLibraryStatus::from_str(&raw.library_status)?;
    let effective_total = effective_progress_total(
        progress_unit,
        raw.progress_total_override,
        raw.page_count,
        raw.audio_duration_minutes,
    );
    let progress_percent = effective_total
        .filter(|total| *total > 0.0)
        .map(|total| (raw.progress_value / total * 100.0).clamp(0.0, 100.0));
    Ok(BookLibraryItem {
        entry_id: raw.entry_id,
        media_title_id: raw.media_title_id,
        title: raw.title,
        subtitle: raw.subtitle,
        primary_author_text: raw.primary_author_text,
        description: raw.description,
        first_publish_year: raw.first_publish_year,
        cover_url: raw.cover_url,
        library_status,
        status_label: library_status
            .label_for(crate::media::domain::MediaContentType::Book)
            .to_string(),
        is_favorite: raw.is_favorite,
        personal_rating: raw.personal_rating,
        notes: raw.notes,
        priority: raw.priority,
        added_at: raw.added_at,
        started_at: raw.started_at,
        completed_at: raw.completed_at,
        metadata_refreshed_at: raw.metadata_refreshed_at,
        progress: BookProgressSummary {
            progress_unit,
            progress_value: raw.progress_value,
            progress_total_override: raw.progress_total_override,
            effective_total,
            progress_percent,
            current_chapter: raw.current_chapter,
            preferred_edition_id: raw.preferred_edition_id,
            ownership_status: BookOwnershipStatus::from_str(&raw.ownership_status)?,
            preferred_format: BookFormat::from_str(&raw.preferred_format)?,
            reading_queue_position: raw.reading_queue_position,
            last_progress_at: raw.last_progress_at,
        },
    })
}

fn load_editions(
    connection: &Connection,
    title_id: i64,
) -> Result<Vec<BookEditionRecord>, MediaError> {
    let mut statement = connection.prepare(
        "
        SELECT id,title,subtitle,format,isbn_10,isbn_13,publisher,published_date,language,
               page_count,audio_duration_minutes,cover_url,is_ebook,access_viewability,is_present_in_source
        FROM obj_media_book_edition WHERE media_title_id=?1
        ORDER BY is_present_in_source DESC,published_date DESC,id
        ",
    )?;
    let raw = statement
        .query_map([title_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, i64>(12)? != 0,
                row.get::<_, String>(13)?,
                row.get::<_, i64>(14)? != 0,
            ))
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    raw.into_iter()
        .map(|value| {
            let source_keys = load_edition_source_keys(connection, value.0)?;
            let is_user_owned =
                source_keys.is_empty() || source_keys.iter().all(|source| source == "manual");
            Ok(BookEditionRecord {
                id: value.0,
                title: value.1,
                subtitle: value.2,
                format: BookFormat::from_str(&value.3)?,
                isbn_10: value.4,
                isbn_13: value.5,
                publisher: value.6,
                published_date: value.7,
                language: value.8,
                page_count: value.9,
                audio_duration_minutes: value.10,
                cover_url: value.11,
                is_ebook: value.12,
                access_viewability: value.13,
                is_present_in_source: value.14,
                source_keys,
                is_user_owned,
            })
        })
        .collect()
}

fn load_edition_source_keys(
    connection: &Connection,
    edition_id: i64,
) -> Result<Vec<String>, MediaError> {
    let mut statement = connection.prepare(
        "SELECT DISTINCT source_key FROM obj_media_book_source_record
         WHERE edition_id=?1 ORDER BY source_key",
    )?;
    let source_keys = statement
        .query_map([edition_id], |row| row.get::<_, String>(0))?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(source_keys)
}

fn load_authors(
    connection: &Connection,
    title_id: i64,
) -> Result<Vec<BookAuthorRecord>, MediaError> {
    let mut statement = connection.prepare(
        "
        SELECT author.id,author.name,mapping.role,mapping.position
        FROM n2n_media_book_work_author mapping
        JOIN obj_media_book_author author ON author.id=mapping.author_id
        WHERE mapping.media_title_id=?1 ORDER BY mapping.position,author.name COLLATE NOCASE
        ",
    )?;
    let records = statement
        .query_map([title_id], |row| {
            Ok(BookAuthorRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                position: row.get(3)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(records)
}

fn load_tags(connection: &Connection, entry_id: i64) -> Result<Vec<MediaTagRecord>, MediaError> {
    let mut statement = connection.prepare(
        "
        SELECT tag.id,tag.name,tag.created_at,tag.updated_at
        FROM obj_media_tag tag
        JOIN n2n_media_library_entry_tag mapping ON mapping.media_tag_id=tag.id
        WHERE mapping.media_library_entry_id=?1
        ORDER BY tag.name COLLATE NOCASE
        ",
    )?;
    let tags = statement
        .query_map([entry_id], |row| {
            Ok(MediaTagRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(tags)
}

fn load_links(connection: &Connection, title_id: i64) -> Result<Vec<BookLinkRecord>, MediaError> {
    let mut statement = connection.prepare(
        "
        SELECT id,edition_id,source_key,link_type,url,region_code,is_user_owned,is_preferred,refreshed_at
        FROM obj_media_book_link WHERE media_title_id=?1
        ORDER BY is_preferred DESC,is_user_owned DESC,
          CASE link_type WHEN 'READ' THEN 1 WHEN 'BORROW' THEN 2 WHEN 'PREVIEW' THEN 3 WHEN 'BUY' THEN 4 ELSE 5 END,id
        ",
    )?;
    let records = statement
        .query_map([title_id], |row| {
            Ok(BookLinkRecord {
                id: row.get(0)?,
                edition_id: row.get(1)?,
                source_key: row.get(2)?,
                link_type: row.get(3)?,
                url: row.get(4)?,
                region_code: row.get(5)?,
                is_user_owned: row.get::<_, i64>(6)? != 0,
                is_preferred: row.get::<_, i64>(7)? != 0,
                refreshed_at: row.get(8)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(records)
}

fn load_series(
    connection: &Connection,
    title_id: i64,
) -> Result<Vec<BookSeriesRecord>, MediaError> {
    let mut statement = connection.prepare(
        "
        SELECT member.id,series.name,member.position_text,member.position_sort,
               member.is_primary,member.source_key,member.is_user_override
        FROM obj_media_book_series_member member
        JOIN obj_media_book_series series ON series.id=member.series_id
        WHERE member.media_title_id=?1
        ORDER BY member.is_user_override DESC,member.is_primary DESC,member.position_sort,series.name
        ",
    )?;
    let records = statement
        .query_map([title_id], |row| {
            Ok(BookSeriesRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                position_text: row.get(2)?,
                position_sort: row.get(3)?,
                is_primary: row.get::<_, i64>(4)? != 0,
                source_key: row.get(5)?,
                is_user_override: row.get::<_, i64>(6)? != 0,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(records)
}

fn load_sources(
    connection: &Connection,
    title_id: i64,
) -> Result<Vec<BookSourceRecord>, MediaError> {
    let mut statement = connection.prepare(
        "
        SELECT source_key,entity_type,external_id,source_url,refreshed_at,last_refresh_status,last_refresh_error
        FROM obj_media_book_source_record WHERE media_title_id=?1 ORDER BY source_key,entity_type
        ",
    )?;
    let records = statement
        .query_map([title_id], |row| {
            Ok(BookSourceRecord {
                source_key: row.get(0)?,
                entity_type: row.get(1)?,
                external_id: row.get(2)?,
                source_url: row.get(3)?,
                refreshed_at: row.get(4)?,
                last_refresh_status: row.get(5)?,
                last_refresh_error: row.get(6)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(records)
}

fn book_title_id(connection: &Connection, entry_id: i64) -> Result<i64, MediaError> {
    connection
        .query_row(
            "
            SELECT title.id FROM obj_media_library_entry entry
            JOIN obj_media_title title ON title.id=entry.media_title_id
            WHERE entry.id=?1 AND title.content_type='BOOK'
            ",
            [entry_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| MediaError::not_found("Book library entry was not found."))
}

fn update_entry_for_progress(
    transaction: &Transaction<'_>,
    entry_id: i64,
    status: MediaLibraryStatus,
    progress_value: f64,
) -> Result<(), MediaError> {
    transaction.execute(
        "
        UPDATE obj_media_library_entry SET library_status=?2,
            started_at=CASE WHEN ?3>0 AND started_at='' THEN datetime('now','localtime') WHEN ?3=0 THEN '' ELSE started_at END,
            completed_at=CASE WHEN ?2='COMPLETED' THEN COALESCE(NULLIF(completed_at,''),datetime('now','localtime')) ELSE '' END,
            updated_at=datetime('now','localtime') WHERE id=?1
        ",
        params![entry_id, status.to_string(), progress_value],
    )?;
    Ok(())
}

fn normalize_reading_queue(transaction: &Transaction<'_>) -> Result<(), MediaError> {
    let ids = {
        let mut statement = transaction.prepare(
            "SELECT media_library_entry_id FROM obj_media_book_reader_state WHERE reading_queue_position IS NOT NULL ORDER BY reading_queue_position,media_library_entry_id",
        )?;
        let ids = statement
            .query_map([], |row| row.get::<_, i64>(0))?
            .collect::<SqlResult<Vec<_>>>()?;
        ids
    };
    transaction.execute(
        "UPDATE obj_media_book_reader_state SET reading_queue_position=-media_library_entry_id WHERE reading_queue_position IS NOT NULL",
        [],
    )?;
    for (index, entry_id) in ids.into_iter().enumerate() {
        transaction.execute(
            "UPDATE obj_media_book_reader_state SET reading_queue_position=?2 WHERE media_library_entry_id=?1",
            params![entry_id, index as i64 + 1],
        )?;
    }
    Ok(())
}

fn normalized_manual_edition(
    input: &ManualBookEditionInput,
) -> Result<NormalizedBookEdition, MediaError> {
    if input.title.trim().is_empty() {
        return Err(MediaError::validation("An edition title is required."));
    }
    if input.page_count.is_some_and(|value| value <= 0)
        || input.audio_duration_minutes.is_some_and(|value| value <= 0)
    {
        return Err(MediaError::validation(
            "Page count and audio duration must be positive.",
        ));
    }
    Ok(NormalizedBookEdition {
        title: input.title.trim().to_string(),
        subtitle: input.subtitle.trim().to_string(),
        format: input.format,
        isbn_10: normalize_optional_isbn(&input.isbn_10, 10)?,
        isbn_13: normalize_optional_isbn(&input.isbn_13, 13)?,
        publisher: input.publisher.trim().to_string(),
        published_date: input.published_date.trim().to_string(),
        language: input.language.trim().to_string(),
        page_count: input.page_count,
        audio_duration_minutes: input.audio_duration_minutes,
        cover_url: validate_optional_url(&input.cover_url)?,
        is_ebook: input.format == BookFormat::Ebook,
        access_viewability: String::new(),
    })
}

fn validate_edition_ownership(
    connection: &Connection,
    title_id: i64,
    edition_id: Option<i64>,
) -> Result<(), MediaError> {
    if let Some(edition_id) = edition_id {
        let valid: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM obj_media_book_edition WHERE id=?1 AND media_title_id=?2)",
            params![edition_id, title_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if !valid {
            return Err(MediaError::validation(
                "The selected edition does not belong to this book.",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::books::domain::{NormalizedBookLink, NormalizedBookSource};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "overlay-forge-books-{test_name}-{}-{unique}.sqlite3",
            std::process::id()
        ))
    }

    fn remove_db_files(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
    }

    fn with_database<T>(
        name: &str,
        test: impl FnOnce(&AppDatabase, &BookRepository<'_>) -> T,
    ) -> T {
        let path = temp_db_path(name);
        remove_db_files(&path);
        let database = AppDatabase::new(path.clone()).expect("database should initialize");
        let result = {
            let repository = BookRepository::new(&database);
            test(&database, &repository)
        };
        drop(database);
        remove_db_files(&path);
        result
    }

    fn manual_input(title: &str, isbn: &str) -> CreateManualBookInput {
        CreateManualBookInput {
            title: title.to_string(),
            subtitle: String::new(),
            authors: vec!["Test Author".to_string()],
            description: "A local book".to_string(),
            first_publish_year: Some(2026),
            isbn_10: String::new(),
            isbn_13: isbn.to_string(),
            publisher: "Test Press".to_string(),
            published_date: "2026".to_string(),
            page_count: Some(100),
            audio_duration_minutes: None,
            language: "en".to_string(),
            cover_url: String::new(),
            preferred_format: BookFormat::Paperback,
            library_status: MediaLibraryStatus::Planned,
            notes: "Keep this note".to_string(),
            personal_rating: None,
            is_favorite: false,
            priority: 0,
            ownership_status: BookOwnershipStatus::None,
            is_read_next: false,
            series_name: String::new(),
            series_position: String::new(),
        }
    }

    fn provider_import(title: &str) -> NormalizedBookImport {
        NormalizedBookImport {
            title: title.to_string(),
            subtitle: "Provider subtitle".to_string(),
            description: "Provider description".to_string(),
            authors: vec!["Provider Author".to_string()],
            first_publish_year: Some(2024),
            subjects: vec!["Fiction".to_string()],
            canonical_cover_url: String::new(),
            community_rating: Some(4.0),
            community_rating_count: Some(10),
            editions: vec![NormalizedBookEdition {
                title: title.to_string(),
                subtitle: String::new(),
                format: BookFormat::Paperback,
                isbn_10: String::new(),
                isbn_13: "9780306406157".to_string(),
                publisher: "Provider Press".to_string(),
                published_date: "2024".to_string(),
                language: "en".to_string(),
                page_count: Some(200),
                audio_duration_minutes: None,
                cover_url: String::new(),
                is_ebook: false,
                access_viewability: String::new(),
            }],
            sources: vec![NormalizedBookSource {
                edition_index: Some(0),
                source_key: "google_books".to_string(),
                entity_type: "EDITION".to_string(),
                external_id: "google-volume-1".to_string(),
                source_url: "https://books.google.com/books?id=google-volume-1".to_string(),
                metadata_json: "{\"version\":1}".to_string(),
            }],
            links: vec![NormalizedBookLink {
                edition_index: Some(0),
                source_key: "google_books".to_string(),
                link_type: "INFO".to_string(),
                url: "https://books.google.com/books?id=google-volume-1".to_string(),
                region_code: String::new(),
            }],
            series: Vec::new(),
        }
    }

    #[test]
    fn populated_pre_book_schema_rebuild_preserves_ids_and_foreign_keys() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys=ON;
                CREATE TABLE obj_media_title (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    source_key TEXT NOT NULL, external_id INTEGER, external_media_type TEXT,
                    content_type TEXT NOT NULL CHECK(content_type IN ('MOVIE','SERIES')),
                    title TEXT NOT NULL, original_title TEXT NOT NULL DEFAULT '', overview TEXT NOT NULL DEFAULT '',
                    original_language TEXT NOT NULL DEFAULT '', release_date TEXT NOT NULL DEFAULT '',
                    first_air_date TEXT NOT NULL DEFAULT '', last_air_date TEXT NOT NULL DEFAULT '',
                    runtime_minutes INTEGER, episode_runtime_minutes INTEGER, external_status TEXT NOT NULL DEFAULT '',
                    series_type TEXT NOT NULL DEFAULT '', poster_path TEXT NOT NULL DEFAULT '', backdrop_path TEXT NOT NULL DEFAULT '',
                    homepage_url TEXT NOT NULL DEFAULT '', total_seasons INTEGER, total_episodes INTEGER,
                    metadata_json TEXT NOT NULL DEFAULT '{}', metadata_refreshed_at TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(source_key,external_media_type,external_id)
                );
                CREATE TABLE dependent (
                    id INTEGER PRIMARY KEY, media_title_id INTEGER NOT NULL,
                    FOREIGN KEY(media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
                );
                INSERT INTO obj_media_title (id,source_key,external_id,external_media_type,content_type,title)
                VALUES (41,'tmdb',99,'movie','MOVIE','Existing Movie');
                INSERT INTO obj_media_title (id,source_key,external_id,external_media_type,content_type,title)
                VALUES (42,'tmdb',100,'tv','SERIES','Existing Series');
                INSERT INTO dependent (id,media_title_id) VALUES (1,41);
                INSERT INTO dependent (id,media_title_id) VALUES (2,42);
                ",
            )
            .expect("pre-book schema");

        migrate_media_title_book_constraint(&connection).expect("migration should succeed");
        migrate_media_title_book_constraint(&connection).expect("migration should be idempotent");
        let preserved = connection
            .prepare(
                "SELECT id,source_key,external_id,external_media_type,content_type,title
                 FROM obj_media_title ORDER BY id",
            )
            .expect("preservation query")
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .expect("preserved titles")
            .collect::<SqlResult<Vec<_>>>()
            .expect("preserved rows");
        assert_eq!(
            preserved,
            vec![
                (
                    41,
                    "tmdb".to_string(),
                    99,
                    "movie".to_string(),
                    "MOVIE".to_string(),
                    "Existing Movie".to_string(),
                ),
                (
                    42,
                    "tmdb".to_string(),
                    100,
                    "tv".to_string(),
                    "SERIES".to_string(),
                    "Existing Series".to_string(),
                ),
            ]
        );
        connection
            .execute(
                "INSERT INTO obj_media_title (source_key,external_media_type,content_type,title) VALUES ('manual','book','BOOK','Book')",
                [],
            )
            .expect("BOOK should be accepted");
        let violations: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        assert_eq!(violations, 0);
    }

    #[test]
    fn failed_book_constraint_rebuild_rolls_back_without_losing_titles() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys=ON;
                CREATE TABLE obj_media_title (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    source_key TEXT NOT NULL,
                    external_id INTEGER,
                    external_media_type TEXT,
                    content_type TEXT NOT NULL CHECK(content_type IN ('MOVIE','SERIES')),
                    title TEXT NOT NULL
                );
                INSERT INTO obj_media_title
                    (id,source_key,external_id,external_media_type,content_type,title)
                VALUES (41,'tmdb',99,'movie','MOVIE','Existing Movie');
                ",
            )
            .expect("intentionally incomplete pre-book schema");

        assert!(migrate_media_title_book_constraint(&connection).is_err());
        let preserved: (i64, String) = connection
            .query_row("SELECT id,title FROM obj_media_title", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("original table should remain");
        assert_eq!(preserved, (41, "Existing Movie".to_string()));
        let temporary_table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='table' AND name='obj_media_title_book_migration'",
                [],
                |row| row.get(0),
            )
            .expect("temporary table query");
        assert_eq!(temporary_table_count, 0);
        let foreign_keys_enabled: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign key state");
        assert_eq!(foreign_keys_enabled, 1);
    }

    #[test]
    fn manual_books_work_without_provider_credentials_and_deduplicate_by_isbn() {
        with_database("manual", |_database, repository| {
            let first = repository
                .create_manual(&manual_input("First", "9780306406157"))
                .expect("manual book should save");
            let duplicate = repository
                .create_manual(&manual_input("Duplicate", "9780306406157"))
                .expect("exact ISBN should return existing detail");
            assert_eq!(first.item.entry_id, duplicate.item.entry_id);
            assert_eq!(duplicate.item.notes, "Keep this note");
        });
    }

    #[test]
    fn manual_book_creation_persists_optional_reader_and_edition_fields() {
        with_database("manual-fields", |_database, repository| {
            let mut input = manual_input("Configured manual book", "");
            input.audio_duration_minutes = Some(480);
            input.preferred_format = BookFormat::Audiobook;
            input.personal_rating = Some(8.5);
            input.is_favorite = true;
            input.priority = 4;
            input.ownership_status = BookOwnershipStatus::Owned;
            input.is_read_next = true;
            input.series_name = "Local Series".to_string();
            input.series_position = "2".to_string();

            let detail = repository
                .create_manual(&input)
                .expect("manual options should save");
            assert_eq!(detail.item.personal_rating, Some(8.5));
            assert!(detail.item.is_favorite);
            assert_eq!(detail.item.priority, 4);
            assert_eq!(
                detail.item.progress.ownership_status,
                BookOwnershipStatus::Owned
            );
            assert_eq!(detail.item.progress.preferred_format, BookFormat::Audiobook);
            assert_eq!(detail.item.progress.reading_queue_position, Some(1));
            assert_eq!(detail.editions[0].audio_duration_minutes, Some(480));
            assert_eq!(detail.editions[0].language, "en");
            assert!(detail.editions[0].is_user_owned);
            assert_eq!(detail.series[0].name, "Local Series");
            assert!(detail.series[0].is_user_override);
        });
    }

    #[test]
    fn preferred_edition_change_preserves_progress_and_warns_above_new_total() {
        with_database("preferred-edition", |_database, repository| {
            let detail = repository
                .create_manual(&manual_input("Edition progress", ""))
                .expect("book should save");
            let entry_id = detail.item.entry_id;
            let detail = repository
                .create_edition(
                    entry_id,
                    &ManualBookEditionInput {
                        title: "Short edition".to_string(),
                        subtitle: String::new(),
                        format: BookFormat::Paperback,
                        isbn_10: String::new(),
                        isbn_13: String::new(),
                        publisher: String::new(),
                        published_date: String::new(),
                        language: "en".to_string(),
                        page_count: Some(40),
                        audio_duration_minutes: None,
                        cover_url: String::new(),
                    },
                )
                .expect("manual edition should save");
            let short_edition = detail
                .editions
                .iter()
                .find(|edition| edition.title == "Short edition")
                .expect("short edition");
            assert!(short_edition.is_user_owned);
            repository
                .set_progress(
                    entry_id,
                    &SetBookProgressInput {
                        progress_unit: BookProgressUnit::Page,
                        progress_value: 75.0,
                        progress_total_override: None,
                        current_chapter: String::new(),
                    },
                )
                .expect("progress should save");
            let changed = repository
                .set_preferred_edition(entry_id, short_edition.id)
                .expect("preferred edition should change");
            assert_eq!(changed.item.progress.progress_value, 75.0);
            assert_eq!(changed.item.progress.effective_total, Some(40.0));
            assert!(!changed.provider_warnings.is_empty());
        });
    }

    #[test]
    fn progress_queue_and_preferred_edition_are_user_owned() {
        with_database("state", |_database, repository| {
            let detail = repository
                .create_manual(&manual_input("Progress", "9780306406157"))
                .expect("book should save");
            let entry_id = detail.item.entry_id;
            let preferred = detail.item.progress.preferred_edition_id;
            let progressed = repository
                .set_progress(
                    entry_id,
                    &SetBookProgressInput {
                        progress_unit: BookProgressUnit::Page,
                        progress_value: 50.0,
                        progress_total_override: None,
                        current_chapter: "Five".to_string(),
                    },
                )
                .expect("progress should save");
            assert_eq!(progressed.item.library_status, MediaLibraryStatus::Watching);
            assert_eq!(progressed.item.progress.progress_percent, Some(50.0));
            let queued = repository
                .add_to_read_next(entry_id)
                .expect("queue should save");
            assert_eq!(queued.item.progress.reading_queue_position, Some(1));
            assert_eq!(queued.item.progress.preferred_edition_id, preferred);
            let completed = repository.mark_read(entry_id).expect("mark read");
            assert_eq!(completed.item.library_status, MediaLibraryStatus::Completed);
            let reset = repository.reset_progress(entry_id).expect("reset");
            assert_eq!(reset.item.library_status, MediaLibraryStatus::Planned);
        });
    }

    #[test]
    fn watch_next_column_remains_null_for_books() {
        with_database("queues", |database, repository| {
            let detail = repository
                .create_manual(&manual_input("Queue", "9780306406157"))
                .expect("book should save");
            repository
                .add_to_read_next(detail.item.entry_id)
                .expect("read next should save");
            let connection = database.connection().expect("connection");
            let watch_next: Option<i64> = connection
                .query_row(
                    "SELECT queue_position FROM obj_media_library_entry WHERE id=?1",
                    [detail.item.entry_id],
                    |row| row.get(0),
                )
                .expect("entry");
            assert_eq!(watch_next, None);
        });
    }

    #[test]
    fn read_next_move_and_remove_keep_positions_unique_and_dense() {
        with_database("queue-order", |_database, repository| {
            let first = repository
                .create_manual(&manual_input("First queue book", ""))
                .expect("first book");
            let second = repository
                .create_manual(&manual_input("Second queue book", ""))
                .expect("second book");
            let third = repository
                .create_manual(&manual_input("Third queue book", ""))
                .expect("third book");
            for entry_id in [
                first.item.entry_id,
                second.item.entry_id,
                third.item.entry_id,
            ] {
                repository
                    .add_to_read_next(entry_id)
                    .expect("queue insertion");
            }
            repository
                .move_read_next(third.item.entry_id, "UP")
                .expect("queue move");
            repository
                .remove_from_read_next(first.item.entry_id)
                .expect("queue removal");

            let mut positions = [second.item.entry_id, third.item.entry_id]
                .into_iter()
                .map(|entry_id| {
                    repository
                        .get_detail(entry_id)
                        .expect("queued detail")
                        .item
                        .progress
                        .reading_queue_position
                        .expect("queue position")
                })
                .collect::<Vec<_>>();
            positions.sort_unstable();
            assert_eq!(positions, vec![1, 2]);
            assert_eq!(
                repository
                    .get_detail(first.item.entry_id)
                    .expect("removed detail")
                    .item
                    .progress
                    .reading_queue_position,
                None
            );
        });
    }

    #[test]
    fn provider_refresh_preserves_every_user_owned_field() {
        with_database("refresh", |database, repository| {
            let initial = repository
                .add_import(
                    &provider_import("Original"),
                    MediaLibraryStatus::Planned,
                    &[],
                )
                .expect("provider import");
            let entry_id = initial.item.entry_id;
            repository
                .update_reader_state(
                    entry_id,
                    &UpdateBookReaderStateInput {
                        ownership_status: BookOwnershipStatus::Owned,
                        preferred_format: BookFormat::Paperback,
                    },
                )
                .expect("reader state");
            repository
                .set_progress(
                    entry_id,
                    &SetBookProgressInput {
                        progress_unit: BookProgressUnit::Page,
                        progress_value: 75.0,
                        progress_total_override: Some(250.0),
                        current_chapter: "Chapter 3".to_string(),
                    },
                )
                .expect("progress");
            repository.add_to_read_next(entry_id).expect("queue");
            repository
                .create_link(
                    entry_id,
                    &BookLinkInput {
                        edition_id: None,
                        link_type: "READ".to_string(),
                        url: "https://example.com/my-copy".to_string(),
                        region_code: "CA".to_string(),
                        is_preferred: true,
                    },
                )
                .expect("user link");
            let with_manual_edition = repository
                .create_edition(
                    entry_id,
                    &ManualBookEditionInput {
                        title: "My edition".to_string(),
                        subtitle: String::new(),
                        format: BookFormat::Hardcover,
                        isbn_10: String::new(),
                        isbn_13: String::new(),
                        publisher: "Local Press".to_string(),
                        published_date: "2025".to_string(),
                        language: "en-CA".to_string(),
                        page_count: Some(250),
                        audio_duration_minutes: None,
                        cover_url: String::new(),
                    },
                )
                .expect("manual edition");
            let manual_edition_id = with_manual_edition
                .editions
                .iter()
                .find(|edition| edition.title == "My edition")
                .expect("manual edition detail")
                .id;
            repository
                .set_preferred_edition(entry_id, manual_edition_id)
                .expect("preferred manual edition");
            repository
                .set_series_override(
                    entry_id,
                    &BookSeriesOverrideInput {
                        name: "My Series".to_string(),
                        position_text: "3".to_string(),
                        position_sort: Some(3.0),
                        is_primary: true,
                    },
                )
                .expect("series override");
            {
                let connection = database.connection().expect("connection");
                connection
                    .execute(
                        "UPDATE obj_media_library_entry SET
                            library_status='ON_HOLD',is_favorite=1,personal_rating=9.0,
                            notes='Personal note',priority=7,
                            started_at='2026-01-02 03:04:05',completed_at='2026-02-03 04:05:06'
                         WHERE id=?1",
                        [entry_id],
                    )
                    .expect("shared user state");
                connection
                    .execute("INSERT INTO obj_media_tag (name) VALUES ('Keeper')", [])
                    .expect("tag");
                let tag_id = connection.last_insert_rowid();
                connection
                    .execute(
                        "INSERT INTO n2n_media_library_entry_tag
                            (media_library_entry_id,media_tag_id) VALUES (?1,?2)",
                        params![entry_id, tag_id],
                    )
                    .expect("tag mapping");
            }

            let mut refreshed = provider_import("Refreshed title");
            refreshed.description = "New provider description".to_string();
            refreshed.editions[0].page_count = Some(220);
            let detail = repository
                .refresh_import(entry_id, &refreshed, &[])
                .expect("refresh should succeed");
            assert_eq!(detail.item.title, "Refreshed title");
            assert_eq!(detail.item.progress.progress_value, 75.0);
            assert_eq!(detail.item.progress.effective_total, Some(250.0));
            assert_eq!(detail.item.progress.current_chapter, "Chapter 3");
            assert_eq!(
                detail.item.progress.ownership_status,
                BookOwnershipStatus::Owned
            );
            assert_eq!(detail.item.progress.reading_queue_position, Some(1));
            assert_eq!(detail.item.library_status, MediaLibraryStatus::OnHold);
            assert!(detail.item.is_favorite);
            assert_eq!(detail.item.personal_rating, Some(9.0));
            assert_eq!(detail.item.notes, "Personal note");
            assert_eq!(detail.item.priority, 7);
            assert_eq!(detail.item.started_at, "2026-01-02 03:04:05");
            assert_eq!(detail.item.completed_at, "2026-02-03 04:05:06");
            assert_eq!(
                detail.item.progress.preferred_edition_id,
                Some(manual_edition_id)
            );
            assert!(detail
                .editions
                .iter()
                .any(|edition| edition.id == manual_edition_id && edition.is_present_in_source));
            assert!(detail
                .series
                .iter()
                .any(|series| series.name == "My Series" && series.is_user_override));
            assert!(detail.tags.iter().any(|tag| tag.name == "Keeper"));
            assert!(detail
                .links
                .iter()
                .any(|link| link.is_user_owned && link.url == "https://example.com/my-copy"));
        });
    }

    #[test]
    fn provider_failure_keeps_cached_payload_and_other_sources_untouched() {
        with_database("provider-failure", |database, repository| {
            let detail = repository
                .add_import(&provider_import("Cached"), MediaLibraryStatus::Planned, &[])
                .expect("provider import");
            repository
                .mark_provider_failure(detail.item.entry_id, "google_books", "temporary outage")
                .expect("failure state");
            let connection = database.connection().expect("connection");
            let (payload, status, error): (String, String, String) = connection
                .query_row(
                    "SELECT metadata_json,last_refresh_status,last_refresh_error FROM obj_media_book_source_record WHERE source_key='google_books'",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .expect("source record");
            assert_eq!(payload, "{\"version\":1}");
            assert_eq!(status, "FAILED");
            assert_eq!(error, "temporary outage");
        });
    }

    #[test]
    fn partial_provider_failure_is_stored_separately_from_successful_sources() {
        with_database("partial-provider-failure", |_database, repository| {
            let failures = vec![BookProviderFailure {
                source_key: "open_library".to_string(),
                message: "temporary outage".to_string(),
            }];
            let detail = repository
                .add_import(
                    &provider_import("Partial success"),
                    MediaLibraryStatus::Planned,
                    &failures,
                )
                .expect("successful provider import should survive optional failure");
            assert!(detail.sources.iter().any(|source| {
                source.source_key == "google_books" && source.last_refresh_status == "SUCCESS"
            }));
            assert!(detail.sources.iter().any(|source| {
                source.source_key == "open_library"
                    && source.last_refresh_status == "FAILED"
                    && source.last_refresh_error == "temporary outage"
            }));
            assert_eq!(detail.provider_warnings.len(), 1);
        });
    }

    #[test]
    fn book_links_reject_unsafe_urls() {
        with_database("unsafe-link", |_database, repository| {
            let detail = repository
                .create_manual(&manual_input("Unsafe", "9780306406157"))
                .expect("book");
            let error = repository
                .create_link(
                    detail.item.entry_id,
                    &BookLinkInput {
                        edition_id: None,
                        link_type: "READ".to_string(),
                        url: "file:///secret.epub".to_string(),
                        region_code: String::new(),
                        is_preferred: false,
                    },
                )
                .expect_err("unsafe URL should fail");
            assert!(error.message.contains("http"));
        });
    }
}
