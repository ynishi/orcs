//! UserProfile DTOs and migrations

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use orcs_core::user::UserProfile;

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

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert UserProfileV1_1 DTO to domain model.
impl IntoDomain<UserProfile> for UserProfileV1_1 {
    fn into_domain(self) -> UserProfile {
        UserProfile {
            nickname: self.nickname,
            background: self.background,
        }
    }
}

/// Convert domain model to UserProfileV1_1 DTO for persistence.
impl version_migrate::FromDomain<UserProfile> for UserProfileV1_1 {
    fn from_domain(profile: UserProfile) -> Self {
        UserProfileV1_1 {
            nickname: profile.nickname,
            background: profile.background,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for UserProfile entities.
///
/// The migrator handles automatic schema migration from V1.0 to V1.1
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0 → V1.1: Adds `background` field with default empty string
/// - V1.1 → UserProfile: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_user_profile_migrator();
/// let profile: UserProfile = migrator.load_flat_from("user_profile", toml_value)?;
/// ```
pub fn create_user_profile_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("user_profile" => [
        UserProfileV1_0,
        UserProfileV1_1,
        UserProfile
    ], save = true)
    .expect("Failed to create user_profile migrator")
}
