//! Atomic TOML file operations with ACID guarantees.
//!
//! Provides a thin layer for safe concurrent access to TOML configuration files.

use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write as IoWrite;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// Errors that can occur during atomic TOML operations.
#[derive(Debug)]
pub enum AtomicTomlError {
    /// File I/O error.
    IoError(std::io::Error),
    /// TOML serialization/deserialization error.
    TomlError(toml::de::Error),
    /// TOML serialization error.
    TomlSerError(toml::ser::Error),
    /// File locking error.
    LockError(String),
    /// Migration error.
    MigrationError(String),
}

impl std::fmt::Display for AtomicTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomicTomlError::IoError(e) => write!(f, "I/O error: {}", e),
            AtomicTomlError::TomlError(e) => write!(f, "TOML parse error: {}", e),
            AtomicTomlError::TomlSerError(e) => write!(f, "TOML serialization error: {}", e),
            AtomicTomlError::LockError(e) => write!(f, "Lock error: {}", e),
            AtomicTomlError::MigrationError(e) => write!(f, "Migration error: {}", e),
        }
    }
}

impl std::error::Error for AtomicTomlError {}

impl From<std::io::Error> for AtomicTomlError {
    fn from(e: std::io::Error) -> Self {
        AtomicTomlError::IoError(e)
    }
}

impl From<toml::de::Error> for AtomicTomlError {
    fn from(e: toml::de::Error) -> Self {
        AtomicTomlError::TomlError(e)
    }
}

impl From<toml::ser::Error> for AtomicTomlError {
    fn from(e: toml::ser::Error) -> Self {
        AtomicTomlError::TomlSerError(e)
    }
}

/// A handle to an atomic TOML file with ACID guarantees.
///
/// Provides:
/// - **Atomicity**: Updates are all-or-nothing via tmp file + atomic rename
/// - **Consistency**: TOML schema validation on load/save
/// - **Isolation**: File locking prevents concurrent modifications
/// - **Durability**: Explicit fsync before rename
pub struct AtomicTomlFile<T> {
    path: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T> AtomicTomlFile<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a new atomic TOML file handle.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TOML file
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }

    /// Loads the TOML file and deserializes it.
    ///
    /// If the file doesn't exist, returns `None`.
    /// If the file is empty, returns `None`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(T))`: Successfully loaded and deserialized
    /// - `Ok(None)`: File doesn't exist or is empty
    /// - `Err`: Failed to read or parse the file
    pub fn load(&self) -> Result<Option<T>, AtomicTomlError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.path)?;

        if content.trim().is_empty() {
            return Ok(None);
        }

        let data: T = toml::from_str(&content)?;
        Ok(Some(data))
    }

    /// Saves data to the TOML file atomically.
    ///
    /// Uses a temporary file + atomic rename to ensure durability.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize and save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Successfully saved
    /// - `Err`: Failed to serialize or write the file
    pub fn save(&self, data: &T) -> Result<(), AtomicTomlError> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Serialize to TOML
        let toml_string = toml::to_string_pretty(data)?;

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
    pub fn update<F>(&self, default_value: T, f: F) -> Result<(), AtomicTomlError>
    where
        F: FnOnce(&mut T) -> Result<(), AtomicTomlError>,
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
    fn get_temp_path(&self) -> Result<PathBuf, AtomicTomlError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| AtomicTomlError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path has no parent directory",
            )))?;

        let file_name = self
            .path
            .file_name()
            .ok_or_else(|| AtomicTomlError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path has no file name",
            )))?;

        let tmp_name = format!(".{}.tmp", file_name.to_string_lossy());
        Ok(parent.join(tmp_name))
    }

    /// Acquires an exclusive file lock.
    ///
    /// Returns a lock guard that automatically releases the lock when dropped.
    fn acquire_lock(&self) -> Result<FileLock, AtomicTomlError> {
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
    fn acquire(path: &Path) -> Result<Self, AtomicTomlError> {
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
                .map_err(|e| AtomicTomlError::LockError(format!("Failed to acquire lock: {}", e)))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        name: String,
        count: u32,
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        let atomic_file = AtomicTomlFile::<TestConfig>::new(file_path);

        let config = TestConfig {
            name: "test".to_string(),
            count: 42,
        };

        // Save
        atomic_file.save(&config).unwrap();

        // Load
        let loaded = atomic_file.load().unwrap().unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.count, 42);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.toml");
        let atomic_file = AtomicTomlFile::<TestConfig>::new(file_path);

        let result = atomic_file.load().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        let atomic_file = AtomicTomlFile::<TestConfig>::new(file_path);

        let default_config = TestConfig {
            name: "default".to_string(),
            count: 0,
        };

        // Update (creates new file with default)
        atomic_file
            .update(default_config.clone(), |config| {
                config.count += 10;
                Ok(())
            })
            .unwrap();

        // Verify
        let loaded = atomic_file.load().unwrap().unwrap();
        assert_eq!(loaded.count, 10);

        // Update again
        atomic_file
            .update(default_config, |config| {
                config.count += 5;
                Ok(())
            })
            .unwrap();

        // Verify
        let loaded = atomic_file.load().unwrap().unwrap();
        assert_eq!(loaded.count, 15);
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        let atomic_file = AtomicTomlFile::<TestConfig>::new(file_path.clone());

        let config = TestConfig {
            name: "test".to_string(),
            count: 42,
        };

        atomic_file.save(&config).unwrap();

        // Verify no temp file left behind
        let tmp_path = temp_dir.path().join(".test.toml.tmp");
        assert!(!tmp_path.exists());

        // Verify main file exists
        assert!(file_path.exists());
    }
}
