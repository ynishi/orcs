//! Error types for the Orcs application.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A shared error type for the entire Orcs application.
///
/// Each crate will have its own specific error enum, but this type can be used
/// for common errors or as a top-level error container if needed.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum OrcsError {
    #[error("An unknown error has occurred.")]
    Unknown,

    #[error("IO error: {0}")]
    Io(String),

    #[error("Data Access error: {0}")]
    DataAccess(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Task execution error: {0}")]
    Execution(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// A type alias for `Result<T, OrcsError>`.
pub type Result<T> = std::result::Result<T, OrcsError>;
