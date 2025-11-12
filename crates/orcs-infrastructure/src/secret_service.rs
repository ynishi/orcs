//! Secret service implementation.
//!
//! This module provides a service for managing secret configuration (API keys)
//! stored in secret.json.

use crate::dto::create_secret_migrator;
use crate::paths::{OrcsPaths, ServiceType};
use anyhow::{Context, Result};
use orcs_core::config::SecretConfig;
use orcs_core::secret::SecretService;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use version_migrate::{FileStorage, FileStorageStrategy, FormatStrategy, LoadBehavior, Migrator};

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
        let path_type = OrcsPaths::new(base_path)
            .get_path(ServiceType::Secret)
            .map_err(|e| anyhow::anyhow!("Failed to get secret path: {}", e))?;
        let file_path = path_type.into_path_buf(); // secret.json

        // Setup migrator (version-aware secret storage)
        let migrator = create_secret_migrator();

        // Ensure legacy flat files are migrated before FileStorage initializes
        Self::migrate_legacy_secret_file(&file_path, &migrator)
            .context("Failed to migrate legacy secret.json")?;

        // Prepare default storage payload in FileService-friendly format
        let default_secret = Self::default_storage_value(&migrator)
            .context("Failed to build default secret.json")?;

        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Json)
            .with_load_behavior(LoadBehavior::SaveIfMissing)
            .with_default_value(default_secret);

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
        let mut storage = self.storage.write().unwrap();
        let secrets: Vec<SecretConfig> = storage
            .query("secret")
            .map_err(|e| format!("Failed to query secret: {}", e))?;

        // If no secrets exist, create default and save it
        let loaded = if secrets.is_empty() {
            let default_config = SecretConfig::default();
            storage
                .update_and_save("secret", vec![default_config.clone()])
                .map_err(|e| format!("Failed to save default secret config: {}", e))?;
            default_config
        } else {
            secrets.into_iter().next().unwrap()
        };

        // Cache it
        {
            let mut write_lock = self.secrets.write().unwrap();
            *write_lock = Some(loaded.clone());
        }

        Ok(loaded)
    }

    /// Builds the default storage payload expected by FileService.
    fn default_storage_value(migrator: &Migrator) -> Result<serde_json::Value> {
        let entry = migrator
            .save_domain_flat("secret", SecretConfig::default())
            .context("Failed to serialize default secret config for storage")?;
        let entry_value: serde_json::Value =
            serde_json::from_str(&entry).context("Failed to parse serialized secret config")?;

        Ok(json!({ "secret": [entry_value] }))
    }

    /// Migrates legacy flat secret.json files into the new FileService format.
    fn migrate_legacy_secret_file(path: &Path, migrator: &Migrator) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read legacy secret file: {}", path.display()))?;

        if content.trim().is_empty() {
            return Ok(());
        }

        let current_value: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse secret file {} as JSON", path.display()))?;

        // Already in the new format (has `secret` key with array)
        if current_value.get("secret").is_some() {
            return Ok(());
        }

        let legacy_config: SecretConfig = serde_json::from_value(current_value)
            .context("Failed to convert legacy secret file into SecretConfig")?;

        let entry = migrator
            .save_domain_flat("secret", legacy_config)
            .context("Failed to serialize migrated secret config")?;
        let entry_value: serde_json::Value = serde_json::from_str(&entry)
            .context("Failed to parse migrated secret config payload")?;

        let migrated = json!({ "secret": [entry_value] });
        let serialized = serde_json::to_string_pretty(&migrated)
            .context("Failed to serialize migrated secret file")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directories for {}", path.display())
            })?;
        }

        fs::write(path, serialized)
            .with_context(|| format!("Failed to write migrated secret file {}", path.display()))?;

        Ok(())
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
        let secret_temp_file =
            tempfile::NamedTempFile::new().expect("secret_temp_file should be created");
        let service = SecretServiceImpl::new(Some(secret_temp_file.path()));
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_secret_file_exists() {
        let secret_temp_file =
            tempfile::NamedTempFile::new().expect("secret_temp_file should be created");
        let service = SecretServiceImpl::new(Some(secret_temp_file.path())).unwrap();
        // File may or may not exist depending on test environment
        assert!(service.secret_file_exists().await);
    }

    #[tokio::test]
    async fn test_migrates_legacy_secret_format() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let legacy_path = temp_dir.path().join("legacy_secret.json");
        let legacy = json!({
            "claude": { "api_key": "claude-legacy" },
            "gemini": { "api_key": "gemini-legacy" },
            "openai": { "api_key": "openai-legacy" }
        });
        fs::write(&legacy_path, serde_json::to_string_pretty(&legacy).unwrap()).unwrap();

        let service = SecretServiceImpl::new(Some(legacy_path.as_path())).unwrap();
        let secrets = service.load_secrets().await.unwrap();

        assert_eq!(secrets.claude.as_ref().unwrap().api_key, "claude-legacy");
        assert_eq!(secrets.gemini.as_ref().unwrap().api_key, "gemini-legacy");
        assert_eq!(secrets.openai.as_ref().unwrap().api_key, "openai-legacy");

        let migrated: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&legacy_path).unwrap()).unwrap();
        assert!(migrated.get("secret").is_some());
    }
}
