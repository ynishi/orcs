//! Data Transfer Objects (DTOs) for persistence.
//!
//! These DTOs represent the versioned schema for persisting data.
//! They are private to the infrastructure layer and handle the evolution
//! of the storage format over time.
//!
//! ## Schema Versioning (Semantic Versioning)
//!
//! We follow semantic versioning for schema changes:
//! - **MAJOR (X.0.0)**: Breaking changes (field removal, type changes)
//! - **MINOR (1.X.0)**: Backward-compatible additions (new optional fields)
//! - **PATCH (1.0.X)**: Backward-compatible fixes (not typically used for schema)
//!
//! ### Session Version History
//! - **1.0.0**: Initial schema with `name` field
//! - **1.1.0**: Renamed `name` to `title`
//! - **2.0.0**: Added `workspace_id` for workspace-based session filtering
//!
//! ### PersonaConfig Version History
//! - **1.0.0**: Initial V1 schema (string-based ID)
//! - **2.0.0**: V2 schema (UUID-based ID)

mod persona;
mod session;
mod uploaded_file;
mod user_profile;
mod workspace;

use serde::{Deserialize, Serialize};

// Re-export persona DTOs and migrator
pub use persona::{
    create_persona_migrator, PersonaBackendDTO, PersonaConfigV1_0_0, PersonaConfigV1_1_0,
    PersonaSourceDTO,
};

// Re-export session DTOs and migrator
pub use session::{create_session_migrator, SessionV1_0_0, SessionV1_1_0, SessionV2_0_0};

// Re-export uploaded_file DTOs and migrator
pub use uploaded_file::{create_uploaded_file_migrator, UploadedFileV1_0_0, UploadedFileV1_1_0};

// Re-export user_profile DTOs and migrator
pub use user_profile::{create_user_profile_migrator, UserProfileDTO, UserProfileV1_0, UserProfileV1_1};

// Re-export workspace DTOs
pub use workspace::{
    GeneratedDocV1, ProjectContextV1, SessionWorkspaceV1, TempFileV1,
    WorkspaceResourcesV1, WorkspaceV1,
};

// ============================================================================
// Root configuration structures
// ============================================================================

/// Root configuration structure V1.0.0 for the application config file.
///
/// Each contained entity (personas, workspaces, etc.) maintains its own version field.
#[derive(Debug, Clone, Serialize, Deserialize, version_migrate::Versioned)]
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

impl Default for ConfigRootV1_0_0 {
    fn default() -> Self {
        Self {
            personas: Vec::new(),
            user_profile: None,
            workspaces: Vec::new(),
        }
    }
}

/// IntoDomain implementation for ConfigRootV1_0_0 (identity conversion since it's the latest version).
impl version_migrate::IntoDomain<ConfigRootV1_0_0> for ConfigRootV1_0_0 {
    fn into_domain(self) -> ConfigRootV1_0_0 {
        self
    }
}

/// Type alias for the latest ConfigRoot version.
pub type ConfigRoot = ConfigRootV1_0_0;

// ============================================================================
// ConfigRoot Migrator
// ============================================================================

/// Creates and configures a Migrator instance for ConfigRoot.
///
/// Currently only V1.0.0 exists, so no migration steps are needed yet.
/// Future versions can add migration paths here.
///
/// # Example
///
/// ```ignore
/// let migrator = create_config_root_migrator();
/// let config: ConfigRoot = migrator.load_flat_from("config_root", json_value)?;
/// ```
pub fn create_config_root_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> ConfigRoot (currently same)
    let config_path = version_migrate::Migrator::define("config_root")
        .from::<ConfigRootV1_0_0>()
        .into::<ConfigRoot>();

    migrator
        .register(config_path)
        .expect("Failed to register config_root migration path");

    migrator
}
