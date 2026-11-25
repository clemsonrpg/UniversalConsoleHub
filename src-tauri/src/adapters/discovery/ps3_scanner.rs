use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::error::DomainError;
use crate::domain::scanner::{
    DiscoveredGameCandidate, DiscoveryBatch, GameDiscoveryPort, ScanProgressEvent,
};
use crate::infrastructure::system::security::SecurityManager;

use super::param_sfo::{infer_ps3_region, parse_param_sfo};

const MAX_SCAN_DEPTH: usize = 16;

pub struct Ps3GameDiscovery;

impl Ps3GameDiscovery {
    fn inspect_ps3_game_dir(
        game_root: &Path,
        seen_eboots: &mut BTreeSet<PathBuf>,
        candidates: &mut Vec<DiscoveredGameCandidate>,
        diagnostics: &mut Vec<String>,
        on_progress: &mut dyn FnMut(ScanProgressEvent),
    ) -> bool {
        let sfo_path = game_root.join("PARAM.SFO");
        let eboot_path = game_root.join("USRDIR").join("EBOOT.BIN");

        if !sfo_path.is_file() || !eboot_path.is_file() {
            return false;
        }

        let canonical_eboot = match SecurityManager::validate_existing_file(&eboot_path) {
            Ok(path) => path,
            Err(err) => {
                diagnostics.push(format!(
                    "EBOOT.BIN inacessível em '{}': {err}",
                    eboot_path.display()
                ));
                return true;
            }
        };

        if !seen_eboots.insert(canonical_eboot.clone()) {
            return true;
        }

        let bytes = match fs::read(&sfo_path) {
            Ok(data) => data,
            Err(err) => {
                diagnostics.push(format!(
                    "Falha ao ler PARAM.SFO em '{}': {err}",
                    sfo_path.display()
                ));
                return true;
            }
        };

        let meta = match parse_param_sfo(&bytes) {
            Ok(parsed) => parsed,
            Err(err) => {
                diagnostics.push(format!(
                    "PARAM.SFO inválido/corrompido ignorado em '{}': {err}",
                    sfo_path.display()
                ));
                return true;
            }
        };

        let title = meta
            .title
            .filter(|t| !t.trim().is_empty())
            .or_else(|| {
                game_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "PS3 Title".to_string());

        let serial_code = meta.title_id.filter(|s| !s.trim().is_empty());
        let region = serial_code
            .as_deref()
            .map(infer_ps3_region)
            .unwrap_or("UNKNOWN")
            .to_string();
        let version_label = meta
            .app_ver
            .or(meta.version)
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "default".to_string());

        on_progress(ScanProgressEvent::GameDiscovered {
            title: title.clone(),
            serial_code: serial_code.clone(),
            file_path: canonical_eboot.to_string_lossy().to_string(),
        });

        candidates.push(DiscoveredGameCandidate {
            console_slug: "ps3".to_string(),
            title,
            serial_code,
            region,
            version_label,
            file_path: canonical_eboot,
            metadata_source: "ps3_param_sfo".to_string(),
        });

        true
    }

    #[allow(clippy::too_many_arguments)]
    fn walk_dir(
        dir: &Path,
        recursive: bool,
        depth: usize,
        files_checked: &mut usize,
        seen_eboots: &mut BTreeSet<PathBuf>,
        candidates: &mut Vec<DiscoveredGameCandidate>,
        diagnostics: &mut Vec<String>,
        on_progress: &mut dyn FnMut(ScanProgressEvent),
    ) {
        if depth > MAX_SCAN_DEPTH {
            return;
        }

        on_progress(ScanProgressEvent::ScanningDirectory {
            directory_path: dir.to_string_lossy().to_string(),
            files_checked: *files_checked,
        });

        // Direct check if this directory itself is a PS3_GAME or HDD game folder
        if Self::inspect_ps3_game_dir(dir, seen_eboots, candidates, diagnostics, on_progress) {
            *files_checked += 2;
            return;
        }

        // Also check direct child `PS3_GAME/` even when recursive == false (disc folder root)
        let ps3_game_child = dir.join("PS3_GAME");
        if ps3_game_child.is_dir()
            && Self::inspect_ps3_game_dir(
                &ps3_game_child,
                seen_eboots,
                candidates,
                diagnostics,
                on_progress,
            )
        {
            *files_checked += 2;
            return;
        }

        let read_dir = match fs::read_dir(dir) {
            Ok(iter) => iter,
            Err(err) => {
                diagnostics.push(format!("Diretório inacessível '{}': {err}", dir.display()));
                return;
            }
        };

        let mut subdirs = Vec::new();
        for entry_res in read_dir {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    diagnostics.push(format!("Erro ao enumerar '{}': {err}", dir.display()));
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_file() {
                *files_checked += 1;
            } else if file_type.is_dir() {
                subdirs.push(entry.path());
            }
        }

        subdirs.sort();
        for sub in subdirs {
            if recursive {
                Self::walk_dir(
                    &sub,
                    true,
                    depth + 1,
                    files_checked,
                    seen_eboots,
                    candidates,
                    diagnostics,
                    on_progress,
                );
            } else {
                // Non-recursive: still inspect immediate game folder children (`<root>/<Game>/PS3_GAME`)
                let child_ps3_game = sub.join("PS3_GAME");
                if child_ps3_game.is_dir() {
                    if Self::inspect_ps3_game_dir(
                        &child_ps3_game,
                        seen_eboots,
                        candidates,
                        diagnostics,
                        on_progress,
                    ) {
                        *files_checked += 2;
                    }
                } else if Self::inspect_ps3_game_dir(
                    &sub,
                    seen_eboots,
                    candidates,
                    diagnostics,
                    on_progress,
                ) {
                    *files_checked += 2;
                }
            }
        }
    }
}

impl GameDiscoveryPort for Ps3GameDiscovery {
    fn console_slug(&self) -> &'static str {
        "ps3"
    }

    fn discover_in_directory(
        &self,
        root_dir: &Path,
        recursive: bool,
        on_progress: &mut dyn FnMut(ScanProgressEvent),
    ) -> Result<DiscoveryBatch, DomainError> {
        if !root_dir.is_dir() {
            return Err(DomainError::NotFound(format!(
                "scan directory not found: '{}'",
                root_dir.display()
            )));
        }

        let mut files_checked = 0usize;
        let mut seen_eboots = BTreeSet::new();
        let mut candidates = Vec::new();
        let mut diagnostics = Vec::new();

        Self::walk_dir(
            root_dir,
            recursive,
            0,
            &mut files_checked,
            &mut seen_eboots,
            &mut candidates,
            &mut diagnostics,
            on_progress,
        );

        candidates.sort_by(|a, b| a.title.cmp(&b.title).then(a.file_path.cmp(&b.file_path)));

        Ok(DiscoveryBatch {
            files_checked,
            candidates,
            diagnostics,
        })
    }
}
