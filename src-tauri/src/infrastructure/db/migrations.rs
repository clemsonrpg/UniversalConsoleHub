use rusqlite::Connection;

use crate::domain::error::DomainError;

const MIGRATION_0001: &str = include_str!("sql/0001_initial_schema.sql");
const MIGRATION_0002: &str = include_str!("sql/0002_console_environments.sql");
pub const LATEST_SCHEMA_VERSION: i64 = 2;

fn run_transactional_migration(
    conn: &mut Connection,
    target_version: i64,
    label: &str,
    sql: &str,
) -> Result<(), DomainError> {
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| DomainError::Database(format!("failed to begin {label} tx: {e}")))?;

    tx.execute_batch(sql)
        .map_err(|e| DomainError::Database(format!("{label} failed: {e}")))?;

    tx.pragma_update(None, "user_version", target_version)
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
            "{label} aborted with {fk_violations} foreign key violations"
        )));
    }

    let quick: String = tx
        .query_row("PRAGMA quick_check;", [], |row| row.get(0))
        .map_err(|e| DomainError::Database(format!("quick_check failed during {label}: {e}")))?;
    if quick != "ok" {
        return Err(DomainError::Database(format!(
            "{label} aborted because PRAGMA quick_check returned: {quick}"
        )));
    }

    tx.commit()
        .map_err(|e| DomainError::Database(format!("failed to commit {label}: {e}")))?;
    Ok(())
}

pub fn check_schema_version_compatible(conn: &Connection) -> Result<i64, DomainError> {
    let current_version: i64 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| DomainError::Database(format!("failed to read user_version: {e}")))?;

    if current_version > LATEST_SCHEMA_VERSION {
        return Err(DomainError::Validation(format!(
            "database schema version {current_version} is newer than supported version {LATEST_SCHEMA_VERSION}; refusing to open without modifying database"
        )));
    }

    Ok(current_version)
}

pub fn apply_migrations(conn: &mut Connection) -> Result<i64, DomainError> {
    let mut current_version = check_schema_version_compatible(conn)?;

    if current_version < 1 {
        run_transactional_migration(conn, 1, "migration 0001", MIGRATION_0001)?;
        current_version = 1;
    }

    if current_version < 2 {
        run_transactional_migration(conn, 2, "migration 0002", MIGRATION_0002)?;
    }

    conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| DomainError::Database(format!("failed to confirm user_version: {e}")))
}
