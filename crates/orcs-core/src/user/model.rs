//! UserProfile domain model.
//!
//! Represents user profile information including display name and background.

use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// User profile domain model.
///
/// Contains user information such as display name (nickname) and background/bio.
/// This is a version-agnostic domain model that works with versioned DTOs
/// through the version-migrate library.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[queryable(entity = "user_profile")]
pub struct UserProfile {
    /// User's display nickname
    pub nickname: String,
    /// User's background or bio
    pub background: String,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            nickname: "You".to_string(),
            background: String::new(),
        }
    }
}
