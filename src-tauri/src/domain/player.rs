use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use super::error::DomainError;

pub fn now_utc_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn validate_utc_timestamp(value: &str) -> Result<(), DomainError> {
    if !value.ends_with('Z') {
        return Err(DomainError::Validation(format!(
            "timestamp must be UTC ending with 'Z': {value}"
        )));
    }
    DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|err| {
            DomainError::Validation(format!("invalid RFC3339 UTC timestamp '{value}': {err}"))
        })
}

pub fn normalize_canonical_key(
    title: &str,
    serial_code: Option<&str>,
) -> Result<String, DomainError> {
    if let Some(raw_serial) = serial_code {
        let cleaned: String = raw_serial
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        if !cleaned.is_empty() {
            return Ok(format!("serial:{cleaned}"));
        }
    }

    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DomainError::Validation(
            "game title cannot be empty when computing canonical key".to_string(),
        ));
    }

    let mut slug = String::with_capacity(trimmed.len());
    let mut last_dash = false;
    for ch in trimmed.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                slug.push(lower);
            }
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        return Err(DomainError::Validation(format!(
            "could not derive canonical key from title: '{title}'"
        )));
    }
    Ok(format!("title:{slug}"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub handle: String,
    pub display_name: String,
    pub avatar_path: Option<String>,
    pub banner_path: Option<String>,
    pub bio: String,
    pub privacy_level: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRecord {
    pub id: String,
    pub console_id: String,
    pub console_slug: String,
    pub canonical_key: String,
    pub title: String,
    pub description: String,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_year: Option<i64>,
    pub genre: Option<String>,
    pub cover_path: Option<String>,
    pub metadata_source: String,
    pub install_state: String,
    pub associated_emulator_id: Option<String>,
    pub total_playtime_seconds: i64,
    pub last_played_at: Option<String>,
    pub completion_percent: i64,
    pub is_favorite: bool,
    pub version_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersionRecord {
    pub id: String,
    pub game_id: String,
    pub version_label: String,
    pub region: String,
    pub serial_code: Option<String>,
    pub file_path: String,
    pub install_state: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatistics {
    pub games_registered: i64,
    pub games_started: i64,
    pub games_completed: i64,
    pub total_playtime_seconds: i64,
    pub total_trophies: i64,
    pub unlocked_trophies: i64,
    pub rare_trophies: i64,
}
