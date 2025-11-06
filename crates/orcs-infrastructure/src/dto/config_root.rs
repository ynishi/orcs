//! ConfigRoot DTOs and migrations
//!
//! This module defines versioned DTOs for the root configuration file (config.toml).
//! The configuration structure has evolved to separate concerns:
//!
//! - V1.0.0: Initial version with personas, user_profile, workspaces
//! - V1.1.0: Added app_state field
//! - V2.0.0: Simplified to only user_profile (personas/workspaces/app_state now managed separately)

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use super::{AppStateDTO, UserProfileDTO, WorkspaceV1};
use orcs_core::config::RootConfig;

// ============================================================================
// ConfigRoot DTOs
// ============================================================================

/// Root configuration structure V1.0.0 for the application config file (legacy).
///
/// This version contained personas, workspaces, and user_profile in a single file.
/// Now deprecated in favor of separate storage:
/// - Personas: DataDir/personas/
/// - Workspaces: DataDir/content/workspaces/
/// - AppState: PrefDir/state.toml
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct ConfigRootV1_0_0 {
    /// Persona configurations (each has its own version field).
    /// Stored as serde_json::Value (intermediate format) to allow version-migrate to handle migration.
    #[serde(rename = "persona", default)]
    pub personas: Vec<serde_json::Value>,

    /// User profile configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_profile: Option<UserProfileDTO>,

    /// Workspace configurations (each has its own version field).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<WorkspaceV1>,
}

/// Root configuration structure V1.1.0 for the application config file (legacy).
///
/// Added app_state field (now moved to separate state.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct ConfigRootV1_1_0 {
    /// Persona configurations (each has its own version field).
    /// Stored as serde_json::Value (intermediate format) to allow version-migrate to handle migration.
    #[serde(rename = "persona", default)]
    pub personas: Vec<serde_json::Value>,

    /// User profile configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_profile: Option<UserProfileDTO>,

    /// Application state configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_state: Option<AppStateDTO>,

    /// Workspace configurations (each has its own version field).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<WorkspaceV1>,
}

/// Root configuration structure V2.0.0 for the application config file (current).
///
/// Simplified to only contain user_profile.
/// Other data now managed separately:
/// - Personas: DataDir/personas/ (AsyncDirPersonaRepository)
/// - Workspaces: DataDir/content/workspaces/ (AsyncDirRepository)
/// - AppState: PrefDir/state.toml (separate file for frequent updates)
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct ConfigRootV2_0_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
}

/// Type alias for the latest ConfigRoot version.
pub type ConfigRoot = ConfigRootV2_0_0;

// ============================================================================
// Default implementations
// ============================================================================

impl Default for ConfigRootV1_0_0 {
    fn default() -> Self {
        Self {
            personas: Vec::new(),
            user_profile: None,
            workspaces: Vec::new(),
        }
    }
}

impl Default for ConfigRootV1_1_0 {
    fn default() -> Self {
        Self {
            personas: Vec::new(),
            user_profile: None,
            app_state: None,
            workspaces: Vec::new(),
        }
    }
}

impl Default for ConfigRootV2_0_0 {
    fn default() -> Self {
        Self {
            user_profile: UserProfileDTO::default(),
        }
    }
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from ConfigRootV1_0_0 to ConfigRootV1_1_0.
/// Adds app_state field with default value.
impl MigratesTo<ConfigRootV1_1_0> for ConfigRootV1_0_0 {
    fn migrate(self) -> ConfigRootV1_1_0 {
        ConfigRootV1_1_0 {
            personas: self.personas,
            user_profile: self.user_profile,
            app_state: None, // Default: no app_state
            workspaces: self.workspaces,
        }
    }
}

/// Migration from ConfigRootV1_1_0 to ConfigRootV2_0_0.
/// Removes personas, workspaces, and app_state (now managed separately).
/// Only keeps user_profile.
impl MigratesTo<ConfigRootV2_0_0> for ConfigRootV1_1_0 {
    fn migrate(self) -> ConfigRootV2_0_0 {
        ConfigRootV2_0_0 {
            user_profile: self.user_profile.unwrap_or_default(),
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// IntoDomain implementation for ConfigRootV2_0_0.
/// Converts DTO to domain RootConfig.
impl IntoDomain<RootConfig> for ConfigRootV2_0_0 {
    fn into_domain(self) -> RootConfig {
        RootConfig {
            user_profile: self.user_profile.into_domain(),
        }
    }
}

/// FromDomain implementation for ConfigRootV2_0_0.
/// Converts domain RootConfig to DTO for persistence.
impl version_migrate::FromDomain<RootConfig> for ConfigRootV2_0_0 {
    fn from_domain(config: RootConfig) -> Self {
        ConfigRootV2_0_0 {
            user_profile: UserProfileDTO::from_domain(config.user_profile),
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for ConfigRoot.
///
/// Handles automatic schema migration through multiple versions.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `app_state` field with default value (None)
/// - V1.1.0 → V2.0.0: Removes `personas`, `workspaces`, `app_state` (now managed separately)
/// - V2.0.0 → RootConfig: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_config_root_migrator();
/// let config: RootConfig = migrator.load_flat_from("config_root", toml_value)?;
/// ```
pub fn create_config_root_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> V1.1.0 -> V2.0.0 -> RootConfig
    let config_path = version_migrate::Migrator::define("config_root")
        .from::<ConfigRootV1_0_0>()
        .step::<ConfigRootV1_1_0>()
        .step::<ConfigRootV2_0_0>()
        .into_with_save::<RootConfig>();

    migrator
        .register(config_path)
        .expect("Failed to register config_root migration path");

    migrator
}
