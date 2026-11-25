use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::State;

use crate::application::backup_service::BackupFileEntry;
use crate::application::environment_service::ConsoleEnvironmentDto;
use crate::application::services::{
    ActivityEntryDto, AppServices, ConsoleDetailsDto, HubBootstrapDto, RegisterGameRequest,
    Rpcs3InspectionDto, UpdateProfileRequest,
};
use crate::domain::console::ConsoleRecord;
use crate::domain::emulator::EmulatorRecord;
use crate::domain::environment::{ConsoleScanPath, ConsoleSetting};
use crate::domain::error::DomainError;
use crate::domain::player::{GameRecord, PlayerProfile};
use crate::domain::scanner::{ScanProgressEvent, ScanSummaryDto};
use crate::domain::session::StatisticsOverview;
use crate::domain::trophy::{AchievementProviderRecord, TrophyRecord};
use crate::infrastructure::db::console_env_repo::{GameDetailsDto, UpdateGameMetadataInput};
use crate::infrastructure::db::manager::{BackupInspectionReport, BackupReport, LiveRestoreReport};
use crate::infrastructure::db::repositories::FirmwareRecordDto;

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
    services: State<'_, AppServices>,
) -> Result<ConsoleScanPath, DomainError> {
    services
        .environment_service()
        .add_scan_path(&console_slug, &directory_path, recursive)
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
    services: State<'_, AppServices>,
) -> Result<ScanSummaryDto, DomainError> {
    let scanner = services.scanner_service().clone();
    tokio::task::spawn_blocking(move || {
        scanner.scan_console(&console_slug, |evt| {
            let _ = on_progress.send(evt);
        })
    })
    .await
    .map_err(|e| DomainError::Io(format!("scanner task failed to join: {e}")))?
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
