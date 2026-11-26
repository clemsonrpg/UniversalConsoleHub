use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::player::{now_utc_rfc3339, validate_utc_timestamp};

use super::manager::DatabaseManager;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleProfileRecord {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub emulator_id: Option<String>,
    pub emulator_slug: Option<String>,
    pub emulator_status: String,
    pub detected_emulator_version: Option<String>,
    pub detected_firmware_version: Option<String>,
    pub emulator_root_dir: Option<String>,
    pub native_save_count: i64,
    pub native_trophy_dir_present: bool,
    pub trophy_sync_readiness: String,
    pub native_metadata_json: String,
    pub sync_status: String,
    pub sync_diagnostics: String,
    pub last_synced_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLocalUserRecord {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub native_user_key: String,
    pub display_name: String,
    pub source_path: String,
    pub availability_status: String,
    pub linked_profile_id: Option<String>,
    pub first_detected_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertConsoleProfileInput {
    pub console_slug: String,
    pub emulator_slug: Option<String>,
    pub emulator_status: String,
    pub detected_emulator_version: Option<String>,
    pub detected_firmware_version: Option<String>,
    pub emulator_root_dir: Option<String>,
    pub native_save_count: i64,
    pub native_trophy_dir_present: bool,
    pub trophy_sync_readiness: String,
    pub native_metadata_json: String,
    pub sync_status: String,
    pub sync_diagnostics: String,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertConsoleLocalUserInput {
    pub console_slug: String,
    pub native_user_key: String,
    pub display_name: String,
    pub source_path: String,
    pub availability_status: String,
    pub linked_profile_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameVersionAvailabilityRow {
    pub version_id: String,
    pub game_id: String,
    pub file_path: String,
    pub availability_status: String,
    pub cover_path: Option<String>,
}

fn validate_availability_status(value: &str) -> Result<(), DomainError> {
    match value {
        "available" | "missing_on_disk" => Ok(()),
        other => Err(DomainError::Validation(format!(
            "invalid availability_status '{other}' (expected 'available' or 'missing_on_disk')"
        ))),
    }
}

fn validate_emulator_status(value: &str) -> Result<(), DomainError> {
    match value {
        "unconfigured" | "available" | "missing_on_disk" => Ok(()),
        other => Err(DomainError::Validation(format!(
            "invalid emulator_status '{other}'"
        ))),
    }
}

fn validate_trophy_sync_readiness(value: &str) -> Result<(), DomainError> {
    match value {
        "unavailable" | "phase4_planned" | "ready" => Ok(()),
        other => Err(DomainError::Validation(format!(
            "invalid trophy_sync_readiness '{other}'"
        ))),
    }
}

fn validate_sync_status(value: &str) -> Result<(), DomainError> {
    match value {
        "never_synced" | "synced" | "partial" | "error" => Ok(()),
        other => Err(DomainError::Validation(format!(
            "invalid sync_status '{other}'"
        ))),
    }
}

#[derive(Clone)]
pub struct ConsoleProfileRepository {
    db: DatabaseManager,
}

impl ConsoleProfileRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn get_console_profile(
        &self,
        console_slug: &str,
    ) -> Result<Option<ConsoleProfileRecord>, DomainError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT cp.id, cp.console_id, c.slug, cp.emulator_id, e.slug, cp.emulator_status,
                        cp.detected_emulator_version, cp.detected_firmware_version, cp.emulator_root_dir,
                        cp.native_save_count, cp.native_trophy_dir_present, cp.trophy_sync_readiness,
                        cp.native_metadata_json, cp.sync_status, cp.sync_diagnostics,
                        cp.last_synced_at, cp.updated_at
                 FROM console_profiles cp
                 JOIN consoles c ON c.id = cp.console_id
                 LEFT JOIN emulators e ON e.id = cp.emulator_id
                 WHERE c.slug = ?1",
                params![console_slug],
                |r| {
                    Ok(ConsoleProfileRecord {
                        id: r.get(0)?,
                        console_id: r.get(1)?,
                        console_slug: r.get(2)?,
                        emulator_id: r.get(3)?,
                        emulator_slug: r.get(4)?,
                        emulator_status: r.get(5)?,
                        detected_emulator_version: r.get(6)?,
                        detected_firmware_version: r.get(7)?,
                        emulator_root_dir: r.get(8)?,
                        native_save_count: r.get(9)?,
                        native_trophy_dir_present: r.get::<_, i64>(10)? != 0,
                        trophy_sync_readiness: r.get(11)?,
                        native_metadata_json: r.get(12)?,
                        sync_status: r.get(13)?,
                        sync_diagnostics: r.get(14)?,
                        last_synced_at: r.get(15)?,
                        updated_at: r.get(16)?,
                    })
                },
            )
            .optional()
            .map_err(|e| DomainError::Database(e.to_string()))
        })
    }

    pub fn upsert_console_profile(
        &self,
        input: &UpsertConsoleProfileInput,
    ) -> Result<ConsoleProfileRecord, DomainError> {
        validate_emulator_status(&input.emulator_status)?;
        validate_trophy_sync_readiness(&input.trophy_sync_readiness)?;
        validate_sync_status(&input.sync_status)?;
        if input.native_save_count < 0 {
            return Err(DomainError::Validation(
                "native_save_count cannot be negative".to_string(),
            ));
        }
        if let Some(ts) = input.last_synced_at.as_deref() {
            validate_utc_timestamp(ts)?;
        }

        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();
        let trophy_present_int = i64::from(input.native_trophy_dir_present);

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let console_id: String = tx
                .query_row(
                    "SELECT id FROM consoles WHERE slug = ?1",
                    params![input.console_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!("console '{}' not found", input.console_slug))
                })?;

            let emulator_id: Option<String> = match input.emulator_slug.as_deref() {
                Some(slug) => Some(
                    tx.query_row(
                        "SELECT id FROM emulators WHERE slug = ?1",
                        params![slug],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(|e| DomainError::Database(e.to_string()))?
                    .ok_or_else(|| DomainError::NotFound(format!("emulator '{slug}' not found")))?,
                ),
                None => None,
            };

            tx.execute(
                "INSERT INTO console_profiles (
                    id, console_id, emulator_id, emulator_status,
                    detected_emulator_version, detected_firmware_version, emulator_root_dir,
                    native_save_count, native_trophy_dir_present, trophy_sync_readiness,
                    native_metadata_json, sync_status, sync_diagnostics,
                    last_synced_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(console_id) DO UPDATE SET
                    emulator_id = excluded.emulator_id,
                    emulator_status = excluded.emulator_status,
                    detected_emulator_version = excluded.detected_emulator_version,
                    detected_firmware_version = excluded.detected_firmware_version,
                    emulator_root_dir = excluded.emulator_root_dir,
                    native_save_count = excluded.native_save_count,
                    native_trophy_dir_present = excluded.native_trophy_dir_present,
                    trophy_sync_readiness = excluded.trophy_sync_readiness,
                    native_metadata_json = excluded.native_metadata_json,
                    sync_status = excluded.sync_status,
                    sync_diagnostics = excluded.sync_diagnostics,
                    last_synced_at = excluded.last_synced_at,
                    updated_at = excluded.updated_at",
                params![
                    new_id,
                    console_id,
                    emulator_id,
                    input.emulator_status,
                    input.detected_emulator_version,
                    input.detected_firmware_version,
                    input.emulator_root_dir,
                    input.native_save_count,
                    trophy_present_int,
                    input.trophy_sync_readiness,
                    input.native_metadata_json,
                    input.sync_status,
                    input.sync_diagnostics,
                    input.last_synced_at,
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(format!("upsert console_profile failed: {e}")))?;

            let record = tx
                .query_row(
                    "SELECT cp.id, cp.console_id, c.slug, cp.emulator_id, e.slug, cp.emulator_status,
                            cp.detected_emulator_version, cp.detected_firmware_version, cp.emulator_root_dir,
                            cp.native_save_count, cp.native_trophy_dir_present, cp.trophy_sync_readiness,
                            cp.native_metadata_json, cp.sync_status, cp.sync_diagnostics,
                            cp.last_synced_at, cp.updated_at
                     FROM console_profiles cp
                     JOIN consoles c ON c.id = cp.console_id
                     LEFT JOIN emulators e ON e.id = cp.emulator_id
                     WHERE cp.console_id = ?1",
                    params![console_id],
                    |r| {
                        Ok(ConsoleProfileRecord {
                            id: r.get(0)?,
                            console_id: r.get(1)?,
                            console_slug: r.get(2)?,
                            emulator_id: r.get(3)?,
                            emulator_slug: r.get(4)?,
                            emulator_status: r.get(5)?,
                            detected_emulator_version: r.get(6)?,
                            detected_firmware_version: r.get(7)?,
                            emulator_root_dir: r.get(8)?,
                            native_save_count: r.get(9)?,
                            native_trophy_dir_present: r.get::<_, i64>(10)? != 0,
                            trophy_sync_readiness: r.get(11)?,
                            native_metadata_json: r.get(12)?,
                            sync_status: r.get(13)?,
                            sync_diagnostics: r.get(14)?,
                            last_synced_at: r.get(15)?,
                            updated_at: r.get(16)?,
                        })
                    },
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(record)
        })
    }

    pub fn list_local_users(
        &self,
        console_slug: &str,
    ) -> Result<Vec<ConsoleLocalUserRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT clu.id, clu.console_id, c.slug, clu.native_user_key, clu.display_name,
                            clu.source_path, clu.availability_status, clu.linked_profile_id,
                            clu.first_detected_at, clu.last_seen_at
                     FROM console_local_users clu
                     JOIN consoles c ON c.id = clu.console_id
                     WHERE c.slug = ?1
                     ORDER BY clu.native_user_key ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![console_slug], |r| {
                    Ok(ConsoleLocalUserRecord {
                        id: r.get(0)?,
                        console_id: r.get(1)?,
                        console_slug: r.get(2)?,
                        native_user_key: r.get(3)?,
                        display_name: r.get(4)?,
                        source_path: r.get(5)?,
                        availability_status: r.get(6)?,
                        linked_profile_id: r.get(7)?,
                        first_detected_at: r.get(8)?,
                        last_seen_at: r.get(9)?,
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

    pub fn upsert_local_user(
        &self,
        input: &UpsertConsoleLocalUserInput,
    ) -> Result<ConsoleLocalUserRecord, DomainError> {
        let user_key = input.native_user_key.trim();
        let display_name = input.display_name.trim();
        let source_path = input.source_path.trim();
        if user_key.is_empty() || display_name.is_empty() || source_path.is_empty() {
            return Err(DomainError::Validation(
                "native_user_key, display_name, and source_path must be non-empty".to_string(),
            ));
        }
        validate_availability_status(&input.availability_status)?;

        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let console_id: String = tx
                .query_row(
                    "SELECT id FROM consoles WHERE slug = ?1",
                    params![input.console_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!("console '{}' not found", input.console_slug))
                })?;

            tx.execute(
                "INSERT INTO console_local_users (
                    id, console_id, native_user_key, display_name, source_path,
                    availability_status, linked_profile_id, first_detected_at, last_seen_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
                 ON CONFLICT(console_id, native_user_key) DO UPDATE SET
                    display_name = excluded.display_name,
                    source_path = excluded.source_path,
                    availability_status = excluded.availability_status,
                    linked_profile_id = COALESCE(excluded.linked_profile_id, console_local_users.linked_profile_id),
                    last_seen_at = excluded.last_seen_at",
                params![
                    new_id,
                    console_id,
                    user_key,
                    display_name,
                    source_path,
                    input.availability_status,
                    input.linked_profile_id,
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(format!("upsert console_local_user failed: {e}")))?;

            let record = tx
                .query_row(
                    "SELECT clu.id, clu.console_id, c.slug, clu.native_user_key, clu.display_name,
                            clu.source_path, clu.availability_status, clu.linked_profile_id,
                            clu.first_detected_at, clu.last_seen_at
                     FROM console_local_users clu
                     JOIN consoles c ON c.id = clu.console_id
                     WHERE clu.console_id = ?1 AND clu.native_user_key = ?2",
                    params![console_id, user_key],
                    |r| {
                        Ok(ConsoleLocalUserRecord {
                            id: r.get(0)?,
                            console_id: r.get(1)?,
                            console_slug: r.get(2)?,
                            native_user_key: r.get(3)?,
                            display_name: r.get(4)?,
                            source_path: r.get(5)?,
                            availability_status: r.get(6)?,
                            linked_profile_id: r.get(7)?,
                            first_detected_at: r.get(8)?,
                            last_seen_at: r.get(9)?,
                        })
                    },
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(record)
        })
    }

    /// Marks any previously detected local users for `console_slug` whose `native_user_key`
    /// is NOT in `seen_native_keys` as `missing_on_disk`, preserving all historical records.
    pub fn mark_unseen_local_users_missing(
        &self,
        console_slug: &str,
        seen_native_keys: &[String],
    ) -> Result<usize, DomainError> {
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
                .ok_or_else(|| {
                    DomainError::NotFound(format!("console '{console_slug}' not found"))
                })?;

            let existing_keys: Vec<(String, String)> = {
                let mut stmt = tx
                    .prepare(
                        "SELECT id, native_user_key FROM console_local_users
                         WHERE console_id = ?1 AND availability_status != 'missing_on_disk'",
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                let rows = stmt
                    .query_map(params![console_id], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                    })
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                let mut out = Vec::new();
                for r in rows {
                    out.push(r.map_err(|e| DomainError::Database(e.to_string()))?);
                }
                out
            };

            let mut marked = 0usize;
            for (id, key) in existing_keys {
                if !seen_native_keys.iter().any(|k| k == &key) {
                    tx.execute(
                        "UPDATE console_local_users SET availability_status = 'missing_on_disk' WHERE id = ?1",
                        params![id],
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                    marked += 1;
                }
            }

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(marked)
        })
    }

    pub fn update_game_version_availability(
        &self,
        version_id: &str,
        availability_status: &str,
        verified_at_utc: &str,
    ) -> Result<(), DomainError> {
        validate_availability_status(availability_status)?;
        validate_utc_timestamp(verified_at_utc)?;

        let install_state = if availability_status == "available" {
            "installed"
        } else {
            "missing_files"
        };

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let game_id: String = tx
                .query_row(
                    "SELECT game_id FROM game_versions WHERE id = ?1",
                    params![version_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!("game_version '{version_id}' not found"))
                })?;

            tx.execute(
                "UPDATE game_versions
                 SET availability_status = ?1, install_state = ?2, last_verified_at = ?3
                 WHERE id = ?4",
                params![availability_status, install_state, verified_at_utc, version_id],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;

            // Recompute parent game install_state from all versions without deleting history
            let available_versions: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM game_versions WHERE game_id = ?1 AND availability_status = 'available'",
                    params![game_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let game_install_state = if available_versions > 0 {
                "installed"
            } else {
                "missing_files"
            };
            tx.execute(
                "UPDATE games SET install_state = ?1, updated_at = ?2 WHERE id = ?3",
                params![game_install_state, verified_at_utc, game_id],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })
    }

    pub fn list_console_game_versions_for_sync(
        &self,
        console_slug: &str,
    ) -> Result<Vec<GameVersionAvailabilityRow>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT gv.id, gv.game_id, gv.file_path, gv.availability_status, g.cover_path
                     FROM game_versions gv
                     JOIN games g ON g.id = gv.game_id
                     JOIN consoles c ON c.id = g.console_id
                     WHERE c.slug = ?1
                     ORDER BY gv.created_at ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![console_slug], |r| {
                    Ok(GameVersionAvailabilityRow {
                        version_id: r.get(0)?,
                        game_id: r.get(1)?,
                        file_path: r.get(2)?,
                        availability_status: r.get(3)?,
                        cover_path: r.get(4)?,
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
}
