use serde::{Deserialize, Serialize};

use super::console::{CapabilityState, ConsoleCapabilities};
use super::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleEnvironmentMode {
    AuthenticSystemCapable,
    IntegratedAlternativeOnly,
}

impl ConsoleEnvironmentMode {
    pub fn from_capabilities(capabilities: &ConsoleCapabilities) -> Self {
        if capabilities.has_state(CapabilityState::FullSystemBoot)
            && capabilities.has_state(CapabilityState::OriginalDashboard)
        {
            Self::AuthenticSystemCapable
        } else {
            Self::IntegratedAlternativeOnly
        }
    }
}

/// Seven independent runtime verification stages (Adjustment E).
/// Validating one stage NEVER automatically validates another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeVerificationFeature {
    EmulatorLaunch,
    SystemBoot,
    OriginalDashboard,
    GameBoot,
    PerGameCompatibility,
    SessionMonitoring,
    HubReturn,
}

impl RuntimeVerificationFeature {
    pub const fn all() -> [Self; 7] {
        [
            Self::EmulatorLaunch,
            Self::SystemBoot,
            Self::OriginalDashboard,
            Self::GameBoot,
            Self::PerGameCompatibility,
            Self::SessionMonitoring,
            Self::HubReturn,
        ]
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EmulatorLaunch => "emulator_launch",
            Self::SystemBoot => "system_boot",
            Self::OriginalDashboard => "original_dashboard",
            Self::GameBoot => "game_boot",
            Self::PerGameCompatibility => "per_game_compatibility",
            Self::SessionMonitoring => "session_monitoring",
            Self::HubReturn => "hub_return",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "emulator_launch" => Ok(Self::EmulatorLaunch),
            "system_boot" => Ok(Self::SystemBoot),
            "original_dashboard" => Ok(Self::OriginalDashboard),
            "game_boot" => Ok(Self::GameBoot),
            "per_game_compatibility" => Ok(Self::PerGameCompatibility),
            "session_monitoring" => Ok(Self::SessionMonitoring),
            "hub_return" => Ok(Self::HubReturn),
            other => Err(DomainError::Validation(format!(
                "invalid runtime verification feature_key: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeVerificationStatus {
    Unverified,
    DocumentedUpstream,
    FilesVerified,
    RuntimeConfirmed,
    Failed,
}

impl RuntimeVerificationStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unverified => "unverified",
            Self::DocumentedUpstream => "documented_upstream",
            Self::FilesVerified => "files_verified",
            Self::RuntimeConfirmed => "runtime_confirmed",
            Self::Failed => "failed",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "unverified" => Ok(Self::Unverified),
            "documented_upstream" => Ok(Self::DocumentedUpstream),
            "files_verified" => Ok(Self::FilesVerified),
            "runtime_confirmed" => Ok(Self::RuntimeConfirmed),
            "failed" => Ok(Self::Failed),
            other => Err(DomainError::Validation(format!(
                "invalid runtime verification status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVerificationRecord {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub emulator_id: String,
    pub emulator_slug: String,
    pub game_id: Option<String>,
    pub scope_key: String,
    pub feature_key: RuntimeVerificationFeature,
    pub emulator_version: String,
    pub status: RuntimeVerificationStatus,
    pub evidence_notes: String,
    pub verified_at: String,
}

impl RuntimeVerificationRecord {
    pub fn compute_scope_key(game_id: Option<&str>) -> Result<String, DomainError> {
        match game_id {
            None => Ok("console".to_string()),
            Some(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Err(DomainError::Validation(
                        "game_id cannot be empty when scoping a runtime verification".to_string(),
                    ));
                }
                Ok(trimmed.to_string())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleScanPath {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub directory_path: String,
    pub recursive: bool,
    pub enabled: bool,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleSetting {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub key: String,
    pub value: String,
    pub updated_at: String,
}
