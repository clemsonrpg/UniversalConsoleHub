use std::path::Path;

use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::adapters::discovery::discovery_for_console;
use crate::domain::error::DomainError;
use crate::domain::player::{normalize_canonical_key, now_utc_rfc3339};
use crate::domain::scanner::{ScanProgressEvent, ScanSummaryDto};
use crate::infrastructure::db::console_env_repo::ConsoleEnvRepository;
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::SqliteRepositories;

#[derive(Clone)]
pub struct ScannerService {
    db: DatabaseManager,
    repos: SqliteRepositories,
    env_repo: ConsoleEnvRepository,
}

impl ScannerService {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            env_repo: ConsoleEnvRepository::new(db.clone()),
            db,
        }
    }

    pub fn scan_console<F>(
        &self,
        console_slug: &str,
        mut on_progress: F,
    ) -> Result<ScanSummaryDto, DomainError>
    where
        F: FnMut(ScanProgressEvent),
    {
        let console = self.repos.get_console_by_slug(console_slug)?;
        let discovery = discovery_for_console(console_slug).ok_or_else(|| {
            DomainError::Validation(format!(
                "nenhum scanner de biblioteca registrado para o console '{console_slug}'"
            ))
        })?;

        let enabled_paths: Vec<_> = self
            .env_repo
            .list_scan_paths(console_slug)?
            .into_iter()
            .filter(|p| p.enabled)
            .collect();

        on_progress(ScanProgressEvent::Started {
            console_slug: console_slug.to_string(),
            total_directories: enabled_paths.len(),
        });

        let mut scanned_directories = 0usize;
        let mut scanned_files = 0usize;
        let mut games_added = 0usize;
        let mut versions_added = 0usize;
        let mut skipped_existing = 0usize;
        let mut errors = Vec::new();
        let now = now_utc_rfc3339();

        for sp in &enabled_paths {
            let dir = Path::new(&sp.directory_path);
            let batch = match discovery.discover_in_directory(dir, sp.recursive, &mut on_progress) {
                Ok(b) => b,
                Err(err) => {
                    let msg = format!("Erro ao escanear '{}': {err}", sp.directory_path);
                    on_progress(ScanProgressEvent::Error {
                        message: msg.clone(),
                    });
                    errors.push(msg);
                    continue;
                }
            };

            scanned_directories += 1;
            scanned_files += batch.files_checked;
            errors.extend(batch.diagnostics);

            for candidate in batch.candidates {
                let file_path_str = candidate.file_path.to_string_lossy().to_string();
                if self
                    .env_repo
                    .find_existing_version_by_file_path(&file_path_str)?
                    .is_some()
                {
                    skipped_existing += 1;
                    continue;
                }

                let canonical_key =
                    normalize_canonical_key(&candidate.title, candidate.serial_code.as_deref())?;
                let existed_before = self.db.with_connection(|conn| {
                    let row: Option<String> = conn
                        .query_row(
                            "SELECT id FROM games WHERE console_id = ?1 AND canonical_key = ?2",
                            params![console.id, canonical_key],
                            |r| r.get(0),
                        )
                        .optional()
                        .map_err(|e| DomainError::Database(e.to_string()))?;
                    Ok(row.is_some())
                })?;

                match self.repos.register_game_with_version(
                    console_slug,
                    &candidate.title,
                    &candidate.version_label,
                    &candidate.region,
                    candidate.serial_code.as_deref(),
                    &file_path_str,
                    &candidate.metadata_source,
                ) {
                    Ok(_) => {
                        if !existed_before {
                            games_added += 1;
                        }
                        versions_added += 1;
                    }
                    Err(DomainError::Conflict(_)) => {
                        skipped_existing += 1;
                    }
                    Err(err) => {
                        errors.push(format!(
                            "Falha ao importar '{}' ({}): {err}",
                            candidate.title, file_path_str
                        ));
                    }
                }
            }

            let _ = self.env_repo.mark_scan_path_scanned(&sp.id, &now);
        }

        if let Ok(profile) = self.repos.get_active_profile() {
            let summary_text = format!(
                "Scan de {} concluído: {} novos jogos, {} versões adicionadas, {} já existentes.",
                console.name, games_added, versions_added, skipped_existing
            );
            let _ = self.db.with_connection_mut(|conn| {
                conn.execute(
                    "INSERT INTO activity_history (id, profile_id, event_type, entity_type, entity_id, summary, occurred_at)
                     VALUES (?1, ?2, 'library_scanned', 'console', ?3, ?4, ?5)",
                    params![
                        Uuid::new_v4().to_string(),
                        profile.id,
                        console.id,
                        summary_text,
                        now,
                    ],
                )
                .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(())
            });
        }

        on_progress(ScanProgressEvent::Completed {
            scanned_files,
            games_added,
            versions_added,
            skipped_existing,
        });

        Ok(ScanSummaryDto {
            console_slug: console_slug.to_string(),
            scanned_directories,
            scanned_files,
            games_added,
            versions_added,
            skipped_existing,
            errors,
        })
    }
}
