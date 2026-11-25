use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::player::{now_utc_rfc3339, validate_utc_timestamp};
use crate::domain::session::{
    compute_elapsed_seconds_utc, ConsolePlaytimeStat, GameSessionRecord, MonthlyPlaytimeStat,
    SessionRecoveryReport, StatisticsOverview, TopGameStat, EXIT_STATUS_RECOVERED_AFTER_CRASH,
    EXIT_STATUS_RUNNING,
};

use super::manager::DatabaseManager;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityHistoryRow {
    pub id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub occurred_at: String,
}

#[derive(Clone)]
pub struct SessionStatsRepository {
    db: DatabaseManager,
}

fn upsert_statistics_delta(
    tx: &rusqlite::Transaction<'_>,
    profile_id: &str,
    scope_type: &str,
    scope_id: &str,
    delta_seconds: i64,
    now_utc: &str,
) -> Result<(), DomainError> {
    tx.execute(
        "INSERT INTO statistics (id, profile_id, scope_type, scope_id, total_playtime_seconds, sessions_count, games_started, games_completed, unlocked_trophies, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 0, 0, 0, ?6)
         ON CONFLICT(profile_id, scope_type, scope_id) DO UPDATE SET
           total_playtime_seconds = statistics.total_playtime_seconds + excluded.total_playtime_seconds,
           sessions_count = statistics.sessions_count + 1,
           updated_at = excluded.updated_at",
        params![
            Uuid::new_v4().to_string(),
            profile_id,
            scope_type,
            scope_id,
            delta_seconds.max(0),
            now_utc,
        ],
    )
    .map_err(|e| DomainError::Database(format!("upsert statistics ({scope_type}) failed: {e}")))?;
    Ok(())
}

impl SessionStatsRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn start_session(
        &self,
        profile_id: &str,
        game_id: &str,
        game_version_id: Option<&str>,
        emulator_id: &str,
        started_at_utc: &str,
    ) -> Result<GameSessionRecord, DomainError> {
        validate_utc_timestamp(started_at_utc)?;
        let session_id = Uuid::new_v4().to_string();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            tx.execute(
                "INSERT INTO game_sessions (id, profile_id, game_id, game_version_id, emulator_id, started_at, last_heartbeat_at, ended_at, duration_seconds, exit_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 0, ?7)",
                params![
                    session_id,
                    profile_id,
                    game_id,
                    game_version_id,
                    emulator_id,
                    started_at_utc,
                    EXIT_STATUS_RUNNING,
                ],
            )
            .map_err(|e| DomainError::Database(format!("insert game_session failed: {e}")))?;

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        self.get_session_by_id(&session_id)
    }

    pub fn heartbeat_session(
        &self,
        session_id: &str,
        heartbeat_at_utc: &str,
    ) -> Result<GameSessionRecord, DomainError> {
        validate_utc_timestamp(heartbeat_at_utc)?;

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let started_at: String = tx
                .query_row(
                    "SELECT started_at FROM game_sessions WHERE id = ?1 AND ended_at IS NULL",
                    params![session_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?
                .ok_or_else(|| {
                    DomainError::NotFound(format!("active session '{session_id}' not found"))
                })?;

            let elapsed = compute_elapsed_seconds_utc(&started_at, heartbeat_at_utc)?;
            tx.execute(
                "UPDATE game_sessions
                 SET last_heartbeat_at = ?1, duration_seconds = ?2
                 WHERE id = ?3 AND ended_at IS NULL",
                params![heartbeat_at_utc, elapsed, session_id],
            )
            .map_err(|e| DomainError::Database(format!("heartbeat update failed: {e}")))?;

            tx.commit()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        self.get_session_by_id(session_id)
    }

    pub fn complete_session(
        &self,
        session_id: &str,
        ended_at_utc: &str,
        exit_status: &str,
    ) -> Result<GameSessionRecord, DomainError> {
        validate_utc_timestamp(ended_at_utc)?;
        let clean_status = exit_status.trim();
        if clean_status.is_empty() {
            return Err(DomainError::Validation(
                "exit_status cannot be empty".to_string(),
            ));
        }

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let row: Option<(String, String, String, String, String)> = tx
                .query_row(
                    "SELECT s.profile_id, s.game_id, g.console_id, g.title, s.started_at
                     FROM game_sessions s
                     JOIN games g ON g.id = s.game_id
                     WHERE s.id = ?1 AND s.ended_at IS NULL",
                    params![session_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                )
                .optional()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let (profile_id, game_id, console_id, game_title, started_at) = row.ok_or_else(|| {
                DomainError::Conflict(format!(
                    "session '{session_id}' does not exist or is already closed"
                ))
            })?;

            let duration = compute_elapsed_seconds_utc(&started_at, ended_at_utc)?;

            let affected = tx
                .execute(
                    "UPDATE game_sessions
                     SET last_heartbeat_at = ?1, ended_at = ?1, duration_seconds = ?2, exit_status = ?3
                     WHERE id = ?4 AND ended_at IS NULL",
                    params![ended_at_utc, duration, clean_status, session_id],
                )
                .map_err(|e| DomainError::Database(format!("complete session failed: {e}")))?;

            if affected == 1 {
                upsert_statistics_delta(&tx, &profile_id, "global", "global", duration, ended_at_utc)?;
                upsert_statistics_delta(&tx, &profile_id, "console", &console_id, duration, ended_at_utc)?;
                upsert_statistics_delta(&tx, &profile_id, "game", &game_id, duration, ended_at_utc)?;

                tx.execute(
                    "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                     VALUES (?1, ?2, 'session_completed', 'game', ?3, ?4, ?5)",
                    params![
                        Uuid::new_v4().to_string(),
                        profile_id,
                        game_id,
                        format!("Sessão de '{game_title}' encerrada ({duration}s — {clean_status})."),
                        ended_at_utc,
                    ],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            }

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;
            Ok(())
        })?;

        self.get_session_by_id(session_id)
    }

    /// Adjustment B: Recovers interrupted sessions (`ended_at IS NULL`) using strictly
    /// `last_heartbeat_at` (never `Utc::now()`), marking `exit_status = 'recovered_after_crash'`.
    /// 100% idempotent across repeated invocations.
    pub fn reconcile_orphaned_sessions(&self) -> Result<SessionRecoveryReport, DomainError> {
        let now = now_utc_rfc3339();

        self.db.with_connection_mut(|conn| {
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut stmt = tx
                .prepare(
                    "SELECT s.id, s.profile_id, s.game_id, g.console_id, g.title, s.started_at, s.last_heartbeat_at
                     FROM game_sessions s
                     JOIN games g ON g.id = s.game_id
                     WHERE s.ended_at IS NULL
                     ORDER BY s.started_at ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let orphans: Vec<(String, String, String, String, String, String, String)> = stmt
                .query_map([], |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                })
                .map_err(|e| DomainError::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            drop(stmt);

            let mut recovered_ids = Vec::with_capacity(orphans.len());
            let mut total_recovered_seconds = 0i64;

            for (session_id, profile_id, game_id, console_id, game_title, started_at, last_hb) in
                orphans
            {
                let duration = compute_elapsed_seconds_utc(&started_at, &last_hb)?;

                let affected = tx
                    .execute(
                        "UPDATE game_sessions
                         SET ended_at = ?1, duration_seconds = ?2, exit_status = ?3
                         WHERE id = ?4 AND ended_at IS NULL",
                        params![last_hb, duration, EXIT_STATUS_RECOVERED_AFTER_CRASH, session_id],
                    )
                    .map_err(|e| DomainError::Database(format!("orphan recovery update failed: {e}")))?;

                if affected == 1 {
                    upsert_statistics_delta(&tx, &profile_id, "global", "global", duration, &now)?;
                    upsert_statistics_delta(&tx, &profile_id, "console", &console_id, duration, &now)?;
                    upsert_statistics_delta(&tx, &profile_id, "game", &game_id, duration, &now)?;

                    tx.execute(
                        "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                         VALUES (?1, ?2, 'session_recovered', 'game', ?3, ?4, ?5)",
                        params![
                            Uuid::new_v4().to_string(),
                            profile_id,
                            game_id,
                            format!(
                                "Sessão interrompida de '{game_title}' recuperada até o último heartbeat ({duration}s)."
                            ),
                            now,
                        ],
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;

                    total_recovered_seconds += duration;
                    recovered_ids.push(session_id);
                }
            }

            tx.commit().map_err(|e| DomainError::Database(e.to_string()))?;

            Ok(SessionRecoveryReport {
                recovered_count: recovered_ids.len(),
                total_recovered_seconds,
                recovered_session_ids: recovered_ids,
            })
        })
    }

    pub fn get_session_by_id(&self, session_id: &str) -> Result<GameSessionRecord, DomainError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT s.id, s.profile_id, s.game_id, g.title, c.slug, s.game_version_id,
                        s.emulator_id, s.started_at, s.last_heartbeat_at, s.ended_at,
                        s.duration_seconds, s.exit_status
                 FROM game_sessions s
                 JOIN games g ON g.id = s.game_id
                 JOIN consoles c ON c.id = g.console_id
                 WHERE s.id = ?1",
                params![session_id],
                |r| {
                    Ok(GameSessionRecord {
                        id: r.get(0)?,
                        profile_id: r.get(1)?,
                        game_id: r.get(2)?,
                        game_title: r.get(3)?,
                        console_slug: r.get(4)?,
                        game_version_id: r.get(5)?,
                        emulator_id: r.get(6)?,
                        started_at: r.get(7)?,
                        last_heartbeat_at: r.get(8)?,
                        ended_at: r.get(9)?,
                        duration_seconds: r.get(10)?,
                        exit_status: r.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|e| DomainError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound(format!("session '{session_id}' not found")))
        })
    }

    pub fn list_sessions_for_game(
        &self,
        profile_id: &str,
        game_id: &str,
        limit: i64,
    ) -> Result<Vec<GameSessionRecord>, DomainError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, s.profile_id, s.game_id, g.title, c.slug, s.game_version_id,
                            s.emulator_id, s.started_at, s.last_heartbeat_at, s.ended_at,
                            s.duration_seconds, s.exit_status
                     FROM game_sessions s
                     JOIN games g ON g.id = s.game_id
                     JOIN consoles c ON c.id = g.console_id
                     WHERE s.profile_id = ?1 AND s.game_id = ?2
                     ORDER BY s.started_at DESC
                     LIMIT ?3",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![profile_id, game_id, limit], |r| {
                    Ok(GameSessionRecord {
                        id: r.get(0)?,
                        profile_id: r.get(1)?,
                        game_id: r.get(2)?,
                        game_title: r.get(3)?,
                        console_slug: r.get(4)?,
                        game_version_id: r.get(5)?,
                        emulator_id: r.get(6)?,
                        started_at: r.get(7)?,
                        last_heartbeat_at: r.get(8)?,
                        ended_at: r.get(9)?,
                        duration_seconds: r.get(10)?,
                        exit_status: r.get(11)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut out = Vec::new();
            for row in rows {
                out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
            }
            Ok(out)
        })
    }

    pub fn get_statistics_overview(
        &self,
        profile_id: &str,
    ) -> Result<StatisticsOverview, DomainError> {
        self.db.with_connection(|conn| {
            let (total_playtime_seconds, total_sessions, games_started): (i64, i64, i64) = conn
                .query_row(
                    "SELECT COALESCE(SUM(duration_seconds), 0), COUNT(*), COUNT(DISTINCT game_id)
                     FROM game_sessions
                     WHERE profile_id = ?1",
                    params![profile_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let (games_registered, games_completed): (i64, i64) = conn
                .query_row(
                    "SELECT COUNT(*), COALESCE(SUM(CASE WHEN completion_percent = 100 THEN 1 ELSE 0 END), 0)
                     FROM games",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let completion_rate_percent = if games_registered > 0 {
                (games_completed * 100) / games_registered
            } else {
                0
            };

            let mut console_stmt = conn
                .prepare(
                    "SELECT c.slug, c.name,
                            (SELECT COUNT(*) FROM games g2 WHERE g2.console_id = c.id) AS games_count,
                            COUNT(s.id) AS sessions_count,
                            COALESCE(SUM(s.duration_seconds), 0) AS playtime
                     FROM consoles c
                     LEFT JOIN games g ON g.console_id = c.id
                     LEFT JOIN game_sessions s ON s.game_id = g.id AND s.profile_id = ?1
                     GROUP BY c.id, c.slug, c.name
                     ORDER BY playtime DESC, games_count DESC, c.name ASC",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let by_console = console_stmt
                .query_map(params![profile_id], |r| {
                    Ok(ConsolePlaytimeStat {
                        console_slug: r.get(0)?,
                        console_name: r.get(1)?,
                        games_count: r.get(2)?,
                        sessions_count: r.get(3)?,
                        total_playtime_seconds: r.get(4)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut top_stmt = conn
                .prepare(
                    "SELECT g.id, g.title, c.slug, COUNT(s.id) AS sessions_count,
                            COALESCE(SUM(s.duration_seconds), 0) AS playtime,
                            g.completion_percent, MAX(s.started_at)
                     FROM games g
                     JOIN consoles c ON c.id = g.console_id
                     LEFT JOIN game_sessions s ON s.game_id = g.id AND s.profile_id = ?1
                     GROUP BY g.id, g.title, c.slug, g.completion_percent
                     HAVING sessions_count > 0 OR playtime > 0
                     ORDER BY playtime DESC, sessions_count DESC, g.title ASC
                     LIMIT 10",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let top_games = top_stmt
                .query_map(params![profile_id], |r| {
                    Ok(TopGameStat {
                        game_id: r.get(0)?,
                        title: r.get(1)?,
                        console_slug: r.get(2)?,
                        sessions_count: r.get(3)?,
                        total_playtime_seconds: r.get(4)?,
                        completion_percent: r.get(5)?,
                        last_played_at: r.get(6)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            let mut month_stmt = conn
                .prepare(
                    "SELECT substr(started_at, 1, 7) AS ym, COUNT(*) AS sessions_count,
                            COALESCE(SUM(duration_seconds), 0) AS playtime
                     FROM game_sessions
                     WHERE profile_id = ?1
                     GROUP BY ym
                     ORDER BY ym DESC
                     LIMIT 12",
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let monthly_playtime = month_stmt
                .query_map(params![profile_id], |r| {
                    Ok(MonthlyPlaytimeStat {
                        month: r.get(0)?,
                        sessions_count: r.get(1)?,
                        total_playtime_seconds: r.get(2)?,
                    })
                })
                .map_err(|e| DomainError::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| DomainError::Database(e.to_string()))?;

            Ok(StatisticsOverview {
                total_playtime_seconds,
                total_sessions,
                games_registered,
                games_started,
                games_completed,
                completion_rate_percent,
                by_console,
                top_games,
                monthly_playtime,
            })
        })
    }

    pub fn list_activity_history(
        &self,
        profile_id: &str,
        event_type_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ActivityHistoryRow>, DomainError> {
        let bounded_limit = limit.clamp(1, 500);
        let clean_filter = event_type_filter
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "all");

        self.db.with_connection(|conn| {
            let mut out = Vec::new();
            if let Some(event_type) = clean_filter {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, event_type, entity_type, entity_id, summary, occurred_at
                         FROM activity_history
                         WHERE profile_id = ?1 AND event_type = ?2
                         ORDER BY occurred_at DESC
                         LIMIT ?3",
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                let rows = stmt
                    .query_map(params![profile_id, event_type, bounded_limit], |r| {
                        Ok(ActivityHistoryRow {
                            id: r.get(0)?,
                            event_type: r.get(1)?,
                            entity_type: r.get(2)?,
                            entity_id: r.get(3)?,
                            summary: r.get(4)?,
                            occurred_at: r.get(5)?,
                        })
                    })
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                for row in rows {
                    out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
                }
            } else {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, event_type, entity_type, entity_id, summary, occurred_at
                         FROM activity_history
                         WHERE profile_id = ?1
                         ORDER BY occurred_at DESC
                         LIMIT ?2",
                    )
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                let rows = stmt
                    .query_map(params![profile_id, bounded_limit], |r| {
                        Ok(ActivityHistoryRow {
                            id: r.get(0)?,
                            event_type: r.get(1)?,
                            entity_type: r.get(2)?,
                            entity_id: r.get(3)?,
                            summary: r.get(4)?,
                            occurred_at: r.get(5)?,
                        })
                    })
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                for row in rows {
                    out.push(row.map_err(|e| DomainError::Database(e.to_string()))?);
                }
            }
            Ok(out)
        })
    }
}
