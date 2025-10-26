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
use toml;

// Re-export persona DTOs
pub use persona::{PersonaBackendDTO, PersonaConfigV1_0_0, PersonaConfigV1_1_0, PersonaSourceDTO};

// Re-export session DTOs
pub use session::{SessionV1_0_0, SessionV1_1_0, SessionV2_0_0};

// Re-export uploaded_file DTOs
pub use uploaded_file::{UploadedFileV1_0_0, UploadedFileV1_1_0};

// Re-export user_profile DTOs
pub use user_profile::{UserProfileDTO, UserProfileV1_0, UserProfileV1_1};

// Re-export workspace DTOs
pub use workspace::{
    GeneratedDocV1, ProjectContextV1, SessionWorkspaceV1, TempFileV1,
    WorkspaceResourcesV1, WorkspaceV1,
};

// ============================================================================
// Root configuration structures
// ============================================================================

/// Root configuration structure for the application config file.
///
/// This structure doesn't have its own version - each contained entity
/// (personas, workspaces, etc.) maintains its own schema version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRoot {
    /// Persona configurations (each has its own schema_version).
    /// Stored as raw TOML values to allow version-migrate to handle migration.
    #[serde(rename = "persona", default)]
    pub personas: Vec<toml::Value>,

    /// User profile configuration.
    #[serde(default)]
    pub user_profile: Option<UserProfileDTO>,

    /// Workspace configurations (each has its own schema_version).
    #[serde(default)]
    pub workspaces: Vec<WorkspaceV1>,
}

impl Default for ConfigRoot {
    fn default() -> Self {
        Self {
            personas: Vec::new(),
            user_profile: None,
            workspaces: Vec::new(),
        }
    }
}
