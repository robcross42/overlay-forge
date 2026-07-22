use super::domain::{
    automatic_series_status, CreateManualMediaInput, MediaContentType, MediaEpisodeRecord,
    MediaError, MediaLibraryDetail, MediaLibraryFilter, MediaLibraryStatus, MediaLibrarySummary,
    MediaNextEpisode, MediaProviderAvailabilityRecord, MediaProviderSnapshotRecord,
    MediaProviderSummary, MediaSeasonRecord, MediaSettingsRecord, MediaStreamingLinkInput,
    MediaStreamingLinkRecord, MediaTagRecord, NormalizedMediaImport, ProgressFacts,
    UpdateMediaEntryInput, UpdateMediaSettingsInput, DEFAULT_MEDIA_REGION,
};
use crate::db::AppDatabase;
use rusqlite::types::Value;
use rusqlite::{
    params, params_from_iter, Connection, OptionalExtension, Result as SqlResult, Transaction,
};
use std::str::FromStr;

pub(crate) fn migrate_schema(connection: &Connection) -> SqlResult<()> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS obj_media_title (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_key TEXT NOT NULL,
            external_id INTEGER,
            external_media_type TEXT,
            content_type TEXT NOT NULL CHECK (content_type IN ('MOVIE', 'SERIES')),
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

        CREATE TABLE IF NOT EXISTS obj_media_library_entry (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL UNIQUE,
            library_status TEXT NOT NULL DEFAULT 'PLANNED'
                CHECK (library_status IN ('PLANNED', 'WATCHING', 'COMPLETED', 'ON_HOLD', 'DROPPED')),
            is_favorite INTEGER NOT NULL DEFAULT 0,
            personal_rating REAL CHECK (
                personal_rating IS NULL OR (personal_rating >= 1.0 AND personal_rating <= 10.0)
            ),
            notes TEXT NOT NULL DEFAULT '',
            priority INTEGER NOT NULL DEFAULT 0,
            queue_position INTEGER UNIQUE,
            new_episodes_count INTEGER NOT NULL DEFAULT 0,
            added_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            started_at TEXT NOT NULL DEFAULT '',
            completed_at TEXT NOT NULL DEFAULT '',
            last_watched_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_season (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            external_id INTEGER,
            season_number INTEGER NOT NULL,
            name TEXT NOT NULL DEFAULT '',
            overview TEXT NOT NULL DEFAULT '',
            air_date TEXT NOT NULL DEFAULT '',
            poster_path TEXT NOT NULL DEFAULT '',
            episode_count INTEGER NOT NULL DEFAULT 0,
            is_present_in_source INTEGER NOT NULL DEFAULT 1,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            metadata_refreshed_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (media_title_id, season_number),
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_episode (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            season_id INTEGER NOT NULL,
            external_id INTEGER,
            season_number INTEGER NOT NULL,
            episode_number INTEGER NOT NULL,
            name TEXT NOT NULL DEFAULT '',
            overview TEXT NOT NULL DEFAULT '',
            air_date TEXT NOT NULL DEFAULT '',
            runtime_minutes INTEGER,
            still_path TEXT NOT NULL DEFAULT '',
            is_present_in_source INTEGER NOT NULL DEFAULT 1,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            metadata_refreshed_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (media_title_id, season_number, episode_number),
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE,
            FOREIGN KEY (season_id) REFERENCES obj_media_season(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_episode_external_identity
            ON obj_media_episode (media_title_id, external_id)
            WHERE external_id IS NOT NULL;

        CREATE TABLE IF NOT EXISTS obj_media_episode_progress (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            episode_id INTEGER NOT NULL UNIQUE,
            is_watched INTEGER NOT NULL DEFAULT 0,
            watched_at TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (episode_id) REFERENCES obj_media_episode(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_provider_snapshot (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_title_id INTEGER NOT NULL,
            region_code TEXT NOT NULL,
            source_key TEXT NOT NULL DEFAULT 'tmdb',
            source_link_url TEXT NOT NULL DEFAULT '',
            refreshed_at TEXT NOT NULL DEFAULT '',
            last_refresh_status TEXT NOT NULL DEFAULT '',
            last_refresh_error TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (media_title_id, region_code),
            FOREIGN KEY (media_title_id) REFERENCES obj_media_title(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_provider_availability (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            provider_external_id INTEGER NOT NULL,
            provider_name TEXT NOT NULL,
            provider_logo_path TEXT NOT NULL DEFAULT '',
            monetization_type TEXT NOT NULL
                CHECK (monetization_type IN ('FLATRATE', 'FREE', 'ADS', 'RENT', 'BUY')),
            display_priority INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (snapshot_id, provider_external_id, monetization_type),
            FOREIGN KEY (snapshot_id) REFERENCES obj_media_provider_snapshot(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_streaming_link (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_library_entry_id INTEGER NOT NULL,
            provider_name TEXT NOT NULL DEFAULT '',
            url TEXT NOT NULL,
            link_type TEXT NOT NULL DEFAULT 'STREAM'
                CHECK (link_type IN ('STREAM', 'RENT', 'BUY', 'OTHER')),
            is_preferred INTEGER NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (media_library_entry_id)
                REFERENCES obj_media_library_entry(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_streaming_link_preferred
            ON obj_media_streaming_link (media_library_entry_id)
            WHERE is_preferred = 1;

        CREATE TABLE IF NOT EXISTS obj_media_tag (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL COLLATE NOCASE UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS n2n_media_library_entry_tag (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_library_entry_id INTEGER NOT NULL,
            media_tag_id INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (media_library_entry_id, media_tag_id),
            FOREIGN KEY (media_library_entry_id)
                REFERENCES obj_media_library_entry(id) ON DELETE CASCADE,
            FOREIGN KEY (media_tag_id) REFERENCES obj_media_tag(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS obj_media_setting (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            region_code TEXT NOT NULL DEFAULT 'CA',
            metadata_language TEXT NOT NULL DEFAULT 'en-CA',
            include_specials_in_completion INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        INSERT OR IGNORE INTO obj_media_setting (
            id, region_code, metadata_language, include_specials_in_completion
        ) VALUES (1, 'CA', 'en-CA', 0);

        CREATE INDEX IF NOT EXISTS idx_media_title_search ON obj_media_title (title COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_media_title_source ON obj_media_title (
            source_key, external_media_type, external_id
        );
        CREATE INDEX IF NOT EXISTS idx_media_library_status ON obj_media_library_entry (library_status);
        CREATE INDEX IF NOT EXISTS idx_media_library_favorite ON obj_media_library_entry (is_favorite);
        CREATE INDEX IF NOT EXISTS idx_media_library_queue ON obj_media_library_entry (queue_position);
        CREATE INDEX IF NOT EXISTS idx_media_library_added ON obj_media_library_entry (added_at);
        CREATE INDEX IF NOT EXISTS idx_media_library_watched ON obj_media_library_entry (last_watched_at);
        CREATE INDEX IF NOT EXISTS idx_media_season_title ON obj_media_season (
            media_title_id, season_number
        );
        CREATE INDEX IF NOT EXISTS idx_media_episode_season ON obj_media_episode (
            season_id, episode_number
        );
        CREATE INDEX IF NOT EXISTS idx_media_episode_title_order ON obj_media_episode (
            media_title_id, season_number, episode_number
        );
        CREATE INDEX IF NOT EXISTS idx_media_episode_progress_watched
            ON obj_media_episode_progress (is_watched);
        CREATE INDEX IF NOT EXISTS idx_media_provider_name
            ON obj_media_provider_availability (provider_name COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_media_tag_mapping_entry
            ON n2n_media_library_entry_tag (media_library_entry_id);
        CREATE INDEX IF NOT EXISTS idx_media_tag_mapping_tag
            ON n2n_media_library_entry_tag (media_tag_id);
        ",
    )?;
    super::books::repository::migrate_schema(connection)
}

pub struct MediaRepository<'a> {
    database: &'a AppDatabase,
}

impl<'a> MediaRepository<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self { database }
    }

    pub fn get_settings(&self) -> Result<MediaSettingsRecord, MediaError> {
        let connection = self.database.connection()?;
        Self::get_settings_from_connection(&connection)
    }

    pub fn update_settings(
        &self,
        input: &UpdateMediaSettingsInput,
    ) -> Result<MediaSettingsRecord, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "
            UPDATE obj_media_setting
            SET region_code = ?1,
                metadata_language = ?2,
                include_specials_in_completion = ?3,
                updated_at = datetime('now', 'localtime')
            WHERE id = 1
            ",
            params![
                input.region_code.trim().to_ascii_uppercase(),
                input.metadata_language.trim(),
                input.include_specials_in_completion as i64,
            ],
        )?;
        let entry_ids = {
            let mut statement = transaction.prepare(
                "
                SELECT entry.id
                FROM obj_media_library_entry entry
                JOIN obj_media_title title ON title.id = entry.media_title_id
                WHERE title.content_type = 'SERIES'
                ",
            )?;
            let rows = statement
                .query_map([], |row| row.get::<_, i64>(0))?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        for entry_id in entry_ids {
            Self::recalculate_series_status(&transaction, entry_id, false)?;
        }
        transaction.commit()?;
        drop(connection);
        self.get_settings()
    }

    fn get_settings_from_connection(
        connection: &Connection,
    ) -> Result<MediaSettingsRecord, MediaError> {
        connection
            .query_row(
                "
                SELECT region_code, metadata_language, include_specials_in_completion, updated_at
                FROM obj_media_setting
                WHERE id = 1
                ",
                [],
                |row| {
                    Ok(MediaSettingsRecord {
                        region_code: row.get(0)?,
                        metadata_language: row.get(1)?,
                        include_specials_in_completion: row.get::<_, i64>(2)? != 0,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .map_err(MediaError::from)
    }

    pub fn find_external_entry(
        &self,
        external_id: i64,
        external_media_type: &str,
    ) -> Result<Option<i64>, MediaError> {
        let connection = self.database.connection()?;
        connection
            .query_row(
                "
                SELECT entry.id
                FROM obj_media_library_entry entry
                JOIN obj_media_title title ON title.id = entry.media_title_id
                WHERE title.source_key = 'tmdb'
                  AND title.external_id = ?1
                  AND title.external_media_type = ?2
                ",
                params![external_id, external_media_type],
                |row| row.get(0),
            )
            .optional()
            .map_err(MediaError::from)
    }

    pub fn source_identity_for_entry(
        &self,
        entry_id: i64,
    ) -> Result<(String, Option<i64>, Option<String>), MediaError> {
        let connection = self.database.connection()?;
        connection
            .query_row(
                "
                SELECT title.source_key, title.external_id, title.external_media_type
                FROM obj_media_library_entry entry
                JOIN obj_media_title title ON title.id = entry.media_title_id
                WHERE entry.id = ?1
                ",
                [entry_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Media library entry was not found."))
    }

    pub fn add_catalog_import(
        &self,
        import: &NormalizedMediaImport,
        status: MediaLibraryStatus,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = Self::upsert_imported_title(&transaction, import)?;
        if let Some(existing_id) = transaction
            .query_row(
                "SELECT id FROM obj_media_library_entry WHERE media_title_id = ?1",
                [title_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
        {
            return Err(MediaError::new(
                super::domain::MediaErrorKind::Duplicate,
                format!("This title is already in the library (entry {existing_id})."),
            ));
        }
        transaction.execute(
            "
            INSERT INTO obj_media_library_entry (media_title_id, library_status)
            VALUES (?1, ?2)
            ",
            params![title_id, status.to_string()],
        )?;
        let entry_id = transaction.last_insert_rowid();
        Self::apply_import_children(&transaction, title_id, import, entry_id, false)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn create_manual_entry(
        &self,
        input: &CreateManualMediaInput,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "
            INSERT INTO obj_media_title (
                source_key, external_id, external_media_type, content_type, title,
                original_title, overview, release_date, first_air_date, runtime_minutes,
                metadata_json, metadata_refreshed_at
            )
            VALUES (
                'manual', NULL, NULL, ?1, ?2, ?2, ?3,
                CASE WHEN ?1 = 'MOVIE' THEN ?4 ELSE '' END,
                CASE WHEN ?1 = 'SERIES' THEN ?4 ELSE '' END,
                ?5, '{}', datetime('now', 'localtime')
            )
            ",
            params![
                input.content_type.to_string(),
                input.title.trim(),
                input.overview.trim(),
                input.release_date.trim(),
                input.runtime_minutes,
            ],
        )?;
        let title_id = transaction.last_insert_rowid();
        let queue_position = if input.add_to_watch_next {
            Some(Self::next_queue_position(&transaction)?)
        } else {
            None
        };
        transaction.execute(
            "
            INSERT INTO obj_media_library_entry (
                media_title_id, library_status, is_favorite, personal_rating,
                notes, priority, queue_position
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                title_id,
                input.library_status.to_string(),
                input.is_favorite as i64,
                input.personal_rating,
                input.notes.trim(),
                input.priority,
                queue_position,
            ],
        )?;
        let entry_id = transaction.last_insert_rowid();
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn refresh_import(
        &self,
        entry_id: i64,
        import: &NormalizedMediaImport,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let expected_title_id: i64 = transaction
            .query_row(
                "SELECT media_title_id FROM obj_media_library_entry WHERE id = ?1",
                [entry_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Media library entry was not found."))?;
        let title_id = Self::upsert_imported_title(&transaction, import)?;
        if expected_title_id != title_id {
            return Err(MediaError::database(
                "The refreshed title did not match the saved library entry.",
            ));
        }
        Self::apply_import_children(&transaction, title_id, import, entry_id, true)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    fn upsert_imported_title(
        transaction: &Transaction<'_>,
        import: &NormalizedMediaImport,
    ) -> Result<i64, MediaError> {
        let title = &import.title;
        transaction.execute(
            "
            INSERT INTO obj_media_title (
                source_key, external_id, external_media_type, content_type, title,
                original_title, overview, original_language, release_date, first_air_date,
                last_air_date, runtime_minutes, episode_runtime_minutes, external_status,
                series_type, poster_path, backdrop_path, homepage_url, total_seasons,
                total_episodes, metadata_json, metadata_refreshed_at
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19, ?20, ?21, datetime('now', 'localtime')
            )
            ON CONFLICT (source_key, external_media_type, external_id) DO UPDATE SET
                content_type = excluded.content_type,
                title = excluded.title,
                original_title = excluded.original_title,
                overview = excluded.overview,
                original_language = excluded.original_language,
                release_date = excluded.release_date,
                first_air_date = excluded.first_air_date,
                last_air_date = excluded.last_air_date,
                runtime_minutes = excluded.runtime_minutes,
                episode_runtime_minutes = excluded.episode_runtime_minutes,
                external_status = excluded.external_status,
                series_type = excluded.series_type,
                poster_path = excluded.poster_path,
                backdrop_path = excluded.backdrop_path,
                homepage_url = excluded.homepage_url,
                total_seasons = excluded.total_seasons,
                total_episodes = excluded.total_episodes,
                metadata_json = excluded.metadata_json,
                metadata_refreshed_at = datetime('now', 'localtime'),
                updated_at = datetime('now', 'localtime')
            ",
            params![
                title.source_key,
                title.external_id,
                title.external_media_type,
                title.content_type.to_string(),
                title.title,
                title.original_title,
                title.overview,
                title.original_language,
                title.release_date,
                title.first_air_date,
                title.last_air_date,
                title.runtime_minutes,
                title.episode_runtime_minutes,
                title.external_status,
                title.series_type,
                title.poster_path,
                title.backdrop_path,
                title.homepage_url,
                title.total_seasons,
                title.total_episodes,
                title.metadata_json,
            ],
        )?;
        transaction
            .query_row(
                "
                SELECT id FROM obj_media_title
                WHERE source_key = ?1 AND external_media_type = ?2 AND external_id = ?3
                ",
                params![
                    title.source_key,
                    title.external_media_type,
                    title.external_id
                ],
                |row| row.get(0),
            )
            .map_err(MediaError::from)
    }

    fn apply_import_children(
        transaction: &Transaction<'_>,
        title_id: i64,
        import: &NormalizedMediaImport,
        entry_id: i64,
        is_refresh: bool,
    ) -> Result<(), MediaError> {
        let include_specials = Self::include_specials(transaction)?;
        let old_total = Self::included_episode_count(transaction, title_id, include_specials)?;
        if import.title.content_type == MediaContentType::Series {
            transaction.execute(
                "UPDATE obj_media_season SET is_present_in_source = 0 WHERE media_title_id = ?1",
                [title_id],
            )?;
            transaction.execute(
                "UPDATE obj_media_episode SET is_present_in_source = 0 WHERE media_title_id = ?1",
                [title_id],
            )?;
            for season in &import.seasons {
                transaction.execute(
                    "
                    INSERT INTO obj_media_season (
                        media_title_id, external_id, season_number, name, overview, air_date,
                        poster_path, episode_count, is_present_in_source, metadata_json,
                        metadata_refreshed_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, datetime('now', 'localtime'))
                    ON CONFLICT (media_title_id, season_number) DO UPDATE SET
                        external_id = excluded.external_id,
                        name = excluded.name,
                        overview = excluded.overview,
                        air_date = excluded.air_date,
                        poster_path = excluded.poster_path,
                        episode_count = excluded.episode_count,
                        is_present_in_source = 1,
                        metadata_json = excluded.metadata_json,
                        metadata_refreshed_at = datetime('now', 'localtime'),
                        updated_at = datetime('now', 'localtime')
                    ",
                    params![
                        title_id,
                        season.external_id,
                        season.season_number,
                        season.name,
                        season.overview,
                        season.air_date,
                        season.poster_path,
                        season.episode_count,
                        season.metadata_json,
                    ],
                )?;
                let season_id: i64 = transaction.query_row(
                    "
                    SELECT id FROM obj_media_season
                    WHERE media_title_id = ?1 AND season_number = ?2
                    ",
                    params![title_id, season.season_number],
                    |row| row.get(0),
                )?;
                for episode in &season.episodes {
                    transaction.execute(
                        "
                        INSERT INTO obj_media_episode (
                            media_title_id, season_id, external_id, season_number,
                            episode_number, name, overview, air_date, runtime_minutes,
                            still_path, is_present_in_source, metadata_json, metadata_refreshed_at
                        )
                        VALUES (
                            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11,
                            datetime('now', 'localtime')
                        )
                        ON CONFLICT (media_title_id, season_number, episode_number) DO UPDATE SET
                            season_id = excluded.season_id,
                            external_id = excluded.external_id,
                            name = excluded.name,
                            overview = excluded.overview,
                            air_date = excluded.air_date,
                            runtime_minutes = excluded.runtime_minutes,
                            still_path = excluded.still_path,
                            is_present_in_source = 1,
                            metadata_json = excluded.metadata_json,
                            metadata_refreshed_at = datetime('now', 'localtime'),
                            updated_at = datetime('now', 'localtime')
                        ",
                        params![
                            title_id,
                            season_id,
                            episode.external_id,
                            episode.season_number,
                            episode.episode_number,
                            episode.name,
                            episode.overview,
                            episode.air_date,
                            episode.runtime_minutes,
                            episode.still_path,
                            episode.metadata_json,
                        ],
                    )?;
                }
            }
            let new_total = Self::included_episode_count(transaction, title_id, include_specials)?;
            if is_refresh && new_total > old_total {
                transaction.execute(
                    "
                    UPDATE obj_media_library_entry
                    SET new_episodes_count = new_episodes_count + ?2,
                        updated_at = datetime('now', 'localtime')
                    WHERE id = ?1
                    ",
                    params![entry_id, new_total - old_total],
                )?;
            }
        }
        Self::replace_or_mark_availability(transaction, title_id, import)?;
        if import.title.content_type == MediaContentType::Series && is_refresh {
            Self::recalculate_series_status(transaction, entry_id, true)?;
        }
        Ok(())
    }

    fn replace_or_mark_availability(
        transaction: &Transaction<'_>,
        title_id: i64,
        import: &NormalizedMediaImport,
    ) -> Result<(), MediaError> {
        if let Some(snapshot) = &import.provider_snapshot {
            transaction.execute(
                "
                INSERT INTO obj_media_provider_snapshot (
                    media_title_id, region_code, source_key, source_link_url,
                    refreshed_at, last_refresh_status, last_refresh_error
                )
                VALUES (
                    ?1, ?2, 'tmdb', ?3, datetime('now', 'localtime'), 'SUCCESS', ''
                )
                ON CONFLICT (media_title_id, region_code) DO UPDATE SET
                    source_link_url = excluded.source_link_url,
                    refreshed_at = datetime('now', 'localtime'),
                    last_refresh_status = 'SUCCESS',
                    last_refresh_error = '',
                    updated_at = datetime('now', 'localtime')
                ",
                params![title_id, snapshot.region_code, snapshot.source_link_url],
            )?;
            let snapshot_id: i64 = transaction.query_row(
                "
                SELECT id FROM obj_media_provider_snapshot
                WHERE media_title_id = ?1 AND region_code = ?2
                ",
                params![title_id, snapshot.region_code],
                |row| row.get(0),
            )?;
            transaction.execute(
                "DELETE FROM obj_media_provider_availability WHERE snapshot_id = ?1",
                [snapshot_id],
            )?;
            for provider in &snapshot.providers {
                transaction.execute(
                    "
                    INSERT INTO obj_media_provider_availability (
                        snapshot_id, provider_external_id, provider_name,
                        provider_logo_path, monetization_type, display_priority
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                    ",
                    params![
                        snapshot_id,
                        provider.provider_external_id,
                        provider.provider_name,
                        provider.provider_logo_path,
                        provider.monetization_type,
                        provider.display_priority,
                    ],
                )?;
            }
        } else if let Some(error) = &import.provider_error {
            let region: String = transaction
                .query_row(
                    "SELECT region_code FROM obj_media_setting WHERE id = 1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| DEFAULT_MEDIA_REGION.to_string());
            transaction.execute(
                "
                INSERT INTO obj_media_provider_snapshot (
                    media_title_id, region_code, source_key, last_refresh_status, last_refresh_error
                )
                VALUES (?1, ?2, 'tmdb', 'FAILED', ?3)
                ON CONFLICT (media_title_id, region_code) DO UPDATE SET
                    last_refresh_status = 'FAILED',
                    last_refresh_error = excluded.last_refresh_error,
                    updated_at = datetime('now', 'localtime')
                ",
                params![title_id, region, error],
            )?;
        }
        Ok(())
    }

    pub fn list_library(
        &self,
        filter: &MediaLibraryFilter,
    ) -> Result<Vec<MediaLibrarySummary>, MediaError> {
        let connection = self.database.connection()?;
        let settings = Self::get_settings_from_connection(&connection)?;
        let include_specials = settings.include_specials_in_completion;
        let mut sql = Self::summary_select(include_specials);
        sql.push_str(" WHERE 1 = 1");
        let mut values = Vec::<Value>::new();

        if !filter.query.trim().is_empty() {
            sql.push_str(
                " AND (
                    lower(title.title) LIKE lower(?)
                    OR lower(title.original_title) LIKE lower(?)
                    OR lower(title.overview) LIKE lower(?)
                    OR lower(entry.notes) LIKE lower(?)
                )",
            );
            let pattern = format!("%{}%", filter.query.trim());
            for _ in 0..4 {
                values.push(Value::Text(pattern.clone()));
            }
        }
        if let Some(content_type) = filter.content_type {
            sql.push_str(" AND title.content_type = ?");
            values.push(Value::Text(content_type.to_string()));
        }
        if let Some(status) = filter.library_status {
            sql.push_str(" AND entry.library_status = ?");
            values.push(Value::Text(status.to_string()));
        }
        if let Some(is_favorite) = filter.is_favorite {
            sql.push_str(" AND entry.is_favorite = ?");
            values.push(Value::Integer(is_favorite as i64));
        }
        if let Some(tag_id) = filter.tag_id {
            sql.push_str(
                " AND EXISTS (
                    SELECT 1 FROM n2n_media_library_entry_tag mapping
                    WHERE mapping.media_library_entry_id = entry.id
                      AND mapping.media_tag_id = ?
                )",
            );
            values.push(Value::Integer(tag_id));
        }
        if !filter.provider_name.trim().is_empty() {
            sql.push_str(
                " AND EXISTS (
                    SELECT 1
                    FROM obj_media_provider_snapshot snapshot
                    JOIN obj_media_provider_availability availability
                      ON availability.snapshot_id = snapshot.id
                    WHERE snapshot.media_title_id = title.id
                      AND lower(availability.provider_name) LIKE lower(?)
                )",
            );
            values.push(Value::Text(format!("%{}%", filter.provider_name.trim())));
        }
        if let Some(is_watch_next) = filter.is_watch_next {
            sql.push_str(if is_watch_next {
                " AND entry.queue_position IS NOT NULL"
            } else {
                " AND entry.queue_position IS NULL"
            });
        }
        if let Some(has_unwatched) = filter.has_unwatched {
            let included_episode = if include_specials {
                "episode.is_present_in_source = 1"
            } else {
                "episode.is_present_in_source = 1 AND episode.season_number <> 0"
            };
            let expression = format!(
                "(
                    (title.content_type = 'MOVIE' AND entry.completed_at = '')
                    OR
                    (title.content_type = 'SERIES' AND EXISTS (
                        SELECT 1
                        FROM obj_media_episode episode
                        LEFT JOIN obj_media_episode_progress progress
                          ON progress.episode_id = episode.id
                        WHERE episode.media_title_id = title.id
                          AND {included_episode}
                          AND COALESCE(progress.is_watched, 0) = 0
                    ))
                )"
            );
            sql.push_str(if has_unwatched { " AND " } else { " AND NOT " });
            sql.push_str(&expression);
        }

        let direction = if filter.sort_direction.eq_ignore_ascii_case("ASC") {
            "ASC"
        } else {
            "DESC"
        };
        let order = match filter.sort.trim().to_ascii_uppercase().as_str() {
            "TITLE" => format!("title.title COLLATE NOCASE {direction}"),
            "RECENTLY_WATCHED" => {
                format!("entry.last_watched_at {direction}, title.title COLLATE NOCASE ASC")
            }
            "PROGRESS" => format!("progress_percent {direction}, title.title COLLATE NOCASE ASC"),
            "RELEASE_DATE" => format!(
                "CASE WHEN title.content_type IN ('MOVIE','BOOK') THEN title.release_date
                      ELSE title.first_air_date END {direction}"
            ),
            "WATCH_NEXT" => {
                "entry.queue_position IS NULL ASC, entry.queue_position ASC".to_string()
            }
            "PRIORITY" => format!("entry.priority {direction}, entry.added_at DESC"),
            _ => format!("entry.added_at {direction}"),
        };
        sql.push_str(" ORDER BY ");
        sql.push_str(&order);
        sql.push_str(" LIMIT ? OFFSET ?");
        values.push(Value::Integer(filter.limit.clamp(1, 250)));
        values.push(Value::Integer(filter.offset.max(0)));

        let raw_rows = {
            let mut statement = connection.prepare(&sql)?;
            let rows = statement
                .query_map(params_from_iter(values), summary_row_from_row)?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        raw_rows
            .into_iter()
            .map(|row| Self::enrich_summary(&connection, row, include_specials))
            .collect()
    }

    pub fn get_detail(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let settings = Self::get_settings_from_connection(&connection)?;
        let entry = Self::load_summary_by_id(
            &connection,
            entry_id,
            settings.include_specials_in_completion,
        )?
        .ok_or_else(|| MediaError::not_found("Media library entry was not found."))?;
        let seasons = Self::load_seasons(
            &connection,
            entry.media_title_id,
            settings.include_specials_in_completion,
        )?;
        let tags = Self::list_entry_tags_from_connection(&connection, entry_id)?;
        let streaming_links = Self::list_streaming_links_from_connection(&connection, entry_id)?;
        let provider_snapshot = connection
            .query_row(
                "
                SELECT snapshot.id, snapshot.region_code, snapshot.source_link_url,
                       snapshot.refreshed_at, snapshot.last_refresh_status,
                       snapshot.last_refresh_error
                FROM obj_media_provider_snapshot snapshot
                WHERE snapshot.media_title_id = ?1
                  AND snapshot.region_code = ?2
                ",
                params![entry.media_title_id, settings.region_code],
                provider_snapshot_from_row,
            )
            .optional()?;
        let providers = if let Some(snapshot) = &provider_snapshot {
            let mut statement = connection.prepare(
                "
                SELECT id, provider_external_id, provider_name, provider_logo_path,
                       monetization_type, display_priority
                FROM obj_media_provider_availability
                WHERE snapshot_id = ?1
                ORDER BY
                    CASE monetization_type
                        WHEN 'FLATRATE' THEN 1
                        WHEN 'FREE' THEN 2
                        WHEN 'ADS' THEN 3
                        WHEN 'RENT' THEN 4
                        ELSE 5
                    END,
                    display_priority,
                    provider_name COLLATE NOCASE
                ",
            )?;
            let rows = statement
                .query_map([snapshot.id], provider_availability_from_row)?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        } else {
            Vec::new()
        };
        let book_detail = if entry.content_type == MediaContentType::Book {
            super::books::repository::load_media_book_detail(
                &connection,
                entry.id,
                entry.media_title_id,
            )?
        } else {
            None
        };
        Ok(MediaLibraryDetail {
            entry,
            seasons,
            tags,
            streaming_links,
            provider_snapshot,
            providers,
            settings,
            book_detail,
        })
    }

    fn summary_select(include_specials: bool) -> String {
        let included = if include_specials {
            "episode.is_present_in_source = 1"
        } else {
            "episode.is_present_in_source = 1 AND episode.season_number <> 0"
        };
        format!(
            "
            SELECT
                entry.id,
                entry.media_title_id,
                title.source_key,
                title.external_id,
                title.external_media_type,
                title.content_type,
                title.title,
                title.original_title,
                title.overview,
                title.original_language,
                title.release_date,
                title.first_air_date,
                title.last_air_date,
                title.runtime_minutes,
                title.episode_runtime_minutes,
                title.external_status,
                title.series_type,
                title.poster_path,
                title.backdrop_path,
                title.homepage_url,
                title.total_seasons,
                title.total_episodes,
                title.metadata_refreshed_at,
                entry.library_status,
                entry.is_favorite,
                entry.personal_rating,
                entry.notes,
                entry.priority,
                entry.queue_position,
                entry.added_at,
                entry.started_at,
                entry.completed_at,
                entry.last_watched_at,
                entry.new_episodes_count,
                COALESCE((
                    SELECT COUNT(*)
                    FROM obj_media_episode episode
                    JOIN obj_media_episode_progress progress ON progress.episode_id = episode.id
                    WHERE episode.media_title_id = title.id
                      AND {included}
                      AND progress.is_watched = 1
                ), 0) AS watched_count,
                COALESCE((
                    SELECT COUNT(*)
                    FROM obj_media_episode episode
                    WHERE episode.media_title_id = title.id AND {included}
                ), 0) AS progress_total,
                CASE
                    WHEN title.content_type = 'MOVIE' THEN
                        CASE WHEN entry.completed_at <> '' THEN 100.0 ELSE 0.0 END
                    WHEN COALESCE((
                        SELECT COUNT(*)
                        FROM obj_media_episode episode
                        WHERE episode.media_title_id = title.id AND {included}
                    ), 0) = 0 THEN 0.0
                    ELSE (
                        100.0 * COALESCE((
                            SELECT COUNT(*)
                            FROM obj_media_episode episode
                            JOIN obj_media_episode_progress progress
                              ON progress.episode_id = episode.id
                            WHERE episode.media_title_id = title.id
                              AND {included}
                              AND progress.is_watched = 1
                        ), 0)
                        /
                        (
                            SELECT COUNT(*)
                            FROM obj_media_episode episode
                            WHERE episode.media_title_id = title.id AND {included}
                        )
                    )
                END AS progress_percent
            FROM obj_media_library_entry entry
            JOIN obj_media_title title ON title.id = entry.media_title_id
            "
        )
    }

    fn load_summary_by_id(
        connection: &Connection,
        entry_id: i64,
        include_specials: bool,
    ) -> Result<Option<MediaLibrarySummary>, MediaError> {
        let mut sql = Self::summary_select(include_specials);
        sql.push_str(" WHERE entry.id = ?1");
        let raw = connection
            .query_row(&sql, [entry_id], summary_row_from_row)
            .optional()?;
        raw.map(|row| Self::enrich_summary(connection, row, include_specials))
            .transpose()
    }

    fn enrich_summary(
        connection: &Connection,
        row: RawMediaSummary,
        include_specials: bool,
    ) -> Result<MediaLibrarySummary, MediaError> {
        let content_type = MediaContentType::from_str(&row.content_type)?;
        let library_status = MediaLibraryStatus::from_str(&row.library_status)?;
        let next_episode = if content_type == MediaContentType::Series {
            Self::next_episode(connection, row.media_title_id, include_specials)?
        } else {
            None
        };
        let subscription_providers = Self::subscription_providers(connection, row.media_title_id)?;
        let availability_is_stale = connection.query_row(
            "
            SELECT CASE
                WHEN COUNT(*) = 0 THEN 0
                WHEN MAX(last_refresh_status) <> 'SUCCESS' THEN 1
                WHEN MAX(refreshed_at) = '' THEN 1
                WHEN datetime(MAX(refreshed_at)) < datetime('now', '-7 days') THEN 1
                ELSE 0
            END
            FROM obj_media_provider_snapshot
            WHERE media_title_id = ?1
            ",
            [row.media_title_id],
            |db_row| Ok(db_row.get::<_, i64>(0)? != 0),
        )?;
        let book_summary = if content_type == MediaContentType::Book {
            super::books::repository::load_media_book_summary(connection, row.id)?
        } else {
            None
        };
        Ok(MediaLibrarySummary {
            id: row.id,
            media_title_id: row.media_title_id,
            source_key: row.source_key,
            external_id: row.external_id,
            external_media_type: row.external_media_type,
            content_type,
            title: row.title,
            original_title: row.original_title,
            overview: row.overview,
            original_language: row.original_language,
            release_date: row.release_date,
            first_air_date: row.first_air_date,
            last_air_date: row.last_air_date,
            runtime_minutes: row.runtime_minutes,
            episode_runtime_minutes: row.episode_runtime_minutes,
            external_status: row.external_status,
            series_type: row.series_type,
            poster_path: row.poster_path,
            backdrop_path: row.backdrop_path,
            homepage_url: row.homepage_url,
            total_seasons: row.total_seasons,
            total_episodes: row.total_episodes,
            metadata_refreshed_at: row.metadata_refreshed_at,
            library_status,
            is_favorite: row.is_favorite,
            personal_rating: row.personal_rating,
            notes: row.notes,
            priority: row.priority,
            queue_position: row.queue_position,
            added_at: row.added_at,
            started_at: row.started_at,
            completed_at: row.completed_at.clone(),
            last_watched_at: row.last_watched_at.clone(),
            movie_watched: content_type == MediaContentType::Movie
                && (!row.completed_at.is_empty() || !row.last_watched_at.is_empty()),
            watched_count: if content_type == MediaContentType::Movie {
                (!row.completed_at.is_empty()) as i64
            } else {
                row.watched_count
            },
            progress_total: if content_type == MediaContentType::Movie {
                1
            } else {
                row.progress_total
            },
            progress_percent: row.progress_percent,
            next_episode,
            new_episodes_count: row.new_episodes_count,
            subscription_providers,
            availability_is_stale,
            book_summary,
        })
    }

    fn next_episode(
        connection: &Connection,
        title_id: i64,
        include_specials: bool,
    ) -> Result<Option<MediaNextEpisode>, MediaError> {
        let special_clause = if include_specials {
            ""
        } else {
            "AND episode.season_number <> 0"
        };
        connection
            .query_row(
                &format!(
                    "
                    SELECT episode.id, episode.season_number, episode.episode_number,
                           episode.name, episode.air_date
                    FROM obj_media_episode episode
                    LEFT JOIN obj_media_episode_progress progress
                      ON progress.episode_id = episode.id
                    WHERE episode.media_title_id = ?1
                      AND episode.is_present_in_source = 1
                      {special_clause}
                      AND COALESCE(progress.is_watched, 0) = 0
                    ORDER BY episode.season_number, episode.episode_number
                    LIMIT 1
                    "
                ),
                [title_id],
                |row| {
                    Ok(MediaNextEpisode {
                        id: row.get(0)?,
                        season_number: row.get(1)?,
                        episode_number: row.get(2)?,
                        name: row.get(3)?,
                        air_date: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(MediaError::from)
    }

    fn subscription_providers(
        connection: &Connection,
        title_id: i64,
    ) -> Result<Vec<MediaProviderSummary>, MediaError> {
        let mut statement = connection.prepare(
            "
            SELECT DISTINCT availability.provider_name, availability.provider_logo_path
            FROM obj_media_provider_snapshot snapshot
            JOIN obj_media_provider_availability availability
              ON availability.snapshot_id = snapshot.id
            WHERE snapshot.media_title_id = ?1
              AND availability.monetization_type = 'FLATRATE'
            ORDER BY availability.display_priority, availability.provider_name COLLATE NOCASE
            LIMIT 8
            ",
        )?;
        let rows = statement
            .query_map([title_id], |row| {
                Ok(MediaProviderSummary {
                    provider_name: row.get(0)?,
                    provider_logo_path: row.get(1)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    fn load_seasons(
        connection: &Connection,
        title_id: i64,
        include_specials: bool,
    ) -> Result<Vec<MediaSeasonRecord>, MediaError> {
        let raw_seasons = {
            let mut statement = connection.prepare(
                "
                SELECT id, external_id, season_number, name, overview, air_date,
                       poster_path, episode_count, is_present_in_source
                FROM obj_media_season
                WHERE media_title_id = ?1
                ORDER BY season_number
                ",
            )?;
            let rows = statement
                .query_map([title_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                        row.get::<_, i64>(8)? != 0,
                    ))
                })?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        let mut seasons = Vec::with_capacity(raw_seasons.len());
        for (
            id,
            external_id,
            season_number,
            name,
            overview,
            air_date,
            poster_path,
            episode_count,
            is_present_in_source,
        ) in raw_seasons
        {
            let episodes = Self::load_episodes(connection, id)?;
            let counts_for_progress = include_specials || season_number != 0;
            let progress_total = if counts_for_progress {
                episodes
                    .iter()
                    .filter(|episode| episode.is_present_in_source)
                    .count() as i64
            } else {
                0
            };
            let watched_count = if counts_for_progress {
                episodes
                    .iter()
                    .filter(|episode| episode.is_present_in_source && episode.is_watched)
                    .count() as i64
            } else {
                0
            };
            seasons.push(MediaSeasonRecord {
                id,
                external_id,
                season_number,
                name,
                overview,
                air_date,
                poster_path,
                episode_count,
                is_present_in_source,
                watched_count,
                progress_total,
                episodes,
            });
        }
        Ok(seasons)
    }

    fn load_episodes(
        connection: &Connection,
        season_id: i64,
    ) -> Result<Vec<MediaEpisodeRecord>, MediaError> {
        let mut statement = connection.prepare(
            "
            SELECT episode.id, episode.external_id, episode.season_number,
                   episode.episode_number, episode.name, episode.overview,
                   episode.air_date, episode.runtime_minutes, episode.still_path,
                   episode.is_present_in_source, COALESCE(progress.is_watched, 0),
                   COALESCE(progress.watched_at, ''), COALESCE(progress.notes, '')
            FROM obj_media_episode episode
            LEFT JOIN obj_media_episode_progress progress ON progress.episode_id = episode.id
            WHERE episode.season_id = ?1
            ORDER BY episode.episode_number
            ",
        )?;
        let rows = statement
            .query_map([season_id], |row| {
                Ok(MediaEpisodeRecord {
                    id: row.get(0)?,
                    external_id: row.get(1)?,
                    season_number: row.get(2)?,
                    episode_number: row.get(3)?,
                    name: row.get(4)?,
                    overview: row.get(5)?,
                    air_date: row.get(6)?,
                    runtime_minutes: row.get(7)?,
                    still_path: row.get(8)?,
                    is_present_in_source: row.get::<_, i64>(9)? != 0,
                    is_watched: row.get::<_, i64>(10)? != 0,
                    watched_at: row.get(11)?,
                    notes: row.get(12)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn update_entry(
        &self,
        entry_id: i64,
        input: &UpdateMediaEntryInput,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let connection = self.database.connection()?;
        let changed = connection.execute(
            "
            UPDATE obj_media_library_entry
            SET library_status = ?2,
                is_favorite = ?3,
                personal_rating = ?4,
                notes = ?5,
                priority = ?6,
                started_at = CASE
                    WHEN ?2 = 'WATCHING' AND started_at = ''
                        THEN datetime('now', 'localtime')
                    ELSE started_at
                END,
                completed_at = CASE
                    WHEN ?2 = 'COMPLETED' AND completed_at = ''
                        THEN datetime('now', 'localtime')
                    ELSE completed_at
                END,
                new_episodes_count = CASE WHEN ?2 = 'COMPLETED' THEN 0 ELSE new_episodes_count END,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?1
            ",
            params![
                entry_id,
                input.library_status.to_string(),
                input.is_favorite as i64,
                input.personal_rating,
                input.notes.trim(),
                input.priority,
            ],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Media library entry was not found."));
        }
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn delete_entry(&self, entry_id: i64) -> Result<(), MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let title_id = transaction
            .query_row(
                "SELECT media_title_id FROM obj_media_library_entry WHERE id = ?1",
                [entry_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Media library entry was not found."))?;
        transaction.execute("DELETE FROM obj_media_title WHERE id = ?1", [title_id])?;
        Self::normalize_queue(&transaction)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn set_movie_watched(
        &self,
        entry_id: i64,
        is_watched: bool,
        watched_at: &str,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let (content_type, current_status): (String, String) = transaction
            .query_row(
                "
                SELECT title.content_type, entry.library_status
                FROM obj_media_library_entry entry
                JOIN obj_media_title title ON title.id = entry.media_title_id
                WHERE entry.id = ?1
                ",
                [entry_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Media library entry was not found."))?;
        if content_type != "MOVIE" {
            return Err(MediaError::validation(
                "Movie watched state can only be set on a movie.",
            ));
        }
        let current_status = MediaLibraryStatus::from_str(&current_status)?;
        if is_watched {
            let next_status = if current_status.protects_from_automatic_transition() {
                current_status
            } else {
                MediaLibraryStatus::Completed
            };
            transaction.execute(
                "
                UPDATE obj_media_library_entry
                SET library_status = ?2,
                    started_at = CASE
                        WHEN started_at = '' THEN COALESCE(NULLIF(?3, ''), datetime('now', 'localtime'))
                        ELSE started_at
                    END,
                    completed_at = COALESCE(NULLIF(?3, ''), datetime('now', 'localtime')),
                    last_watched_at = COALESCE(NULLIF(?3, ''), datetime('now', 'localtime')),
                    updated_at = datetime('now', 'localtime')
                WHERE id = ?1
                ",
                params![entry_id, next_status.to_string(), watched_at.trim()],
            )?;
        } else {
            let next_status = if current_status == MediaLibraryStatus::Completed {
                MediaLibraryStatus::Planned
            } else {
                current_status
            };
            transaction.execute(
                "
                UPDATE obj_media_library_entry
                SET library_status = ?2,
                    completed_at = '',
                    last_watched_at = '',
                    updated_at = datetime('now', 'localtime')
                WHERE id = ?1
                ",
                params![entry_id, next_status.to_string()],
            )?;
        }
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn set_episode_watched(
        &self,
        entry_id: i64,
        episode_id: i64,
        is_watched: bool,
        watched_at: &str,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        Self::assert_episode_ownership(&transaction, entry_id, episode_id)?;
        Self::write_episode_progress(&transaction, episode_id, is_watched, watched_at)?;
        Self::recalculate_series_status(&transaction, entry_id, true)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn set_season_watched(
        &self,
        entry_id: i64,
        season_number: i64,
        is_watched: bool,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let episode_ids = {
            let mut statement = transaction.prepare(
                "
                SELECT episode.id
                FROM obj_media_episode episode
                JOIN obj_media_library_entry entry ON entry.media_title_id = episode.media_title_id
                WHERE entry.id = ?1
                  AND episode.season_number = ?2
                  AND episode.is_present_in_source = 1
                ORDER BY episode.episode_number
                ",
            )?;
            let rows = statement
                .query_map(params![entry_id, season_number], |row| row.get::<_, i64>(0))?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        if episode_ids.is_empty() {
            return Err(MediaError::not_found(
                "No episodes were found for that season.",
            ));
        }
        for episode_id in episode_ids {
            Self::write_episode_progress(&transaction, episode_id, is_watched, "")?;
        }
        Self::recalculate_series_status(&transaction, entry_id, true)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn set_series_watched(
        &self,
        entry_id: i64,
        is_watched: bool,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let episode_ids = {
            let mut statement = transaction.prepare(
                "
                SELECT episode.id
                FROM obj_media_episode episode
                JOIN obj_media_library_entry entry ON entry.media_title_id = episode.media_title_id
                WHERE entry.id = ?1 AND episode.is_present_in_source = 1
                ORDER BY episode.season_number, episode.episode_number
                ",
            )?;
            let rows = statement
                .query_map([entry_id], |row| row.get::<_, i64>(0))?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        if episode_ids.is_empty() {
            return Err(MediaError::validation(
                "This series has no imported episodes to update.",
            ));
        }
        for episode_id in episode_ids {
            Self::write_episode_progress(&transaction, episode_id, is_watched, "")?;
        }
        Self::recalculate_series_status(&transaction, entry_id, true)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn mark_watched_through(
        &self,
        entry_id: i64,
        episode_id: i64,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let (target_season, target_episode): (i64, i64) =
            Self::assert_episode_ownership(&transaction, entry_id, episode_id)?;
        let include_specials = Self::include_specials(&transaction)?;
        let episode_ids = {
            let mut statement = transaction.prepare(
                "
                SELECT episode.id
                FROM obj_media_episode episode
                JOIN obj_media_library_entry entry ON entry.media_title_id = episode.media_title_id
                WHERE entry.id = ?1
                  AND episode.is_present_in_source = 1
                  AND (?4 = 1 OR episode.season_number <> 0)
                  AND (
                    episode.season_number < ?2
                    OR (
                        episode.season_number = ?2
                        AND episode.episode_number <= ?3
                    )
                  )
                ORDER BY episode.season_number, episode.episode_number
                ",
            )?;
            let rows = statement
                .query_map(
                    params![
                        entry_id,
                        target_season,
                        target_episode,
                        include_specials as i64
                    ],
                    |row| row.get::<_, i64>(0),
                )?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        for id in episode_ids {
            Self::write_episode_progress(&transaction, id, true, "")?;
        }
        Self::recalculate_series_status(&transaction, entry_id, true)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    fn assert_episode_ownership(
        transaction: &Transaction<'_>,
        entry_id: i64,
        episode_id: i64,
    ) -> Result<(i64, i64), MediaError> {
        transaction
            .query_row(
                "
                SELECT episode.season_number, episode.episode_number
                FROM obj_media_episode episode
                JOIN obj_media_library_entry entry ON entry.media_title_id = episode.media_title_id
                WHERE entry.id = ?1 AND episode.id = ?2
                ",
                params![entry_id, episode_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| {
                MediaError::not_found("The episode does not belong to this library entry.")
            })
    }

    fn write_episode_progress(
        transaction: &Transaction<'_>,
        episode_id: i64,
        is_watched: bool,
        watched_at: &str,
    ) -> Result<(), MediaError> {
        transaction.execute(
            "
            INSERT INTO obj_media_episode_progress (
                episode_id, is_watched, watched_at
            )
            VALUES (
                ?1, ?2,
                CASE
                    WHEN ?2 = 1 THEN COALESCE(NULLIF(?3, ''), datetime('now', 'localtime'))
                    ELSE ''
                END
            )
            ON CONFLICT (episode_id) DO UPDATE SET
                is_watched = excluded.is_watched,
                watched_at = excluded.watched_at,
                updated_at = datetime('now', 'localtime')
            ",
            params![episode_id, is_watched as i64, watched_at.trim()],
        )?;
        Ok(())
    }

    fn recalculate_series_status(
        transaction: &Transaction<'_>,
        entry_id: i64,
        clear_new_when_complete: bool,
    ) -> Result<(), MediaError> {
        let (title_id, current_status): (i64, String) = transaction
            .query_row(
                "
                SELECT entry.media_title_id, entry.library_status
                FROM obj_media_library_entry entry
                JOIN obj_media_title title ON title.id = entry.media_title_id
                WHERE entry.id = ?1 AND title.content_type = 'SERIES'
                ",
                [entry_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Series library entry was not found."))?;
        let include_specials = Self::include_specials(transaction)?;
        let total = Self::included_episode_count(transaction, title_id, include_specials)?;
        let watched = Self::included_watched_count(transaction, title_id, include_specials)?;
        let current = MediaLibraryStatus::from_str(&current_status)?;
        let next = automatic_series_status(current, ProgressFacts { watched, total });
        let last_watched: String = transaction.query_row(
            "
            SELECT COALESCE(MAX(progress.watched_at), '')
            FROM obj_media_episode episode
            JOIN obj_media_episode_progress progress ON progress.episode_id = episode.id
            WHERE episode.media_title_id = ?1 AND progress.is_watched = 1
            ",
            [title_id],
            |row| row.get(0),
        )?;
        transaction.execute(
            "
            UPDATE obj_media_library_entry
            SET library_status = ?2,
                started_at = CASE
                    WHEN ?2 IN ('WATCHING', 'COMPLETED') AND started_at = ''
                        THEN COALESCE(NULLIF(?3, ''), datetime('now', 'localtime'))
                    ELSE started_at
                END,
                completed_at = CASE
                    WHEN ?2 = 'COMPLETED' AND completed_at = ''
                        THEN datetime('now', 'localtime')
                    WHEN ?2 <> 'COMPLETED' THEN ''
                    ELSE completed_at
                END,
                last_watched_at = ?3,
                new_episodes_count = CASE
                    WHEN ?4 = 1 AND ?2 = 'COMPLETED' THEN 0
                    ELSE new_episodes_count
                END,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?1
            ",
            params![
                entry_id,
                next.to_string(),
                last_watched,
                clear_new_when_complete as i64
            ],
        )?;
        Ok(())
    }

    fn include_specials(connection: &Connection) -> Result<bool, MediaError> {
        connection
            .query_row(
                "
                SELECT include_specials_in_completion
                FROM obj_media_setting WHERE id = 1
                ",
                [],
                |row| Ok(row.get::<_, i64>(0)? != 0),
            )
            .map_err(MediaError::from)
    }

    fn included_episode_count(
        connection: &Connection,
        title_id: i64,
        include_specials: bool,
    ) -> Result<i64, MediaError> {
        connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM obj_media_episode
                WHERE media_title_id = ?1
                  AND is_present_in_source = 1
                  AND (?2 = 1 OR season_number <> 0)
                ",
                params![title_id, include_specials as i64],
                |row| row.get(0),
            )
            .map_err(MediaError::from)
    }

    fn included_watched_count(
        connection: &Connection,
        title_id: i64,
        include_specials: bool,
    ) -> Result<i64, MediaError> {
        connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM obj_media_episode episode
                JOIN obj_media_episode_progress progress ON progress.episode_id = episode.id
                WHERE episode.media_title_id = ?1
                  AND episode.is_present_in_source = 1
                  AND (?2 = 1 OR episode.season_number <> 0)
                  AND progress.is_watched = 1
                ",
                params![title_id, include_specials as i64],
                |row| row.get(0),
            )
            .map_err(MediaError::from)
    }

    pub fn add_to_watch_next(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM obj_media_library_entry entry
                JOIN obj_media_title title ON title.id = entry.media_title_id
                WHERE entry.id = ?1 AND title.content_type <> 'BOOK'
            )",
            [entry_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if !exists {
            return Err(MediaError::validation(
                "Books use the separate Read Next queue.",
            ));
        }
        let position = Self::next_queue_position(&transaction)?;
        transaction.execute(
            "
            UPDATE obj_media_library_entry
            SET queue_position = COALESCE(queue_position, ?2),
                updated_at = datetime('now', 'localtime')
            WHERE id = ?1
            ",
            params![entry_id, position],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn remove_from_watch_next(&self, entry_id: i64) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "
            UPDATE obj_media_library_entry
            SET queue_position = NULL, updated_at = datetime('now', 'localtime')
            WHERE id = ?1
            ",
            [entry_id],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Media library entry was not found."));
        }
        Self::normalize_queue(&transaction)?;
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    pub fn move_watch_next(
        &self,
        entry_id: i64,
        direction: &str,
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let current_position: i64 = transaction
            .query_row(
                "SELECT queue_position FROM obj_media_library_entry WHERE id = ?1",
                [entry_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| MediaError::validation("This title is not currently in Watch Next."))?;
        let target_position = match direction.trim().to_ascii_uppercase().as_str() {
            "UP" => current_position - 1,
            "DOWN" => current_position + 1,
            _ => {
                return Err(MediaError::validation(
                    "Watch Next direction must be UP or DOWN.",
                ))
            }
        };
        if target_position < 1 {
            transaction.commit()?;
            drop(connection);
            return self.get_detail(entry_id);
        }
        let target_id = transaction
            .query_row(
                "SELECT id FROM obj_media_library_entry WHERE queue_position = ?1",
                [target_position],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if let Some(target_id) = target_id {
            transaction.execute(
                "UPDATE obj_media_library_entry SET queue_position = -1 WHERE id = ?1",
                [entry_id],
            )?;
            transaction.execute(
                "UPDATE obj_media_library_entry SET queue_position = ?2 WHERE id = ?1",
                params![target_id, current_position],
            )?;
            transaction.execute(
                "
                UPDATE obj_media_library_entry
                SET queue_position = ?2, updated_at = datetime('now', 'localtime')
                WHERE id = ?1
                ",
                params![entry_id, target_position],
            )?;
        }
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    fn next_queue_position(connection: &Connection) -> Result<i64, MediaError> {
        connection
            .query_row(
                "SELECT COALESCE(MAX(queue_position), 0) + 1 FROM obj_media_library_entry",
                [],
                |row| row.get(0),
            )
            .map_err(MediaError::from)
    }

    fn normalize_queue(transaction: &Transaction<'_>) -> Result<(), MediaError> {
        let entry_ids = {
            let mut statement = transaction.prepare(
                "
                SELECT id FROM obj_media_library_entry
                WHERE queue_position IS NOT NULL
                ORDER BY queue_position, id
                ",
            )?;
            let rows = statement
                .query_map([], |row| row.get::<_, i64>(0))?
                .collect::<SqlResult<Vec<_>>>()?;
            rows
        };
        transaction.execute(
            "
            UPDATE obj_media_library_entry
            SET queue_position = -id
            WHERE queue_position IS NOT NULL
            ",
            [],
        )?;
        for (index, entry_id) in entry_ids.into_iter().enumerate() {
            transaction.execute(
                "UPDATE obj_media_library_entry SET queue_position = ?2 WHERE id = ?1",
                params![entry_id, index as i64 + 1],
            )?;
        }
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<MediaTagRecord>, MediaError> {
        let connection = self.database.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, name, created_at, updated_at
            FROM obj_media_tag
            ORDER BY name COLLATE NOCASE
            ",
        )?;
        let rows = statement
            .query_map([], tag_from_row)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn create_tag(&self, name: &str) -> Result<MediaTagRecord, MediaError> {
        let connection = self.database.connection()?;
        let name = name.trim();
        connection.execute(
            "INSERT OR IGNORE INTO obj_media_tag (name) VALUES (?1)",
            [name],
        )?;
        connection
            .query_row(
                "
                SELECT id, name, created_at, updated_at
                FROM obj_media_tag WHERE name = ?1 COLLATE NOCASE
                ",
                [name],
                tag_from_row,
            )
            .map_err(MediaError::from)
    }

    pub fn delete_tag(&self, tag_id: i64) -> Result<(), MediaError> {
        let connection = self.database.connection()?;
        let changed = connection.execute("DELETE FROM obj_media_tag WHERE id = ?1", [tag_id])?;
        if changed == 0 {
            return Err(MediaError::not_found("Media tag was not found."));
        }
        Ok(())
    }

    pub fn set_entry_tags(
        &self,
        entry_id: i64,
        tag_ids: &[i64],
    ) -> Result<MediaLibraryDetail, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM obj_media_library_entry WHERE id = ?1)",
            [entry_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if !exists {
            return Err(MediaError::not_found("Media library entry was not found."));
        }
        transaction.execute(
            "DELETE FROM n2n_media_library_entry_tag WHERE media_library_entry_id = ?1",
            [entry_id],
        )?;
        for tag_id in tag_ids {
            let inserted = transaction.execute(
                "
                INSERT OR IGNORE INTO n2n_media_library_entry_tag (
                    media_library_entry_id, media_tag_id
                )
                SELECT ?1, id FROM obj_media_tag WHERE id = ?2
                ",
                params![entry_id, tag_id],
            )?;
            if inserted == 0 {
                return Err(MediaError::not_found(format!(
                    "Media tag {tag_id} was not found."
                )));
            }
        }
        transaction.commit()?;
        drop(connection);
        self.get_detail(entry_id)
    }

    fn list_entry_tags_from_connection(
        connection: &Connection,
        entry_id: i64,
    ) -> Result<Vec<MediaTagRecord>, MediaError> {
        let mut statement = connection.prepare(
            "
            SELECT tag.id, tag.name, tag.created_at, tag.updated_at
            FROM obj_media_tag tag
            JOIN n2n_media_library_entry_tag mapping ON mapping.media_tag_id = tag.id
            WHERE mapping.media_library_entry_id = ?1
            ORDER BY tag.name COLLATE NOCASE
            ",
        )?;
        let rows = statement
            .query_map([entry_id], tag_from_row)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn list_streaming_links(
        &self,
        entry_id: i64,
    ) -> Result<Vec<MediaStreamingLinkRecord>, MediaError> {
        let connection = self.database.connection()?;
        Self::list_streaming_links_from_connection(&connection, entry_id)
    }

    fn list_streaming_links_from_connection(
        connection: &Connection,
        entry_id: i64,
    ) -> Result<Vec<MediaStreamingLinkRecord>, MediaError> {
        let mut statement = connection.prepare(
            "
            SELECT id, media_library_entry_id, provider_name, url, link_type,
                   is_preferred, notes, created_at, updated_at
            FROM obj_media_streaming_link
            WHERE media_library_entry_id = ?1
            ORDER BY is_preferred DESC, provider_name COLLATE NOCASE, id
            ",
        )?;
        let rows = statement
            .query_map([entry_id], streaming_link_from_row)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn create_streaming_link(
        &self,
        entry_id: i64,
        input: &MediaStreamingLinkInput,
    ) -> Result<MediaStreamingLinkRecord, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        if input.is_preferred {
            transaction.execute(
                "
                UPDATE obj_media_streaming_link SET is_preferred = 0
                WHERE media_library_entry_id = ?1
                ",
                [entry_id],
            )?;
        }
        let inserted = transaction.execute(
            "
            INSERT INTO obj_media_streaming_link (
                media_library_entry_id, provider_name, url, link_type, is_preferred, notes
            )
            SELECT ?1, ?2, ?3, ?4, ?5, ?6
            WHERE EXISTS (SELECT 1 FROM obj_media_library_entry WHERE id = ?1)
            ",
            params![
                entry_id,
                input.provider_name.trim(),
                input.url.trim(),
                input.link_type.trim().to_ascii_uppercase(),
                input.is_preferred as i64,
                input.notes.trim(),
            ],
        )?;
        if inserted == 0 {
            return Err(MediaError::not_found("Media library entry was not found."));
        }
        let id = transaction.last_insert_rowid();
        let record = transaction.query_row(
            "
            SELECT id, media_library_entry_id, provider_name, url, link_type,
                   is_preferred, notes, created_at, updated_at
            FROM obj_media_streaming_link WHERE id = ?1
            ",
            [id],
            streaming_link_from_row,
        )?;
        transaction.commit()?;
        Ok(record)
    }

    pub fn update_streaming_link(
        &self,
        link_id: i64,
        input: &MediaStreamingLinkInput,
    ) -> Result<MediaStreamingLinkRecord, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let entry_id: i64 = transaction
            .query_row(
                "
                SELECT media_library_entry_id FROM obj_media_streaming_link WHERE id = ?1
                ",
                [link_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| MediaError::not_found("Manual streaming link was not found."))?;
        if input.is_preferred {
            transaction.execute(
                "
                UPDATE obj_media_streaming_link SET is_preferred = 0
                WHERE media_library_entry_id = ?1
                ",
                [entry_id],
            )?;
        }
        transaction.execute(
            "
            UPDATE obj_media_streaming_link
            SET provider_name = ?2,
                url = ?3,
                link_type = ?4,
                is_preferred = ?5,
                notes = ?6,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?1
            ",
            params![
                link_id,
                input.provider_name.trim(),
                input.url.trim(),
                input.link_type.trim().to_ascii_uppercase(),
                input.is_preferred as i64,
                input.notes.trim(),
            ],
        )?;
        let record = transaction.query_row(
            "
            SELECT id, media_library_entry_id, provider_name, url, link_type,
                   is_preferred, notes, created_at, updated_at
            FROM obj_media_streaming_link WHERE id = ?1
            ",
            [link_id],
            streaming_link_from_row,
        )?;
        transaction.commit()?;
        Ok(record)
    }

    pub fn delete_streaming_link(&self, link_id: i64) -> Result<(), MediaError> {
        let connection = self.database.connection()?;
        let changed = connection.execute(
            "DELETE FROM obj_media_streaming_link WHERE id = ?1",
            [link_id],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found(
                "Manual streaming link was not found.",
            ));
        }
        Ok(())
    }

    pub fn set_preferred_streaming_link(
        &self,
        entry_id: i64,
        link_id: i64,
    ) -> Result<Vec<MediaStreamingLinkRecord>, MediaError> {
        let mut connection = self.database.connection()?;
        let transaction = connection.transaction()?;
        let belongs: bool = transaction.query_row(
            "
            SELECT EXISTS(
                SELECT 1 FROM obj_media_streaming_link
                WHERE id = ?1 AND media_library_entry_id = ?2
            )
            ",
            params![link_id, entry_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )?;
        if !belongs {
            return Err(MediaError::not_found(
                "Manual streaming link does not belong to this entry.",
            ));
        }
        transaction.execute(
            "
            UPDATE obj_media_streaming_link
            SET is_preferred = CASE WHEN id = ?2 THEN 1 ELSE 0 END,
                updated_at = datetime('now', 'localtime')
            WHERE media_library_entry_id = ?1
            ",
            params![entry_id, link_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.list_streaming_links(entry_id)
    }

    pub fn resolve_streaming_target(&self, entry_id: i64) -> Result<String, MediaError> {
        let connection = self.database.connection()?;
        if let Some(url) = connection
            .query_row(
                "
                SELECT url FROM obj_media_streaming_link
                WHERE media_library_entry_id = ?1 AND is_preferred = 1
                LIMIT 1
                ",
                [entry_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(url);
        }
        connection
            .query_row(
                "
                SELECT snapshot.source_link_url
                FROM obj_media_library_entry entry
                JOIN obj_media_provider_snapshot snapshot
                  ON snapshot.media_title_id = entry.media_title_id
                JOIN obj_media_setting setting ON setting.id = 1
                WHERE entry.id = ?1
                  AND snapshot.region_code = setting.region_code
                  AND snapshot.source_link_url <> ''
                LIMIT 1
                ",
                [entry_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| {
                MediaError::not_found(
                    "No preferred manual link or TMDB availability page is saved for this title.",
                )
            })
    }

    pub fn mark_refresh_failure(&self, entry_id: i64, error: &str) -> Result<(), MediaError> {
        let connection = self.database.connection()?;
        let changed = connection.execute(
            "
            INSERT INTO obj_media_provider_snapshot (
                media_title_id, region_code, source_key, last_refresh_status, last_refresh_error
            )
            SELECT entry.media_title_id, setting.region_code, 'tmdb', 'FAILED', ?2
            FROM obj_media_library_entry entry
            JOIN obj_media_setting setting ON setting.id = 1
            WHERE entry.id = ?1
            ON CONFLICT (media_title_id, region_code) DO UPDATE SET
                last_refresh_status = 'FAILED',
                last_refresh_error = excluded.last_refresh_error,
                updated_at = datetime('now', 'localtime')
            ",
            params![entry_id, error],
        )?;
        if changed == 0 {
            return Err(MediaError::not_found("Media library entry was not found."));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RawMediaSummary {
    id: i64,
    media_title_id: i64,
    source_key: String,
    external_id: Option<i64>,
    external_media_type: Option<String>,
    content_type: String,
    title: String,
    original_title: String,
    overview: String,
    original_language: String,
    release_date: String,
    first_air_date: String,
    last_air_date: String,
    runtime_minutes: Option<i64>,
    episode_runtime_minutes: Option<i64>,
    external_status: String,
    series_type: String,
    poster_path: String,
    backdrop_path: String,
    homepage_url: String,
    total_seasons: Option<i64>,
    total_episodes: Option<i64>,
    metadata_refreshed_at: String,
    library_status: String,
    is_favorite: bool,
    personal_rating: Option<f64>,
    notes: String,
    priority: i64,
    queue_position: Option<i64>,
    added_at: String,
    started_at: String,
    completed_at: String,
    last_watched_at: String,
    new_episodes_count: i64,
    watched_count: i64,
    progress_total: i64,
    progress_percent: f64,
}

fn summary_row_from_row(row: &rusqlite::Row<'_>) -> SqlResult<RawMediaSummary> {
    Ok(RawMediaSummary {
        id: row.get(0)?,
        media_title_id: row.get(1)?,
        source_key: row.get(2)?,
        external_id: row.get(3)?,
        external_media_type: row.get(4)?,
        content_type: row.get(5)?,
        title: row.get(6)?,
        original_title: row.get(7)?,
        overview: row.get(8)?,
        original_language: row.get(9)?,
        release_date: row.get(10)?,
        first_air_date: row.get(11)?,
        last_air_date: row.get(12)?,
        runtime_minutes: row.get(13)?,
        episode_runtime_minutes: row.get(14)?,
        external_status: row.get(15)?,
        series_type: row.get(16)?,
        poster_path: row.get(17)?,
        backdrop_path: row.get(18)?,
        homepage_url: row.get(19)?,
        total_seasons: row.get(20)?,
        total_episodes: row.get(21)?,
        metadata_refreshed_at: row.get(22)?,
        library_status: row.get(23)?,
        is_favorite: row.get::<_, i64>(24)? != 0,
        personal_rating: row.get(25)?,
        notes: row.get(26)?,
        priority: row.get(27)?,
        queue_position: row.get(28)?,
        added_at: row.get(29)?,
        started_at: row.get(30)?,
        completed_at: row.get(31)?,
        last_watched_at: row.get(32)?,
        new_episodes_count: row.get(33)?,
        watched_count: row.get(34)?,
        progress_total: row.get(35)?,
        progress_percent: row.get(36)?,
    })
}

fn tag_from_row(row: &rusqlite::Row<'_>) -> SqlResult<MediaTagRecord> {
    Ok(MediaTagRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

fn streaming_link_from_row(row: &rusqlite::Row<'_>) -> SqlResult<MediaStreamingLinkRecord> {
    Ok(MediaStreamingLinkRecord {
        id: row.get(0)?,
        media_library_entry_id: row.get(1)?,
        provider_name: row.get(2)?,
        url: row.get(3)?,
        link_type: row.get(4)?,
        is_preferred: row.get::<_, i64>(5)? != 0,
        notes: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn provider_snapshot_from_row(row: &rusqlite::Row<'_>) -> SqlResult<MediaProviderSnapshotRecord> {
    Ok(MediaProviderSnapshotRecord {
        id: row.get(0)?,
        region_code: row.get(1)?,
        source_link_url: row.get(2)?,
        refreshed_at: row.get(3)?,
        last_refresh_status: row.get(4)?,
        last_refresh_error: row.get(5)?,
    })
}

fn provider_availability_from_row(
    row: &rusqlite::Row<'_>,
) -> SqlResult<MediaProviderAvailabilityRecord> {
    Ok(MediaProviderAvailabilityRecord {
        id: row.get(0)?,
        provider_external_id: row.get(1)?,
        provider_name: row.get(2)?,
        provider_logo_path: row.get(3)?,
        monetization_type: row.get(4)?,
        display_priority: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::domain::{
        MediaStreamingLinkInput, NormalizedMediaEpisode, NormalizedMediaSeason,
        NormalizedMediaTitle, NormalizedProviderAvailability, NormalizedProviderSnapshot,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "overlay-forge-media-{test_name}-{}-{unique}.sqlite3",
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
        test: impl FnOnce(&AppDatabase, &MediaRepository<'_>) -> T,
    ) -> T {
        let path = temp_db_path(name);
        remove_db_files(&path);
        let database = AppDatabase::new(path.clone()).expect("test database should initialize");
        let result = {
            let repository = MediaRepository::new(&database);
            test(&database, &repository)
        };
        drop(database);
        remove_db_files(&path);
        result
    }

    fn episode(external_id: i64, season: i64, number: i64) -> NormalizedMediaEpisode {
        NormalizedMediaEpisode {
            external_id: Some(external_id),
            season_number: season,
            episode_number: number,
            name: format!("S{season}E{number}"),
            overview: String::new(),
            air_date: format!("2026-01-{:02}", number.clamp(1, 28)),
            runtime_minutes: Some(45),
            still_path: String::new(),
            metadata_json: "{}".to_string(),
        }
    }

    fn series_import(include_third_regular_episode: bool) -> NormalizedMediaImport {
        let mut regular_episodes = vec![episode(101, 1, 1), episode(102, 1, 2)];
        if include_third_regular_episode {
            regular_episodes.push(episode(103, 1, 3));
        }
        NormalizedMediaImport {
            title: NormalizedMediaTitle {
                source_key: "tmdb".to_string(),
                external_id: Some(42),
                external_media_type: Some("tv".to_string()),
                content_type: MediaContentType::Series,
                title: "Test Series".to_string(),
                original_title: "Test Series".to_string(),
                overview: "A test series.".to_string(),
                original_language: "en".to_string(),
                release_date: String::new(),
                first_air_date: "2026-01-01".to_string(),
                last_air_date: "2026-01-03".to_string(),
                runtime_minutes: None,
                episode_runtime_minutes: Some(45),
                external_status: "Returning Series".to_string(),
                series_type: "Scripted".to_string(),
                poster_path: "/poster.jpg".to_string(),
                backdrop_path: String::new(),
                homepage_url: String::new(),
                total_seasons: Some(1),
                total_episodes: Some(if include_third_regular_episode { 4 } else { 3 }),
                metadata_json: "{}".to_string(),
            },
            seasons: vec![
                NormalizedMediaSeason {
                    external_id: Some(400),
                    season_number: 0,
                    name: "Specials".to_string(),
                    overview: String::new(),
                    air_date: String::new(),
                    poster_path: String::new(),
                    episode_count: 1,
                    metadata_json: "{}".to_string(),
                    episodes: vec![episode(100, 0, 1)],
                },
                NormalizedMediaSeason {
                    external_id: Some(401),
                    season_number: 1,
                    name: "Season 1".to_string(),
                    overview: String::new(),
                    air_date: "2026-01-01".to_string(),
                    poster_path: String::new(),
                    episode_count: regular_episodes.len() as i64,
                    metadata_json: "{}".to_string(),
                    episodes: regular_episodes,
                },
            ],
            provider_snapshot: Some(NormalizedProviderSnapshot {
                region_code: "CA".to_string(),
                source_link_url: "https://www.themoviedb.org/tv/42/watch".to_string(),
                providers: vec![NormalizedProviderAvailability {
                    provider_external_id: 8,
                    provider_name: "Example Stream".to_string(),
                    provider_logo_path: "/provider.jpg".to_string(),
                    monetization_type: "FLATRATE".to_string(),
                    display_priority: 1,
                }],
            }),
            provider_error: None,
        }
    }

    fn manual_input(title: &str) -> CreateManualMediaInput {
        CreateManualMediaInput {
            content_type: MediaContentType::Movie,
            title: title.to_string(),
            release_date: String::new(),
            overview: String::new(),
            runtime_minutes: Some(90),
            library_status: MediaLibraryStatus::Planned,
            notes: String::new(),
            personal_rating: None,
            is_favorite: false,
            priority: 0,
            add_to_watch_next: false,
        }
    }

    #[test]
    fn schema_and_external_identity_prevent_duplicate_catalog_entries() {
        with_database("schema-duplicates", |database, repository| {
            let connection = database.connection().expect("database should lock");
            for table in [
                "obj_media_title",
                "obj_media_library_entry",
                "obj_media_season",
                "obj_media_episode",
                "obj_media_episode_progress",
                "obj_media_provider_snapshot",
                "obj_media_provider_availability",
                "obj_media_streaming_link",
                "obj_media_tag",
                "n2n_media_library_entry_tag",
                "obj_media_setting",
            ] {
                let exists: i64 = connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                        [table],
                        |row| row.get(0),
                    )
                    .expect("schema query should work");
                assert_eq!(exists, 1, "{table} should exist");
            }
            drop(connection);

            repository
                .add_catalog_import(&series_import(false), MediaLibraryStatus::Planned)
                .expect("first import should work");
            let duplicate = repository
                .add_catalog_import(&series_import(false), MediaLibraryStatus::Planned)
                .expect_err("duplicate should be rejected");
            assert_eq!(
                duplicate.kind,
                super::super::domain::MediaErrorKind::Duplicate
            );
        });
    }

    #[test]
    fn progress_transitions_and_specials_setting_are_consistent() {
        with_database("progress", |_database, repository| {
            let detail = repository
                .add_catalog_import(&series_import(false), MediaLibraryStatus::Planned)
                .expect("series should import");
            let regular = &detail.seasons[1].episodes;
            let special_id = detail.seasons[0].episodes[0].id;

            let detail = repository
                .set_episode_watched(detail.entry.id, regular[0].id, true, "")
                .expect("first episode should update");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Watching);

            let detail = repository
                .set_episode_watched(detail.entry.id, regular[1].id, true, "")
                .expect("second episode should update");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Completed);
            assert_eq!(detail.entry.progress_total, 2);

            let detail = repository
                .set_episode_watched(detail.entry.id, regular[1].id, false, "")
                .expect("episode should become unwatched");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Watching);

            let held = repository
                .update_entry(
                    detail.entry.id,
                    &UpdateMediaEntryInput {
                        library_status: MediaLibraryStatus::OnHold,
                        is_favorite: false,
                        personal_rating: None,
                        notes: String::new(),
                        priority: 0,
                    },
                )
                .expect("status should update");
            let held = repository
                .set_series_watched(held.entry.id, true)
                .expect("bulk watch should work");
            assert_eq!(held.entry.library_status, MediaLibraryStatus::OnHold);

            repository
                .update_entry(
                    held.entry.id,
                    &UpdateMediaEntryInput {
                        library_status: MediaLibraryStatus::Watching,
                        is_favorite: false,
                        personal_rating: None,
                        notes: String::new(),
                        priority: 0,
                    },
                )
                .expect("explicit status should win");
            let completed = repository
                .set_episode_watched(held.entry.id, special_id, false, "")
                .expect("special should become unwatched");
            assert_eq!(
                completed.entry.library_status,
                MediaLibraryStatus::Completed,
                "specials are excluded by default"
            );

            repository
                .update_settings(&UpdateMediaSettingsInput {
                    region_code: "CA".to_string(),
                    metadata_language: "en-CA".to_string(),
                    include_specials_in_completion: true,
                })
                .expect("settings should update");
            let detail = repository
                .get_detail(held.entry.id)
                .expect("detail should reload");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Watching);
            assert_eq!(detail.entry.progress_total, 3);

            let detail = repository
                .mark_watched_through(detail.entry.id, regular[1].id)
                .expect("mark through should update preceding episodes");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Completed);
            assert!(detail.seasons[0].episodes[0].is_watched);
        });
    }

    #[test]
    fn refresh_preserves_user_owned_state_and_old_provider_rows() {
        with_database("refresh-preserves", |_database, repository| {
            let mut detail = repository
                .add_catalog_import(&series_import(false), MediaLibraryStatus::Watching)
                .expect("series should import");
            detail = repository
                .update_entry(
                    detail.entry.id,
                    &UpdateMediaEntryInput {
                        library_status: MediaLibraryStatus::Watching,
                        is_favorite: true,
                        personal_rating: Some(9.0),
                        notes: "Keep this note".to_string(),
                        priority: 7,
                    },
                )
                .expect("entry should update");
            detail = repository
                .add_to_watch_next(detail.entry.id)
                .expect("queue should update");
            let tag = repository.create_tag("Comfort").expect("tag should create");
            detail = repository
                .set_entry_tags(detail.entry.id, &[tag.id])
                .expect("tag should attach");
            repository
                .create_streaming_link(
                    detail.entry.id,
                    &MediaStreamingLinkInput {
                        provider_name: "My Service".to_string(),
                        url: "https://example.com/watch".to_string(),
                        link_type: "STREAM".to_string(),
                        is_preferred: true,
                        notes: "Keep this link".to_string(),
                    },
                )
                .expect("manual link should create");
            detail = repository
                .set_season_watched(detail.entry.id, 1, true)
                .expect("regular season should complete");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Completed);

            detail = repository
                .refresh_import(detail.entry.id, &series_import(true))
                .expect("refresh should work");
            assert_eq!(detail.entry.library_status, MediaLibraryStatus::Watching);
            assert_eq!(detail.entry.new_episodes_count, 1);
            assert_eq!(detail.entry.notes, "Keep this note");
            assert_eq!(detail.entry.personal_rating, Some(9.0));
            assert!(detail.entry.is_favorite);
            assert_eq!(detail.entry.priority, 7);
            assert_eq!(detail.entry.queue_position, Some(1));
            assert_eq!(detail.tags[0].name, "Comfort");
            assert_eq!(detail.streaming_links[0].url, "https://example.com/watch");
            assert!(detail.seasons[1].episodes[0].is_watched);
            assert!(detail.seasons[1].episodes[1].is_watched);
            assert!(!detail.seasons[1].episodes[2].is_watched);

            let mut failed_availability_import = series_import(true);
            failed_availability_import.provider_snapshot = None;
            failed_availability_import.provider_error =
                Some("Network: temporary provider error".to_string());
            detail = repository
                .refresh_import(detail.entry.id, &failed_availability_import)
                .expect("metadata refresh should retain old provider rows");
            assert_eq!(detail.providers.len(), 1);
            assert_eq!(
                detail
                    .provider_snapshot
                    .as_ref()
                    .expect("snapshot should remain")
                    .last_refresh_status,
                "FAILED"
            );
        });
    }

    #[test]
    fn queue_tags_and_preferred_links_normalize_without_duplicates() {
        with_database("queue-tags-links", |_database, repository| {
            let first = repository
                .create_manual_entry(&manual_input("First"))
                .expect("first should create");
            let second = repository
                .create_manual_entry(&manual_input("Second"))
                .expect("second should create");
            let third = repository
                .create_manual_entry(&manual_input("Third"))
                .expect("third should create");
            repository
                .add_to_watch_next(first.entry.id)
                .expect("first should queue");
            repository
                .add_to_watch_next(second.entry.id)
                .expect("second should queue");
            repository
                .add_to_watch_next(third.entry.id)
                .expect("third should queue");
            repository
                .remove_from_watch_next(second.entry.id)
                .expect("middle should leave queue");
            let third = repository
                .get_detail(third.entry.id)
                .expect("third should reload");
            assert_eq!(third.entry.queue_position, Some(2));
            let third = repository
                .move_watch_next(third.entry.id, "UP")
                .expect("third should move up");
            assert_eq!(third.entry.queue_position, Some(1));

            let first_tag = repository.create_tag("Anime").expect("tag should create");
            let same_tag = repository
                .create_tag("anime")
                .expect("tag should be reused");
            assert_eq!(first_tag.id, same_tag.id);

            let first_link = repository
                .create_streaming_link(
                    first.entry.id,
                    &MediaStreamingLinkInput {
                        provider_name: "One".to_string(),
                        url: "https://example.com/one".to_string(),
                        link_type: "STREAM".to_string(),
                        is_preferred: true,
                        notes: String::new(),
                    },
                )
                .expect("first link should create");
            let second_link = repository
                .create_streaming_link(
                    first.entry.id,
                    &MediaStreamingLinkInput {
                        provider_name: "Two".to_string(),
                        url: "https://example.com/two".to_string(),
                        link_type: "STREAM".to_string(),
                        is_preferred: true,
                        notes: String::new(),
                    },
                )
                .expect("second link should create");
            let links = repository
                .list_streaming_links(first.entry.id)
                .expect("links should list");
            assert_eq!(links.iter().filter(|link| link.is_preferred).count(), 1);
            assert_eq!(
                links
                    .iter()
                    .find(|link| link.is_preferred)
                    .expect("preferred link should exist")
                    .id,
                second_link.id
            );
            assert_ne!(first_link.id, second_link.id);
        });
    }
}
