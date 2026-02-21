//! Error types for OpenAPI parser

use thiserror::Error;

/// Result type for OpenAPI parser operations
pub type Result<T> = std::result::Result<T, OpenApiError>;

/// Errors that can occur during OpenAPI parsing
#[derive(Error, Debug)]
pub enum OpenApiError {
    /// Failed to parse YAML
    #[error("Failed to parse YAML: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    /// Failed to parse JSON
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Invalid OpenAPI specification
    #[error("Invalid OpenAPI specification: {0}")]
    InvalidSpec(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Unsupported schema type
    #[error("Unsupported schema type: {0}")]
    UnsupportedType(String),

    /// Reference resolution failed
    #[error("Failed to resolve reference '{reference}': {reason}")]
    ReferenceResolution {
        /// The reference that couldn't be resolved
        reference: String,
        /// Why it failed
        reason: String,
    },

    /// Circular reference detected
    #[error("Circular reference detected: {0}")]
    CircularReference(String),

    /// Invalid schema composition
    #[error("Invalid schema composition (allOf/oneOf/anyOf): {0}")]
    InvalidComposition(String),

    /// HTTP fetch error
    #[cfg(feature = "openapi")]
    #[error("Failed to fetch OpenAPI spec from URL: {0}")]
    FetchError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl OpenApiError {
    /// Create a new invalid spec error
    pub fn invalid_spec(msg: impl Into<String>) -> Self {
        Self::InvalidSpec(msg.into())
    }

    /// Create a new missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField(field.into())
    }

    /// Create a new unsupported type error
    pub fn unsupported_type(type_name: impl Into<String>) -> Self {
        Self::UnsupportedType(type_name.into())
    }

    /// Create a new reference resolution error
    pub fn reference_resolution(reference: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ReferenceResolution {
            reference: reference.into(),
            reason: reason.into(),
        }
    }

    /// Create a new circular reference error
    pub fn circular_reference(path: impl Into<String>) -> Self {
        Self::CircularReference(path.into())
    }

    /// Create a new invalid composition error
    pub fn invalid_composition(msg: impl Into<String>) -> Self {
        Self::InvalidComposition(msg.into())
    }
}
