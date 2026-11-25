use crate::domain::error::DomainError;
use crate::domain::session::StatisticsOverview;
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::SqliteRepositories;
use crate::infrastructure::db::session_stats_repo::SessionStatsRepository;

use super::services::ActivityEntryDto;

#[derive(Clone)]
pub struct StatisticsService {
    repos: SqliteRepositories,
    stats_repo: SessionStatsRepository,
}

impl StatisticsService {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            stats_repo: SessionStatsRepository::new(db),
        }
    }

    pub fn get_statistics_overview(&self) -> Result<StatisticsOverview, DomainError> {
        let profile = self.repos.get_active_profile()?;
        self.stats_repo.get_statistics_overview(&profile.id)
    }

    pub fn list_activity_history(
        &self,
        event_type_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ActivityEntryDto>, DomainError> {
        let profile = self.repos.get_active_profile()?;
        let rows = self
            .stats_repo
            .list_activity_history(&profile.id, event_type_filter, limit)?;
        Ok(rows
            .into_iter()
            .map(|r| ActivityEntryDto {
                id: r.id,
                event_type: r.event_type,
                entity_type: r.entity_type,
                entity_id: r.entity_id,
                summary: r.summary,
                occurred_at: r.occurred_at,
            })
            .collect())
    }
}
