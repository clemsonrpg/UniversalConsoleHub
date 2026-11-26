use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::adapters::trophies::trophy_reader_for_console;
use crate::domain::error::DomainError;
use crate::domain::player::{normalize_canonical_key, now_utc_rfc3339};
use crate::domain::trophy::{
    compute_trophy_completion_percent, AchievementProviderRecord, NewlyUnlockedTrophy,
    ParsedTrophySet, TrophyCategory, TrophyOrigin, TrophySyncReport,
};
use crate::infrastructure::db::console_profile_repo::ConsoleProfileRepository;
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::SqliteRepositories;
use crate::infrastructure::db::trophy_repo::{
    DetailedTrophyRow, TrophyRepository, TrophySetSummaryRow, UpsertTrophyDefInput,
    UpsertTrophySetInput,
};
use crate::infrastructure::system::cover_cache::CoverCache;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophyTotalsDto {
    pub total_sets: i64,
    pub completed_sets: i64,
    pub total_trophies: i64,
    pub unlocked_trophies: i64,
    pub bronze_total: i64,
    pub bronze_unlocked: i64,
    pub silver_total: i64,
    pub silver_unlocked: i64,
    pub gold_total: i64,
    pub gold_unlocked: i64,
    pub platinum_total: i64,
    pub platinum_unlocked: i64,
    pub completion_percent: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophyCenterOverviewDto {
    pub providers: Vec<AchievementProviderRecord>,
    pub sets: Vec<TrophySetSummaryRow>,
    pub trophies: Vec<DetailedTrophyRow>,
    pub recent_unlocks: Vec<DetailedTrophyRow>,
    pub totals: TrophyTotalsDto,
}

#[derive(Clone)]
pub struct TrophySyncService {
    db: DatabaseManager,
    repos: SqliteRepositories,
    trophy_repo: TrophyRepository,
    profile_repo: ConsoleProfileRepository,
    cover_cache_dir: PathBuf,
}

impl TrophySyncService {
    pub fn new(db: DatabaseManager, cover_cache_dir: PathBuf) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            trophy_repo: TrophyRepository::new(db.clone()),
            profile_repo: ConsoleProfileRepository::new(db.clone()),
            db,
            cover_cache_dir,
        }
    }

    pub fn sync_console_trophies(
        &self,
        console_slug: &str,
    ) -> Result<TrophySyncReport, DomainError> {
        let console = self.repos.get_console_by_slug(console_slug)?;
        let active_profile = self.repos.get_active_profile()?;
        let started_at = now_utc_rfc3339();

        let Some(reader) = trophy_reader_for_console(console_slug) else {
            let synced_at = now_utc_rfc3339();
            return Ok(TrophySyncReport {
                console_slug: console_slug.to_string(),
                sets_synced: 0,
                trophies_synced: 0,
                unlocks_recorded: 0,
                newly_unlocked: Vec::new(),
                icons_cached: 0,
                diagnostics: vec![format!(
                    "O console '{}' não possui leitor nativo de troféus implementado nesta fase.",
                    console.name
                )],
                synced_at,
            });
        };

        let root_dir = self.resolve_emulator_root_dir(console_slug)?;
        let Some(root_dir) = root_dir.filter(|p| p.is_dir()) else {
            let finished_at = now_utc_rfc3339();
            let msg = format!(
                "Diretório raiz do emulador de '{}' não está configurado ou ausente no disco; histórico de troféus preservado.",
                console.name
            );
            let _ = self.trophy_repo.record_sync_history(
                "emulator",
                &active_profile.id,
                "unavailable",
                0,
                Some(&msg),
                &started_at,
                &finished_at,
            );
            return Ok(TrophySyncReport {
                console_slug: console_slug.to_string(),
                sets_synced: 0,
                trophies_synced: 0,
                unlocks_recorded: 0,
                newly_unlocked: Vec::new(),
                icons_cached: 0,
                diagnostics: vec![msg],
                synced_at: finished_at,
            });
        };

        let batch = reader.read_trophy_sets(&root_dir);
        let mut diagnostics = batch.diagnostics;

        let local_user_map: HashMap<String, String> = self
            .profile_repo
            .list_local_users(console_slug)?
            .into_iter()
            .map(|u| (u.native_user_key, u.id))
            .collect();

        let mut sets_synced = 0usize;
        let mut trophies_synced = 0usize;
        let mut unlocks_recorded = 0usize;
        let mut icons_cached = 0usize;
        let mut newly_unlocked = Vec::new();
        let mut affected_game_ids = Vec::new();

        for set in batch.sets {
            let cached_set_icon = self.cache_optional_icon(
                set.icon_path.as_deref(),
                &mut icons_cached,
                &mut diagnostics,
            );

            let game_id = self.resolve_or_create_game_for_trophy_set(
                &console.id,
                console_slug,
                &set,
                cached_set_icon.as_deref(),
            )?;
            if !affected_game_ids.contains(&game_id) {
                affected_game_ids.push(game_id.clone());
            }

            let set_record = self.trophy_repo.upsert_trophy_set(&UpsertTrophySetInput {
                console_slug: console_slug.to_string(),
                game_id: game_id.clone(),
                provider_slug: "emulator".to_string(),
                external_set_id: set.external_set_id.clone(),
                title: set.title.clone(),
                detail: set.detail.clone(),
                icon_path: cached_set_icon,
                total_count: set.trophies.len() as i64,
                supports_platinum: set.supports_platinum(),
            })?;
            sets_synced += 1;

            let mut trophy_by_ext_id: BTreeMap<String, DetailedTrophyRow> = BTreeMap::new();
            for def in &set.trophies {
                let cached_icon = self.cache_optional_icon(
                    def.icon_path.as_deref(),
                    &mut icons_cached,
                    &mut diagnostics,
                );
                let upserted =
                    self.trophy_repo
                        .upsert_trophy_definition(&UpsertTrophyDefInput {
                            game_id: game_id.clone(),
                            console_slug: console_slug.to_string(),
                            provider_slug: "emulator".to_string(),
                            trophy_set_id: Some(set_record.id.clone()),
                            external_trophy_id: def.external_trophy_id.clone(),
                            name: def.name.clone(),
                            description: def.description.clone(),
                            icon_path: cached_icon,
                            category: def.category,
                            rarity_percent: None, // Offline RPCS3 trophies NEVER invent global rarity
                            origin: TrophyOrigin::Emulator,
                            is_hidden: def.is_hidden,
                        })?;
                trophies_synced += 1;
                trophy_by_ext_id.insert(def.external_trophy_id.clone(), upserted);
            }

            for unlock in &set.unlocks {
                let Some(trophy_row) = trophy_by_ext_id.get(&unlock.external_trophy_id) else {
                    diagnostics.push(format!(
                        "Desbloqueio ignorado para troféu inexistente '{}' em '{}'.",
                        unlock.external_trophy_id, set.external_set_id
                    ));
                    continue;
                };

                let local_user_id = local_user_map
                    .get(&unlock.native_user_key)
                    .or_else(|| local_user_map.get(&set.native_user_key))
                    .map(String::as_str);

                match self.trophy_repo.record_trophy_unlock(
                    &active_profile.id,
                    &trophy_row.id,
                    TrophyOrigin::Emulator,
                    &unlock.unlocked_at_utc,
                    local_user_id,
                ) {
                    Ok(rec) => {
                        unlocks_recorded += 1;
                        if rec.newly_unlocked {
                            newly_unlocked.push(NewlyUnlockedTrophy {
                                trophy_id: trophy_row.id.clone(),
                                external_trophy_id: trophy_row.external_trophy_id.clone(),
                                game_id: trophy_row.game_id.clone(),
                                game_title: trophy_row.game_title.clone(),
                                name: trophy_row.name.clone(),
                                category: trophy_row.category,
                                unlocked_at: rec.unlocked_at,
                                icon_path: CoverCache::sanitize_cover_for_frontend(
                                    trophy_row.icon_path.as_deref(),
                                    &self.cover_cache_dir,
                                ),
                            });
                        }
                    }
                    Err(err) => {
                        diagnostics.push(format!(
                            "Falha ao registrar desbloqueio de '{}' ({}): {err}",
                            trophy_row.name, set.external_set_id
                        ));
                    }
                }
            }
        }

        // Recompute real completion_percent on every touched game
        for game_id in &affected_game_ids {
            let _ = self.recompute_game_completion_percent(&active_profile.id, game_id);
        }

        let finished_at = now_utc_rfc3339();
        let diag_summary = if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics.join(" | "))
        };
        let sync_status = if sets_synced > 0 || diagnostics.is_empty() {
            "completed"
        } else {
            "partial"
        };

        self.trophy_repo.record_sync_history(
            "emulator",
            &active_profile.id,
            sync_status,
            trophies_synced as i64,
            diag_summary.as_deref(),
            &started_at,
            &finished_at,
        )?;

        if !newly_unlocked.is_empty() {
            let summary = if newly_unlocked.len() == 1 {
                format!(
                    "Troféu desbloqueado em {}: '{}'.",
                    newly_unlocked[0].game_title, newly_unlocked[0].name
                )
            } else {
                format!(
                    "{} novos troféus sincronizados em {} via leitor local do emulador.",
                    newly_unlocked.len(),
                    console.name
                )
            };
            let _ = self.db.with_connection_mut(|conn| {
                conn.execute(
                    "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                     VALUES (?1, ?2, 'trophies_unlocked', 'console', ?3, ?4, ?5)",
                    params![
                        Uuid::new_v4().to_string(),
                        active_profile.id,
                        console.id,
                        summary,
                        finished_at,
                    ],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(())
            });
        }

        Ok(TrophySyncReport {
            console_slug: console_slug.to_string(),
            sets_synced,
            trophies_synced,
            unlocks_recorded,
            newly_unlocked,
            icons_cached,
            diagnostics,
            synced_at: finished_at,
        })
    }

    pub fn get_trophy_center_overview(
        &self,
        console_slug: Option<&str>,
    ) -> Result<TrophyCenterOverviewDto, DomainError> {
        let active_profile = self.repos.get_active_profile()?;
        let providers = self.repos.list_achievement_providers()?;
        let mut sets = self
            .trophy_repo
            .list_trophy_set_summaries(&active_profile.id, console_slug)?;
        let mut trophies =
            self.trophy_repo
                .list_detailed_trophies(&active_profile.id, None, console_slug)?;

        for set in &mut sets {
            set.icon_path = CoverCache::sanitize_cover_for_frontend(
                set.icon_path.as_deref(),
                &self.cover_cache_dir,
            );
        }
        for trophy in &mut trophies {
            trophy.icon_path = CoverCache::sanitize_cover_for_frontend(
                trophy.icon_path.as_deref(),
                &self.cover_cache_dir,
            );
        }

        let mut totals = TrophyTotalsDto {
            total_sets: sets.len() as i64,
            completed_sets: sets.iter().filter(|s| s.completion_percent == 100).count() as i64,
            total_trophies: trophies.len() as i64,
            ..TrophyTotalsDto::default()
        };

        for t in &trophies {
            if t.is_unlocked {
                totals.unlocked_trophies += 1;
            }
            match t.category {
                TrophyCategory::Bronze => {
                    totals.bronze_total += 1;
                    if t.is_unlocked {
                        totals.bronze_unlocked += 1;
                    }
                }
                TrophyCategory::Silver => {
                    totals.silver_total += 1;
                    if t.is_unlocked {
                        totals.silver_unlocked += 1;
                    }
                }
                TrophyCategory::Gold => {
                    totals.gold_total += 1;
                    if t.is_unlocked {
                        totals.gold_unlocked += 1;
                    }
                }
                TrophyCategory::Platinum => {
                    totals.platinum_total += 1;
                    if t.is_unlocked {
                        totals.platinum_unlocked += 1;
                    }
                }
                _ => {}
            }
        }
        totals.completion_percent =
            compute_trophy_completion_percent(totals.unlocked_trophies, totals.total_trophies);

        let mut recent_unlocks: Vec<DetailedTrophyRow> = trophies
            .iter()
            .filter(|t| t.is_unlocked && t.unlocked_at.is_some())
            .cloned()
            .collect();
        recent_unlocks.sort_by(|a, b| b.unlocked_at.cmp(&a.unlocked_at));
        recent_unlocks.truncate(12);

        Ok(TrophyCenterOverviewDto {
            providers,
            sets,
            trophies,
            recent_unlocks,
            totals,
        })
    }

    pub fn list_game_trophies(&self, game_id: &str) -> Result<Vec<DetailedTrophyRow>, DomainError> {
        let active_profile = self.repos.get_active_profile()?;
        let mut rows =
            self.trophy_repo
                .list_detailed_trophies(&active_profile.id, Some(game_id), None)?;
        for row in &mut rows {
            row.icon_path = CoverCache::sanitize_cover_for_frontend(
                row.icon_path.as_deref(),
                &self.cover_cache_dir,
            );
        }
        Ok(rows)
    }

    fn resolve_emulator_root_dir(
        &self,
        console_slug: &str,
    ) -> Result<Option<PathBuf>, DomainError> {
        if let Some(emu) = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.console_slug == console_slug)
        {
            if let Some(exe) = emu.executable_path.as_deref() {
                if let Some(parent) = Path::new(exe).parent() {
                    return Ok(Some(parent.to_path_buf()));
                }
            }
        }
        if let Some(profile) = self.profile_repo.get_console_profile(console_slug)? {
            if let Some(root) = profile.emulator_root_dir {
                return Ok(Some(PathBuf::from(root)));
            }
        }
        Ok(None)
    }

    fn cache_optional_icon(
        &self,
        icon_path: Option<&Path>,
        icons_cached: &mut usize,
        diagnostics: &mut Vec<String>,
    ) -> Option<String> {
        let src = icon_path?;
        match CoverCache::import_local_image(src, &self.cover_cache_dir) {
            Ok(cached) => {
                *icons_cached += 1;
                Some(cached.to_string_lossy().to_string())
            }
            Err(err) => {
                diagnostics.push(format!(
                    "Ícone de troféu ignorado ('{}'): {err}",
                    src.display()
                ));
                None
            }
        }
    }

    fn resolve_or_create_game_for_trophy_set(
        &self,
        console_id: &str,
        _console_slug: &str,
        set: &ParsedTrophySet,
        cached_icon_path: Option<&str>,
    ) -> Result<String, DomainError> {
        let title_key = normalize_canonical_key(&set.title, None)?;
        let fallback_key = normalize_canonical_key(&set.title, Some(&set.external_set_id))?;
        let trimmed_title = set.title.trim();
        let now = now_utc_rfc3339();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            // 1. Prefer existing trophy_set link if already persisted
            if let Some(existing_game_id) = tx
                .query_row(
                    "SELECT ts.game_id
                     FROM trophy_sets ts
                     JOIN achievement_providers p ON p.id = ts.provider_id
                     WHERE p.slug = 'emulator' AND ts.external_set_id = ?1",
                    params![set.external_set_id],
                    |r| r.get::<_, String>(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
            {
                tx.commit()
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                return Ok(existing_game_id);
            }

            // 2. Match existing game on the same console by case-insensitive title or canonical key
            let mut stmt = tx
                .prepare("SELECT id, title, canonical_key FROM games WHERE console_id = ?1")
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let existing_games: Vec<(String, String, String)> = stmt
                .query_map(params![console_id], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?
                .filter_map(Result::ok)
                .collect();
            drop(stmt);

            for (gid, gtitle, gkey) in &existing_games {
                if gtitle.trim().eq_ignore_ascii_case(trimmed_title)
                    || gkey == &title_key
                    || gkey == &fallback_key
                {
                    tx.commit()
                        .map_err(|e| DomainError::Database(e.to_string()))?;
                    return Ok(gid.clone());
                }
                if let Ok(existing_title_key) = normalize_canonical_key(gtitle, None) {
                    if existing_title_key == title_key {
                        tx.commit()
                            .map_err(|e| DomainError::Database(e.to_string()))?;
                        return Ok(gid.clone());
                    }
                }
            }

            // 3. Create honest canonical game entry sourced from RPCS3 trophy set (no fake game_versions)
            let default_emu_id: Option<String> = tx
                .query_row(
                    "SELECT id FROM emulators WHERE console_id = ?1 LIMIT 1",
                    params![console_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let new_game_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO games (
                    id, console_id, canonical_key, title, description, cover_path,
                    metadata_source, install_state, associated_emulator_id,
                    completion_percent, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'rpcs3_trophy_set', 'missing_files', ?7, 0, ?8, ?8)",
                params![
                    new_game_id,
                    console_id,
                    fallback_key,
                    trimmed_title,
                    set.detail.trim(),
                    cached_icon_path,
                    default_emu_id,
                    now,
                ],
            )
            .map_err(|e| DomainError::Database(format!("insert game from trophy set failed: {e}")))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(new_game_id)
        })
    }

    fn recompute_game_completion_percent(
        &self,
        profile_id: &str,
        game_id: &str,
    ) -> Result<(), DomainError> {
        let now = now_utc_rfc3339();
        self.db.with_connection_mut(|conn| {
            let total: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM trophies WHERE game_id = ?1",
                    params![game_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            if total == 0 {
                return Ok(());
            }
            let unlocked: i64 = conn
                .query_row(
                    "SELECT COUNT(*)
                     FROM trophies t
                     JOIN trophy_unlocks u ON u.trophy_id = t.id AND u.profile_id = ?1
                     WHERE t.game_id = ?2",
                    params![profile_id, game_id],
                    |r| r.get(0),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let pct = compute_trophy_completion_percent(unlocked, total);
            conn.execute(
                "UPDATE games SET completion_percent = ?1, updated_at = ?2 WHERE id = ?3",
                params![pct, now, game_id],
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })
    }
}
