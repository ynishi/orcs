//! Secret configuration file storage.
//!
//! Provides secure loading of secret configuration from ~/.config/orcs/secret.json.

use crate::paths::OrcsPaths;
use orcs_core::config::SecretConfig;
use std::fs;
use std::path::PathBuf;

/// Errors that can occur during secret storage operations.
#[derive(Debug)]
pub enum SecretStorageError {
    /// Configuration file not found.
    NotFound(PathBuf),
    /// File I/O error.
    IoError(std::io::Error),
    /// JSON parsing error.
    ParseError(serde_json::Error),
    /// Config directory not found.
    ConfigDirNotFound,
}

impl std::fmt::Display for SecretStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretStorageError::NotFound(path) => {
                write!(f, "Configuration file not found at: {}", path.display())
            }
            SecretStorageError::IoError(e) => write!(f, "I/O error: {}", e),
            SecretStorageError::ParseError(e) => write!(f, "JSON parse error: {}", e),
            SecretStorageError::ConfigDirNotFound => {
                write!(f, "Could not determine home directory")
            }
        }
    }
}

impl std::error::Error for SecretStorageError {}

impl From<std::io::Error> for SecretStorageError {
    fn from(e: std::io::Error) -> Self {
        SecretStorageError::IoError(e)
    }
}

impl From<serde_json::Error> for SecretStorageError {
    fn from(e: serde_json::Error) -> Self {
        SecretStorageError::ParseError(e)
    }
}

/// Storage for secret configuration file (secret.json).
///
/// Responsibilities:
/// - Load secret.json from ~/.config/orcs/
/// - Parse JSON into SecretConfig domain model
/// - Provide error handling for missing or invalid files
///
/// Does NOT:
/// - Write or modify secret files (read-only)
/// - Validate API keys or credentials
/// - Handle encryption (plaintext JSON storage)
///
/// # Security Note
///
/// This storage reads plaintext JSON files. The secret.json file should have
/// appropriate file permissions (e.g., 600) to prevent unauthorized access.
pub struct SecretStorage {
    path: PathBuf,
}

impl SecretStorage {
    /// Creates a new SecretStorage with the default path (~/.config/orcs/secret.json).
    ///
    /// # Returns
    ///
    /// - `Ok(SecretStorage)`: Successfully determined config path
    /// - `Err(SecretStorageError::ConfigDirNotFound)`: Could not find home directory
    pub fn new() -> Result<Self, SecretStorageError> {
        let path = Self::default_path()?;
        Ok(Self { path })
    }

    /// Creates a new SecretStorage with a custom path (for testing).
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Loads the secret configuration from the JSON file.
    ///
    /// # Returns
    ///
    /// - `Ok(SecretConfig)`: Successfully loaded and parsed
    /// - `Err(SecretStorageError::NotFound)`: File doesn't exist
    /// - `Err(SecretStorageError::IoError)`: Failed to read file
    /// - `Err(SecretStorageError::ParseError)`: Invalid JSON format
    pub fn load(&self) -> Result<SecretConfig, SecretStorageError> {
        if !self.path.exists() {
            return Err(SecretStorageError::NotFound(self.path.clone()));
        }

        let content = fs::read_to_string(&self.path)?;
        let config = serde_json::from_str(&content)?;

        Ok(config)
    }

    /// Returns the default path to secret.json: ~/.config/orcs/secret.json
    fn default_path() -> Result<PathBuf, SecretStorageError> {
        OrcsPaths::secret_file().map_err(|_| SecretStorageError::ConfigDirNotFound)
    }

    /// Returns the path to the secret file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Default for SecretStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create SecretStorage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("secret.json");
        let storage = SecretStorage::with_path(file_path.clone());

        let result = storage.load();
        assert!(result.is_err());
        match result {
            Err(SecretStorageError::NotFound(path)) => {
                assert_eq!(path, file_path);
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_load_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("secret.json");

        let json_content = r#"{
            "gemini": {
                "api_key": "test-key-123",
                "model_name": "gemini-pro"
            }
        }"#;

        fs::write(&file_path, json_content).unwrap();

        let storage = SecretStorage::with_path(file_path);
        let config = storage.load().unwrap();

        assert!(config.gemini.is_some());
        let gemini = config.gemini.unwrap();
        assert_eq!(gemini.api_key, "test-key-123");
        assert_eq!(gemini.model_name, Some("gemini-pro".to_string()));
    }

    #[test]
    fn test_load_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("secret.json");

        let json_content = r#"{}"#;
        fs::write(&file_path, json_content).unwrap();

        let storage = SecretStorage::with_path(file_path);
        let config = storage.load().unwrap();

        assert!(config.gemini.is_none());
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("secret.json");

        let invalid_json = r#"{ invalid json"#;
        fs::write(&file_path, invalid_json).unwrap();

        let storage = SecretStorage::with_path(file_path);
        let result = storage.load();

        assert!(result.is_err());
        assert!(matches!(result, Err(SecretStorageError::ParseError(_))));
    }
}
