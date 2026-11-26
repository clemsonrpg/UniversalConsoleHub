use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::player::{now_utc_rfc3339, validate_utc_timestamp};
use crate::domain::trophy::{TrophyCategory, TrophyOrigin};

use super::manager::DatabaseManager;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophySetRecord {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub game_id: String,
    pub provider_id: String,
    pub provider_slug: String,
    pub external_set_id: String,
    pub title: String,
    pub detail: String,
    pub icon_path: Option<String>,
    pub total_count: i64,
    pub supports_platinum: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophySetSummaryRow {
    pub id: String,
    pub console_slug: String,
    pub game_id: String,
    pub game_title: String,
    pub provider_slug: String,
    pub external_set_id: String,
    pub title: String,
    pub detail: String,
    pub icon_path: Option<String>,
    pub total_count: i64,
    pub unlocked_count: i64,
    pub bronze_unlocked: i64,
    pub silver_unlocked: i64,
    pub gold_unlocked: i64,
    pub platinum_unlocked: i64,
    pub supports_platinum: bool,
    pub completion_percent: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedTrophyRow {
    pub id: String,
    pub trophy_set_id: Option<String>,
    pub game_id: String,
    pub game_title: String,
    pub console_id: String,
    pub console_slug: String,
    pub provider_id: String,
    pub provider_slug: String,
    pub external_trophy_id: String,
    pub name: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub category: TrophyCategory,
    pub rarity_percent: Option<i64>,
    pub origin: TrophyOrigin,
    pub is_hidden: bool,
    pub is_unlocked: bool,
    pub unlocked_at: Option<String>,
    pub console_local_user_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertTrophySetInput {
    pub console_slug: String,
    pub game_id: String,
    pub provider_slug: String,
    pub external_set_id: String,
    pub title: String,
    pub detail: String,
    pub icon_path: Option<String>,
    pub total_count: i64,
    pub supports_platinum: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertTrophyDefInput {
    pub game_id: String,
    pub console_slug: String,
    pub provider_slug: String,
    pub trophy_set_id: Option<String>,
    pub external_trophy_id: String,
    pub name: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub category: TrophyCategory,
    pub rarity_percent: Option<i64>,
    pub origin: TrophyOrigin,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordUnlockResult {
    pub unlock_id: String,
    pub newly_unlocked: bool,
    pub unlocked_at: String,
}

#[derive(Clone)]
pub struct TrophyRepository {
    db: DatabaseManager,
}

impl TrophyRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn upsert_trophy_set(
        &self,
        input: &UpsertTrophySetInput,
    ) -> Result<TrophySetRecord, DomainError> {
        let ext_id = input.external_set_id.trim();
        let title = input.title.trim();
        if ext_id.is_empty() || title.is_empty() {
            return Err(DomainError::Validation(
                "external_set_id and title must be non-empty".to_string(),
            ));
        }

        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();
        let supports_plat_int = i64::from(input.supports_platinum);

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

            let provider_id: String = tx
                .query_row(
                    "SELECT id FROM achievement_providers WHERE slug = ?1",
                    params![input.provider_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!(
                        "achievement_provider '{}' not found",
                        input.provider_slug
                    ))
                })?;

            tx.execute(
                "INSERT INTO trophy_sets (
                    id, console_id, game_id, provider_id, external_set_id,
                    title, detail, icon_path, total_count, supports_platinum, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(provider_id, external_set_id) DO UPDATE SET
                    game_id = excluded.game_id,
                    title = excluded.title,
                    detail = excluded.detail,
                    icon_path = COALESCE(excluded.icon_path, trophy_sets.icon_path),
                    total_count = excluded.total_count,
                    supports_platinum = excluded.supports_platinum,
                    updated_at = excluded.updated_at",
                params![
                    new_id,
                    console_id,
                    input.game_id,
                    provider_id,
                    ext_id,
                    title,
                    input.detail.trim(),
                    input.icon_path,
                    input.total_count.max(0),
                    supports_plat_int,
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(format!("upsert trophy_set failed: {e}")))?;

            let record = tx
                .query_row(
                    "SELECT ts.id, ts.console_id, c.slug, ts.game_id, ts.provider_id, p.slug,
                            ts.external_set_id, ts.title, ts.detail, ts.icon_path,
                            ts.total_count, ts.supports_platinum, ts.updated_at
                     FROM trophy_sets ts
                     JOIN consoles c ON c.id = ts.console_id
                     JOIN achievement_providers p ON p.id = ts.provider_id
                     WHERE ts.provider_id = ?1 AND ts.external_set_id = ?2",
                    params![provider_id, ext_id],
                    |r| {
                        Ok(TrophySetRecord {
                            id: r.get(0)?,
                            console_id: r.get(1)?,
                            console_slug: r.get(2)?,
                            game_id: r.get(3)?,
                            provider_id: r.get(4)?,
                            provider_slug: r.get(5)?,
                            external_set_id: r.get(6)?,
                            title: r.get(7)?,
                            detail: r.get(8)?,
                            icon_path: r.get(9)?,
                            total_count: r.get(10)?,
                            supports_platinum: r.get::<_, i64>(11)? != 0,
                            updated_at: r.get(12)?,
                        })
                    },
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(record)
        })
    }

    pub fn upsert_trophy_definition(
        &self,
        input: &UpsertTrophyDefInput,
    ) -> Result<DetailedTrophyRow, DomainError> {
        let ext_id = input.external_trophy_id.trim();
        let name = input.name.trim();
        if ext_id.is_empty() || name.is_empty() {
            return Err(DomainError::Validation(
                "external_trophy_id and trophy name must be non-empty".to_string(),
            ));
        }

        let now = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();
        let hidden_int = i64::from(input.is_hidden);

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

            let provider_id: String = tx
                .query_row(
                    "SELECT id FROM achievement_providers WHERE slug = ?1",
                    params![input.provider_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!(
                        "achievement_provider '{}' not found",
                        input.provider_slug
                    ))
                })?;

            // Never update `origin` on conflict (protected by trg_trophies_immutable_origin)
            tx.execute(
                "INSERT INTO trophies (
                    id, game_id, console_id, provider_id, trophy_set_id, external_trophy_id,
                    name, description, icon_path, category, rarity_percent, origin, is_hidden, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(game_id, provider_id, external_trophy_id) DO UPDATE SET
                    trophy_set_id = COALESCE(excluded.trophy_set_id, trophies.trophy_set_id),
                    name = excluded.name,
                    description = excluded.description,
                    icon_path = COALESCE(excluded.icon_path, trophies.icon_path),
                    category = excluded.category,
                    rarity_percent = COALESCE(excluded.rarity_percent, trophies.rarity_percent),
                    is_hidden = excluded.is_hidden",
                params![
                    new_id,
                    input.game_id,
                    console_id,
                    provider_id,
                    input.trophy_set_id,
                    ext_id,
                    name,
                    input.description.trim(),
                    input.icon_path,
                    input.category.as_str(),
                    input.rarity_percent,
                    input.origin.as_str(),
                    hidden_int,
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(format!("upsert trophy failed: {e}")))?;

            let row = tx
                .query_row(
                    "SELECT t.id, t.trophy_set_id, t.game_id, g.title, t.console_id, c.slug,
                            t.provider_id, p.slug, t.external_trophy_id, t.name, t.description,
                            t.icon_path, t.category, t.rarity_percent, t.origin, t.is_hidden
                     FROM trophies t
                     JOIN games g ON g.id = t.game_id
                     JOIN consoles c ON c.id = t.console_id
                     JOIN achievement_providers p ON p.id = t.provider_id
                     WHERE t.game_id = ?1 AND t.provider_id = ?2 AND t.external_trophy_id = ?3",
                    params![input.game_id, provider_id, ext_id],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, Option<String>>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, String>(3)?,
                            r.get::<_, String>(4)?,
                            r.get::<_, String>(5)?,
                            r.get::<_, String>(6)?,
                            r.get::<_, String>(7)?,
                            r.get::<_, String>(8)?,
                            r.get::<_, String>(9)?,
                            r.get::<_, String>(10)?,
                            r.get::<_, Option<String>>(11)?,
                            r.get::<_, String>(12)?,
                            r.get::<_, Option<i64>>(13)?,
                            r.get::<_, String>(14)?,
                            r.get::<_, i64>(15)? != 0,
                        ))
                    },
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            Ok(DetailedTrophyRow {
                id: row.0,
                trophy_set_id: row.1,
                game_id: row.2,
                game_title: row.3,
                console_id: row.4,
                console_slug: row.5,
                provider_id: row.6,
                provider_slug: row.7,
                external_trophy_id: row.8,
                name: row.9,
                description: row.10,
                icon_path: row.11,
                category: TrophyCategory::parse(&row.12)?,
                rarity_percent: row.13,
                origin: TrophyOrigin::parse(&row.14)?,
                is_hidden: row.15,
                is_unlocked: false,
                unlocked_at: None,
                console_local_user_name: None,
            })
        })
    }

    /// Idempotently records a trophy unlock while preserving the earliest verified `unlocked_at`
    /// timestamp and never regressing an already-unlocked trophy back to locked.
    pub fn record_trophy_unlock(
        &self,
        profile_id: &str,
        trophy_id: &str,
        origin: TrophyOrigin,
        unlocked_at_utc: &str,
        console_local_user_id: Option<&str>,
    ) -> Result<RecordUnlockResult, DomainError> {
        validate_utc_timestamp(unlocked_at_utc)?;
        let synced_at = now_utc_rfc3339();
        let new_id = Uuid::new_v4().to_string();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let existing: Option<(String, String)> = tx
                .query_row(
                    "SELECT id, unlocked_at FROM trophy_unlocks WHERE profile_id = ?1 AND trophy_id = ?2",
                    params![profile_id, trophy_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let (unlock_id, newly_unlocked, final_unlocked_at) = match existing {
                None => {
                    tx.execute(
                        "INSERT INTO trophy_unlocks (id, profile_id, trophy_id, origin, unlocked_at, synced_at, console_local_user_id)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            new_id,
                            profile_id,
                            trophy_id,
                            origin.as_str(),
                            unlocked_at_utc,
                            synced_at,
                            console_local_user_id,
                        ],
                    )
                    .map_err(|e| DomainError::Database(format!("insert trophy_unlock failed: {e}")))?;
                    (new_id, true, unlocked_at_utc.to_string())
                }
                Some((existing_id, existing_ts)) => {
                    let earliest = if unlocked_at_utc < existing_ts.as_str() {
                        unlocked_at_utc.to_string()
                    } else {
                        existing_ts
                    };
                    tx.execute(
                        "UPDATE trophy_unlocks
                         SET unlocked_at = ?1,
                             synced_at = ?2,
                             console_local_user_id = COALESCE(?3, console_local_user_id)
                         WHERE id = ?4",
                        params![earliest, synced_at, console_local_user_id, existing_id],
                    )
                    .map_err(|e| DomainError::Database(format!("update trophy_unlock failed: {e}")))?;
                    (existing_id, false, earliest)
                }
            };

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(RecordUnlockResult {
                unlock_id,
                newly_unlocked,
                unlocked_at: final_unlocked_at,
            })
        })
    }

    pub fn list_trophy_set_summaries(
        &self,
        profile_id: &str,
        console_slug: Option<&str>,
    ) -> Result<Vec<TrophySetSummaryRow>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT ts.id, c.slug, ts.game_id, g.title, p.slug, ts.external_set_id,
                            ts.title, ts.detail, ts.icon_path,
                            (SELECT COUNT(*) FROM trophies t WHERE t.trophy_set_id = ts.id) AS actual_total,
                            (SELECT COUNT(*) FROM trophies t
                             JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                             WHERE t.trophy_set_id = ts.id) AS unlocked_total,
                            (SELECT COUNT(*) FROM trophies t
                             JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                             WHERE t.trophy_set_id = ts.id AND t.category = 'bronze') AS bronze_u,
                            (SELECT COUNT(*) FROM trophies t
                             JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                             WHERE t.trophy_set_id = ts.id AND t.category = 'silver') AS silver_u,
                            (SELECT COUNT(*) FROM trophies t
                             JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                             WHERE t.trophy_set_id = ts.id AND t.category = 'gold') AS gold_u,
                            (SELECT COUNT(*) FROM trophies t
                             JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                             WHERE t.trophy_set_id = ts.id AND t.category = 'platinum') AS plat_u,
                            ts.supports_platinum, ts.total_count, ts.updated_at
                     FROM trophy_sets ts
                     JOIN consoles c ON c.id = ts.console_id
                     JOIN games g ON g.id = ts.game_id
                     JOIN achievement_providers p ON p.id = ts.provider_id
                     WHERE (?2 IS NULL OR c.slug = ?2)
                     ORDER BY unlocked_total DESC, ts.title ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id, console_slug], |r| {
                    let actual_total: i64 = r.get(9)?;
                    let unlocked_count: i64 = r.get(10)?;
                    let declared_total: i64 = r.get(16)?;
                    let total_count = actual_total.max(declared_total);
                    let completion_percent = if total_count > 0 {
                        ((unlocked_count * 100) / total_count).clamp(0, 100)
                    } else {
                        0
                    };
                    Ok(TrophySetSummaryRow {
                        id: r.get(0)?,
                        console_slug: r.get(1)?,
                        game_id: r.get(2)?,
                        game_title: r.get(3)?,
                        provider_slug: r.get(4)?,
                        external_set_id: r.get(5)?,
                        title: r.get(6)?,
                        detail: r.get(7)?,
                        icon_path: r.get(8)?,
                        total_count,
                        unlocked_count,
                        bronze_unlocked: r.get(11)?,
                        silver_unlocked: r.get(12)?,
                        gold_unlocked: r.get(13)?,
                        platinum_unlocked: r.get(14)?,
                        supports_platinum: r.get::<_, i64>(15)? != 0,
                        completion_percent,
                        updated_at: r.get(17)?,
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

    pub fn list_detailed_trophies(
        &self,
        profile_id: &str,
        game_id: Option<&str>,
        console_slug: Option<&str>,
    ) -> Result<Vec<DetailedTrophyRow>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT t.id, t.trophy_set_id, t.game_id, g.title, t.console_id, c.slug,
                            t.provider_id, p.slug, t.external_trophy_id, t.name, t.description,
                            t.icon_path, t.category, t.rarity_percent, t.origin, t.is_hidden,
                            u.unlocked_at, clu.display_name
                     FROM trophies t
                     JOIN games g ON g.id = t.game_id
                     JOIN consoles c ON c.id = t.console_id
                     JOIN achievement_providers p ON p.id = t.provider_id
                     LEFT JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                     LEFT JOIN console_local_users clu ON clu.id = u.console_local_user_id
                     WHERE (?2 IS NULL OR t.game_id = ?2)
                       AND (?3 IS NULL OR c.slug = ?3)
                     ORDER BY u.unlocked_at DESC, t.external_trophy_id ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id, game_id, console_slug], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                        r.get::<_, String>(7)?,
                        r.get::<_, String>(8)?,
                        r.get::<_, String>(9)?,
                        r.get::<_, String>(10)?,
                        r.get::<_, Option<String>>(11)?,
                        r.get::<_, String>(12)?,
                        r.get::<_, Option<i64>>(13)?,
                        r.get::<_, String>(14)?,
                        r.get::<_, i64>(15)? != 0,
                        r.get::<_, Option<String>>(16)?,
                        r.get::<_, Option<String>>(17)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                let r = row.map_err(|e| DomainError::Database(e.to_string()))?;
                out.push(DetailedTrophyRow {
                    id: r.0,
                    trophy_set_id: r.1,
                    game_id: r.2,
                    game_title: r.3,
                    console_id: r.4,
                    console_slug: r.5,
                    provider_id: r.6,
                    provider_slug: r.7,
                    external_trophy_id: r.8,
                    name: r.9,
                    description: r.10,
                    icon_path: r.11,
                    category: TrophyCategory::parse(&r.12)?,
                    rarity_percent: r.13,
                    origin: TrophyOrigin::parse(&r.14)?,
                    is_hidden: r.15,
                    is_unlocked: r.16.is_some(),
                    unlocked_at: r.16,
                    console_local_user_name: r.17,
                });
            }
            Ok(out)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_sync_history(
        &self,
        provider_slug: &str,
        profile_id: &str,
        status: &str,
        items_synced: i64,
        error_message: Option<&str>,
        started_at_utc: &str,
        finished_at_utc: &str,
    ) -> Result<(), DomainError> {
        validate_utc_timestamp(started_at_utc)?;
        validate_utc_timestamp(finished_at_utc)?;

        self.db.with_connection_mut(|conn| {
            let provider_id: String = conn
                .query_row(
                    "SELECT id FROM achievement_providers WHERE slug = ?1",
                    params![provider_slug],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!(
                        "achievement_provider '{provider_slug}' not found"
                    ))
                })?;

            conn.execute(
                "INSERT INTO synchronization_history (id, provider_id, profile_id, status, items_synced, error_message, started_at, finished_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    Uuid::new_v4().to_string(),
                    provider_id,
                    profile_id,
                    status,
                    items_synced.max(0),
                    error_message,
                    started_at_utc,
                    finished_at_utc,
                ],
            )
            .map_err(|e| DomainError::Database(format!("insert synchronization_history failed: {e}")))?;
            Ok(())
        })
    }
}
