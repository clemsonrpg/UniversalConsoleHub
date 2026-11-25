use rusqlite::Connection;

use crate::domain::error::DomainError;

const MIGRATION_0001: &str = include_str!("sql/0001_initial_schema.sql");
pub const LATEST_SCHEMA_VERSION: i64 = 1;

pub fn apply_migrations(conn: &mut Connection) -> Result<i64, DomainError> {
    let current_version: i64 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| DomainError::Database(format!("failed to read user_version: {e}")))?;

    if current_version < 1 {
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| DomainError::Database(format!("failed to begin migration tx: {e}")))?;

        tx.execute_batch(MIGRATION_0001)
            .map_err(|e| DomainError::Database(format!("migration 0001 failed: {e}")))?;

        tx.pragma_update(None, "user_version", LATEST_SCHEMA_VERSION)
            .map_err(|e| DomainError::Database(format!("failed to update user_version: {e}")))?;

        let fk_violations: i64 = tx
            .prepare("PRAGMA foreign_key_check;")
            .and_then(|mut s| {
                s.query_map([], |_| Ok(1i64))
                    .map(|rows| rows.count() as i64)
            })
            .map_err(|e| DomainError::Database(format!("foreign_key_check failed: {e}")))?;

        if fk_violations > 0 {
            return Err(DomainError::Database(format!(
                "migration 0001 aborted with {fk_violations} foreign key violations"
            )));
        }

        tx.commit()
            .map_err(|e| DomainError::Database(format!("failed to commit migration 0001: {e}")))?;
    }

    conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| DomainError::Database(format!("failed to confirm user_version: {e}")))
}
