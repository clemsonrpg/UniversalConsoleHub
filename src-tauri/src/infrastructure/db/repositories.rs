use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::console::{ConsoleCapabilities, ConsoleRecord};
use crate::domain::emulator::EmulatorRecord;
use crate::domain::error::DomainError;
use crate::domain::player::{
    normalize_canonical_key, now_utc_rfc3339, GameRecord, GameVersionRecord, PlayerProfile,
    ProfileStatistics,
};
use crate::domain::trophy::{
    AchievementProviderRecord, TrophyCategory, TrophyOrigin, TrophyRecord,
};

use super::manager::DatabaseManager;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareRecordDto {
    pub id: String,
    pub console_slug: String,
    pub emulator_slug: String,
    pub file_role: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub sha256: String,
    pub validation_status: String,
    pub notes: String,
    pub validated_at: String,
}

#[derive(Clone)]
pub struct SqliteRepositories {
    pub(crate) db: DatabaseManager,
}

fn parse_origin(raw: &str) -> Result<TrophyOrigin, DomainError> {
    match raw {
        "official" => Ok(TrophyOrigin::Official),
        "emulator" => Ok(TrophyOrigin::Emulator),
        "local" => Ok(TrophyOrigin::Local),
        other => Err(DomainError::Database(format!(
            "invalid trophy origin in db: {other}"
        ))),
    }
}

fn parse_category(raw: &str) -> Result<TrophyCategory, DomainError> {
    match raw {
        "bronze" => Ok(TrophyCategory::Bronze),
        "silver" => Ok(TrophyCategory::Silver),
        "gold" => Ok(TrophyCategory::Gold),
        "platinum" => Ok(TrophyCategory::Platinum),
        "xbox_achievement" => Ok(TrophyCategory::XboxAchievement),
        "steam_achievement" => Ok(TrophyCategory::SteamAchievement),
        "emulator_achievement" => Ok(TrophyCategory::EmulatorAchievement),
        "hub_local" => Ok(TrophyCategory::HubLocal),
        other => Err(DomainError::Database(format!(
            "invalid trophy category in db: {other}"
        ))),
    }
}

impl SqliteRepositories {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DatabaseManager {
        &self.db
    }

    pub fn list_consoles(&self) -> Result<Vec<ConsoleRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, slug, name, manufacturer, generation, release_year, phase_status, capabilities_json, notes
                     FROM consoles
                     ORDER BY CASE slug
                        WHEN 'ps3' THEN 1
                        WHEN 'ps2' THEN 2
                        WHEN 'wii' THEN 3
                        WHEN 'xbox' THEN 4
                        WHEN 'xbox360' THEN 5
                        WHEN 'ps4' THEN 6
                        ELSE 99
                     END, name ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    let caps_raw: String = row.get(7)?;
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        caps_raw,
                        row.get::<_, String>(8)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for item in rows {
                let (id, slug, name, manufacturer, generation, release_year, phase_status, caps_raw, notes) =
                    item.map_err(|e| DomainError::Database(e.to_string()))?;
                let capabilities: ConsoleCapabilities = serde_json::from_str(&caps_raw)
                    .map_err(|e| DomainError::Database(format!("bad capabilities JSON: {e}")))?;
                out.push(ConsoleRecord {
                    id,
                    slug,
                    name,
                    manufacturer,
                    generation,
                    release_year,
                    phase_status,
                    capabilities,
                    notes,
                });
            }
            Ok(out)
        })
    }

    pub fn get_console_by_slug(&self, slug: &str) -> Result<ConsoleRecord, DomainError> {
        self.list_consoles()?
            .into_iter()
            .find(|c| c.slug == slug)
            .ok_or_else(|| DomainError::NotFound(format!("console '{slug}' not found")))
    }

    pub fn list_emulators(&self) -> Result<Vec<EmulatorRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT e.id, e.slug, e.name, e.console_id, c.slug, e.executable_path, e.config_path, e.detected_version, e.integration_phase, e.capabilities_json
                     FROM emulators e
                     JOIN consoles c ON c.id = e.console_id
                     ORDER BY CASE e.slug
                        WHEN 'rpcs3' THEN 1
                        WHEN 'pcsx2' THEN 2
                        WHEN 'dolphin' THEN 3
                        WHEN 'xemu' THEN 4
                        WHEN 'xenia-canary' THEN 5
                        WHEN 'shadps4' THEN 6
                        ELSE 99
                     END",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for item in rows {
                let (
                    id,
                    slug,
                    name,
                    console_id,
                    console_slug,
                    executable_path,
                    config_path,
                    detected_version,
                    integration_phase,
                    caps_raw,
                ) = item.map_err(|e| DomainError::Database(e.to_string()))?;
                let capabilities: ConsoleCapabilities = serde_json::from_str(&caps_raw)
                    .map_err(|e| DomainError::Database(format!("bad emulator caps JSON: {e}")))?;
                out.push(EmulatorRecord {
                    id,
                    slug,
                    name,
                    console_id,
                    console_slug,
                    executable_path,
                    config_path,
                    detected_version,
                    integration_phase,
                    capabilities,
                });
            }
            Ok(out)
        })
    }

    pub fn update_emulator_executable(
        &self,
        slug: &str,
        exe_path: &str,
        detected_version: Option<&str>,
    ) -> Result<EmulatorRecord, DomainError> {
        let now = now_utc_rfc3339();
        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let affected = tx
                .execute(
                    "UPDATE emulators SET executable_path = ?1, detected_version = ?2, updated_at = ?3 WHERE slug = ?4",
                    params![exe_path, detected_version, now, slug],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            if affected == 0 {
                return Err(DomainError::NotFound(format!("emulator '{slug}' not found")));
            }
            if let Some(version) = detected_version {
                let emu_id: String = tx
                    .query_row("SELECT id FROM emulators WHERE slug = ?1", params![slug], |r| r.get(0))
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                tx.execute(
                    "INSERT OR IGNORE INTO emulator_versions (id, emulator_id, version, executable_sha256, detected_at)
                     VALUES (?1, ?2, ?3, NULL, ?4)",
                    params![Uuid::new_v4().to_string(), emu_id, version, now],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            }
            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        self.list_emulators()?
            .into_iter()
            .find(|e| e.slug == slug)
            .ok_or_else(|| DomainError::NotFound(format!("emulator '{slug}' not found")))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_firmware_record(
        &self,
        console_slug: &str,
        emulator_slug: &str,
        file_role: &str,
        file_path: &str,
        file_size_bytes: u64,
        sha256: &str,
        validation_status: &str,
        notes: &str,
    ) -> Result<FirmwareRecordDto, DomainError> {
        let now = now_utc_rfc3339();
        let id = Uuid::new_v4().to_string();
        let size_i64 = i64::try_from(file_size_bytes)
            .map_err(|_| DomainError::Validation("file size exceeds i64".to_string()))?;

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
                "INSERT INTO console_firmware (id, console_id, emulator_id, file_role, file_path, file_size_bytes, sha256, validation_status, notes, validated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(console_id, file_role, file_path) DO UPDATE SET
                   file_size_bytes = excluded.file_size_bytes,
                   sha256 = excluded.sha256,
                   validation_status = excluded.validation_status,
                   notes = excluded.notes,
                   validated_at = excluded.validated_at",
                params![
                    id,
                    console_id,
                    emulator_id,
                    file_role,
                    file_path,
                    size_i64,
                    sha256,
                    validation_status,
                    notes,
                    now
                ],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;
            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        Ok(FirmwareRecordDto {
            id,
            console_slug: console_slug.to_string(),
            emulator_slug: emulator_slug.to_string(),
            file_role: file_role.to_string(),
            file_path: file_path.to_string(),
            file_size_bytes: size_i64,
            sha256: sha256.to_string(),
            validation_status: validation_status.to_string(),
            notes: notes.to_string(),
            validated_at: now,
        })
    }

    pub fn list_firmware_records(&self) -> Result<Vec<FirmwareRecordDto>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT f.id, c.slug, e.slug, f.file_role, f.file_path, f.file_size_bytes, f.sha256, f.validation_status, f.notes, f.validated_at
                     FROM console_firmware f
                     JOIN consoles c ON c.id = f.console_id
                     JOIN emulators e ON e.id = f.emulator_id
                     ORDER BY f.validated_at DESC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(FirmwareRecordDto {
                        id: r.get(0)?,
                        console_slug: r.get(1)?,
                        emulator_slug: r.get(2)?,
                        file_role: r.get(3)?,
                        file_path: r.get(4)?,
                        file_size_bytes: r.get(5)?,
                        sha256: r.get(6)?,
                        validation_status: r.get(7)?,
                        notes: r.get(8)?,
                        validated_at: r.get(9)?,
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

    pub fn get_active_profile(&self) -> Result<PlayerProfile, DomainError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT p.id, p.user_id, u.username, p.handle, p.display_name, p.avatar_path, p.banner_path, p.bio, p.privacy_level, p.created_at, p.updated_at
                 FROM profiles p
                 JOIN users u ON u.id = p.user_id
                 WHERE p.is_active = 1
                 LIMIT 1",
                [],
                |row| {
                    Ok(PlayerProfile {
                        id: row.get(0)?,
                        user_id: row.get(1)?,
                        username: row.get(2)?,
                        handle: row.get(3)?,
                        display_name: row.get(4)?,
                        avatar_path: row.get(5)?,
                        banner_path: row.get(6)?,
                        bio: row.get(7)?,
                        privacy_level: row.get(8)?,
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(|e| DomainError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("no active profile configured".to_string()))
        })
    }

    pub fn update_active_profile(
        &self,
        username: &str,
        display_name: &str,
        bio: &str,
        privacy_level: &str,
    ) -> Result<PlayerProfile, DomainError> {
        let trimmed_user = username.trim();
        let trimmed_name = display_name.trim();
        if trimmed_user.is_empty() || trimmed_name.is_empty() {
            return Err(DomainError::Validation(
                "username and display_name cannot be empty".to_string(),
            ));
        }
        if !matches!(privacy_level, "private" | "friends" | "public") {
            return Err(DomainError::Validation(format!(
                "invalid privacy_level: {privacy_level}"
            )));
        }

        let current = self.get_active_profile()?;
        let now = now_utc_rfc3339();
        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.execute(
                "UPDATE users SET username = ?1, updated_at = ?2 WHERE id = ?3",
                params![trimmed_user, now, current.user_id],
            )
            .map_err(|e| DomainError::Database(format!("update username failed: {e}")))?;

            tx.execute(
                "UPDATE profiles SET handle = ?1, display_name = ?2, bio = ?3, privacy_level = ?4, updated_at = ?5 WHERE id = ?6",
                params![trimmed_user, trimmed_name, bio.trim(), privacy_level, now, current.id],
            )
            .map_err(|e| DomainError::Database(format!("update profile failed: {e}")))?;

            tx.execute(
                "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                 VALUES (?1, ?2, 'profile_updated', 'profile', ?3, ?4, ?5)",
                params![
                    Uuid::new_v4().to_string(),
                    current.id,
                    current.id,
                    format!("Perfil atualizado para '{trimmed_name}' (@{trimmed_user})."),
                    now
                ],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        self.get_active_profile()
    }

    pub fn get_profile_statistics(
        &self,
        profile_id: &str,
    ) -> Result<ProfileStatistics, DomainError> {
        self.db.with_connection(|conn| {
            let games_registered: i64 = conn
                .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let games_started: i64 = conn
                .query_row(
                    "SELECT COUNT(DISTINCT game_id) FROM game_sessions WHERE profile_id = ?1",
                    params![profile_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let games_completed: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM games WHERE completion_percent = 100",
                    [],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let total_playtime_seconds: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(duration_seconds), 0) FROM game_sessions WHERE profile_id = ?1",
                    params![profile_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let total_trophies: i64 = conn
                .query_row("SELECT COUNT(*) FROM trophies", [], |r| r.get(0))
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let unlocked_trophies: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM trophy_unlocks WHERE profile_id = ?1",
                    params![profile_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let rare_trophies: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM trophy_unlocks u
                     JOIN trophies t ON t.id = u.trophy_id
                     WHERE u.profile_id = ?1 AND t.rarity_percent IS NOT NULL AND t.rarity_percent <= 15",
                    params![profile_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            Ok(ProfileStatistics {
                games_registered,
                games_started,
                games_completed,
                total_playtime_seconds,
                total_trophies,
                unlocked_trophies,
                rare_trophies,
            })
        })
    }

    pub fn list_games(&self, profile_id: &str) -> Result<Vec<GameRecord>, DomainError> {
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
                     ORDER BY g.title ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id], |r| {
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
                out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
            }
            Ok(out)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_game_with_version(
        &self,
        console_slug: &str,
        title: &str,
        version_label: &str,
        region: &str,
        serial_code: Option<&str>,
        file_path: &str,
        metadata_source: &str,
    ) -> Result<(GameRecord, GameVersionRecord), DomainError> {
        let trimmed_title = title.trim();
        let trimmed_path = file_path.trim();
        if trimmed_title.is_empty() || trimmed_path.is_empty() {
            return Err(DomainError::Validation(
                "title and file_path are required".to_string(),
            ));
        }

        let canonical_key = normalize_canonical_key(trimmed_title, serial_code)?;
        let now = now_utc_rfc3339();
        let profile = self.get_active_profile()?;
        let version_id = Uuid::new_v4().to_string();
        let clean_version = if version_label.trim().is_empty() {
            "default".to_string()
        } else {
            version_label.trim().to_string()
        };
        let clean_region = if region.trim().is_empty() {
            "UNKNOWN".to_string()
        } else {
            region.trim().to_string()
        };
        let clean_source = if metadata_source.trim().is_empty() {
            "manual_user_entry".to_string()
        } else {
            metadata_source.trim().to_string()
        };

        let game_id = self.db.with_connection_mut(|conn| {
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

            let default_emu_id: Option<String> = tx
                .query_row(
                    "SELECT id FROM emulators WHERE console_id = ?1 LIMIT 1",
                    params![console_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let existing_game_id: Option<String> = tx
                .query_row(
                    "SELECT id FROM games WHERE console_id = ?1 AND canonical_key = ?2",
                    params![console_id, canonical_key],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let gid = match existing_game_id {
                Some(id) => id,
                None => {
                    let new_id = Uuid::new_v4().to_string();
                    tx.execute(
                        "INSERT INTO games (id, console_id, canonical_key, title, description, metadata_source, install_state, associated_emulator_id, completion_percent, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, '', ?5, 'registered', ?6, 0, ?7, ?7)",
                        params![
                            new_id,
                            console_id,
                            canonical_key,
                            trimmed_title,
                            clean_source,
                            default_emu_id,
                            now
                        ],
                    )
                    .map_err(|e| DomainError::Database(format!("insert game failed: {e}")))?;
                    new_id
                }
            };

            tx.execute(
                "INSERT INTO game_versions (id, game_id, version_label, region, serial_code, file_path, install_state, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'registered', ?7)",
                params![
                    version_id,
                    gid,
                    clean_version,
                    clean_region,
                    serial_code,
                    trimmed_path,
                    now
                ],
            )
            .map_err(|e| DomainError::Conflict(format!("duplicate or invalid game version: {e}")))?;

            tx.execute(
                "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                 VALUES (?1, ?2, 'game_registered', 'game', ?3, ?4, ?5)",
                params![
                    Uuid::new_v4().to_string(),
                    profile.id,
                    gid,
                    format!("Jogo '{trimmed_title}' ({clean_region} / {clean_version}) registrado na biblioteca."),
                    now
                ],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(gid)
        })?;

        let game = self
            .list_games(&profile.id)?
            .into_iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| DomainError::NotFound("registered game missing".to_string()))?;

        let version = GameVersionRecord {
            id: version_id,
            game_id,
            version_label: clean_version,
            region: clean_region,
            serial_code: serial_code.map(str::to_string),
            file_path: trimmed_path.to_string(),
            install_state: "registered".to_string(),
            created_at: now,
        };

        Ok((game, version))
    }

    pub fn toggle_favorite(&self, profile_id: &str, game_id: &str) -> Result<bool, DomainError> {
        let now = now_utc_rfc3339();
        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let existing: Option<String> = tx
                .query_row(
                    "SELECT id FROM favorites WHERE profile_id = ?1 AND game_id = ?2",
                    params![profile_id, game_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let now_favorite = if let Some(fav_id) = existing {
                tx.execute("DELETE FROM favorites WHERE id = ?1", params![fav_id])
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                false
            } else {
                tx.execute(
                    "INSERT INTO favorites (id, profile_id, game_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                    params![Uuid::new_v4().to_string(), profile_id, game_id, now],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
                true
            };

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(now_favorite)
        })
    }

    pub fn list_achievement_providers(
        &self,
    ) -> Result<Vec<AchievementProviderRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, slug, name, origin, availability_status, supports_auth, supports_sync, notes
                     FROM achievement_providers
                     ORDER BY CASE slug
                        WHEN 'local' THEN 1
                        WHEN 'emulator' THEN 2
                        WHEN 'playstation' THEN 3
                        WHEN 'xbox' THEN 4
                        WHEN 'steam' THEN 5
                        ELSE 99
                     END",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, i64>(5)? != 0,
                        r.get::<_, i64>(6)? != 0,
                        r.get::<_, String>(7)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                let (id, slug, name, origin_raw, availability_status, supports_auth, supports_sync, notes) =
                    row.map_err(|e| DomainError::Database(e.to_string()))?;
                out.push(AchievementProviderRecord {
                    id,
                    slug,
                    name,
                    origin: parse_origin(&origin_raw)?,
                    availability_status,
                    supports_auth,
                    supports_sync,
                    notes,
                });
            }
            Ok(out)
        })
    }

    pub fn list_trophies(&self, profile_id: &str) -> Result<Vec<TrophyRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT t.id, t.game_id, t.console_id, t.provider_id, t.external_trophy_id,
                            t.name, t.description, t.icon_path, t.category, t.rarity_percent,
                            t.origin, u.unlocked_at
                     FROM trophies t
                     LEFT JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                     ORDER BY u.unlocked_at DESC, t.name ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                        r.get::<_, Option<String>>(7)?,
                        r.get::<_, String>(8)?,
                        r.get::<_, Option<i64>>(9)?,
                        r.get::<_, String>(10)?,
                        r.get::<_, Option<String>>(11)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                let (
                    id,
                    game_id,
                    console_id,
                    provider_id,
                    external_trophy_id,
                    name,
                    description,
                    icon_path,
                    cat_raw,
                    rarity_percent,
                    origin_raw,
                    unlocked_at,
                ) = row.map_err(|e| DomainError::Database(e.to_string()))?;
                out.push(TrophyRecord {
                    id,
                    game_id,
                    console_id,
                    provider_id,
                    external_trophy_id,
                    name,
                    description,
                    icon_path,
                    category: parse_category(&cat_raw)?,
                    rarity_percent,
                    origin: parse_origin(&origin_raw)?,
                    is_unlocked: unlocked_at.is_some(),
                    unlocked_at,
                });
            }
            Ok(out)
        })
    }

    pub fn get_all_settings(&self) -> Result<BTreeMap<String, String>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare("SELECT key, value FROM application_settings ORDER BY key ASC")
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let mut map = BTreeMap::new();
            for row in rows {
                let (k, v) = row.map_err(|e| DomainError::Database(e.to_string()))?;
                map.insert(k, v);
            }
            Ok(map)
        })
    }

    pub fn upsert_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let trimmed_key = key.trim();
        if trimmed_key.is_empty() {
            return Err(DomainError::Validation(
                "setting key cannot be empty".to_string(),
            ));
        }
        let now = now_utc_rfc3339();
        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;
            tx.execute(
                "INSERT INTO application_settings (key, value, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![trimmed_key, value, now],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;
            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })
    }
}
