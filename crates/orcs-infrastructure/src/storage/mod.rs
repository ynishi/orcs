//! Storage layer for atomic file operations.

mod atomic_toml;
mod config_storage;

pub use atomic_toml::{AtomicTomlError, AtomicTomlFile};
pub use config_storage::{ConfigStorage, ConfigStorageError};
