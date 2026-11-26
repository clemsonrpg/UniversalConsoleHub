use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::DomainError;

/// Mandatory explicit provenance for every field exposed on a console profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataOrigin {
    NativePlatform,
    HubRecorded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileField<T> {
    pub value: Option<T>,
    pub origin: DataOrigin,
    pub notes: Option<String>,
}

impl<T> ProfileField<T> {
    pub fn native(value: T) -> Self {
        Self {
            value: Some(value),
            origin: DataOrigin::NativePlatform,
            notes: None,
        }
    }

    pub fn native_with_note(value: T, notes: impl Into<String>) -> Self {
        Self {
            value: Some(value),
            origin: DataOrigin::NativePlatform,
            notes: Some(notes.into()),
        }
    }

    pub fn hub(value: T) -> Self {
        Self {
            value: Some(value),
            origin: DataOrigin::HubRecorded,
            notes: None,
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            value: None,
            origin: DataOrigin::Unavailable,
            notes: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    Available,
    MissingOnDisk,
}

impl AvailabilityStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::MissingOnDisk => "missing_on_disk",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "available" => Ok(Self::Available),
            "missing_on_disk" => Ok(Self::MissingOnDisk),
            other => Err(DomainError::Validation(format!(
                "invalid availability status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmulatorStatus {
    Unconfigured,
    Available,
    MissingOnDisk,
}

impl EmulatorStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unconfigured => "unconfigured",
            Self::Available => "available",
            Self::MissingOnDisk => "missing_on_disk",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "unconfigured" => Ok(Self::Unconfigured),
            "available" => Ok(Self::Available),
            "missing_on_disk" => Ok(Self::MissingOnDisk),
            other => Err(DomainError::Validation(format!(
                "invalid emulator status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrophySyncReadiness {
    Unavailable,
    Phase4Planned,
    Ready,
}

impl TrophySyncReadiness {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Phase4Planned => "phase4_planned",
            Self::Ready => "ready",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "unavailable" => Ok(Self::Unavailable),
            "phase4_planned" => Ok(Self::Phase4Planned),
            "ready" => Ok(Self::Ready),
            other => Err(DomainError::Validation(format!(
                "invalid trophy sync readiness: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncTrigger {
    Manual,
    DirectoryChanged,
    ScanCompleted,
    EmulatorConfigured,
    PreLaunch,
    SessionEnded,
}

impl SyncTrigger {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::DirectoryChanged => "directory_changed",
            Self::ScanCompleted => "scan_completed",
            Self::EmulatorConfigured => "emulator_configured",
            Self::PreLaunch => "pre_launch",
            Self::SessionEnded => "session_ended",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "manual" => Ok(Self::Manual),
            "directory_changed" => Ok(Self::DirectoryChanged),
            "scan_completed" => Ok(Self::ScanCompleted),
            "emulator_configured" => Ok(Self::EmulatorConfigured),
            "pre_launch" => Ok(Self::PreLaunch),
            "session_ended" => Ok(Self::SessionEnded),
            other => Err(DomainError::Validation(format!(
                "invalid sync trigger: {other}"
            ))),
        }
    }
}

/// Real local console user detected directly on disk (e.g. RPCS3 `dev_hdd0/home/<id>/localusername`).
/// Never fabricated when a platform does not expose local user identification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedLocalUser {
    pub native_user_key: String,
    pub display_name: String,
    pub source_path: PathBuf,
}

/// Read-only native artwork candidate discovered alongside a real game title (e.g. PS3 ICON0.PNG / PIC1.PNG).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeConsoleArtworkCandidate {
    pub game_file_path: PathBuf,
    pub icon_path: Option<PathBuf>,
    pub background_path: Option<PathBuf>,
}

/// Read-only snapshot collected by a platform-specific `ConsoleProfileProvider`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeConsoleProfileSnapshot {
    pub console_slug: String,
    pub emulator_slug: Option<String>,
    pub emulator_status: EmulatorStatus,
    pub detected_emulator_version: Option<String>,
    pub detected_firmware_version: Option<String>,
    pub emulator_root_dir: Option<PathBuf>,
    pub vsh_or_system_boot_ready: bool,
    pub local_users: Vec<DetectedLocalUser>,
    pub native_game_scan_dirs: Vec<PathBuf>,
    pub native_save_count: i64,
    pub native_trophy_dir_present: bool,
    pub trophy_sync_readiness: TrophySyncReadiness,
    pub native_metadata_json: String,
    pub diagnostics: Vec<String>,
}

/// Port implemented independently by each supported console platform.
pub trait ConsoleProfileProvider: Send + Sync {
    fn console_slug(&self) -> &'static str;

    fn collect_native_snapshot(
        &self,
        configured_exe: Option<&Path>,
    ) -> NativeConsoleProfileSnapshot;

    fn find_native_artwork_for_game(
        &self,
        game_file_path: &Path,
    ) -> Option<NativeConsoleArtworkCandidate>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleSyncReport {
    pub console_slug: String,
    pub trigger: SyncTrigger,
    pub sync_status: String,
    pub emulator_status: EmulatorStatus,
    pub local_users_detected: usize,
    pub local_users_marked_missing: usize,
    pub games_discovered: usize,
    pub versions_available: usize,
    pub versions_marked_missing: usize,
    pub covers_imported: usize,
    pub diagnostics: Vec<String>,
    pub synced_at: String,
}
