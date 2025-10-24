//! User service for managing user information.
//!
//! This module provides services for retrieving and managing user information
//! across the application. Currently returns a constant value, but designed
//! to support future enhancements like user configuration and preferences.

/// Service for managing user information.
///
/// Future enhancements:
/// - Load from configuration file
/// - Support user preferences
/// - Multiple user profiles
pub trait UserService: Send + Sync {
    /// Returns the current user's display name.
    fn get_user_name(&self) -> String;
}

/// Default implementation that returns a constant user name.
///
/// This is a simple implementation suitable for single-user scenarios.
/// Future versions may load from configuration or allow runtime changes.
pub struct DefaultUserService;

impl UserService for DefaultUserService {
    fn get_user_name(&self) -> String {
        "user".to_string()
    }
}

impl Default for DefaultUserService {
    fn default() -> Self {
        Self
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
