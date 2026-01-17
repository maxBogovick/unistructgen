//! Error types for Schema Registry

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Team not found: {0}")]
    TeamNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Breaking change detected: {0}")]
    BreakingChange(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("{0}")]
    Custom(String),
}

impl Error {
    pub fn custom(msg: impl Into<String>) -> Self {
        Error::Custom(msg.into())
    }
}
