//! Configuration-based user service implementation.
//!
//! This module provides a UserService implementation that loads user information
//! from the configuration file (~/.config/orcs/config.toml).

use crate::dto::create_user_profile_migrator;
use crate::paths::OrcsPaths;
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
        // Get config path
        let config_path = Self::get_config_path()?;

        // Create FileStorage with migrator
        let migrator = create_user_profile_migrator();
        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Toml)
            .with_load_behavior(LoadBehavior::CreateIfMissing);

        let storage = FileStorage::new(config_path, migrator, strategy)
            .map_err(|e| format!("Failed to create FileStorage: {}", e))?;

        // Query user_profile
        let profiles: Vec<UserProfile> = storage
            .query("user_profile")
            .map_err(|e| format!("Failed to query user_profile: {}", e))?;

        // user_profile is a single object, but query returns Vec
        // If empty or error, fall back to default
        let profile = profiles.into_iter().next().unwrap_or_default();

        Ok(profile.nickname)
    }

    /// Gets the default config path (~/.config/orcs/config.toml).
    fn get_config_path() -> Result<PathBuf, String> {
        OrcsPaths::config_file().map_err(|e| e.to_string())
    }
}

impl Default for ConfigBasedUserService {
    fn default() -> Self {
        Self::new()
    }
}

impl UserService for ConfigBasedUserService {
    fn get_user_name(&self) -> String {
        self.load_nickname()
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
}
