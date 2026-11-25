use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;
use crate::infrastructure::db::manager::{
    BackupInspectionReport, BackupReport, DatabaseManager, LiveRestoreReport,
};
use crate::infrastructure::system::security::SecurityManager;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileEntry {
    pub file_name: String,
    pub backup_path: PathBuf,
    pub size_bytes: u64,
    pub modified_at: String,
}

#[derive(Clone)]
pub struct BackupService {
    db: DatabaseManager,
    backup_dir: PathBuf,
}

impl BackupService {
    pub fn new(db: DatabaseManager, backup_dir: PathBuf) -> Self {
        Self { db, backup_dir }
    }

    pub fn list_backups(&self) -> Result<Vec<BackupFileEntry>, DomainError> {
        if !self.backup_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let read_dir = fs::read_dir(&self.backup_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to list backups in '{}': {e}",
                self.backup_dir.display()
            ))
        })?;

        for item in read_dir {
            let entry = match item {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let is_db = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("db"))
                .unwrap_or(false);
            if !is_db {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let modified_at = meta
                .modified()
                .ok()
                .map(|st| DateTime::<Utc>::from(st).to_rfc3339_opts(SecondsFormat::Secs, true))
                .unwrap_or_else(now_utc_rfc3339);

            let canonical = SecurityManager::validate_existing_file(&path).unwrap_or(path);
            let file_name = canonical
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("backup.db")
                .to_string();

            entries.push(BackupFileEntry {
                file_name,
                backup_path: canonical,
                size_bytes: meta.len(),
                modified_at,
            });
        }

        entries.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        Ok(entries)
    }

    pub fn inspect_backup(
        &self,
        backup_path: &Path,
    ) -> Result<BackupInspectionReport, DomainError> {
        let valid_path = SecurityManager::validate_existing_file(backup_path)?;
        DatabaseManager::inspect_backup(&valid_path)
    }

    pub fn create_backup(&self, file_name: Option<String>) -> Result<BackupReport, DomainError> {
        fs::create_dir_all(&self.backup_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to prepare backup directory '{}': {e}",
                self.backup_dir.display()
            ))
        })?;

        let safe_name = match file_name {
            Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
            _ => {
                let ts = now_utc_rfc3339().replace(':', "-");
                format!("uch_backup_{ts}.db")
            }
        };

        if safe_name.contains(['/', '\\']) || safe_name.contains("..") {
            return Err(DomainError::Validation(
                "nome do backup deve ser apenas um nome de arquivo simples sem separadores ou '..'"
                    .to_string(),
            ));
        }

        let target = SecurityManager::ensure_within_base(&self.backup_dir, Path::new(&safe_name))?;
        self.db.create_backup(&target)
    }

    pub fn restore_backup(&self, backup_path: &Path) -> Result<LiveRestoreReport, DomainError> {
        let valid_path = SecurityManager::validate_existing_file(backup_path)?;
        let safety_dir = self.backup_dir.join("pre_restore");
        self.db.restore_from_backup_live(&valid_path, &safety_dir)
    }
}
