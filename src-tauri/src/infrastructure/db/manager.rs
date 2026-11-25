use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;

use super::migrations::apply_migrations;
use super::seed::seed_defaults;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupReport {
    pub backup_path: PathBuf,
    pub size_bytes: u64,
    pub integrity_ok: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub source_backup_path: PathBuf,
    pub restored_db_path: PathBuf,
    pub integrity_ok: bool,
    pub schema_version: i64,
    pub restored_at: String,
}

#[derive(Clone)]
pub struct DatabaseManager {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) db_path: Option<PathBuf>,
}

fn configure_connection(conn: &Connection, enable_wal: bool) -> Result<(), DomainError> {
    if enable_wal {
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| DomainError::Database(format!("failed to enable WAL: {e}")))?;
    }
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| DomainError::Database(format!("failed to set synchronous=NORMAL: {e}")))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| DomainError::Database(format!("failed to enable foreign_keys: {e}")))?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| DomainError::Database(format!("failed to set busy_timeout: {e}")))?;
    conn.pragma_update(None, "cache_size", -64000)
        .map_err(|e| DomainError::Database(format!("failed to set cache_size: {e}")))?;
    conn.pragma_update(None, "temp_store", "MEMORY")
        .map_err(|e| DomainError::Database(format!("failed to set temp_store: {e}")))?;
    Ok(())
}

fn run_integrity_checks(conn: &Connection) -> Result<bool, DomainError> {
    let quick: String = conn
        .query_row("PRAGMA quick_check;", [], |r| r.get(0))
        .map_err(|e| DomainError::Database(format!("PRAGMA quick_check failed: {e}")))?;
    if quick != "ok" {
        return Ok(false);
    }

    let fk_errors: usize = conn
        .prepare("PRAGMA foreign_key_check;")
        .and_then(|mut stmt| stmt.query_map([], |_| Ok(())).map(|rows| rows.count()))
        .map_err(|e| DomainError::Database(format!("PRAGMA foreign_key_check failed: {e}")))?;

    Ok(fk_errors == 0)
}

impl DatabaseManager {
    pub fn open(path: &Path) -> Result<Self, DomainError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DomainError::Io(format!(
                    "failed to create database directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        let mut conn = Connection::open(path).map_err(|e| {
            DomainError::Database(format!("failed to open SQLite '{}': {e}", path.display()))
        })?;
        configure_connection(&conn, true)?;
        apply_migrations(&mut conn)?;
        seed_defaults(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: Some(path.to_path_buf()),
        })
    }

    pub fn open_in_memory() -> Result<Self, DomainError> {
        let mut conn = Connection::open_in_memory()
            .map_err(|e| DomainError::Database(format!("failed to open in-memory SQLite: {e}")))?;
        configure_connection(&conn, false)?;
        apply_migrations(&mut conn)?;
        seed_defaults(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: None,
        })
    }

    pub fn with_connection<T, F>(&self, f: F) -> Result<T, DomainError>
    where
        F: FnOnce(&Connection) -> Result<T, DomainError>,
    {
        let guard = self
            .conn
            .lock()
            .map_err(|_| DomainError::Database("database mutex poisoned".to_string()))?;
        f(&guard)
    }

    pub fn with_connection_mut<T, F>(&self, f: F) -> Result<T, DomainError>
    where
        F: FnOnce(&mut Connection) -> Result<T, DomainError>,
    {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| DomainError::Database("database mutex poisoned".to_string()))?;
        f(&mut guard)
    }

    pub fn db_path(&self) -> Option<&Path> {
        self.db_path.as_deref()
    }

    pub fn verify_integrity(&self) -> Result<bool, DomainError> {
        self.with_connection(run_integrity_checks)
    }

    pub fn create_backup(&self, destination: &Path) -> Result<BackupReport, DomainError> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DomainError::Io(format!(
                    "failed to create backup directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        if destination.exists() {
            fs::remove_file(destination).map_err(|e| {
                DomainError::Io(format!(
                    "failed to overwrite existing backup '{}': {e}",
                    destination.display()
                ))
            })?;
        }

        let dest_str = destination.to_str().ok_or_else(|| {
            DomainError::Validation("backup destination path must be valid UTF-8".to_string())
        })?;

        self.with_connection(|conn| {
            conn.execute("VACUUM INTO ?1", params![dest_str])
                .map_err(|e| DomainError::Database(format!("VACUUM INTO backup failed: {e}")))?;
            Ok(())
        })?;

        // Validate backup integrity independently in read-only mode
        let verify_conn = Connection::open_with_flags(
            destination,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| DomainError::Database(format!("failed to open backup for validation: {e}")))?;
        let integrity_ok = run_integrity_checks(&verify_conn)?;
        drop(verify_conn);

        if !integrity_ok {
            return Err(DomainError::Database(
                "created backup failed integrity verification".to_string(),
            ));
        }

        let metadata = fs::metadata(destination)
            .map_err(|e| DomainError::Io(format!("failed to stat backup file: {e}")))?;

        Ok(BackupReport {
            backup_path: destination.to_path_buf(),
            size_bytes: metadata.len(),
            integrity_ok,
            created_at: now_utc_rfc3339(),
        })
    }

    /// Validates a backup snapshot and safely restores it onto `target_db_path`.
    /// Callers must ensure active write handles to `target_db_path` are closed before restoring.
    pub fn restore_backup(
        backup_path: &Path,
        target_db_path: &Path,
    ) -> Result<RestoreReport, DomainError> {
        if !backup_path.is_file() {
            return Err(DomainError::NotFound(format!(
                "backup file not found: '{}'",
                backup_path.display()
            )));
        }

        // 1. Validate source backup integrity & schema version in read-only mode
        let src_conn = Connection::open_with_flags(
            backup_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| DomainError::Database(format!("invalid SQLite backup file: {e}")))?;
        configure_connection(&src_conn, false)?;

        let src_ok = run_integrity_checks(&src_conn)?;
        if !src_ok {
            return Err(DomainError::Database(
                "backup file failed PRAGMA quick_check / foreign_key_check; restore aborted"
                    .to_string(),
            ));
        }

        let schema_version: i64 = src_conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .map_err(|e| DomainError::Database(format!("failed to read backup version: {e}")))?;
        if schema_version < 1 {
            return Err(DomainError::Validation(format!(
                "backup schema version {schema_version} is not a valid Universal Console Hub database"
            )));
        }

        if let Some(parent) = target_db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DomainError::Io(format!(
                    "failed to create target directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        // 2. Stage clean copy via VACUUM INTO a temp file beside target, then swap cleanly
        let staged_path = target_db_path.with_extension("restore.tmp");
        if staged_path.exists() {
            fs::remove_file(&staged_path).map_err(|e| {
                DomainError::Io(format!("failed to clear staged restore file: {e}"))
            })?;
        }
        let staged_str = staged_path.to_str().ok_or_else(|| {
            DomainError::Validation("target path must be valid UTF-8".to_string())
        })?;
        src_conn
            .execute("VACUUM INTO ?1", params![staged_str])
            .map_err(|e| DomainError::Database(format!("failed to stage restore copy: {e}")))?;
        drop(src_conn);

        // 3. Remove stale WAL/SHM sidecars so SQLite never replays old WAL frames over restored DB
        for suffix in ["-wal", "-shm"] {
            let mut sidecar = target_db_path.as_os_str().to_os_string();
            sidecar.push(suffix);
            let sidecar_path = PathBuf::from(sidecar);
            if sidecar_path.exists() {
                fs::remove_file(&sidecar_path).map_err(|e| {
                    DomainError::Io(format!(
                        "failed to remove stale sidecar '{}': {e}",
                        sidecar_path.display()
                    ))
                })?;
            }
        }
        if target_db_path.exists() {
            fs::remove_file(target_db_path).map_err(|e| {
                DomainError::Io(format!(
                    "failed to replace target database '{}': {e}",
                    target_db_path.display()
                ))
            })?;
        }
        fs::rename(&staged_path, target_db_path).map_err(|e| {
            DomainError::Io(format!(
                "failed to promote restored database '{}': {e}",
                target_db_path.display()
            ))
        })?;

        // 4. Re-open restored database and confirm integrity
        let verify_conn = Connection::open(target_db_path).map_err(|e| {
            DomainError::Database(format!("failed to verify restored database: {e}"))
        })?;
        configure_connection(&verify_conn, true)?;
        let integrity_ok = run_integrity_checks(&verify_conn)?;

        Ok(RestoreReport {
            source_backup_path: backup_path.to_path_buf(),
            restored_db_path: target_db_path.to_path_buf(),
            integrity_ok,
            schema_version,
            restored_at: now_utc_rfc3339(),
        })
    }
}
