//! UserProfile DTOs and migrations

use serde::{Deserialize, Serialize};
use version_migrate::{MigratesTo, Versioned};

/// User profile configuration V1.0.0 (initial version with nickname only).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct UserProfileV1_0 {
    /// User's display nickname.
    pub nickname: String,
}

/// User profile configuration V1.1.0 (added background field).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct UserProfileV1_1 {
    /// User's display nickname.
    pub nickname: String,

    /// User's background or bio.
    #[serde(default)]
    pub background: String,
}

/// Type alias for the latest UserProfile version.
pub type UserProfileDTO = UserProfileV1_1;

impl Default for UserProfileV1_1 {
    fn default() -> Self {
        Self {
            nickname: "You".to_string(),
            background: String::new(),
        }
    }
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from UserProfileV1_0 to UserProfileV1_1.
impl MigratesTo<UserProfileV1_1> for UserProfileV1_0 {
    fn migrate(self) -> UserProfileV1_1 {
        UserProfileV1_1 {
            nickname: self.nickname,
            background: String::new(),
        }
    }
}
