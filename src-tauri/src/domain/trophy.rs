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
