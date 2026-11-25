use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::error::DomainError;
use crate::domain::scanner::{
    DiscoveredGameCandidate, DiscoveryBatch, GameDiscoveryPort, ScanProgressEvent,
};
use crate::infrastructure::system::security::SecurityManager;

const MAX_SCAN_DEPTH: usize = 16;

#[derive(Debug, Clone, Copy)]
enum MatchStrategy {
    Extensions(&'static [&'static str]),
    ExactFileName(&'static str),
}

pub struct PlatformExtensionDiscovery {
    slug: &'static str,
    strategy: MatchStrategy,
}

impl PlatformExtensionDiscovery {
    pub const fn for_ps2() -> Self {
        Self {
            slug: "ps2",
            strategy: MatchStrategy::Extensions(&["iso", "chd", "cso", "gz", "bin"]),
        }
    }

    pub const fn for_wii() -> Self {
        Self {
            slug: "wii",
            strategy: MatchStrategy::Extensions(&["rvz", "wbfs", "iso", "gcz", "ciso", "wia"]),
        }
    }

    pub const fn for_xbox() -> Self {
        Self {
            slug: "xbox",
            strategy: MatchStrategy::Extensions(&["xiso", "iso"]),
        }
    }

    pub const fn for_xbox360() -> Self {
        Self {
            slug: "xbox360",
            strategy: MatchStrategy::Extensions(&["xex", "iso"]),
        }
    }

    pub const fn for_ps4() -> Self {
        Self {
            slug: "ps4",
            strategy: MatchStrategy::ExactFileName("eboot.bin"),
        }
    }

    fn matches_file(&self, path: &Path) -> bool {
        match self.strategy {
            MatchStrategy::Extensions(exts) => path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| exts.contains(&e.to_ascii_lowercase().as_str()))
                .unwrap_or(false),
            MatchStrategy::ExactFileName(expected) => path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case(expected))
                .unwrap_or(false),
        }
    }

    fn derive_title(&self, path: &Path) -> String {
        match self.strategy {
            MatchStrategy::ExactFileName(_) => path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("Untitled")
                .to_string(),
            MatchStrategy::Extensions(_) => path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("Untitled")
                .to_string(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn walk_dir(
        &self,
        dir: &Path,
        recursive: bool,
        depth: usize,
        files_checked: &mut usize,
        seen: &mut BTreeSet<PathBuf>,
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
            let path = entry.path();
            if file_type.is_dir() {
                if recursive {
                    subdirs.push(path);
                }
                continue;
            }
            if !file_type.is_file() {
                continue;
            }

            *files_checked += 1;
            if !self.matches_file(&path) {
                continue;
            }

            let canonical = match SecurityManager::validate_existing_file(&path) {
                Ok(p) => p,
                Err(err) => {
                    diagnostics.push(format!("Arquivo inacessível '{}': {err}", path.display()));
                    continue;
                }
            };

            if !seen.insert(canonical.clone()) {
                continue;
            }

            let title = self.derive_title(&canonical);
            on_progress(ScanProgressEvent::GameDiscovered {
                title: title.clone(),
                serial_code: None,
                file_path: canonical.to_string_lossy().to_string(),
            });

            candidates.push(DiscoveredGameCandidate {
                console_slug: self.slug.to_string(),
                title,
                serial_code: None,
                region: "UNKNOWN".to_string(),
                version_label: "default".to_string(),
                file_path: canonical,
                metadata_source: "filesystem_filename_scan".to_string(),
            });
        }

        subdirs.sort();
        for sub in subdirs {
            self.walk_dir(
                &sub,
                recursive,
                depth + 1,
                files_checked,
                seen,
                candidates,
                diagnostics,
                on_progress,
            );
        }
    }
}

impl GameDiscoveryPort for PlatformExtensionDiscovery {
    fn console_slug(&self) -> &'static str {
        self.slug
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
        let mut seen = BTreeSet::new();
        let mut candidates = Vec::new();
        let mut diagnostics = Vec::new();

        self.walk_dir(
            root_dir,
            recursive,
            0,
            &mut files_checked,
            &mut seen,
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
