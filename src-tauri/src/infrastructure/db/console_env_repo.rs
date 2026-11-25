use std::path::Path;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::environment::{
    ConsoleScanPath, ConsoleSetting, RuntimeVerificationFeature, RuntimeVerificationRecord,
    RuntimeVerificationStatus,
};
use crate::domain::error::DomainError;
use crate::domain::player::{now_utc_rfc3339, GameRecord, GameVersionRecord};
use crate::domain::session::GameSessionRecord;
use crate::infrastructure::system::cover_cache::CoverCache;

use super::manager::DatabaseManager;
use super::session_stats_repo::SessionStatsRepository;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetailsDto {
    pub game: GameRecord,
    pub versions: Vec<GameVersionRecord>,
    pub sessions: Vec<GameSessionRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGameMetadataInput {
    pub game_id: String,
    pub description: String,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_year: Option<i64>,
    pub genre: Option<String>,
    pub completion_percent: i64,
}

#[derive(Clone)]
pub struct ConsoleEnvRepository {
    db: DatabaseManager,
}

impl ConsoleEnvRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn add_scan_path(
        &self,
        console_slug: &str,
        directory_path: &str,
        recursive: bool,
    ) -> Result<ConsoleScanPath, DomainError> {
        let trimmed = directory_path.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "directory_path cannot be empty".to_string(),
            ));
        }

        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();
        let recursive_int = i64::from(recursive);

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let console_id: String = tx
                .query_row(
                    "SELECT id FROM consoles WHERE slug = ?1",
                    params![console_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| DomainError::NotFound(format!("console '{console_slug}'")))?;

            tx.execute(
                "INSERT INTO console_scan_paths (id, console_id, directory_path, recursive, enabled, last_scanned_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, 1, NULL, ?5)
                 ON CONFLICT(console_id, directory_path) DO UPDATE SET
                   recursive = excluded.recursive,
                   enabled = 1",
                params![new_id, console_id, trimmed, recursive_int, now],
            )
            .map_err(|e| DomainError::Database(format!("upsert scan path failed: {e}")))?;

            let record = tx
                .query_row(
                    "SELECT sp.id, sp.console_id, c.slug, sp.directory_path, sp.recursive, sp.enabled, sp.last_scanned_at, sp.created_at
                     FROM console_scan_paths sp
                     JOIN consoles c ON c.id = sp.console_id
                     WHERE sp.console_id = ?1 AND sp.directory_path = ?2",
                    params![console_id, trimmed],
                    |r| {
                        Ok(ConsoleScanPath {
                            id: r.get(0)?,
                            console_id: r.get(1)?,
                            console_slug: r.get(2)?,
                            directory_path: r.get(3)?,
                            recursive: r.get::<_, i64>(4)? != 0,
                            enabled: r.get::<_, i64>(5)? != 0,
                            last_scanned_at: r.get(6)?,
                            created_at: r.get(7)?,
                        })
                    },
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(record)
        })
    }

    pub fn list_scan_paths(&self, console_slug: &str) -> Result<Vec<ConsoleScanPath>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT sp.id, sp.console_id, c.slug, sp.directory_path, sp.recursive, sp.enabled, sp.last_scanned_at, sp.created_at
                     FROM console_scan_paths sp
                     JOIN consoles c ON c.id = sp.console_id
                     WHERE c.slug = ?1
                     ORDER BY sp.created_at ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![console_slug], |r| {
                    Ok(ConsoleScanPath {
                        id: r.get(0)?,
                        console_id: r.get(1)?,
                        console_slug: r.get(2)?,
                        directory_path: r.get(3)?,
                        recursive: r.get::<_, i64>(4)? != 0,
                        enabled: r.get::<_, i64>(5)? != 0,
                        last_scanned_at: r.get(6)?,
                        created_at: r.get(7)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
            }
            Ok(out)
        })
    }

    pub fn remove_scan_path(&self, path_id: &str) -> Result<bool, DomainError> {
        self.db.with_connection_mut(|conn| {
            let affected = conn
                .execute(
                    "DELETE FROM console_scan_paths WHERE id = ?1",
                    params![path_id],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(affected > 0)
        })
    }

    pub fn mark_scan_path_scanned(
        &self,
        path_id: &str,
        scanned_at_utc: &str,
    ) -> Result<(), DomainError> {
        self.db.with_connection_mut(|conn| {
            conn.execute(
                "UPDATE console_scan_paths SET last_scanned_at = ?1 WHERE id = ?2",
                params![scanned_at_utc, path_id],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })
    }

    pub fn list_console_settings(
        &self,
        console_slug: &str,
    ) -> Result<Vec<ConsoleSetting>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT cs.id, cs.console_id, c.slug, cs.key, cs.value, cs.updated_at
                     FROM console_settings cs
                     JOIN consoles c ON c.id = cs.console_id
                     WHERE c.slug = ?1
                     ORDER BY cs.key ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![console_slug], |r| {
                    Ok(ConsoleSetting {
                        id: r.get(0)?,
                        console_id: r.get(1)?,
                        console_slug: r.get(2)?,
                        key: r.get(3)?,
                        value: r.get(4)?,
                        updated_at: r.get(5)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
            }
            Ok(out)
        })
    }

    pub fn upsert_console_setting(
        &self,
        console_slug: &str,
        key: &str,
        value: &str,
    ) -> Result<ConsoleSetting, DomainError> {
        let trimmed_key = key.trim();
        if trimmed_key.is_empty() {
            return Err(DomainError::Validation(
                "console setting key cannot be empty".to_string(),
            ));
        }

        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let console_id: String = tx
                .query_row(
                    "SELECT id FROM consoles WHERE slug = ?1",
                    params![console_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| DomainError::NotFound(format!("console '{console_slug}'")))?;

            tx.execute(
                "INSERT INTO console_settings (id, console_id, key, value, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(console_id, key) DO UPDATE SET
                   value = excluded.value,
                   updated_at = excluded.updated_at",
                params![new_id, console_id, trimmed_key, value, now],
            )
            .map_err(|e| DomainError::Database(format!("upsert console setting failed: {e}")))?;

            let record = tx
                .query_row(
                    "SELECT cs.id, cs.console_id, c.slug, cs.key, cs.value, cs.updated_at
                     FROM console_settings cs
                     JOIN consoles c ON c.id = cs.console_id
                     WHERE cs.console_id = ?1 AND cs.key = ?2",
                    params![console_id, trimmed_key],
                    |r| {
                        Ok(ConsoleSetting {
                            id: r.get(0)?,
                            console_id: r.get(1)?,
                            console_slug: r.get(2)?,
                            key: r.get(3)?,
                            value: r.get(4)?,
                            updated_at: r.get(5)?,
                        })
                    },
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(record)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_runtime_verification(
        &self,
        console_slug: &str,
        emulator_slug: &str,
        game_id: Option<&str>,
        feature_key: RuntimeVerificationFeature,
        emulator_version: &str,
        status: RuntimeVerificationStatus,
        evidence_notes: &str,
    ) -> Result<RuntimeVerificationRecord, DomainError> {
        let clean_version = emulator_version.trim();
        if clean_version.is_empty() {
            return Err(DomainError::Validation(
                "emulator_version cannot be empty".to_string(),
            ));
        }

        let scope_key = RuntimeVerificationRecord::compute_scope_key(game_id)?;
        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let console_id: String = tx
                .query_row(
                    "SELECT id FROM consoles WHERE slug = ?1",
                    params![console_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| DomainError::NotFound(format!("console '{console_slug}'")))?;

            let emulator_id: String = tx
                .query_row(
                    "SELECT id FROM emulators WHERE slug = ?1",
                    params![emulator_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| DomainError::NotFound(format!("emulator '{emulator_slug}'")))?;

            tx.execute(
                "INSERT INTO console_runtime_verifications
                    (id, console_id, emulator_id, game_id, scope_key, feature_key, emulator_version, status, evidence_notes, verified_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(console_id, emulator_id, feature_key, emulator_version, scope_key) DO UPDATE SET
                    status = excluded.status,
                    evidence_notes = excluded.evidence_notes,
                    verified_at = excluded.verified_at",
                params![
                    new_id,
                    console_id,
                    emulator_id,
                    game_id,
                    scope_key,
                    feature_key.as_str(),
                    clean_version,
                    status.as_str(),
                    evidence_notes,
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(format!("upsert runtime verification failed: {e}")))?;

            let row_id: String = tx
                .query_row(
                    "SELECT id FROM console_runtime_verifications
                     WHERE console_id = ?1 AND emulator_id = ?2 AND feature_key = ?3 AND emulator_version = ?4 AND scope_key = ?5",
                    params![console_id, emulator_id, feature_key.as_str(), clean_version, scope_key],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;

            Ok(RuntimeVerificationRecord {
                id: row_id,
                console_id,
                console_slug: console_slug.to_string(),
                emulator_id,
                emulator_slug: emulator_slug.to_string(),
                game_id: game_id.map(str::to_string),
                scope_key,
                feature_key,
                emulator_version: clean_version.to_string(),
                status,
                evidence_notes: evidence_notes.to_string(),
                verified_at: now,
            })
        })
    }

    pub fn list_runtime_verifications(
        &self,
        console_slug: &str,
    ) -> Result<Vec<RuntimeVerificationRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT rv.id, rv.console_id, c.slug, rv.emulator_id, e.slug, rv.game_id,
                            rv.scope_key, rv.feature_key, rv.emulator_version, rv.status,
                            rv.evidence_notes, rv.verified_at
                     FROM console_runtime_verifications rv
                     JOIN consoles c ON c.id = rv.console_id
                     JOIN emulators e ON e.id = rv.emulator_id
                     WHERE c.slug = ?1
                     ORDER BY rv.verified_at DESC, rv.feature_key ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![console_slug], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, Option<String>>(5)?,
                        r.get::<_, String>(6)?,
                        r.get::<_, String>(7)?,
                        r.get::<_, String>(8)?,
                        r.get::<_, String>(9)?,
                        r.get::<_, String>(10)?,
                        r.get::<_, String>(11)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                let (
                    id,
                    console_id,
                    c_slug,
                    emulator_id,
                    e_slug,
                    game_id,
                    scope_key,
                    feat_raw,
                    emulator_version,
                    status_raw,
                    evidence_notes,
                    verified_at,
                ) = row.map_err(|e| DomainError::Database(e.to_string()))?;
                out.push(RuntimeVerificationRecord {
                    id,
                    console_id,
                    console_slug: c_slug,
                    emulator_id,
                    emulator_slug: e_slug,
                    game_id,
                    scope_key,
                    feature_key: RuntimeVerificationFeature::parse(&feat_raw)?,
                    emulator_version,
                    status: RuntimeVerificationStatus::parse(&status_raw)?,
                    evidence_notes,
                    verified_at,
                });
            }
            Ok(out)
        })
    }

    pub fn list_games_for_console(
        &self,
        profile_id: &str,
        console_slug: &str,
        cover_cache_dir: Option<&Path>,
    ) -> Result<Vec<GameRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT g.id, g.console_id, c.slug, g.canonical_key, g.title, g.description,
                            g.developer, g.publisher, g.release_year, g.genre, g.cover_path,
                            g.metadata_source, g.install_state, g.associated_emulator_id,
                            COALESCE((SELECT SUM(s.duration_seconds) FROM game_sessions s WHERE s.game_id = g.id AND s.profile_id = ?1), 0),
                            (SELECT MAX(s.started_at) FROM game_sessions s WHERE s.game_id = g.id AND s.profile_id = ?1),
                            g.completion_percent,
                            EXISTS(SELECT 1 FROM favorites f WHERE f.game_id = g.id AND f.profile_id = ?1),
                            (SELECT COUNT(*) FROM game_versions gv WHERE gv.game_id = g.id)
                     FROM games g
                     JOIN consoles c ON c.id = g.console_id
                     WHERE c.slug = ?2
                     ORDER BY g.title ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id, console_slug], |r| {
                    Ok(GameRecord {
                        id: r.get(0)?,
                        console_id: r.get(1)?,
                        console_slug: r.get(2)?,
                        canonical_key: r.get(3)?,
                        title: r.get(4)?,
                        description: r.get(5)?,
                        developer: r.get(6)?,
                        publisher: r.get(7)?,
                        release_year: r.get(8)?,
                        genre: r.get(9)?,
                        cover_path: r.get(10)?,
                        metadata_source: r.get(11)?,
                        install_state: r.get(12)?,
                        associated_emulator_id: r.get(13)?,
                        total_playtime_seconds: r.get(14)?,
                        last_played_at: r.get(15)?,
                        completion_percent: r.get(16)?,
                        is_favorite: r.get::<_, i64>(17)? != 0,
                        version_count: r.get(18)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                let mut game = row.map_err(|e| DomainError::Database(e.to_string()))?;
                if let Some(cache_dir) = cover_cache_dir {
                    game.cover_path =
                        CoverCache::sanitize_cover_for_frontend(game.cover_path.as_deref(), cache_dir);
                }
                out.push(game);
            }
            Ok(out)
        })
    }

    pub fn find_existing_version_by_file_path(
        &self,
        file_path: &str,
    ) -> Result<Option<(String, String)>, DomainError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT id, game_id FROM game_versions WHERE file_path = ?1 LIMIT 1",
                params![file_path],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|e| DomainError::Database(e.to_string()))
        })
    }
}

#[derive(Clone)]
pub struct GameDetailsRepository {
    db: DatabaseManager,
}

impl GameDetailsRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn get_game_record(
        &self,
        profile_id: &str,
        game_id: &str,
        cover_cache_dir: Option<&Path>,
    ) -> Result<GameRecord, DomainError> {
        let mut game = self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT g.id, g.console_id, c.slug, g.canonical_key, g.title, g.description,
                        g.developer, g.publisher, g.release_year, g.genre, g.cover_path,
                        g.metadata_source, g.install_state, g.associated_emulator_id,
                        COALESCE((SELECT SUM(s.duration_seconds) FROM game_sessions s WHERE s.game_id = g.id AND s.profile_id = ?1), 0),
                        (SELECT MAX(s.started_at) FROM game_sessions s WHERE s.game_id = g.id AND s.profile_id = ?1),
                        g.completion_percent,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.game_id = g.id AND f.profile_id = ?1),
                        (SELECT COUNT(*) FROM game_versions gv WHERE gv.game_id = g.id)
                 FROM games g
                 JOIN consoles c ON c.id = g.console_id
                 WHERE g.id = ?2",
                params![profile_id, game_id],
                |r| {
                    Ok(GameRecord {
                        id: r.get(0)?,
                        console_id: r.get(1)?,
                        console_slug: r.get(2)?,
                        canonical_key: r.get(3)?,
                        title: r.get(4)?,
                        description: r.get(5)?,
                        developer: r.get(6)?,
                        publisher: r.get(7)?,
                        release_year: r.get(8)?,
                        genre: r.get(9)?,
                        cover_path: r.get(10)?,
                        metadata_source: r.get(11)?,
                        install_state: r.get(12)?,
                        associated_emulator_id: r.get(13)?,
                        total_playtime_seconds: r.get(14)?,
                        last_played_at: r.get(15)?,
                        completion_percent: r.get(16)?,
                        is_favorite: r.get::<_, i64>(17)? != 0,
                        version_count: r.get(18)?,
                    })
                },
            )
            .optional()
            .map_err(|e| DomainError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound(format!("game '{game_id}' not found")))
        })?;

        if let Some(cache_dir) = cover_cache_dir {
            game.cover_path =
                CoverCache::sanitize_cover_for_frontend(game.cover_path.as_deref(), cache_dir);
        }
        Ok(game)
    }

    pub fn list_game_versions(&self, game_id: &str) -> Result<Vec<GameVersionRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, game_id, version_label, region, serial_code, file_path, install_state, created_at
                     FROM game_versions
                     WHERE game_id = ?1
                     ORDER BY created_at ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![game_id], |r| {
                    Ok(GameVersionRecord {
                        id: r.get(0)?,
                        game_id: r.get(1)?,
                        version_label: r.get(2)?,
                        region: r.get(3)?,
                        serial_code: r.get(4)?,
                        file_path: r.get(5)?,
                        install_state: r.get(6)?,
                        created_at: r.get(7)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
            }
            Ok(out)
        })
    }

    pub fn get_game_details(
        &self,
        profile_id: &str,
        game_id: &str,
        cover_cache_dir: Option<&Path>,
    ) -> Result<GameDetailsDto, DomainError> {
        let game = self.get_game_record(profile_id, game_id, cover_cache_dir)?;
        let versions = self.list_game_versions(game_id)?;
        let stats_repo = SessionStatsRepository::new(self.db.clone());
        let sessions = stats_repo.list_sessions_for_game(profile_id, game_id, 50)?;
        Ok(GameDetailsDto {
            game,
            versions,
            sessions,
        })
    }

    pub fn update_game_metadata(
        &self,
        profile_id: &str,
        input: &UpdateGameMetadataInput,
    ) -> Result<GameRecord, DomainError> {
        if !(0..=100).contains(&input.completion_percent) {
            return Err(DomainError::Validation(
                "completion_percent must be between 0 and 100".to_string(),
            ));
        }
        if let Some(year) = input.release_year {
            if !(1970..=2100).contains(&year) {
                return Err(DomainError::Validation(format!(
                    "release_year out of valid range: {year}"
                )));
            }
        }

        let clean_opt = |opt: &Option<String>| -> Option<String> {
            opt.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };

        let dev = clean_opt(&input.developer);
        let pub_name = clean_opt(&input.publisher);
        let genre = clean_opt(&input.genre);
        let now = now_utc_rfc3339();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let title: String = tx
                .query_row(
                    "SELECT title FROM games WHERE id = ?1",
                    params![input.game_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| DomainError::NotFound(format!("game '{}' not found", input.game_id)))?;

            tx.execute(
                "UPDATE games
                 SET description = ?1, developer = ?2, publisher = ?3, release_year = ?4,
                     genre = ?5, completion_percent = ?6, updated_at = ?7
                 WHERE id = ?8",
                params![
                    input.description.trim(),
                    dev,
                    pub_name,
                    input.release_year,
                    genre,
                    input.completion_percent,
                    now,
                    input.game_id,
                ],
            )
            .map_err(|e| DomainError::Database(format!("update game metadata failed: {e}")))?;

            tx.execute(
                "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                 VALUES (?1, ?2, 'game_metadata_updated', 'game', ?3, ?4, ?5)",
                params![
                    Uuid::new_v4().to_string(),
                    profile_id,
                    input.game_id,
                    format!("Metadados de '{title}' atualizados ({}% concluído).", input.completion_percent),
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        self.get_game_record(profile_id, &input.game_id, None)
    }

    pub fn set_game_cover(
        &self,
        game_id: &str,
        authorized_cover_path: &Path,
        cover_cache_dir: &Path,
    ) -> Result<String, DomainError> {
        let verified =
            CoverCache::validate_authorized_cover_path(authorized_cover_path, cover_cache_dir)?;
        let verified_str = verified.to_string_lossy().to_string();
        let now = now_utc_rfc3339();

        self.db.with_connection_mut(|conn| {
            let affected = conn
                .execute(
                    "UPDATE games SET cover_path = ?1, updated_at = ?2 WHERE id = ?3",
                    params![verified_str, now, game_id],
                )
                .map_err(|e| DomainError::Database(format!("update game cover failed: {e}")))?;
            if affected == 0 {
                return Err(DomainError::NotFound(format!("game '{game_id}' not found")));
            }
            Ok(())
        })?;

        Ok(verified_str)
    }
}
