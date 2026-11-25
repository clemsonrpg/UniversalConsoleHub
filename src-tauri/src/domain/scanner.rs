use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredGameCandidate {
    pub console_slug: String,
    pub title: String,
    pub serial_code: Option<String>,
    pub region: String,
    pub version_label: String,
    pub file_path: PathBuf,
    pub metadata_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum ScanProgressEvent {
    Started {
        console_slug: String,
        total_directories: usize,
    },
    ScanningDirectory {
        directory_path: String,
        files_checked: usize,
    },
    GameDiscovered {
        title: String,
        serial_code: Option<String>,
        file_path: String,
    },
    Completed {
        scanned_files: usize,
        games_added: usize,
        versions_added: usize,
        skipped_existing: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummaryDto {
    pub console_slug: String,
    pub scanned_directories: usize,
    pub scanned_files: usize,
    pub games_added: usize,
    pub versions_added: usize,
    pub skipped_existing: usize,
    pub errors: Vec<String>,
}

pub struct DiscoveryBatch {
    pub files_checked: usize,
    pub candidates: Vec<DiscoveredGameCandidate>,
    pub diagnostics: Vec<String>,
}

pub trait GameDiscoveryPort: Send + Sync {
    fn console_slug(&self) -> &'static str;

    fn discover_in_directory(
        &self,
        root_dir: &Path,
        recursive: bool,
        on_progress: &mut dyn FnMut(ScanProgressEvent),
    ) -> Result<DiscoveryBatch, DomainError>;
}
