use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::domain::error::DomainError;

use super::security::SecurityManager;

pub const MAX_COVER_BYTES: u64 = 10 * 1024 * 1024;
const ALLOWED_CACHE_EXTENSIONS: &[&str] = &["png", "jpg", "webp"];

pub struct CoverCache;

impl CoverCache {
    pub fn detect_image_extension(header: &[u8]) -> Option<&'static str> {
        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if header.len() >= 8 && header[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return Some("png");
        }
        // JPEG: FF D8 FF
        if header.len() >= 3 && header[0..3] == [0xFF, 0xD8, 0xFF] {
            return Some("jpg");
        }
        // WebP: "RIFF" + 4 bytes size + "WEBP"
        if header.len() >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
            return Some("webp");
        }
        None
    }

    /// Validates an input image strictly by real magic bytes, size (<= 10 MB), and SHA-256,
    /// then stores it deduplicated strictly inside `cache_dir` (`<sha256>.<ext>`).
    pub fn import_local_image(
        source_path: &Path,
        cache_dir: &Path,
    ) -> Result<PathBuf, DomainError> {
        let valid_src = SecurityManager::validate_existing_file(source_path)?;
        let meta = fs::metadata(&valid_src).map_err(|e| {
            DomainError::Io(format!(
                "failed to stat cover image '{}': {e}",
                valid_src.display()
            ))
        })?;
        let size = meta.len();
        if size == 0 {
            return Err(DomainError::Validation(
                "cover image file cannot be empty".to_string(),
            ));
        }
        if size > MAX_COVER_BYTES {
            return Err(DomainError::Validation(format!(
                "cover image exceeds maximum allowed size of {} bytes (got {} bytes)",
                MAX_COVER_BYTES, size
            )));
        }

        let mut file = File::open(&valid_src).map_err(|e| {
            DomainError::Io(format!(
                "failed to open cover image '{}': {e}",
                valid_src.display()
            ))
        })?;
        let mut header = [0u8; 16];
        let read_len = file.read(&mut header).map_err(|e| {
            DomainError::Io(format!(
                "failed to read cover image header '{}': {e}",
                valid_src.display()
            ))
        })?;
        drop(file);

        let ext = Self::detect_image_extension(&header[..read_len]).ok_or_else(|| {
            DomainError::Validation(
                "unsupported or spoofed image format (only genuine PNG, JPEG, or WebP files are accepted)"
                    .to_string(),
            )
        })?;

        let (sha256, hashed_size) = SecurityManager::compute_sha256_streaming(&valid_src)?;
        if hashed_size > MAX_COVER_BYTES {
            return Err(DomainError::Validation(
                "cover image exceeds maximum allowed size of 10 MB".to_string(),
            ));
        }

        fs::create_dir_all(cache_dir).map_err(|e| {
            DomainError::Io(format!(
                "failed to create cover cache directory '{}': {e}",
                cache_dir.display()
            ))
        })?;

        let file_name = format!("{sha256}.{ext}");
        let destination = SecurityManager::ensure_within_base(cache_dir, Path::new(&file_name))?;

        if !destination.is_file() {
            let tmp_name = format!("{sha256}.{ext}.tmp");
            let tmp_dest = SecurityManager::ensure_within_base(cache_dir, Path::new(&tmp_name))?;
            fs::copy(&valid_src, &tmp_dest).map_err(|e| {
                DomainError::Io(format!(
                    "failed to copy cover image into cache '{}': {e}",
                    tmp_dest.display()
                ))
            })?;
            if let Err(err) = fs::rename(&tmp_dest, &destination) {
                let _ = fs::remove_file(&tmp_dest);
                if !destination.is_file() {
                    return Err(DomainError::Io(format!(
                        "failed to finalize cached cover '{}': {err}",
                        destination.display()
                    )));
                }
            }
        }

        Self::validate_authorized_cover_path(&destination, cache_dir)
    }

    /// Enforces Adjustment C: any persisted or returned cover path MUST exist strictly
    /// inside `<app_local_data_dir>/cache/covers/` and carry an allowlisted image extension.
    pub fn validate_authorized_cover_path(
        cover_path: &Path,
        cache_dir: &Path,
    ) -> Result<PathBuf, DomainError> {
        if !cache_dir.is_dir() {
            return Err(DomainError::Validation(
                "authorized cover cache directory does not exist".to_string(),
            ));
        }

        let valid_file = SecurityManager::validate_existing_file(cover_path)?;
        let within = SecurityManager::ensure_within_base(cache_dir, &valid_file)?;

        let ext_ok = within
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| ALLOWED_CACHE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
            .unwrap_or(false);
        if !ext_ok {
            return Err(DomainError::Validation(format!(
                "cover path '{}' does not have an allowed cached image extension",
                within.display()
            )));
        }

        Ok(within)
    }

    /// Returns `Some(normalized_path)` only if the path is confirmed to reside strictly
    /// inside `cache_dir`; otherwise strips it (`None`) so unauthorized paths never reach UI.
    pub fn sanitize_cover_for_frontend(
        raw_cover_path: Option<&str>,
        cache_dir: &Path,
    ) -> Option<String> {
        let raw = raw_cover_path?.trim();
        if raw.is_empty() {
            return None;
        }
        Self::validate_authorized_cover_path(Path::new(raw), cache_dir)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }
}
