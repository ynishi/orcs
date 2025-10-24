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
}
