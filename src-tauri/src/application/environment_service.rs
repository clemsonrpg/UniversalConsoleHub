use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::console::ConsoleRecord;
use crate::domain::emulator::EmulatorRecord;
use crate::domain::environment::{
    ConsoleEnvironmentMode, ConsoleScanPath, ConsoleSetting, RuntimeVerificationRecord,
};
use crate::domain::error::DomainError;
use crate::domain::player::GameRecord;
use crate::infrastructure::db::console_env_repo::{
    ConsoleEnvRepository, GameDetailsDto, GameDetailsRepository, UpdateGameMetadataInput,
};
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::{FirmwareRecordDto, SqliteRepositories};
use crate::infrastructure::system::cover_cache::CoverCache;
use crate::infrastructure::system::security::SecurityManager;

use super::services::Rpcs3InspectionDto;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleEnvironmentDto {
    pub console: ConsoleRecord,
    pub emulator: Option<EmulatorRecord>,
    pub environment_mode: ConsoleEnvironmentMode,
    pub games: Vec<GameRecord>,
    pub scan_paths: Vec<ConsoleScanPath>,
    pub settings: Vec<ConsoleSetting>,
    pub firmware_records: Vec<FirmwareRecordDto>,
    pub runtime_verifications: Vec<RuntimeVerificationRecord>,
    pub rpcs3_inspection: Option<Rpcs3InspectionDto>,
}

#[derive(Clone)]
pub struct EnvironmentService {
    repos: SqliteRepositories,
    env_repo: ConsoleEnvRepository,
    details_repo: GameDetailsRepository,
    cover_cache_dir: PathBuf,
}

impl EnvironmentService {
    pub fn new(db: DatabaseManager, cover_cache_dir: PathBuf) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            env_repo: ConsoleEnvRepository::new(db.clone()),
            details_repo: GameDetailsRepository::new(db),
            cover_cache_dir,
        }
    }

    pub fn get_console_environment(
        &self,
        slug: &str,
        rpcs3_inspection: Option<Rpcs3InspectionDto>,
    ) -> Result<ConsoleEnvironmentDto, DomainError> {
        let console = self.repos.get_console_by_slug(slug)?;
        let environment_mode = ConsoleEnvironmentMode::from_capabilities(&console.capabilities);
        let emulator = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.console_slug == slug);
        let profile = self.repos.get_active_profile()?;
        let games =
            self.env_repo
                .list_games_for_console(&profile.id, slug, Some(&self.cover_cache_dir))?;
        let scan_paths = self.env_repo.list_scan_paths(slug)?;
        let settings = self.env_repo.list_console_settings(slug)?;
        let firmware_records = self
            .repos
            .list_firmware_records()?
            .into_iter()
            .filter(|f| f.console_slug == slug)
            .collect();
        let runtime_verifications = self.env_repo.list_runtime_verifications(slug)?;

        Ok(ConsoleEnvironmentDto {
            console,
            emulator,
            environment_mode,
            games,
            scan_paths,
            settings,
            firmware_records,
            runtime_verifications,
            rpcs3_inspection,
        })
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
                "caminho da pasta de jogos não pode estar vazio".to_string(),
            ));
        }
        let valid_dir = SecurityManager::validate_existing_directory(Path::new(trimmed))?;
        self.env_repo
            .add_scan_path(console_slug, &valid_dir.to_string_lossy(), recursive)
    }

    pub fn remove_scan_path(&self, path_id: &str) -> Result<bool, DomainError> {
        self.env_repo.remove_scan_path(path_id)
    }

    pub fn update_console_setting(
        &self,
        console_slug: &str,
        key: &str,
        value: &str,
    ) -> Result<ConsoleSetting, DomainError> {
        self.env_repo
            .upsert_console_setting(console_slug, key, value)
    }

    pub fn get_game_details(&self, game_id: &str) -> Result<GameDetailsDto, DomainError> {
        let profile = self.repos.get_active_profile()?;
        self.details_repo
            .get_game_details(&profile.id, game_id, Some(&self.cover_cache_dir))
    }

    pub fn update_game_metadata(
        &self,
        input: UpdateGameMetadataInput,
    ) -> Result<GameRecord, DomainError> {
        let profile = self.repos.get_active_profile()?;
        let mut updated = self
            .details_repo
            .update_game_metadata(&profile.id, &input)?;
        updated.cover_path = CoverCache::sanitize_cover_for_frontend(
            updated.cover_path.as_deref(),
            &self.cover_cache_dir,
        );
        Ok(updated)
    }

    pub fn set_game_cover(
        &self,
        game_id: &str,
        source_image_path: &str,
    ) -> Result<GameRecord, DomainError> {
        let trimmed = source_image_path.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "caminho da imagem de capa não pode estar vazio".to_string(),
            ));
        }
        let cached = CoverCache::import_local_image(Path::new(trimmed), &self.cover_cache_dir)?;
        let _ = self
            .details_repo
            .set_game_cover(game_id, &cached, &self.cover_cache_dir)?;
        let profile = self.repos.get_active_profile()?;
        self.details_repo
            .get_game_record(&profile.id, game_id, Some(&self.cover_cache_dir))
    }
}
