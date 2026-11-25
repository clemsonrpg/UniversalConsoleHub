use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("entity not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("firmware or system file missing: {0}")]
    FirmwareMissing(String),
    #[error(
        "immutable origin violation: cannot change trophy origin from {existing} to {attempted}"
    )]
    ImmutableTrophyOrigin { existing: String, attempted: String },
    #[error("unsafe command rejected: {0}")]
    UnsafeCommand(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("process error: {0}")]
    Process(String),
}

impl Serialize for DomainError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
