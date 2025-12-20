//! Error types for manifest parsing.

use thiserror::Error;

/// Errors that can occur when parsing manifests.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// IO error reading manifest file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Invalid manifest format
    #[error("Invalid manifest format: {0}")]
    InvalidFormat(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid version string
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}
