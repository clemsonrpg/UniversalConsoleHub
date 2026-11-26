use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::adapters::discovery::discovery_for_console;
use crate::adapters::profiles::profile_provider_for_console;
use crate::domain::console_profile::{
    ConsoleSyncReport, EmulatorStatus, ProfileField, SyncTrigger, TrophySyncReadiness,
};
use crate::domain::environment::ConsoleEnvironmentMode;
use crate::domain::error::DomainError;
use crate::domain::player::{now_utc_rfc3339, GameRecord};
use crate::domain::session::{GameSessionRecord, TopGameStat};
use crate::infrastructure::db::console_env_repo::{ConsoleEnvRepository, GameDetailsRepository};
use crate::infrastructure::db::console_profile_repo::{
    ConsoleLocalUserRecord, ConsoleProfileRecord, ConsoleProfileRepository,
    UpsertConsoleLocalUserInput, UpsertConsoleProfileInput,
};
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::SqliteRepositories;
use crate::infrastructure::system::cover_cache::CoverCache;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleHubProfileStatsDto {
    pub total_playtime_seconds: i64,
    pub total_sessions: i64,
    pub games_total: i64,
    pub games_available: i64,
    pub games_missing_on_disk: i64,
    pub most_played: Vec<TopGameStat>,
    pub recently_played: Vec<GameRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleProfileOverviewDto {
    pub console_slug: String,
    pub console_name: String,
    pub environment_mode: ConsoleEnvironmentMode,
    pub profile: Option<ConsoleProfileRecord>,
    pub local_users: Vec<ConsoleLocalUserRecord>,
    pub emulator_status: ProfileField<EmulatorStatus>,
    pub emulator_version: ProfileField<String>,
    pub firmware_version: ProfileField<String>,
    pub emulator_root_dir: ProfileField<String>,
    pub native_save_count: ProfileField<i64>,
    pub trophy_readiness: ProfileField<TrophySyncReadiness>,
    pub hub_stats: ConsoleHubProfileStatsDto,
    pub recent_sessions: Vec<GameSessionRecord>,
}

#[derive(Clone)]
pub struct ConsoleProfileSyncService {
    db: DatabaseManager,
    repos: SqliteRepositories,
    env_repo: ConsoleEnvRepository,
    details_repo: GameDetailsRepository,
    profile_repo: ConsoleProfileRepository,
    cover_cache_dir: PathBuf,
}

impl ConsoleProfileSyncService {
    pub fn new(db: DatabaseManager, cover_cache_dir: PathBuf) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            env_repo: ConsoleEnvRepository::new(db.clone()),
            details_repo: GameDetailsRepository::new(db.clone()),
            profile_repo: ConsoleProfileRepository::new(db.clone()),
            db,
            cover_cache_dir,
        }
    }

    pub fn sync_console_profile(
        &self,
        console_slug: &str,
        trigger: SyncTrigger,
    ) -> Result<ConsoleSyncReport, DomainError> {
        let console = self.repos.get_console_by_slug(console_slug)?;
        let emulator = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.console_slug == console_slug);

        let configured_exe = emulator
            .as_ref()
            .and_then(|e| e.executable_path.as_deref())
            .map(PathBuf::from);

        let provider = profile_provider_for_console(console_slug);
        let snapshot = provider.collect_native_snapshot(configured_exe.as_deref());
        let now = now_utc_rfc3339();
        let mut diagnostics = snapshot.diagnostics.clone();

        // Keep emulator detected_version synchronized when real version is found
        if let (Some(emu), Some(ver)) = (
            emulator.as_ref(),
            snapshot.detected_emulator_version.as_deref(),
        ) {
            if let Some(exe) = emu.executable_path.as_deref() {
                if emu.detected_version.as_deref() != Some(ver) {
                    let _ = self
                        .repos
                        .update_emulator_executable(&emu.slug, exe, Some(ver));
                }
            }
        }

        // 1. Reconcile real local console users (Adjustment A & B: never invent users, never delete history)
        let mut seen_user_keys = Vec::with_capacity(snapshot.local_users.len());
        for detected in &snapshot.local_users {
            seen_user_keys.push(detected.native_user_key.clone());
            self.profile_repo
                .upsert_local_user(&UpsertConsoleLocalUserInput {
                    console_slug: console_slug.to_string(),
                    native_user_key: detected.native_user_key.clone(),
                    display_name: detected.display_name.clone(),
                    source_path: detected.source_path.to_string_lossy().to_string(),
                    availability_status: "available".to_string(),
                    linked_profile_id: None,
                })?;
        }
        let local_users_marked_missing = self
            .profile_repo
            .mark_unseen_local_users_missing(console_slug, &seen_user_keys)?;

        // 2. Auto-discover games from native emulator dirs (e.g. dev_hdd0/game) + user-configured scan paths
        let mut games_discovered = 0usize;
        if let Some(discovery) = discovery_for_console(console_slug) {
            let mut dirs_to_scan: Vec<(PathBuf, bool)> = Vec::new();
            let mut seen_dirs: BTreeSet<PathBuf> = BTreeSet::new();

            for native_dir in &snapshot.native_game_scan_dirs {
                if seen_dirs.insert(native_dir.clone()) {
                    dirs_to_scan.push((native_dir.clone(), true));
                }
            }
            for sp in self.env_repo.list_scan_paths(console_slug)? {
                if sp.enabled {
                    let p = PathBuf::from(&sp.directory_path);
                    if seen_dirs.insert(p.clone()) {
                        dirs_to_scan.push((p, sp.recursive));
                    }
                }
            }

            for (dir, recursive) in dirs_to_scan {
                if !dir.is_dir() {
                    diagnostics.push(format!(
                        "Diretório de jogos ausente no disco: '{}'.",
                        dir.display()
                    ));
                    continue;
                }
                match discovery.discover_in_directory(&dir, recursive, &mut |_| {}) {
                    Ok(batch) => {
                        diagnostics.extend(batch.diagnostics);
                        for candidate in batch.candidates {
                            let file_path_str = candidate.file_path.to_string_lossy().to_string();
                            if self
                                .env_repo
                                .find_existing_version_by_file_path(&file_path_str)?
                                .is_some()
                            {
                                continue;
                            }
                            match self.repos.register_game_with_version(
                                console_slug,
                                &candidate.title,
                                &candidate.version_label,
                                &candidate.region,
                                candidate.serial_code.as_deref(),
                                &file_path_str,
                                &candidate.metadata_source,
                            ) {
                                Ok(_) => {
                                    games_discovered += 1;
                                }
                                Err(DomainError::Conflict(_)) => {}
                                Err(err) => {
                                    diagnostics.push(format!(
                                        "Falha ao sincronizar '{}': {err}",
                                        candidate.title
                                    ));
                                }
                            }
                        }
                    }
                    Err(err) => {
                        diagnostics
                            .push(format!("Falha na descoberta em '{}': {err}", dir.display()));
                    }
                }
            }
        }

        // 3. Verify disk availability of every registered version & auto-import native artwork via CoverCache
        let versions = self
            .profile_repo
            .list_console_game_versions_for_sync(console_slug)?;
        let mut versions_available = 0usize;
        let mut versions_marked_missing = 0usize;
        let mut covers_imported = 0usize;
        let mut games_with_cover_checked: BTreeSet<String> = BTreeSet::new();

        for ver in versions {
            let file_path = Path::new(&ver.file_path);
            if file_path.is_file() {
                versions_available += 1;
                self.profile_repo.update_game_version_availability(
                    &ver.version_id,
                    "available",
                    &now,
                )?;

                // Import native artwork (e.g. PS3 ICON0.PNG / PIC1.PNG) if game has no valid cached cover yet
                let existing_cover_valid = ver.cover_path.as_deref().is_some_and(|cp| {
                    CoverCache::sanitize_cover_for_frontend(Some(cp), &self.cover_cache_dir)
                        .is_some()
                });
                if !existing_cover_valid && games_with_cover_checked.insert(ver.game_id.clone()) {
                    if let Some(art) = provider.find_native_artwork_for_game(file_path) {
                        let chosen_art = art.icon_path.or(art.background_path);
                        if let Some(src_art) = chosen_art {
                            match CoverCache::import_local_image(&src_art, &self.cover_cache_dir) {
                                Ok(cached_path) => {
                                    if self
                                        .details_repo
                                        .set_game_cover(
                                            &ver.game_id,
                                            &cached_path,
                                            &self.cover_cache_dir,
                                        )
                                        .is_ok()
                                    {
                                        covers_imported += 1;
                                    }
                                }
                                Err(err) => {
                                    diagnostics.push(format!(
                                        "Arte nativa ignorada ('{}'): {err}",
                                        src_art.display()
                                    ));
                                }
                            }
                        }
                    }
                }
            } else {
                versions_marked_missing += 1;
                self.profile_repo.update_game_version_availability(
                    &ver.version_id,
                    "missing_on_disk",
                    &now,
                )?;
            }
        }

        let sync_status = if snapshot.emulator_status == EmulatorStatus::MissingOnDisk
            || versions_marked_missing > 0
        {
            "partial".to_string()
        } else {
            "synced".to_string()
        };

        // 4. Persist console_profiles (#23) snapshot
        let _ = self
            .profile_repo
            .upsert_console_profile(&UpsertConsoleProfileInput {
                console_slug: console_slug.to_string(),
                emulator_slug: emulator
                    .as_ref()
                    .map(|e| e.slug.clone())
                    .or(snapshot.emulator_slug),
                emulator_status: snapshot.emulator_status.as_str().to_string(),
                detected_emulator_version: snapshot.detected_emulator_version,
                detected_firmware_version: snapshot.detected_firmware_version,
                emulator_root_dir: snapshot
                    .emulator_root_dir
                    .map(|p| p.to_string_lossy().to_string()),
                native_save_count: snapshot.native_save_count,
                native_trophy_dir_present: snapshot.native_trophy_dir_present,
                trophy_sync_readiness: snapshot.trophy_sync_readiness.as_str().to_string(),
                native_metadata_json: snapshot.native_metadata_json,
                sync_status: sync_status.clone(),
                sync_diagnostics: diagnostics.join(" | "),
                last_synced_at: Some(now.clone()),
            })?;

        // Record concise sync event in activity_history (never mutating achievement synchronization_history)
        if let Ok(active_profile) = self.repos.get_active_profile() {
            let summary = format!(
                "Perfil de {} sincronizado ({}): {} jogos disponíveis, {} ausentes, {} usuários locais reais.",
                console.name,
                trigger.as_str(),
                versions_available,
                versions_marked_missing,
                snapshot.local_users.len()
            );
            let _ = self.db.with_connection_mut(|conn| {
                conn.execute(
                    "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                     VALUES (?1, ?2, 'console_profile_synced', 'console_profile', ?3, ?4, ?5)",
                    params![
                        Uuid::new_v4().to_string(),
                        active_profile.id,
                        console.id,
                        summary,
                        now,
                    ],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(())
            });
        }

        Ok(ConsoleSyncReport {
            console_slug: console_slug.to_string(),
            trigger,
            sync_status,
            emulator_status: snapshot.emulator_status,
            local_users_detected: snapshot.local_users.len(),
            local_users_marked_missing,
            games_discovered,
            versions_available,
            versions_marked_missing,
            covers_imported,
            diagnostics,
            synced_at: now,
        })
    }

    pub fn get_console_profile_overview(
        &self,
        console_slug: &str,
    ) -> Result<ConsoleProfileOverviewDto, DomainError> {
        let console = self.repos.get_console_by_slug(console_slug)?;
        let environment_mode = ConsoleEnvironmentMode::from_capabilities(&console.capabilities);
        let active_profile = self.repos.get_active_profile()?;
        let profile = self.profile_repo.get_console_profile(console_slug)?;
        let local_users = self.profile_repo.list_local_users(console_slug)?;
        let console_games = self.env_repo.list_games_for_console(
            &active_profile.id,
            console_slug,
            Some(&self.cover_cache_dir),
        )?;

        let (
            emulator_status,
            emulator_version,
            firmware_version,
            emulator_root_dir,
            native_save_count,
            trophy_readiness,
        ) = if let Some(rec) = profile.as_ref() {
            let emu_status_parsed =
                EmulatorStatus::parse(&rec.emulator_status).unwrap_or(EmulatorStatus::Unconfigured);
            let emu_status_field = match emu_status_parsed {
                EmulatorStatus::Unconfigured => {
                    ProfileField::unavailable("Emulador ainda não configurado para este console.")
                }
                status => ProfileField::native(status),
            };
            let emu_ver = match rec.detected_emulator_version.clone() {
                Some(v) => ProfileField::native(v),
                None => ProfileField::unavailable(
                    "Versão do emulador não detectada automaticamente no disco.",
                ),
            };
            let fw_ver = match rec.detected_firmware_version.clone() {
                Some(v) => ProfileField::native(v),
                None => ProfileField::unavailable(
                    "Versão de firmware/sistema não detectada no diretório do emulador.",
                ),
            };
            let root_dir = match rec.emulator_root_dir.clone() {
                Some(v) => ProfileField::native(v),
                None => ProfileField::unavailable("Diretório raiz do emulador não configurado."),
            };
            let saves = if emu_status_parsed == EmulatorStatus::Available {
                ProfileField::native(rec.native_save_count)
            } else {
                ProfileField::unavailable(
                    "Contagem de saves indisponível sem diretório ativo do emulador.",
                )
            };
            let readiness_parsed = TrophySyncReadiness::parse(&rec.trophy_sync_readiness)
                .unwrap_or(TrophySyncReadiness::Unavailable);
            let trophy_field = match readiness_parsed {
                TrophySyncReadiness::Unavailable => ProfileField::unavailable(
                    "Esta plataforma não possui sistema nativo de troféus/conquistas.",
                ),
                TrophySyncReadiness::Phase4Planned => ProfileField::native_with_note(
                    TrophySyncReadiness::Phase4Planned,
                    if rec.native_trophy_dir_present {
                        "Diretório local de troféus detectado; leitura real de TROPCONF.SFM/TROPUSR.DAT planejada para a Fase 4."
                    } else {
                        "Integração real de troféus planejada para a Fase 4 (diretório local ainda não detectado)."
                    },
                ),
                TrophySyncReadiness::Ready => ProfileField::native(TrophySyncReadiness::Ready),
            };
            (
                emu_status_field,
                emu_ver,
                fw_ver,
                root_dir,
                saves,
                trophy_field,
            )
        } else {
            (
                ProfileField::unavailable("Perfil do console ainda não sincronizado."),
                ProfileField::unavailable(
                    "Sincronize o perfil para detectar a versão do emulador.",
                ),
                ProfileField::unavailable("Sincronize o perfil para detectar o firmware."),
                ProfileField::unavailable("Diretório do emulador não configurado."),
                ProfileField::unavailable("Sem dados nativos sincronizados."),
                ProfileField::unavailable("Prontidão de troféus aguardando sincronização."),
            )
        };

        let hub_stats = self.compute_console_hub_stats(
            &active_profile.id,
            &console.id,
            console_slug,
            &console_games,
        )?;
        let recent_sessions =
            self.list_recent_console_sessions(&active_profile.id, &console.id, 15)?;

        Ok(ConsoleProfileOverviewDto {
            console_slug: console.slug,
            console_name: console.name,
            environment_mode,
            profile,
            local_users,
            emulator_status,
            emulator_version,
            firmware_version,
            emulator_root_dir,
            native_save_count,
            trophy_readiness,
            hub_stats,
            recent_sessions,
        })
    }

    fn compute_console_hub_stats(
        &self,
        profile_id: &str,
        console_id: &str,
        console_slug: &str,
        games: &[GameRecord],
    ) -> Result<ConsoleHubProfileStatsDto, DomainError> {
        let (total_playtime_seconds, total_sessions, games_available, games_missing_on_disk) =
            self.db.with_connection(|conn| {
                let (playtime, sessions): (i64, i64) = conn
                    .query_row(
                        "SELECT COALESCE(SUM(s.duration_seconds), 0), COUNT(s.id)
                         FROM game_sessions s
                         JOIN games g ON g.id = s.game_id
                         WHERE s.profile_id = ?1 AND g.console_id = ?2",
                        params![profile_id, console_id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;

                let available: i64 = conn
                    .query_row(
                        "SELECT COUNT(DISTINCT g.id)
                         FROM games g
                         JOIN game_versions gv ON gv.game_id = g.id
                         WHERE g.console_id = ?1 AND gv.availability_status = 'available'",
                        params![console_id],
                        |r| r.get(0),
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;

                let total_games = games.len() as i64;
                let missing = (total_games - available).max(0);
                Ok((playtime, sessions, available, missing))
            })?;

        let most_played = self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT g.id, g.title, COUNT(s.id) AS sessions_count,
                            COALESCE(SUM(s.duration_seconds), 0) AS total_seconds,
                            g.completion_percent, MAX(s.started_at) AS last_played
                     FROM games g
                     JOIN game_sessions s ON s.game_id = g.id AND s.profile_id = ?1
                     WHERE g.console_id = ?2
                     GROUP BY g.id, g.title, g.completion_percent
                     ORDER BY total_seconds DESC, sessions_count DESC
                     LIMIT 5",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![profile_id, console_id], |r| {
                    Ok(TopGameStat {
                        game_id: r.get(0)?,
                        title: r.get(1)?,
                        console_slug: console_slug.to_string(),
                        sessions_count: r.get(2)?,
                        total_playtime_seconds: r.get(3)?,
                        completion_percent: r.get(4)?,
                        last_played_at: r.get(5)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
            }
            Ok(out)
        })?;

        let mut recently_played: Vec<GameRecord> = games
            .iter()
            .filter(|g| g.last_played_at.is_some())
            .cloned()
            .collect();
        recently_played.sort_by(|a, b| b.last_played_at.cmp(&a.last_played_at));
        recently_played.truncate(5);

        Ok(ConsoleHubProfileStatsDto {
            total_playtime_seconds,
            total_sessions,
            games_total: games.len() as i64,
            games_available,
            games_missing_on_disk,
            most_played,
            recently_played,
        })
    }

    fn list_recent_console_sessions(
        &self,
        profile_id: &str,
        console_id: &str,
        limit: i64,
    ) -> Result<Vec<GameSessionRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, s.profile_id, s.game_id, g.title, c.slug, s.game_version_id,
                            s.emulator_id, s.started_at, s.last_heartbeat_at, s.ended_at,
                            s.duration_seconds, s.exit_status
                     FROM game_sessions s
                     JOIN games g ON g.id = s.game_id
                     JOIN consoles c ON c.id = g.console_id
                     WHERE s.profile_id = ?1 AND g.console_id = ?2
                     ORDER BY s.started_at DESC
                     LIMIT ?3",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id, console_id, limit], |r| {
                    Ok(GameSessionRecord {
                        id: r.get(0)?,
                        profile_id: r.get(1)?,
                        game_id: r.get(2)?,
                        game_title: r.get(3)?,
                        console_slug: r.get(4)?,
                        game_version_id: r.get(5)?,
                        emulator_id: r.get(6)?,
                        started_at: r.get(7)?,
                        last_heartbeat_at: r.get(8)?,
                        ended_at: r.get(9)?,
                        duration_seconds: r.get(10)?,
                        exit_status: r.get(11)?,
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
