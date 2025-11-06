//! Secret service implementation.
//!
//! This module provides a service for managing secret configuration (API keys)
//! stored in secret.json.

use crate::paths::{OrcsPaths, ServiceType};
use orcs_core::config::SecretConfig;
use orcs_core::secret::SecretService;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use version_migrate::{FileStorage, FileStorageStrategy, FormatStrategy, LoadBehavior};

/// Service for managing secret configuration.
///
/// This implementation reads secret configuration using FileStorage
/// and caches it to avoid repeated file I/O operations.
///
/// # Example
///
/// ```ignore
/// use orcs_infrastructure::SecretServiceImpl;
/// use orcs_core::secret::SecretService;
///
/// let service = SecretServiceImpl::new()?;
/// let secrets = service.load_secrets()?;
/// ```
#[derive(Clone)]
pub struct SecretServiceImpl {
    /// Cached secret config loaded from storage.
    /// Uses RwLock for thread-safe lazy loading.
    secrets: Arc<RwLock<Option<SecretConfig>>>,
    /// FileStorage instance for persistence.
    storage: Arc<RwLock<FileStorage>>,
}

impl SecretServiceImpl {
    pub fn default() -> Result<Self> {
        Self::new(None)
    }
    /// Creates a new SecretServiceImpl.
    ///
    /// This method ensures that the secret file path is resolved.
    /// If the file doesn't exist, it will be created with default values
    /// on first access via LoadBehavior::CreateIfMissing.
    ///
    /// Uses the centralized path management via `ServiceType::Secret`.
    pub fn new(base_path: Option<&Path>) -> Result<Self> {
        // Get file path for Secret via centralized path management
        let path_type = OrcsPaths::new(base_path).get_path(ServiceType::Secret)
            .map_err(|e| anyhow::anyhow!("Failed to get secret path: {}", e))?;
        let file_path = path_type.into_path_buf(); // secret.json

        // Setup migrator (no versioning for secrets, just load/save)
        let migrator = version_migrate::Migrator::builder().build();

        // Setup storage strategy: JSON format, CreateIfMissing
        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Json)
            .with_load_behavior(LoadBehavior::CreateIfMissing);

        // Create FileStorage (automatically loads or creates empty config)
        let storage = FileStorage::new(file_path.clone(), migrator, strategy)
            .map_err(|e| anyhow::anyhow!("Failed to create FileStorage: {}", e))?;

        Ok(Self {
            secrets: Arc::new(RwLock::new(None)),
            storage: Arc::new(RwLock::new(storage)),
        })
    }

    /// Loads the secrets from storage if not already cached.
    fn load_secrets_internal(&self) -> Result<SecretConfig, String> {
        // Check if already cached
        {
            let read_lock = self.secrets.read().unwrap();
            if let Some(ref cached) = *read_lock {
                return Ok(cached.clone());
            }
        }

        // Load from FileStorage
        let storage = self.storage.read().unwrap();
        let secrets: Vec<SecretConfig> = storage
            .query("secret")
            .map_err(|e| format!("Failed to query secret: {}", e))?;

        // secret is a single object, take first or return default
        let loaded = secrets.into_iter().next().unwrap_or_default();

        // Cache it
        {
            let mut write_lock = self.secrets.write().unwrap();
            *write_lock = Some(loaded.clone());
        }

        Ok(loaded)
    }
}

#[async_trait::async_trait]
impl SecretService for SecretServiceImpl {
    async fn load_secrets(&self) -> Result<SecretConfig, String> {
        self.load_secrets_internal()
    }

    async fn secret_file_exists(&self) -> bool {
        // Try to load secrets - if successful, file exists
        self.load_secrets_internal().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_service_creation() {
        let secret_temp_file = tempfile::NamedTempFile::new().expect("secret_temp_file should be created");
        let service = SecretServiceImpl::new(secret_temp_file);
        assert!(service.is_ok());
    }

    #[test]
    fn test_secret_file_exists() {
        let secret_temp_file = tempfile::NamedTempFile::new().expect("secret_temp_file should be created");
        let service = SecretServiceImpl::new(Some(secret_temp_file.into_temp_path())).unwrap();
        // File may or may not exist depending on test environment
        assert!(service.secret_file_exists());
    }
}
