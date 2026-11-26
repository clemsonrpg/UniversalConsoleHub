use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;
use crate::domain::session::{GameSessionRecord, SessionRecoveryReport};
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::SqliteRepositories;
use crate::infrastructure::db::session_stats_repo::SessionStatsRepository;

#[derive(Clone)]
pub struct SessionService {
    repos: SqliteRepositories,
    stats_repo: SessionStatsRepository,
}

impl SessionService {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            stats_repo: SessionStatsRepository::new(db),
        }
    }

    pub fn start_recorded_session(
        &self,
        game_id: &str,
        game_version_id: Option<&str>,
        emulator_id: &str,
        started_at_utc: Option<&str>,
    ) -> Result<GameSessionRecord, DomainError> {
        self.start_recorded_session_for_local_user(
            game_id,
            game_version_id,
            emulator_id,
            None,
            started_at_utc,
        )
    }

    pub fn start_recorded_session_for_local_user(
        &self,
        game_id: &str,
        game_version_id: Option<&str>,
        emulator_id: &str,
        console_local_user_id: Option<&str>,
        started_at_utc: Option<&str>,
    ) -> Result<GameSessionRecord, DomainError> {
        let profile = self.repos.get_active_profile()?;
        let ts = started_at_utc
            .map(str::to_string)
            .unwrap_or_else(now_utc_rfc3339);
        self.stats_repo.start_session_with_local_user(
            &profile.id,
            game_id,
            game_version_id,
            emulator_id,
            console_local_user_id,
            &ts,
        )
    }

    pub fn heartbeat_session(
        &self,
        session_id: &str,
        heartbeat_at_utc: Option<&str>,
    ) -> Result<GameSessionRecord, DomainError> {
        let ts = heartbeat_at_utc
            .map(str::to_string)
            .unwrap_or_else(now_utc_rfc3339);
        self.stats_repo.heartbeat_session(session_id, &ts)
    }

    pub fn finish_session(
        &self,
        session_id: &str,
        ended_at_utc: Option<&str>,
        exit_status: &str,
    ) -> Result<GameSessionRecord, DomainError> {
        let ts = ended_at_utc
            .map(str::to_string)
            .unwrap_or_else(now_utc_rfc3339);
        self.stats_repo
            .complete_session(session_id, &ts, exit_status)
    }

    /// Reconciles interrupted sessions using strictly `last_heartbeat_at` (idempotent).
    pub fn reconcile_orphaned_sessions(&self) -> Result<SessionRecoveryReport, DomainError> {
        self.stats_repo.reconcile_orphaned_sessions()
    }
}
