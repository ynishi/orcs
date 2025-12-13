//! Error types for the Orcs application.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A shared error type for the entire Orcs application.
///
/// This provides typed, structured error variants with automatic conversion
/// from common error types via the `From` trait.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum OrcsError {
    /// Entity not found error with type information
    #[error("Entity not found: {entity_type} '{id}'")]
    NotFound {
        entity_type: &'static str,
        id: String,
    },

    /// IO error (file system operations)
    #[error("IO error: {message}")]
    Io { message: String },

    /// Data access error (repository/storage layer)
    #[error("Data access error: {0}")]
    DataAccess(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {format} - {message}")]
    Serialization {
        format: String, // "TOML", "JSON", etc.
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Security/authentication error
    #[error("Security error: {0}")]
    Security(String),

    /// Task execution error
    #[error("Task execution error: {0}")]
    Execution(String),

    /// Internal error (should not happen in normal operation)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Multiple errors
    #[error("Multiple errors occurred ({} total)", .0.len())]
    Multiple(Vec<OrcsError>),
}

impl OrcsError {
    // ============================================================================
    // Constructor helpers
    // ============================================================================

    /// Creates a NotFound error
    pub fn not_found(entity_type: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type,
            id: id.into(),
        }
    }

    /// Creates an IO error
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    /// Creates a Config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Creates a DataAccess error
    pub fn data_access(message: impl Into<String>) -> Self {
        Self::DataAccess(message.into())
    }

    /// Creates an Internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Creates a Migration error
    pub fn migration(message: impl Into<String>) -> Self {
        Self::Migration(message.into())
    }

    // ============================================================================
    // Type checking methods
    // ============================================================================

    /// Check if this is a NotFound error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Check if this is an IO error
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io { .. })
    }

    /// Check if this is a serialization error
    pub fn is_serialization(&self) -> bool {
        matches!(self, Self::Serialization { .. })
    }

    /// Check if this is a config error
    pub fn is_config(&self) -> bool {
        matches!(self, Self::Config(_))
    }

    /// Check if this error indicates a file/entity was not found.
    ///
    /// Returns true for:
    /// - `NotFound` errors
    /// - `Io` errors with "File not found" or "not found" in the message
    ///
    /// This helper centralizes the logic for detecting "not found" conditions
    /// across different error types.
    pub fn is_not_found_or_missing(&self) -> bool {
        match self {
            Self::NotFound { .. } => true,
            Self::Io { message } => {
                let lower = message.to_lowercase();
                lower.contains("file not found") || lower.contains("not found")
            }
            _ => false,
        }
    }
}

// ============================================================================
// From implementations for automatic conversion
// ============================================================================

impl From<std::io::Error> for OrcsError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: format!("{} (kind: {:?})", err, err.kind()),
        }
    }
}

impl From<serde_json::Error> for OrcsError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            format: "JSON".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<toml::de::Error> for OrcsError {
    fn from(err: toml::de::Error) -> Self {
        Self::Serialization {
            format: "TOML".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<toml::ser::Error> for OrcsError {
    fn from(err: toml::ser::Error) -> Self {
        Self::Serialization {
            format: "TOML".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<version_migrate::MigrationError> for OrcsError {
    fn from(err: version_migrate::MigrationError) -> Self {
        use version_migrate::MigrationError;

        match err {
            MigrationError::EntityNotFound(id) => Self::not_found("entity", id),
            MigrationError::DeserializationError(_) => Self::Serialization {
                format: "migration".to_string(),
                message: err.to_string(),
            },
            MigrationError::SerializationError(_) => Self::Serialization {
                format: "migration".to_string(),
                message: err.to_string(),
            },
            MigrationError::TomlParseError(_) | MigrationError::TomlSerializeError(_) => {
                Self::Serialization {
                    format: "TOML".to_string(),
                    message: err.to_string(),
                }
            }
            MigrationError::IoError { .. } => Self::Io {
                message: err.to_string(),
            },
            _ => Self::Migration(err.to_string()),
        }
    }
}

/// Conversion from anyhow::Error (transitional, should be removed eventually)
impl From<anyhow::Error> for OrcsError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

/// Conversion from String (for error messages)
impl From<String> for OrcsError {
    fn from(err: String) -> Self {
        Self::Internal(err)
    }
}

/// A type alias for `Result<T, OrcsError>`.
pub type Result<T> = std::result::Result<T, OrcsError>;
