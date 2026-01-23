//! Configuration service implementation.
//!
//! This module provides a ConfigService that loads the root configuration
//! from the configuration file (~/.config/orcs/config.toml).

use crate::dto::create_config_root_migrator;
use crate::paths::{OrcsPaths, ServiceType};
use orcs_core::config::RootConfig;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use version_migrate::{FileStorage, FileStorageStrategy, FormatStrategy, LoadBehavior};

/// Configuration service that loads and caches the root configuration.
///
/// This implementation reads the configuration from config.toml
/// and caches it to avoid repeated file I/O operations.
#[derive(Debug, Clone)]
pub struct ConfigService {
    /// Cached configuration loaded from file.
    /// Uses RwLock for thread-safe lazy loading.
    config: Arc<RwLock<Option<RootConfig>>>,
}

impl ConfigService {
    /// Creates a new ConfigService.
    ///
    /// The configuration is loaded lazily on first access to avoid blocking
    /// during initialization.
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
        }
    }

    /// Gets the root configuration, loading from file if not cached.
    pub fn get_config(&self) -> RootConfig {
        // Check if already cached
        {
            let read_lock = self.config.read().unwrap();
            if let Some(ref cached) = *read_lock {
                return cached.clone();
            }
        }

        // Load from FileStorage
        let loaded = Self::load_config().unwrap_or_default();

        // Cache it
        {
            let mut write_lock = self.config.write().unwrap();
            *write_lock = Some(loaded.clone());
        }

        loaded
    }

    /// Invalidates the cache, forcing a reload on next access.
    pub fn invalidate_cache(&self) {
        let mut write_lock = self.config.write().unwrap();
        *write_lock = None;
    }

    /// Loads RootConfig from config file using FileStorage.
    fn load_config() -> Result<RootConfig, String> {
        let config_path = Self::get_config_path()?;

        let migrator = create_config_root_migrator();
        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Toml)
            .with_load_behavior(LoadBehavior::CreateIfMissing);

        let mut storage = FileStorage::new(config_path.clone(), migrator, strategy)
            .map_err(|e| format!("Failed to create FileStorage: {}", e))?;

        let configs: Vec<RootConfig> = storage
            .query("config_root")
            .map_err(|e| format!("Failed to query config_root: {}", e))?;

        if configs.is_empty() {
            let default_config = RootConfig::default();
            storage
                .update_and_save("config_root", vec![default_config.clone()])
                .map_err(|e| format!("Failed to save default config: {}", e))?;
            Ok(default_config)
        } else {
            Ok(configs.into_iter().next().unwrap_or_default())
        }
    }

    fn get_config_path() -> Result<PathBuf, String> {
        let path_type = OrcsPaths::new(None)
            .get_path(ServiceType::Config)
            .map_err(|e| e.to_string())?;
        Ok(path_type.into_path_buf())
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}
