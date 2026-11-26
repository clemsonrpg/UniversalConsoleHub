use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::adapters::registry::AdapterRegistry;
use crate::domain::console::ConsoleRecord;
use crate::domain::console_profile::SyncTrigger;
use crate::domain::emulator::{
    EmulatorInstallation, EmulatorRecord, FirmwarePackageReport, SystemBootReadiness,
};
use crate::domain::error::DomainError;
use crate::domain::player::{GameRecord, GameVersionRecord, PlayerProfile, ProfileStatistics};
use crate::domain::trophy::{AchievementProviderRecord, TrophyRecord};
use crate::infrastructure::db::manager::{BackupReport, DatabaseManager};
use crate::infrastructure::db::repositories::{FirmwareRecordDto, SqliteRepositories};
use crate::infrastructure::system::cover_cache::CoverCache;
use crate::infrastructure::system::process_manager::ProcessManager;
use crate::infrastructure::system::security::SecurityManager;

use super::backup_service::BackupService;
use super::console_profile_sync_service::ConsoleProfileSyncService;
use super::environment_service::{ConsoleEnvironmentDto, EnvironmentService};
use super::scanner_service::ScannerService;
use super::session_service::SessionService;
use super::statistics_service::StatisticsService;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntryDto {
    pub id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rpcs3InspectionDto {
    pub configured: bool,
    pub installation: Option<EmulatorInstallation>,
    pub readiness: Option<SystemBootReadiness>,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleDetailsDto {
    pub console: ConsoleRecord,
    pub emulator: Option<EmulatorRecord>,
    pub firmware_records: Vec<FirmwareRecordDto>,
    pub rpcs3_inspection: Option<Rpcs3InspectionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HubBootstrapDto {
    pub app_name: String,
    pub version: String,
    pub phase: u8,
    pub profile: PlayerProfile,
    pub statistics: ProfileStatistics,
    pub consoles: Vec<ConsoleRecord>,
    pub emulators: Vec<EmulatorRecord>,
    pub games: Vec<GameRecord>,
    pub achievement_providers: Vec<AchievementProviderRecord>,
    pub trophies: Vec<TrophyRecord>,
    pub firmware_records: Vec<FirmwareRecordDto>,
    pub recent_activity: Vec<ActivityEntryDto>,
    pub settings: BTreeMap<String, String>,
    pub rpcs3_inspection: Rpcs3InspectionDto,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub privacy_level: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterGameRequest {
    pub console_slug: String,
    pub title: String,
    pub version_label: String,
    pub region: String,
    pub serial_code: Option<String>,
    pub file_path: String,
    pub metadata_source: String,
}

#[derive(Clone)]
pub struct AppServices {
    pub repos: SqliteRepositories,
    pub adapters: AdapterRegistry,
    pub process_manager: ProcessManager,
    pub backup_dir: PathBuf,
    pub cover_cache_dir: PathBuf,
    environment: EnvironmentService,
    console_profile_sync: ConsoleProfileSyncService,
    scanner: ScannerService,
    sessions: SessionService,
    statistics: StatisticsService,
    backups: BackupService,
}

impl AppServices {
    /// Preserved Phase 1 constructor signature for full backward compatibility.
    pub fn new(db: DatabaseManager, adapters: AdapterRegistry, backup_dir: PathBuf) -> Self {
        let cover_cache_dir = backup_dir
            .parent()
            .unwrap_or(&backup_dir)
            .join("cache")
            .join("covers");
        Self::with_dirs(db, adapters, backup_dir, cover_cache_dir)
    }

    pub fn with_dirs(
        db: DatabaseManager,
        adapters: AdapterRegistry,
        backup_dir: PathBuf,
        cover_cache_dir: PathBuf,
    ) -> Self {
        let environment = EnvironmentService::new(db.clone(), cover_cache_dir.clone());
        let console_profile_sync =
            ConsoleProfileSyncService::new(db.clone(), cover_cache_dir.clone());
        let scanner = ScannerService::new(db.clone());
        let sessions = SessionService::new(db.clone());
        let statistics = StatisticsService::new(db.clone());
        let backups = BackupService::new(db.clone(), backup_dir.clone());
        Self {
            repos: SqliteRepositories::new(db),
            adapters,
            process_manager: ProcessManager::new(),
            backup_dir,
            cover_cache_dir,
            environment,
            console_profile_sync,
            scanner,
            sessions,
            statistics,
            backups,
        }
    }

    pub fn environment_service(&self) -> &EnvironmentService {
        &self.environment
    }

    pub fn console_profile_sync_service(&self) -> &ConsoleProfileSyncService {
        &self.console_profile_sync
    }

    pub fn scanner_service(&self) -> &ScannerService {
        &self.scanner
    }

    pub fn session_service(&self) -> &SessionService {
        &self.sessions
    }

    pub fn statistics_service(&self) -> &StatisticsService {
        &self.statistics
    }

    pub fn backup_service(&self) -> &BackupService {
        &self.backups
    }

    pub fn get_console_environment(
        &self,
        slug: &str,
    ) -> Result<ConsoleEnvironmentDto, DomainError> {
        let rpcs3_inspection = if slug == "ps3" {
            Some(self.inspect_rpcs3()?)
        } else {
            None
        };
        self.environment
            .get_console_environment(slug, rpcs3_inspection)
    }

    pub fn list_games_sanitized(&self) -> Result<Vec<GameRecord>, DomainError> {
        let profile = self.repos.get_active_profile()?;
        let mut games = self.repos.list_games(&profile.id)?;
        for game in &mut games {
            game.cover_path = CoverCache::sanitize_cover_for_frontend(
                game.cover_path.as_deref(),
                &self.cover_cache_dir,
            );
        }
        Ok(games)
    }

    pub fn list_recent_activity(
        &self,
        profile_id: &str,
        limit: i64,
    ) -> Result<Vec<ActivityEntryDto>, DomainError> {
        self.repos.db().with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, event_type, entity_type, entity_id, summary, occurred_at
                     FROM activity_history
                     WHERE profile_id = ?1
                     ORDER BY occurred_at DESC
                     LIMIT ?2",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id, limit], |r| {
                    Ok(ActivityEntryDto {
                        id: r.get(0)?,
                        event_type: r.get(1)?,
                        entity_type: r.get(2)?,
                        entity_id: r.get(3)?,
                        summary: r.get(4)?,
                        occurred_at: r.get(5)?,
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

    pub fn get_bootstrap(&self) -> Result<HubBootstrapDto, DomainError> {
        let profile = self.repos.get_active_profile()?;
        let statistics = self.repos.get_profile_statistics(&profile.id)?;
        let consoles = self.repos.list_consoles()?;
        let emulators = self.repos.list_emulators()?;
        let games = self.list_games_sanitized()?;
        let achievement_providers = self.repos.list_achievement_providers()?;
        let trophies = self.repos.list_trophies(&profile.id)?;
        let firmware_records = self.repos.list_firmware_records()?;
        let recent_activity = self.list_recent_activity(&profile.id, 15)?;
        let settings = self.repos.get_all_settings()?;
        let rpcs3_inspection = self.inspect_rpcs3()?;

        Ok(HubBootstrapDto {
            app_name: "Universal Console Hub".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            phase: 1,
            profile,
            statistics,
            consoles,
            emulators,
            games,
            achievement_providers,
            trophies,
            firmware_records,
            recent_activity,
            settings,
            rpcs3_inspection,
        })
    }

    pub fn get_console_details(&self, slug: &str) -> Result<ConsoleDetailsDto, DomainError> {
        let console = self.repos.get_console_by_slug(slug)?;
        let emulator = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.console_slug == slug);
        let firmware_records = self
            .repos
            .list_firmware_records()?
            .into_iter()
            .filter(|f| f.console_slug == slug)
            .collect();
        let rpcs3_inspection = if slug == "ps3" {
            Some(self.inspect_rpcs3()?)
        } else {
            None
        };

        Ok(ConsoleDetailsDto {
            console,
            emulator,
            firmware_records,
            rpcs3_inspection,
        })
    }

    pub fn configure_emulator_path(
        &self,
        slug: &str,
        exe_path: &str,
    ) -> Result<EmulatorRecord, DomainError> {
        let trimmed = exe_path.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "caminho do executável não pode estar vazio".to_string(),
            ));
        }

        let detected_version = if let Some(adapter) = self.adapters.get(slug) {
            let install = adapter.detect_installation(Some(Path::new(trimmed)))?;
            install.detected_version
        } else {
            // For future Phase 5 emulators, still validate local executable file cleanly
            let _ = SecurityManager::validate_existing_file(Path::new(trimmed))?;
            None
        };

        let record =
            self.repos
                .update_emulator_executable(slug, trimmed, detected_version.as_deref())?;
        // Automatically synchronize real console profile after emulator configuration
        let _ = self
            .console_profile_sync
            .sync_console_profile(&record.console_slug, SyncTrigger::EmulatorConfigured);
        Ok(record)
    }

    pub fn inspect_rpcs3(&self) -> Result<Rpcs3InspectionDto, DomainError> {
        let adapter = self
            .adapters
            .get("rpcs3")
            .ok_or_else(|| DomainError::NotFound("adaptador RPCS3 não registrado".to_string()))?;

        let emu = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.slug == "rpcs3");

        let Some(exe_str) = emu.and_then(|e| e.executable_path) else {
            return Ok(Rpcs3InspectionDto {
                configured: false,
                installation: None,
                readiness: None,
                diagnostic: "RPCS3 ainda não configurado. Informe o caminho local do rpcs3.exe para inspecionar arquivos de sistema e firmware.".to_string(),
            });
        };

        let install = match adapter.detect_installation(Some(Path::new(&exe_str))) {
            Ok(inst) => inst,
            Err(err) => {
                return Ok(Rpcs3InspectionDto {
                    configured: true,
                    installation: None,
                    readiness: None,
                    diagnostic: format!(
                        "Caminho configurado '{exe_str}' não pôde ser validado: {err}"
                    ),
                });
            }
        };

        // Check whether a validated PS3 firmware package record exists in SQLite
        let latest_fw = self
            .repos
            .list_firmware_records()?
            .into_iter()
            .find(|f| f.console_slug == "ps3" && f.emulator_slug == "rpcs3");

        let fw_report = latest_fw.map(|rec| FirmwarePackageReport {
            file_path: PathBuf::from(&rec.file_path),
            file_size_bytes: u64::try_from(rec.file_size_bytes).unwrap_or(0),
            sha256: rec.sha256,
            header_valid: rec.validation_status == "package_header_and_hash_validated",
            validation_status: rec.validation_status,
            notes: vec![rec.notes],
        });

        let readiness = adapter.evaluate_boot_readiness(&install, fw_report.as_ref());
        let diagnostic = if readiness.system_files_present && readiness.command_buildable {
            "RPCS3 detectado com arquivos de sistema presentes e comando seguro pronto; confirmação real em runtime do XMB reservada à Fase 3.".to_string()
        } else {
            "RPCS3 detectado, porém arquivos de sistema (dev_flash/vsh/module/vsh.self e dev_hdd0) ainda estão ausentes.".to_string()
        };

        Ok(Rpcs3InspectionDto {
            configured: true,
            installation: Some(install),
            readiness: Some(readiness),
            diagnostic,
        })
    }

    pub fn register_firmware_file(
        &self,
        console_slug: &str,
        emulator_slug: &str,
        file_path: &str,
    ) -> Result<FirmwareRecordDto, DomainError> {
        let trimmed = file_path.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "caminho do arquivo de firmware não pode estar vazio".to_string(),
            ));
        }

        let record = if let Some(adapter) = self.adapters.get(emulator_slug) {
            let report = adapter.validate_firmware_package(Path::new(trimmed))?;
            let notes = report.notes.join(" ");
            self.repos.upsert_firmware_record(
                console_slug,
                emulator_slug,
                "system_firmware_package",
                &report.file_path.to_string_lossy(),
                report.file_size_bytes,
                &report.sha256,
                &report.validation_status,
                &notes,
            )?
        } else {
            let valid = SecurityManager::validate_existing_file(Path::new(trimmed))?;
            let (sha256, size_bytes) = SecurityManager::compute_sha256_streaming(&valid)?;
            self.repos.upsert_firmware_record(
                console_slug,
                emulator_slug,
                "user_system_file",
                &valid.to_string_lossy(),
                size_bytes,
                &sha256,
                "file_hash_recorded_phase_1",
                "Hash SHA-256 calculado localmente sem armazenar nem distribuir o arquivo proprietário.",
            )?
        };
        let _ = self
            .console_profile_sync
            .sync_console_profile(console_slug, SyncTrigger::EmulatorConfigured);
        Ok(record)
    }

    pub fn update_active_profile(
        &self,
        req: UpdateProfileRequest,
    ) -> Result<PlayerProfile, DomainError> {
        self.repos.update_active_profile(
            &req.username,
            &req.display_name,
            &req.bio,
            &req.privacy_level,
        )
    }

    pub fn register_game(
        &self,
        req: RegisterGameRequest,
    ) -> Result<(GameRecord, GameVersionRecord), DomainError> {
        self.repos.register_game_with_version(
            &req.console_slug,
            &req.title,
            &req.version_label,
            &req.region,
            req.serial_code.as_deref(),
            &req.file_path,
            &req.metadata_source,
        )
    }

    pub fn toggle_game_favorite(&self, game_id: &str) -> Result<bool, DomainError> {
        let profile = self.repos.get_active_profile()?;
        self.repos.toggle_favorite(&profile.id, game_id)
    }

    pub fn update_setting(
        &self,
        key: &str,
        value: &str,
    ) -> Result<BTreeMap<String, String>, DomainError> {
        self.repos.upsert_setting(key, value)?;
        self.repos.get_all_settings()
    }

    pub fn create_database_backup(
        &self,
        file_name: Option<String>,
    ) -> Result<BackupReport, DomainError> {
        self.backups.create_backup(file_name)
    }
}
