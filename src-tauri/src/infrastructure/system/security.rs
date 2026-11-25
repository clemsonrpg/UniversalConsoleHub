use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::domain::error::DomainError;

pub struct SecurityManager;

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(s) = path.to_str() {
            if let Some(unc) = s.strip_prefix(r"\\?\UNC\") {
                return PathBuf::from(format!(r"\\{unc}"));
            }
            if let Some(drive) = s.strip_prefix(r"\\?\") {
                return PathBuf::from(drive);
            }
        }
        path
    }
    #[cfg(not(windows))]
    {
        path
    }
}

impl SecurityManager {
    pub fn validate_existing_file(path: &Path) -> Result<PathBuf, DomainError> {
        let meta = fs::metadata(path).map_err(|e| {
            DomainError::NotFound(format!("file '{}' not accessible: {e}", path.display()))
        })?;
        if !meta.is_file() {
            return Err(DomainError::Validation(format!(
                "path '{}' is not a regular file",
                path.display()
            )));
        }
        fs::canonicalize(path)
            .map(strip_windows_verbatim_prefix)
            .map_err(|e| {
                DomainError::Io(format!("failed to canonicalize '{}': {e}", path.display()))
            })
    }

    pub fn ensure_within_base(base_dir: &Path, candidate: &Path) -> Result<PathBuf, DomainError> {
        let canonical_base = fs::canonicalize(base_dir).map_err(|e| {
            DomainError::Io(format!(
                "base directory '{}' cannot be canonicalized: {e}",
                base_dir.display()
            ))
        })?;
        let joined = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            canonical_base.join(candidate)
        };
        let resolved = if joined.exists() {
            fs::canonicalize(&joined).map_err(|e| {
                DomainError::Io(format!(
                    "failed to canonicalize '{}': {e}",
                    joined.display()
                ))
            })?
        } else {
            // Normalize parent if it exists, then check lexical components for traversal
            let normalized_base = normalize_lexical(&canonical_base);
            let normalized_target = if candidate.is_absolute() {
                // Canonicalize existing ancestor to handle Windows \\?\ prefixes cleanly
                let mut ancestor = joined.as_path();
                let mut suffix_parts = Vec::new();
                while !ancestor.exists() {
                    if let Some(name) = ancestor.file_name() {
                        suffix_parts.push(name.to_os_string());
                    } else {
                        break;
                    }
                    match ancestor.parent() {
                        Some(p) => ancestor = p,
                        None => break,
                    }
                }
                let mut rebuilt = if ancestor.exists() {
                    fs::canonicalize(ancestor).unwrap_or_else(|_| ancestor.to_path_buf())
                } else {
                    normalize_lexical(&joined)
                };
                for part in suffix_parts.into_iter().rev() {
                    rebuilt.push(part);
                }
                // Reject any explicit `..` components on non-existent target
                if candidate
                    .components()
                    .any(|c| matches!(c, Component::ParentDir))
                {
                    let lex = normalize_lexical(&joined);
                    let plain_base = normalize_lexical(base_dir);
                    if !lex.starts_with(&plain_base) {
                        return Err(DomainError::Validation(format!(
                            "path '{}' escapes allowed base directory '{}'",
                            candidate.display(),
                            base_dir.display()
                        )));
                    }
                }
                rebuilt
            } else {
                let mut target = normalized_base.clone();
                for comp in candidate.components() {
                    match comp {
                        Component::CurDir => {}
                        Component::ParentDir => {
                            if !target.pop() || !target.starts_with(&normalized_base) {
                                return Err(DomainError::Validation(format!(
                                    "path '{}' escapes allowed base directory '{}'",
                                    candidate.display(),
                                    base_dir.display()
                                )));
                            }
                        }
                        Component::Normal(p) => target.push(p),
                        _ => {}
                    }
                }
                target
            };
            normalized_target
        };

        if !resolved.starts_with(&canonical_base) {
            return Err(DomainError::Validation(format!(
                "path '{}' escapes allowed base directory '{}'",
                candidate.display(),
                base_dir.display()
            )));
        }
        Ok(strip_windows_verbatim_prefix(resolved))
    }

    pub fn compute_sha256_streaming(path: &Path) -> Result<(String, u64), DomainError> {
        let valid_path = Self::validate_existing_file(path)?;
        let file = File::open(&valid_path).map_err(|e| {
            DomainError::Io(format!("failed to open '{}': {e}", valid_path.display()))
        })?;
        let mut reader = BufReader::with_capacity(64 * 1024, file);
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];
        let mut total_bytes = 0u64;

        loop {
            let read_len = reader.read(&mut buf).map_err(|e| {
                DomainError::Io(format!(
                    "failed reading '{}' during SHA-256: {e}",
                    valid_path.display()
                ))
            })?;
            if read_len == 0 {
                break;
            }
            hasher.update(&buf[..read_len]);
            total_bytes += read_len as u64;
        }

        let digest = hasher.finalize();
        let mut hex_out = String::with_capacity(digest.len() * 2);
        for byte in digest {
            hex_out.push_str(&format!("{byte:02x}"));
        }
        Ok((hex_out, total_bytes))
    }
}
