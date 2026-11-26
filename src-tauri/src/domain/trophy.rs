use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrophyOrigin {
    Official,
    Emulator,
    Local,
}

impl TrophyOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Emulator => "emulator",
            Self::Local => "local",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "official" => Ok(Self::Official),
            "emulator" => Ok(Self::Emulator),
            "local" => Ok(Self::Local),
            other => Err(DomainError::Validation(format!(
                "invalid trophy origin: {other}"
            ))),
        }
    }
}

pub fn ensure_immutable_origin(
    existing: TrophyOrigin,
    attempted: TrophyOrigin,
) -> Result<(), DomainError> {
    if existing != attempted {
        return Err(DomainError::ImmutableTrophyOrigin {
            existing: existing.as_str().to_string(),
            attempted: attempted.as_str().to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrophyCategory {
    Bronze,
    Silver,
    Gold,
    Platinum,
    XboxAchievement,
    SteamAchievement,
    EmulatorAchievement,
    HubLocal,
}

impl TrophyCategory {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Bronze => "bronze",
            Self::Silver => "silver",
            Self::Gold => "gold",
            Self::Platinum => "platinum",
            Self::XboxAchievement => "xbox_achievement",
            Self::SteamAchievement => "steam_achievement",
            Self::EmulatorAchievement => "emulator_achievement",
            Self::HubLocal => "hub_local",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "bronze" => Ok(Self::Bronze),
            "silver" => Ok(Self::Silver),
            "gold" => Ok(Self::Gold),
            "platinum" => Ok(Self::Platinum),
            "xbox_achievement" => Ok(Self::XboxAchievement),
            "steam_achievement" => Ok(Self::SteamAchievement),
            "emulator_achievement" => Ok(Self::EmulatorAchievement),
            "hub_local" => Ok(Self::HubLocal),
            other => Err(DomainError::Validation(format!(
                "invalid trophy category: {other}"
            ))),
        }
    }

    /// Maps Sony PS3 `TROPCONF.SFM` `ttype` attribute (`B`, `S`, `G`, `P`).
    pub fn from_ps3_ttype(raw: &str) -> Result<Self, DomainError> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "B" => Ok(Self::Bronze),
            "S" => Ok(Self::Silver),
            "G" => Ok(Self::Gold),
            "P" => Ok(Self::Platinum),
            other => Err(DomainError::Validation(format!(
                "unrecognized PS3 trophy ttype: '{other}'"
            ))),
        }
    }
}

pub fn compute_trophy_completion_percent(unlocked: i64, total: i64) -> i64 {
    if total <= 0 || unlocked <= 0 {
        return 0;
    }
    ((unlocked * 100) / total).clamp(0, 100)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementProviderRecord {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub origin: TrophyOrigin,
    pub availability_status: String,
    pub supports_auth: bool,
    pub supports_sync: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophyRecord {
    pub id: String,
    pub game_id: String,
    pub console_id: String,
    pub provider_id: String,
    pub external_trophy_id: String,
    pub name: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub category: TrophyCategory,
    pub rarity_percent: Option<i64>,
    pub origin: TrophyOrigin,
    pub is_unlocked: bool,
    pub unlocked_at: Option<String>,
}

pub trait AchievementProvider: Send + Sync {
    fn slug(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn origin(&self) -> TrophyOrigin;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTrophyDef {
    pub external_trophy_id: String,
    pub name: String,
    pub description: String,
    pub category: TrophyCategory,
    pub is_hidden: bool,
    pub icon_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTrophyUnlock {
    pub external_trophy_id: String,
    pub unlocked_at_utc: String,
    pub native_user_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTrophySet {
    pub console_slug: String,
    pub native_user_key: String,
    pub external_set_id: String,
    pub title: String,
    pub detail: String,
    pub icon_path: Option<PathBuf>,
    pub trophies: Vec<ParsedTrophyDef>,
    pub unlocks: Vec<ParsedTrophyUnlock>,
}

impl ParsedTrophySet {
    pub fn supports_platinum(&self) -> bool {
        self.trophies
            .iter()
            .any(|t| t.category == TrophyCategory::Platinum)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrophyReadBatch {
    pub sets: Vec<ParsedTrophySet>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewlyUnlockedTrophy {
    pub trophy_id: String,
    pub external_trophy_id: String,
    pub game_id: String,
    pub game_title: String,
    pub name: String,
    pub category: TrophyCategory,
    pub unlocked_at: String,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophySyncReport {
    pub console_slug: String,
    pub sets_synced: usize,
    pub trophies_synced: usize,
    pub unlocks_recorded: usize,
    pub newly_unlocked: Vec<NewlyUnlockedTrophy>,
    pub icons_cached: usize,
    pub diagnostics: Vec<String>,
    pub synced_at: String,
}

/// Read-only port implemented by platform-specific local trophy readers (starting with RPCS3 PS3).
pub trait TrophyReaderPort: Send + Sync {
    fn console_slug(&self) -> &'static str;
    fn read_trophy_sets(&self, emulator_root_dir: &Path) -> TrophyReadBatch;
}
