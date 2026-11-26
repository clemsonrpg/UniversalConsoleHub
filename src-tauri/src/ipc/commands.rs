use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, State};

use crate::application::backup_service::BackupFileEntry;
use crate::application::console_profile_sync_service::ConsoleProfileOverviewDto;
use crate::application::environment_service::ConsoleEnvironmentDto;
use crate::application::services::{
    ActivityEntryDto, AppServices, ConsoleDetailsDto, HubBootstrapDto, RegisterGameRequest,
    Rpcs3InspectionDto, UpdateProfileRequest,
};
use crate::application::trophy_sync_service::TrophyCenterOverviewDto;
use crate::domain::console::ConsoleRecord;
use crate::domain::console_profile::{ConsoleSyncReport, SyncTrigger};
use crate::domain::emulator::EmulatorRecord;
use crate::domain::environment::{
    ConsoleScanPath, ConsoleSetting, RuntimeVerificationFeature, RuntimeVerificationRecord,
    RuntimeVerificationStatus,
};
use crate::domain::error::DomainError;
use crate::domain::player::{GameRecord, PlayerProfile};
use crate::domain::scanner::{ScanProgressEvent, ScanSummaryDto};
use crate::domain::session::{ActiveSessionSnapshot, StatisticsOverview};
use crate::domain::theme::InstalledThemePackage;
use crate::domain::trophy::{AchievementProviderRecord, TrophyRecord, TrophySyncReport};
use crate::infrastructure::db::console_env_repo::{GameDetailsDto, UpdateGameMetadataInput};
use crate::infrastructure::db::manager::{BackupInspectionReport, BackupReport, LiveRestoreReport};
use crate::infrastructure::db::repositories::FirmwareRecordDto;
use crate::infrastructure::db::trophy_repo::DetailedTrophyRow;
use crate::infrastructure::system::windows_gamepad::ConnectedGamepadInfo;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubHealthDto {
    pub app_name: String,
    pub version: String,
    pub phase: u8,
}

#[tauri::command]
pub fn ping_hub() -> HubHealthDto {
    HubHealthDto {
        app_name: "Universal Console Hub".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        phase: 1,
    }
}

#[tauri::command]
pub fn get_hub_bootstrap(services: State<'_, AppServices>) -> Result<HubBootstrapDto, DomainError> {
    services.get_bootstrap()
}

#[tauri::command]
pub fn list_consoles(services: State<'_, AppServices>) -> Result<Vec<ConsoleRecord>, DomainError> {
    services.repos.list_consoles()
}

#[tauri::command]
pub fn get_console_details(
    slug: String,
    services: State<'_, AppServices>,
) -> Result<ConsoleDetailsDto, DomainError> {
    services.get_console_details(&slug)
}

#[tauri::command]
pub fn list_emulators(
    services: State<'_, AppServices>,
) -> Result<Vec<EmulatorRecord>, DomainError> {
    services.repos.list_emulators()
}

#[tauri::command]
pub fn configure_emulator_path(
    slug: String,
    exe_path: String,
    services: State<'_, AppServices>,
) -> Result<EmulatorRecord, DomainError> {
    services.configure_emulator_path(&slug, &exe_path)
}

#[tauri::command]
pub fn inspect_rpcs3(services: State<'_, AppServices>) -> Result<Rpcs3InspectionDto, DomainError> {
    services.inspect_rpcs3()
}

#[tauri::command]
pub fn register_firmware_file(
    console_slug: String,
    emulator_slug: String,
    file_path: String,
    services: State<'_, AppServices>,
) -> Result<FirmwareRecordDto, DomainError> {
    services.register_firmware_file(&console_slug, &emulator_slug, &file_path)
}

#[tauri::command]
pub fn get_active_profile(services: State<'_, AppServices>) -> Result<PlayerProfile, DomainError> {
    services.repos.get_active_profile()
}

#[tauri::command]
pub fn update_active_profile(
    request: UpdateProfileRequest,
    services: State<'_, AppServices>,
) -> Result<PlayerProfile, DomainError> {
    services.update_active_profile(request)
}

#[tauri::command]
pub fn list_games(services: State<'_, AppServices>) -> Result<Vec<GameRecord>, DomainError> {
    services.list_games_sanitized()
}

#[tauri::command]
pub fn register_game(
    request: RegisterGameRequest,
    services: State<'_, AppServices>,
) -> Result<GameRecord, DomainError> {
    let (game, _) = services.register_game(request)?;
    Ok(game)
}

#[tauri::command]
pub fn toggle_game_favorite(
    game_id: String,
    services: State<'_, AppServices>,
) -> Result<bool, DomainError> {
    services.toggle_game_favorite(&game_id)
}

#[tauri::command]
pub fn list_achievement_providers(
    services: State<'_, AppServices>,
) -> Result<Vec<AchievementProviderRecord>, DomainError> {
    services.repos.list_achievement_providers()
}

#[tauri::command]
pub fn list_trophies(services: State<'_, AppServices>) -> Result<Vec<TrophyRecord>, DomainError> {
    let profile = services.repos.get_active_profile()?;
    services.repos.list_trophies(&profile.id)
}

#[tauri::command]
pub fn get_app_settings(
    services: State<'_, AppServices>,
) -> Result<BTreeMap<String, String>, DomainError> {
    services.repos.get_all_settings()
}

#[tauri::command]
pub fn update_app_setting(
    key: String,
    value: String,
    services: State<'_, AppServices>,
) -> Result<BTreeMap<String, String>, DomainError> {
    services.update_setting(&key, &value)
}

#[tauri::command]
pub fn create_database_backup(
    file_name: Option<String>,
    services: State<'_, AppServices>,
) -> Result<BackupReport, DomainError> {
    services.create_database_backup(file_name)
}

// ===== Phase 2 Commands (13 new commands, total = 31) =====

#[tauri::command]
pub fn get_console_environment(
    slug: String,
    services: State<'_, AppServices>,
) -> Result<ConsoleEnvironmentDto, DomainError> {
    services.get_console_environment(&slug)
}

#[tauri::command]
pub fn add_console_scan_path(
    console_slug: String,
    directory_path: String,
    recursive: bool,
    app: AppHandle,
    services: State<'_, AppServices>,
) -> Result<ConsoleScanPath, DomainError> {
    let added =
        services
            .environment_service()
            .add_scan_path(&console_slug, &directory_path, recursive)?;
    if let Ok(report) = services
        .console_profile_sync_service()
        .sync_console_profile(&console_slug, SyncTrigger::DirectoryChanged)
    {
        let _ = app.emit("profile:synced", &report);
    }
    Ok(added)
}

#[tauri::command]
pub fn remove_console_scan_path(
    path_id: String,
    services: State<'_, AppServices>,
) -> Result<bool, DomainError> {
    services.environment_service().remove_scan_path(&path_id)
}

#[tauri::command]
pub async fn scan_console_library(
    console_slug: String,
    on_progress: Channel<ScanProgressEvent>,
    app: AppHandle,
    services: State<'_, AppServices>,
) -> Result<ScanSummaryDto, DomainError> {
    let scanner = services.scanner_service().clone();
    let sync = services.console_profile_sync_service().clone();
    let slug_for_sync = console_slug.clone();
    let (summary, sync_report) = tokio::task::spawn_blocking(move || {
        let summary = scanner.scan_console(&console_slug, |evt| {
            let _ = on_progress.send(evt);
        })?;
        let report = sync
            .sync_console_profile(&slug_for_sync, SyncTrigger::ScanCompleted)
            .ok();
        Ok::<_, DomainError>((summary, report))
    })
    .await
    .map_err(|e| DomainError::Io(format!("scanner task failed to join: {e}")))??;

    if let Some(report) = sync_report {
        let _ = app.emit("profile:synced", &report);
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_console_setting(
    console_slug: String,
    key: String,
    value: String,
    services: State<'_, AppServices>,
) -> Result<ConsoleSetting, DomainError> {
    services
        .environment_service()
        .update_console_setting(&console_slug, &key, &value)
}

#[tauri::command]
pub fn get_game_details(
    game_id: String,
    services: State<'_, AppServices>,
) -> Result<GameDetailsDto, DomainError> {
    services.environment_service().get_game_details(&game_id)
}

#[tauri::command]
pub fn update_game_metadata(
    request: UpdateGameMetadataInput,
    services: State<'_, AppServices>,
) -> Result<GameRecord, DomainError> {
    services.environment_service().update_game_metadata(request)
}

#[tauri::command]
pub fn set_game_cover(
    game_id: String,
    source_image_path: String,
    services: State<'_, AppServices>,
) -> Result<GameRecord, DomainError> {
    services
        .environment_service()
        .set_game_cover(&game_id, &source_image_path)
}

#[tauri::command]
pub fn get_statistics_overview(
    services: State<'_, AppServices>,
) -> Result<StatisticsOverview, DomainError> {
    services.statistics_service().get_statistics_overview()
}

#[tauri::command]
pub fn list_activity_history(
    event_type_filter: Option<String>,
    limit: Option<i64>,
    services: State<'_, AppServices>,
) -> Result<Vec<ActivityEntryDto>, DomainError> {
    services
        .statistics_service()
        .list_activity_history(event_type_filter.as_deref(), limit.unwrap_or(100))
}

#[tauri::command]
pub fn list_database_backups(
    services: State<'_, AppServices>,
) -> Result<Vec<BackupFileEntry>, DomainError> {
    services.backup_service().list_backups()
}

#[tauri::command]
pub fn inspect_database_backup(
    backup_path: String,
    services: State<'_, AppServices>,
) -> Result<BackupInspectionReport, DomainError> {
    services
        .backup_service()
        .inspect_backup(Path::new(&backup_path))
}

#[tauri::command]
pub fn restore_database_backup(
    backup_path: String,
    services: State<'_, AppServices>,
) -> Result<LiveRestoreReport, DomainError> {
    services
        .backup_service()
        .restore_backup(Path::new(&backup_path))
}

// ===== Phase 3 Commands (7 new commands, total = 38) =====

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordRuntimeVerificationRequest {
    pub console_slug: String,
    pub emulator_slug: String,
    pub game_id: Option<String>,
    pub feature_key: RuntimeVerificationFeature,
    pub emulator_version: String,
    pub status: RuntimeVerificationStatus,
    pub evidence_notes: String,
}

#[tauri::command]
pub fn get_console_profile(
    console_slug: String,
    services: State<'_, AppServices>,
) -> Result<ConsoleProfileOverviewDto, DomainError> {
    services
        .console_profile_sync_service()
        .get_console_profile_overview(&console_slug)
}

#[tauri::command]
pub async fn sync_console_profile(
    console_slug: String,
    trigger: Option<String>,
    app: AppHandle,
    services: State<'_, AppServices>,
) -> Result<ConsoleSyncReport, DomainError> {
    let parsed_trigger = match trigger.as_deref() {
        Some(raw) => SyncTrigger::parse(raw)?,
        None => SyncTrigger::Manual,
    };
    let sync = services.console_profile_sync_service().clone();
    let report = tokio::task::spawn_blocking(move || {
        sync.sync_console_profile(&console_slug, parsed_trigger)
    })
    .await
    .map_err(|e| DomainError::Io(format!("profile sync task failed to join: {e}")))??;
    let _ = app.emit("profile:synced", &report);
    Ok(report)
}

#[tauri::command]
pub async fn launch_console_system(
    console_slug: String,
    services: State<'_, AppServices>,
) -> Result<ActiveSessionSnapshot, DomainError> {
    services
        .live_session_orchestrator()
        .launch_console_system(&console_slug)
        .await
}

#[tauri::command]
pub async fn launch_game(
    game_id: String,
    game_version_id: Option<String>,
    services: State<'_, AppServices>,
) -> Result<ActiveSessionSnapshot, DomainError> {
    services
        .live_session_orchestrator()
        .launch_game(&game_id, game_version_id.as_deref())
        .await
}

#[tauri::command]
pub fn get_active_session(services: State<'_, AppServices>) -> ActiveSessionSnapshot {
    services.live_session_orchestrator().get_active_session()
}

#[tauri::command]
pub fn stop_active_session(services: State<'_, AppServices>) -> Result<bool, DomainError> {
    services.live_session_orchestrator().stop_active_session()
}

#[tauri::command]
pub fn record_runtime_verification(
    request: RecordRuntimeVerificationRequest,
    services: State<'_, AppServices>,
) -> Result<RuntimeVerificationRecord, DomainError> {
    services
        .live_session_orchestrator()
        .record_runtime_verification(
            &request.console_slug,
            &request.emulator_slug,
            request.game_id.as_deref(),
            request.feature_key,
            &request.emulator_version,
            request.status,
            &request.evidence_notes,
        )
}

// ===== Phase 4 Commands (3 new — total 41) =====

#[tauri::command]
pub async fn sync_console_trophies(
    console_slug: String,
    app_handle: AppHandle,
    services: State<'_, AppServices>,
) -> Result<TrophySyncReport, DomainError> {
    let trophy_sync = services.trophy_sync_service().clone();
    let report =
        tokio::task::spawn_blocking(move || trophy_sync.sync_console_trophies(&console_slug))
            .await
            .map_err(|e| DomainError::Process(format!("trophy sync task failed: {e}")))??;
    let _ = app_handle.emit("trophies:synced", &report);
    if !report.newly_unlocked.is_empty() {
        let _ = app_handle.emit("trophies:unlocked", &report.newly_unlocked);
    }
    Ok(report)
}

#[tauri::command]
pub fn get_trophy_center_overview(
    console_slug: Option<String>,
    services: State<'_, AppServices>,
) -> Result<TrophyCenterOverviewDto, DomainError> {
    services
        .trophy_sync_service()
        .get_trophy_center_overview(console_slug.as_deref())
}

#[tauri::command]
pub fn list_game_trophies(
    game_id: String,
    services: State<'_, AppServices>,
) -> Result<Vec<DetailedTrophyRow>, DomainError> {
    services.trophy_sync_service().list_game_trophies(&game_id)
}

// ===== Visual Redesign: Safe Theme Packages & Windows Gamepad IPC (4 new — total 45) =====

#[tauri::command]
pub fn list_installed_themes(
    services: State<'_, AppServices>,
) -> Result<Vec<InstalledThemePackage>, DomainError> {
    services.theme_service().list_installed_themes()
}

#[tauri::command]
pub fn import_theme_package(
    source_path: String,
    services: State<'_, AppServices>,
) -> Result<InstalledThemePackage, DomainError> {
    services
        .theme_service()
        .import_theme_package(Path::new(source_path.trim()))
}

#[tauri::command]
pub fn remove_installed_theme(
    theme_id: String,
    services: State<'_, AppServices>,
) -> Result<bool, DomainError> {
    services.theme_service().remove_installed_theme(&theme_id)
}

#[tauri::command]
pub fn list_connected_gamepads(services: State<'_, AppServices>) -> Vec<ConnectedGamepadInfo> {
    services.gamepad_monitor().list_connected_gamepads()
}
