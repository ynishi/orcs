//! User service for managing user information.
//!
//! This module provides services for retrieving and managing user information
//! across the application. Currently returns a constant value, but designed
//! to support future enhancements like user configuration and preferences.

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

impl UserService for DefaultUserService {
    fn get_user_name(&self) -> String {
        "user".to_string()
    }

    fn get_user_profile(&self) -> super::model::UserProfile {
        super::model::UserProfile::default()
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
