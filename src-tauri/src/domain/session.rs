use chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::error::DomainError;
use super::player::validate_utc_timestamp;

pub const EXIT_STATUS_RUNNING: &str = "running";
pub const EXIT_STATUS_CLEAN: &str = "clean_exit";
pub const EXIT_STATUS_NON_ZERO: &str = "non_zero_exit";
pub const EXIT_STATUS_USER_STOPPED: &str = "user_stopped";
pub const EXIT_STATUS_RECOVERED_AFTER_CRASH: &str = "recovered_after_crash";
pub const EXIT_STATUS_LAUNCH_FAILED: &str = "launch_failed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveSessionPhase {
    Idle,
    Launching,
    Running,
    Exiting,
}

impl ActiveSessionPhase {
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Idle, Self::Launching)
                | (Self::Launching, Self::Running)
                | (Self::Launching, Self::Idle)
                | (Self::Running, Self::Exiting)
                | (Self::Exiting, Self::Idle)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum LaunchTarget {
    #[serde(rename_all = "camelCase")]
    SystemBoot { console_slug: String },
    #[serde(rename_all = "camelCase")]
    DirectGameBoot {
        console_slug: String,
        game_id: String,
        game_version_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSessionSnapshot {
    pub phase: ActiveSessionPhase,
    pub session_id: Option<String>,
    pub target: Option<LaunchTarget>,
    pub console_slug: Option<String>,
    pub game_id: Option<String>,
    pub game_title: Option<String>,
    pub emulator_slug: Option<String>,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    pub last_heartbeat_at: Option<String>,
    pub elapsed_seconds: i64,
}

impl ActiveSessionSnapshot {
    pub fn idle() -> Self {
        Self {
            phase: ActiveSessionPhase::Idle,
            session_id: None,
            target: None,
            console_slug: None,
            game_id: None,
            game_title: None,
            emulator_slug: None,
            pid: None,
            started_at: None,
            last_heartbeat_at: None,
            elapsed_seconds: 0,
        }
    }
}

pub fn compute_elapsed_seconds_utc(
    started_at: &str,
    reference_at: &str,
) -> Result<i64, DomainError> {
    validate_utc_timestamp(started_at)?;
    validate_utc_timestamp(reference_at)?;

    let start = DateTime::parse_from_rfc3339(started_at)
        .map_err(|e| DomainError::Validation(format!("invalid started_at '{started_at}': {e}")))?;
    let end = DateTime::parse_from_rfc3339(reference_at).map_err(|e| {
        DomainError::Validation(format!("invalid reference_at '{reference_at}': {e}"))
    })?;

    Ok((end - start).num_seconds().max(0))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSessionRecord {
    pub id: String,
    pub profile_id: String,
    pub game_id: String,
    pub game_title: String,
    pub console_slug: String,
    pub game_version_id: Option<String>,
    pub emulator_id: String,
    pub started_at: String,
    pub last_heartbeat_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: i64,
    pub exit_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecoveryReport {
    pub recovered_count: usize,
    pub total_recovered_seconds: i64,
    pub recovered_session_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolePlaytimeStat {
    pub console_slug: String,
    pub console_name: String,
    pub games_count: i64,
    pub sessions_count: i64,
    pub total_playtime_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopGameStat {
    pub game_id: String,
    pub title: String,
    pub console_slug: String,
    pub sessions_count: i64,
    pub total_playtime_seconds: i64,
    pub completion_percent: i64,
    pub last_played_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyPlaytimeStat {
    pub month: String,
    pub sessions_count: i64,
    pub total_playtime_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsOverview {
    pub total_playtime_seconds: i64,
    pub total_sessions: i64,
    pub games_registered: i64,
    pub games_started: i64,
    pub games_completed: i64,
    pub completion_rate_percent: i64,
    pub by_console: Vec<ConsolePlaytimeStat>,
    pub top_games: Vec<TopGameStat>,
    pub monthly_playtime: Vec<MonthlyPlaytimeStat>,
}
