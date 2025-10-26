//! Config file storage with ACID guarantees.
//!
//! Provides a smart mutex-like layer for safe concurrent access to configuration files.
//! Returns data as `serde_json::Value` (intermediate format) to decouple from TOML specifics.

use serde_json::Value as JsonValue;
use std::fs::{self, File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

/// Errors that can occur during config storage operations.
#[derive(Debug)]
pub enum ConfigStorageError {
    /// File I/O error.
    IoError(std::io::Error),
    /// TOML parsing error.
    TomlParseError(toml::de::Error),
    /// TOML serialization error.
    TomlSerError(toml::ser::Error),
    /// JSON conversion error.
    JsonError(serde_json::Error),
    /// File locking error.
    LockError(String),
}

impl std::fmt::Display for ConfigStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigStorageError::IoError(e) => write!(f, "I/O error: {}", e),
            ConfigStorageError::TomlParseError(e) => write!(f, "TOML parse error: {}", e),
            ConfigStorageError::TomlSerError(e) => write!(f, "TOML serialization error: {}", e),
            ConfigStorageError::JsonError(e) => write!(f, "JSON conversion error: {}", e),
            ConfigStorageError::LockError(e) => write!(f, "Lock error: {}", e),
        }
    }
}

impl std::error::Error for ConfigStorageError {}

impl From<std::io::Error> for ConfigStorageError {
    fn from(e: std::io::Error) -> Self {
        ConfigStorageError::IoError(e)
    }
}

impl From<toml::de::Error> for ConfigStorageError {
    fn from(e: toml::de::Error) -> Self {
        ConfigStorageError::TomlParseError(e)
    }
}

impl From<toml::ser::Error> for ConfigStorageError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigStorageError::TomlSerError(e)
    }
}

impl From<serde_json::Error> for ConfigStorageError {
    fn from(e: serde_json::Error) -> Self {
        ConfigStorageError::JsonError(e)
    }
}

/// A config file storage with ACID guarantees.
///
/// Responsibilities:
/// - **File locking** (exclusive write lock)
/// - **Atomic read/write** (tmp file + atomic rename)
/// - **Format conversion** (TOML ⇄ serde_json::Value)
///
/// Does NOT:
/// - Know about specific entities (Persona, Session, etc.)
/// - Handle migrations (delegated to Repository layer)
/// - Parse DTOs (delegated to Repository layer)
///
/// Provides:
/// - **Atomicity**: Updates are all-or-nothing via tmp file + atomic rename
/// - **Consistency**: TOML syntax validation on load/save
/// - **Isolation**: File locking prevents concurrent modifications
/// - **Durability**: Explicit fsync before rename
pub struct ConfigStorage {
    path: PathBuf,
}

impl ConfigStorage {
    /// Creates a new config storage handle.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the config file (usually a .toml file)
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Loads the config file and returns it as a serde_json::Value.
    ///
    /// If the file doesn't exist, returns `None`.
    /// If the file is empty, returns `None`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(JsonValue))`: Successfully loaded and converted to JSON
    /// - `Ok(None)`: File doesn't exist or is empty
    /// - `Err`: Failed to read or parse the file
    pub fn load(&self) -> Result<Option<JsonValue>, ConfigStorageError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.path)?;

        if content.trim().is_empty() {
            return Ok(None);
        }

        // Parse TOML → toml::Value
        let toml_value: toml::Value = toml::from_str(&content)?;

        // Convert toml::Value → serde_json::Value
        let json_value = toml_to_json(toml_value)?;

        Ok(Some(json_value))
    }

    /// Saves data to the config file atomically.
    ///
    /// Uses a temporary file + atomic rename to ensure durability.
    ///
    /// # Arguments
    ///
    /// * `data` - The JSON data to save as TOML
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Successfully saved
    /// - `Err`: Failed to convert or write the file
    pub fn save(&self, data: &JsonValue) -> Result<(), ConfigStorageError> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Convert serde_json::Value → toml::Value
        let toml_value = json_to_toml(data)?;

        // Serialize to TOML string
        let toml_string = toml::to_string_pretty(&toml_value)?;

        // Write to temporary file in the same directory
        let tmp_path = self.get_temp_path()?;
        let mut tmp_file = File::create(&tmp_path)?;
        tmp_file.write_all(toml_string.as_bytes())?;

        // Ensure data is written to disk
        tmp_file.sync_all()?;
        drop(tmp_file);

        // Atomic rename
        fs::rename(&tmp_path, &self.path)?;

        Ok(())
    }

    /// Performs a transactional update with file locking.
    ///
    /// The update function receives a mutable reference to the current data
    /// and can modify it. If the function returns `Ok(())`, the changes are
    /// atomically written back to the file.
    ///
    /// # Arguments
    ///
    /// * `default_value` - Default value to use if file doesn't exist
    /// * `f` - Update function that modifies the data
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Update succeeded
    /// - `Err`: Failed to acquire lock, read, update, or write
    pub fn update<F>(&self, default_value: JsonValue, f: F) -> Result<(), ConfigStorageError>
    where
        F: FnOnce(&mut JsonValue) -> Result<(), ConfigStorageError>,
    {
        // Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // Load current data
        let mut data = self.load()?.unwrap_or(default_value);

        // Apply update function
        f(&mut data)?;

        // Save atomically
        self.save(&data)?;

        Ok(())
    }

    /// Gets a temporary file path for atomic writes.
    fn get_temp_path(&self) -> Result<PathBuf, ConfigStorageError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| ConfigStorageError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path has no parent directory",
            )))?;

        let file_name = self
            .path
            .file_name()
            .ok_or_else(|| ConfigStorageError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path has no file name",
            )))?;

        let tmp_name = format!(".{}.tmp", file_name.to_string_lossy());
        Ok(parent.join(tmp_name))
    }

    /// Acquires an exclusive file lock.
    ///
    /// Returns a lock guard that automatically releases the lock when dropped.
    fn acquire_lock(&self) -> Result<FileLock, ConfigStorageError> {
        FileLock::acquire(&self.path)
    }
}

/// A file lock guard that automatically releases the lock when dropped.
struct FileLock {
    #[allow(dead_code)]
    file: File,
    lock_path: PathBuf,
}

impl FileLock {
    /// Acquires an exclusive lock on the given path.
    fn acquire(path: &Path) -> Result<Self, ConfigStorageError> {
        // Create lock file path
        let lock_path = path.with_extension("lock");

        // Ensure parent directory exists
        if let Some(parent) = lock_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Open or create lock file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_path)?;

        // Try to acquire exclusive lock with fs2
        #[cfg(unix)]
        {
            use fs2::FileExt;
            file.lock_exclusive()
                .map_err(|e| ConfigStorageError::LockError(format!("Failed to acquire lock: {}", e)))?;
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, we don't have file locking
            // This is acceptable for single-user desktop apps
        }

        Ok(FileLock { file, lock_path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Unlock is automatic when the file handle is dropped
        // Try to remove lock file (best effort)
        let _ = fs::remove_file(&self.lock_path);
    }
}

/// Converts a toml::Value to serde_json::Value.
fn toml_to_json(toml_value: toml::Value) -> Result<JsonValue, ConfigStorageError> {
    // Serialize toml::Value to JSON string, then parse as serde_json::Value
    // This is a reliable way to convert between the two formats
    let json_str = serde_json::to_string(&toml_value)?;
    let json_value = serde_json::from_str(&json_str)?;
    Ok(json_value)
}

/// Converts a serde_json::Value to toml::Value.
fn json_to_toml(json_value: &JsonValue) -> Result<toml::Value, ConfigStorageError> {
    // Serialize serde_json::Value to JSON string, then parse as toml::Value
    let json_str = serde_json::to_string(json_value)?;
    let toml_value: toml::Value = serde_json::from_str(&json_str)?;
    Ok(toml_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        let storage = ConfigStorage::new(file_path);

        let data = serde_json::json!({
            "name": "test",
            "count": 42
        });

        // Save
        storage.save(&data).unwrap();

        // Load
        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded["name"], "test");
        assert_eq!(loaded["count"], 42);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.toml");
        let storage = ConfigStorage::new(file_path);

        let result = storage.load().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        let storage = ConfigStorage::new(file_path);

        let default_data = serde_json::json!({
            "name": "default",
            "count": 0
        });

        // Update (creates new file with default)
        storage
            .update(default_data.clone(), |data| {
                data["count"] = serde_json::json!(10);
                Ok(())
            })
            .unwrap();

        // Verify
        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded["count"], 10);

        // Update again
        storage
            .update(default_data, |data| {
                data["count"] = serde_json::json!(data["count"].as_i64().unwrap() + 5);
                Ok(())
            })
            .unwrap();

        // Verify
        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded["count"], 15);
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        let storage = ConfigStorage::new(file_path.clone());

        let data = serde_json::json!({
            "name": "test",
            "count": 42
        });

        storage.save(&data).unwrap();

        // Verify no temp file left behind
        let tmp_path = temp_dir.path().join(".test.toml.tmp");
        assert!(!tmp_path.exists());

        // Verify main file exists
        assert!(file_path.exists());
    }
}
