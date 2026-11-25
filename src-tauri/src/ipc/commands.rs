use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;

use crate::application::services::{
    AppServices, ConsoleDetailsDto, HubBootstrapDto, RegisterGameRequest, Rpcs3InspectionDto,
    UpdateProfileRequest,
};
use crate::domain::console::ConsoleRecord;
use crate::domain::emulator::EmulatorRecord;
use crate::domain::error::DomainError;
use crate::domain::player::{GameRecord, PlayerProfile};
use crate::domain::trophy::{AchievementProviderRecord, TrophyRecord};
use crate::infrastructure::db::manager::BackupReport;
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
    let profile = services.repos.get_active_profile()?;
    services.repos.list_games(&profile.id)
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
