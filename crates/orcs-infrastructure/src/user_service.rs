//! Configuration-based user service implementation.
//!
//! This module provides a UserService implementation that loads user information
//! from the configuration file (~/.config/orcs/config.toml).
//!
//! Also provides helper functions to load RootConfig for accessing other settings like EnvSettings.

use crate::dto::create_config_root_migrator;
use crate::paths::{OrcsPaths, ServiceType};
use orcs_core::config::{DebugSettings, RootConfig};
use orcs_core::user::{UserProfile, UserService};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use version_migrate::{FileStorage, FileStorageStrategy, FormatStrategy, LoadBehavior};

/// User service that loads user information from config.toml.
///
/// This implementation reads the user's nickname from the configuration file
/// and caches it to avoid repeated file I/O operations.
///
/// # Example
///
/// ```
/// use orcs_infrastructure::user_service::ConfigBasedUserService;
/// use orcs_core::user::UserService;
///
/// let service = ConfigBasedUserService::new();
/// let nickname = service.get_user_name();
/// println!("User nickname: {}", nickname);
/// ```
#[derive(Debug, Clone)]
pub struct ConfigBasedUserService {
    /// Cached nickname loaded from config.
    /// Uses RwLock for thread-safe lazy loading.
    nickname: Arc<RwLock<Option<String>>>,
}

impl ConfigBasedUserService {
    /// Creates a new ConfigBasedUserService.
    ///
    /// The nickname is loaded lazily on first access to avoid blocking
    /// during initialization.
    pub fn new() -> Self {
        Self {
            nickname: Arc::new(RwLock::new(None)),
        }
    }

    /// Loads the nickname from config if not already cached.
    fn load_nickname(&self) -> String {
        // Check if already cached
        {
            let read_lock = self.nickname.read().unwrap();
            if let Some(ref cached) = *read_lock {
                return cached.clone();
            }
        }

        // Load from FileStorage
        let loaded = Self::load_from_config().unwrap_or_else(|_| "You".to_string());

        // Cache it
        {
            let mut write_lock = self.nickname.write().unwrap();
            *write_lock = Some(loaded.clone());
        }

        loaded
    }

    /// Loads UserProfile from config file using FileStorage.
    fn load_from_config() -> Result<String, String> {
        Self::load_profile_from_config().map(|profile| profile.nickname)
    }

    /// Loads complete UserProfile from config file using FileStorage.
    fn load_profile_from_config() -> Result<UserProfile, String> {
        // Get config path
        let config_path = Self::get_config_path()?;

        // Create FileStorage with migrator for ConfigRoot
        let migrator = create_config_root_migrator();
        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Toml)
            .with_load_behavior(LoadBehavior::CreateIfMissing);

        let mut storage = FileStorage::new(config_path.clone(), migrator, strategy)
            .map_err(|e| format!("Failed to create FileStorage: {}", e))?;

        // Query config_root (not user_profile directly)
        let configs: Vec<RootConfig> = storage
            .query("config_root")
            .map_err(|e| format!("Failed to query config_root: {}", e))?;

        // If no config exists, create default and save it
        if configs.is_empty() {
            let default_config = RootConfig::default();
            storage
                .update_and_save("config_root", vec![default_config.clone()])
                .map_err(|e| format!("Failed to save default config: {}", e))?;
            Ok(default_config.user_profile)
        } else {
            // Extract user_profile from RootConfig
            Ok(configs
                .into_iter()
                .next()
                .map(|config| config.user_profile)
                .unwrap_or_default())
        }
    }

    /// Gets the default config path via centralized path management.
    fn get_config_path() -> Result<PathBuf, String> {
        let path_type = OrcsPaths::new(None)
            .get_path(ServiceType::Config)
            .map_err(|e| e.to_string())?;
        // ServiceType::Config returns PathType::File(config.toml)
        Ok(path_type.into_path_buf())
    }
}

impl Default for ConfigBasedUserService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl UserService for ConfigBasedUserService {
    fn get_user_name(&self) -> String {
        self.load_nickname()
    }

    fn get_user_profile(&self) -> UserProfile {
        Self::load_profile_from_config().unwrap_or_default()
    }

    fn get_debug_settings(&self) -> DebugSettings {
        load_root_config()
            .map(|config| config.debug_settings)
            .unwrap_or_default()
    }

    async fn update_debug_settings(
        &self,
        enable_llm_debug: bool,
        log_level: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Load current config
        let mut config = load_root_config()?;

        // Update debug settings
        config.debug_settings.enable_llm_debug = enable_llm_debug;
        config.debug_settings.log_level = log_level;

        // Save to file
        save_root_config(config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_based_user_service() {
        let service = ConfigBasedUserService::new();
        let nickname = service.get_user_name();

        // Should return either the configured nickname or default "You"
        assert!(!nickname.is_empty());
    }

    #[test]
    fn test_nickname_is_cached() {
        let service = ConfigBasedUserService::new();

        // First call loads from config
        let first = service.get_user_name();

        // Second call should use cache
        let second = service.get_user_name();

        assert_eq!(first, second);
    }

    #[test]
    fn test_load_root_config() {
        let result = load_root_config();

        // Should either load successfully or create default
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.user_profile.nickname.is_empty());
    }
}

/// Loads the complete RootConfig from config.toml.
///
/// This function is useful for accessing configuration settings like EnvSettings,
/// ModelSettings, etc. from various parts of the application.
///
/// # Returns
///
/// Returns `Ok(RootConfig)` if successfully loaded or created with defaults.
/// Returns `Err(String)` if an unrecoverable error occurs.
///
/// # Example
///
/// ```ignore
/// use orcs_infrastructure::user_service::load_root_config;
///
/// let config = load_root_config().expect("Failed to load config");
/// let env_settings = config.env_settings;
/// ```
pub fn load_root_config() -> Result<RootConfig, String> {
    // Get config path
    let config_path = OrcsPaths::new(None)
        .get_path(ServiceType::Config)
        .map_err(|e| e.to_string())?
        .into_path_buf();

    // Create FileStorage with migrator for ConfigRoot
    let migrator = create_config_root_migrator();
    let strategy = FileStorageStrategy::new()
        .with_format(FormatStrategy::Toml)
        .with_load_behavior(LoadBehavior::CreateIfMissing);

    let mut storage = FileStorage::new(config_path.clone(), migrator, strategy)
        .map_err(|e| format!("Failed to create FileStorage: {}", e))?;

    // Query config_root
    let configs: Vec<RootConfig> = storage
        .query("config_root")
        .map_err(|e| format!("Failed to query config_root: {}", e))?;

    // If no config exists, create default and save it
    if configs.is_empty() {
        let default_config = RootConfig::default();
        storage
            .update_and_save("config_root", vec![default_config.clone()])
            .map_err(|e| format!("Failed to save default config: {}", e))?;
        Ok(default_config)
    } else {
        // Return the first config
        configs
            .into_iter()
            .next()
            .ok_or_else(|| "No config found after query".to_string())
    }
}

/// Saves the RootConfig to config.toml.
///
/// This function is useful for persisting configuration changes made at runtime.
///
/// # Arguments
///
/// * `config` - The RootConfig to save
///
/// # Returns
///
/// Returns `Ok(())` if successfully saved.
/// Returns `Err(String)` if an error occurs.
///
/// # Example
///
/// ```ignore
/// use orcs_infrastructure::user_service::{load_root_config, save_root_config};
///
/// let mut config = load_root_config().expect("Failed to load config");
/// config.debug_settings.enable_llm_debug = true;
/// save_root_config(config).expect("Failed to save config");
/// ```
pub fn save_root_config(config: RootConfig) -> Result<(), String> {
    // Get config path
    let config_path = OrcsPaths::new(None)
        .get_path(ServiceType::Config)
        .map_err(|e| e.to_string())?
        .into_path_buf();

    // Create FileStorage with migrator for ConfigRoot
    let migrator = create_config_root_migrator();
    let strategy = FileStorageStrategy::new()
        .with_format(FormatStrategy::Toml)
        .with_load_behavior(LoadBehavior::CreateIfMissing);

    let mut storage = FileStorage::new(config_path.clone(), migrator, strategy)
        .map_err(|e| format!("Failed to create FileStorage: {}", e))?;

    // Update and save
    storage
        .update_and_save("config_root", vec![config])
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}
