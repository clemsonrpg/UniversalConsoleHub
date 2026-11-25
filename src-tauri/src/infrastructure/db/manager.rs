use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;

use super::migrations::{apply_migrations, check_schema_version_compatible, LATEST_SCHEMA_VERSION};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInspectionReport {
    pub backup_path: PathBuf,
    pub size_bytes: u64,
    pub integrity_ok: bool,
    pub schema_version: i64,
    pub profiles_count: i64,
    pub games_count: i64,
    pub inspected_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveRestoreReport {
    pub source_backup_path: PathBuf,
    pub restored_db_path: PathBuf,
    pub pre_restore_snapshot_path: PathBuf,
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

fn sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    let mut os = db_path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

fn remove_wal_sidecars(db_path: &Path) -> Result<(), DomainError> {
    for suffix in ["-wal", "-shm"] {
        let sidecar = sidecar_path(db_path, suffix);
        if sidecar.exists() {
            fs::remove_file(&sidecar).map_err(|e| {
                DomainError::Io(format!(
                    "failed to remove stale sidecar '{}': {e}",
                    sidecar.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn validate_and_stage_backup(
    backup_path: &Path,
    target_db_path: &Path,
) -> Result<(PathBuf, i64), DomainError> {
    let inspection = DatabaseManager::inspect_backup(backup_path)?;
    if !inspection.integrity_ok {
        return Err(DomainError::Database(
            "backup file failed PRAGMA quick_check / foreign_key_check; restore aborted"
                .to_string(),
        ));
    }

    if let Some(parent) = target_db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            DomainError::Io(format!(
                "failed to create target directory '{}': {e}",
                parent.display()
            ))
        })?;
    }

    let staged_path = sidecar_path(target_db_path, ".restore.tmp");
    if staged_path.exists() {
        fs::remove_file(&staged_path)
            .map_err(|e| DomainError::Io(format!("failed to clear staged restore file: {e}")))?;
    }

    let staged_str = staged_path
        .to_str()
        .ok_or_else(|| DomainError::Validation("target path must be valid UTF-8".to_string()))?;

    let src_conn = Connection::open_with_flags(
        backup_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| DomainError::Database(format!("invalid SQLite backup file: {e}")))?;

    src_conn
        .execute("VACUUM INTO ?1", params![staged_str])
        .map_err(|e| DomainError::Database(format!("failed to stage restore copy: {e}")))?;
    drop(src_conn);

    Ok((staged_path, inspection.schema_version))
}

fn swap_staged_file_with_rollback(
    staged_path: &Path,
    target_db_path: &Path,
    rollback_path: &Path,
) -> Result<(), DomainError> {
    if rollback_path.exists() {
        fs::remove_file(rollback_path).map_err(|e| {
            DomainError::Io(format!(
                "failed to remove stale rollback file '{}': {e}",
                rollback_path.display()
            ))
        })?;
    }

    remove_wal_sidecars(target_db_path)?;

    if target_db_path.exists() {
        fs::rename(target_db_path, rollback_path).map_err(|e| {
            DomainError::Io(format!(
                "failed to move current database '{}' to rollback slot: {e}",
                target_db_path.display()
            ))
        })?;
    }

    if let Err(err) = fs::rename(staged_path, target_db_path) {
        // Attempt immediate rollback of the original DB file
        if rollback_path.exists() && !target_db_path.exists() {
            let _ = fs::rename(rollback_path, target_db_path);
        }
        return Err(DomainError::Io(format!(
            "failed to promote restored database '{}': {err}",
            target_db_path.display()
        )));
    }

    Ok(())
}

fn rollback_target_file(
    target_db_path: &Path,
    rollback_path: &Path,
    fallback_snapshot: Option<&Path>,
) {
    let _ = remove_wal_sidecars(target_db_path);
    if target_db_path.exists() {
        let _ = fs::remove_file(target_db_path);
    }
    if rollback_path.exists() {
        let _ = fs::rename(rollback_path, target_db_path);
    } else if let Some(snapshot) = fallback_snapshot {
        if snapshot.is_file() {
            let _ = fs::copy(snapshot, target_db_path);
        }
    }
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

        // Check future schema compatibility BEFORE enabling WAL so an incompatible
        // future-version database file is never modified on disk.
        if path.is_file() {
            let ro = Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .map_err(|e| {
                DomainError::Database(format!("failed to open SQLite '{}': {e}", path.display()))
            })?;
            let _ = check_schema_version_compatible(&ro)?;
            drop(ro);
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

    /// Inspects a SQLite backup file in read-only mode before any destructive operation.
    pub fn inspect_backup(backup_path: &Path) -> Result<BackupInspectionReport, DomainError> {
        if !backup_path.is_file() {
            return Err(DomainError::NotFound(format!(
                "backup file not found: '{}'",
                backup_path.display()
            )));
        }

        let metadata = fs::metadata(backup_path)
            .map_err(|e| DomainError::Io(format!("failed to stat backup file: {e}")))?;

        let conn = Connection::open_with_flags(
            backup_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| DomainError::Database(format!("invalid SQLite backup file: {e}")))?;

        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| DomainError::Database(format!("failed to enable foreign_keys: {e}")))?;

        let integrity_ok = run_integrity_checks(&conn)?;
        if !integrity_ok {
            return Err(DomainError::Database(
                "backup file failed PRAGMA quick_check / foreign_key_check".to_string(),
            ));
        }

        let schema_version: i64 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .map_err(|e| DomainError::Database(format!("failed to read backup version: {e}")))?;

        if schema_version < 1 {
            return Err(DomainError::Validation(format!(
                "backup schema version {schema_version} is not a valid Universal Console Hub database"
            )));
        }
        if schema_version > LATEST_SCHEMA_VERSION {
            return Err(DomainError::Validation(format!(
                "backup schema version {schema_version} is newer than supported version {LATEST_SCHEMA_VERSION}"
            )));
        }

        let profiles_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))
            .map_err(|e| {
                DomainError::Validation(format!(
                    "backup is missing Universal Console Hub 'profiles' table: {e}"
                ))
            })?;
        let games_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .map_err(|e| {
                DomainError::Validation(format!(
                    "backup is missing Universal Console Hub 'games' table: {e}"
                ))
            })?;

        Ok(BackupInspectionReport {
            backup_path: backup_path.to_path_buf(),
            size_bytes: metadata.len(),
            integrity_ok,
            schema_version,
            profiles_count,
            games_count,
            inspected_at: now_utc_rfc3339(),
        })
    }

    /// Validates a backup snapshot and safely restores it onto `target_db_path`.
    /// Callers must ensure active write handles to `target_db_path` are closed before restoring.
    pub fn restore_backup(
        backup_path: &Path,
        target_db_path: &Path,
    ) -> Result<RestoreReport, DomainError> {
        let (staged_path, _src_version) = validate_and_stage_backup(backup_path, target_db_path)?;
        let rollback_path = sidecar_path(target_db_path, ".rollback.tmp");

        swap_staged_file_with_rollback(&staged_path, target_db_path, &rollback_path)?;

        let verify_result = (|| -> Result<i64, DomainError> {
            let mut verify_conn = Connection::open(target_db_path).map_err(|e| {
                DomainError::Database(format!("failed to verify restored database: {e}"))
            })?;
            configure_connection(&verify_conn, true)?;
            let final_version = apply_migrations(&mut verify_conn)?;
            seed_defaults(&mut verify_conn)?;
            if !run_integrity_checks(&verify_conn)? {
                return Err(DomainError::Database(
                    "restored database failed post-restore integrity verification".to_string(),
                ));
            }
            Ok(final_version)
        })();

        match verify_result {
            Ok(final_version) => {
                if rollback_path.exists() {
                    let _ = fs::remove_file(&rollback_path);
                }
                Ok(RestoreReport {
                    source_backup_path: backup_path.to_path_buf(),
                    restored_db_path: target_db_path.to_path_buf(),
                    integrity_ok: true,
                    schema_version: final_version,
                    restored_at: now_utc_rfc3339(),
                })
            }
            Err(err) => {
                rollback_target_file(target_db_path, &rollback_path, None);
                let _ = fs::remove_file(&staged_path);
                Err(err)
            }
        }
    }

    /// Windows-safe live restore while this `DatabaseManager` holds an active WAL connection:
    /// 1. Validates the source backup integrally before touching active data.
    /// 2. Creates a verified pre-restore snapshot (`pre_restore_<utc>.db`) via `VACUUM INTO`.
    /// 3. Stages a clean copy of the backup to `.restore.tmp`.
    /// 4. Under exclusive `Mutex<Connection>`, drops the active OS connection handle completely.
    /// 5. Swaps the database file with `.rollback.tmp` protection and clears stale `-wal`/`-shm`.
    /// 6. Reopens with WAL + FK, auto-migrates if restoring a v1 backup, verifies integrity,
    ///    and automatically rolls back to the original database on any error.
    pub fn restore_from_backup_live(
        &self,
        backup_path: &Path,
        safety_backup_dir: &Path,
    ) -> Result<LiveRestoreReport, DomainError> {
        let target_db_path = self.db_path.as_ref().ok_or_else(|| {
            DomainError::Validation(
                "live restore is only supported for file-backed SQLite databases".to_string(),
            )
        })?;

        // 1. Validate source backup completely before creating snapshots or touching live DB
        let _ = Self::inspect_backup(backup_path)?;

        if let (Ok(canon_backup), Ok(canon_target)) = (
            fs::canonicalize(backup_path),
            fs::canonicalize(target_db_path),
        ) {
            if canon_backup == canon_target {
                return Err(DomainError::Validation(
                    "backup source cannot be the active live database file".to_string(),
                ));
            }
        }

        // 2. Create a consistent pre-restore safety snapshot of the active WAL database
        fs::create_dir_all(safety_backup_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to prepare pre-restore snapshot directory '{}': {e}",
                safety_backup_dir.display()
            ))
        })?;
        let ts = now_utc_rfc3339().replace(':', "-");
        let short_id = &Uuid::new_v4().simple().to_string()[..8];
        let snapshot_path = safety_backup_dir.join(format!("pre_restore_{ts}_{short_id}.db"));
        self.create_backup(&snapshot_path)?;

        // 3. Stage clean copy of validated backup beside target database
        let (staged_path, _src_version) = validate_and_stage_backup(backup_path, target_db_path)?;
        let rollback_path = sidecar_path(target_db_path, ".rollback.tmp");

        // 4. Exclusively lock & release all OS file handles to the active WAL database
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| DomainError::Database("database mutex poisoned".to_string()))?;

        let placeholder = Connection::open_in_memory().map_err(|e| {
            DomainError::Database(format!("failed to create temporary placeholder conn: {e}"))
        })?;
        let old_conn = std::mem::replace(&mut *guard, placeholder);
        drop(old_conn);

        // 5 & 6. Swap files, reopen with WAL/FK, migrate v1 backups if needed, verify integrity
        let swap_and_reopen = (|| -> Result<(Connection, i64), DomainError> {
            swap_staged_file_with_rollback(&staged_path, target_db_path, &rollback_path)?;

            let mut new_conn = Connection::open(target_db_path).map_err(|e| {
                DomainError::Database(format!("failed to reopen restored database: {e}"))
            })?;
            configure_connection(&new_conn, true)?;
            let final_version = apply_migrations(&mut new_conn)?;
            seed_defaults(&mut new_conn)?;
            if !run_integrity_checks(&new_conn)? {
                return Err(DomainError::Database(
                    "restored database failed post-restore PRAGMA quick_check / foreign_key_check"
                        .to_string(),
                ));
            }
            Ok((new_conn, final_version))
        })();

        match swap_and_reopen {
            Ok((new_conn, final_version)) => {
                *guard = new_conn;
                if rollback_path.exists() {
                    let _ = fs::remove_file(&rollback_path);
                }
                Ok(LiveRestoreReport {
                    source_backup_path: backup_path.to_path_buf(),
                    restored_db_path: target_db_path.to_path_buf(),
                    pre_restore_snapshot_path: snapshot_path,
                    integrity_ok: true,
                    schema_version: final_version,
                    restored_at: now_utc_rfc3339(),
                })
            }
            Err(err) => {
                // Automatic rollback recovery: restore original file and reopen active connection
                let _ = fs::remove_file(&staged_path);
                rollback_target_file(target_db_path, &rollback_path, Some(&snapshot_path));
                if let Ok(recovered_conn) = Connection::open(target_db_path) {
                    let _ = configure_connection(&recovered_conn, true);
                    *guard = recovered_conn;
                }
                Err(err)
            }
        }
    }
}
