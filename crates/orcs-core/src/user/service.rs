//! User service for managing user information.
//!
//! This module provides services for retrieving and managing user information
//! across the application. Currently returns a constant value, but designed
//! to support future enhancements like user configuration and preferences.

use crate::config::{DebugSettings, MemorySyncSettings};

/// Service for managing user information.
///
/// This trait abstracts user-related operations, allowing different
/// implementations for various scenarios (single-user, multi-user,
/// configuration-based, etc.).
///
/// # Future Enhancements
/// - Load from configuration file
/// - Support user preferences
/// - Multiple user profiles
/// - User authentication
#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    /// Returns the current user's display name.
    ///
    /// # Returns
    ///
    /// A string containing the user's display name.
    fn get_user_name(&self) -> String;

    /// Returns the complete user profile.
    ///
    /// # Returns
    ///
    /// A UserProfile containing nickname and background.
    fn get_user_profile(&self) -> super::model::UserProfile;

    /// Returns the current debug settings.
    ///
    /// # Returns
    ///
    /// A DebugSettings containing enable_llm_debug and log_level.
    fn get_debug_settings(&self) -> DebugSettings;

    /// Updates the debug settings.
    ///
    /// # Arguments
    ///
    /// * `enable_llm_debug` - Whether to enable LLM debug mode
    /// * `log_level` - The log level to use
    async fn update_debug_settings(
        &self,
        enable_llm_debug: bool,
        log_level: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Returns the current memory sync settings.
    ///
    /// # Returns
    ///
    /// A MemorySyncSettings containing enabled flag and interval_secs.
    fn get_memory_sync_settings(&self) -> MemorySyncSettings;
}

/// Default implementation that returns a constant user name.
///
/// This is a simple implementation suitable for single-user scenarios
/// or development environments. For production use, consider implementing
/// a configuration-based or authentication-based UserService.
///
/// # Example
///
/// ```
/// use orcs_core::user::{UserService, DefaultUserService};
///
/// let service = DefaultUserService::default();
/// assert_eq!(service.get_user_name(), "user");
/// ```
#[derive(Debug, Clone, Default)]
pub struct DefaultUserService;

#[async_trait::async_trait]
impl UserService for DefaultUserService {
    fn get_user_name(&self) -> String {
        "user".to_string()
    }

    fn get_user_profile(&self) -> super::model::UserProfile {
        super::model::UserProfile::default()
    }

    fn get_debug_settings(&self) -> DebugSettings {
        DebugSettings::default()
    }

    async fn update_debug_settings(
        &self,
        _enable_llm_debug: bool,
        _log_level: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Default implementation does nothing
        Ok(())
    }

    fn get_memory_sync_settings(&self) -> MemorySyncSettings {
        MemorySyncSettings::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_user_service() {
        let service = DefaultUserService::default();
        assert_eq!(service.get_user_name(), "user");
    }
}
