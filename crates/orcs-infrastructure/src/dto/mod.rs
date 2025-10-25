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

// Re-export persona DTOs
pub use persona::{PersonaConfigV1, PersonaConfigV2, PersonaSourceDTO};

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

/// Root configuration structure for personas (DTO V2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRootV2 {
    #[serde(rename = "persona")]
    pub personas: Vec<PersonaConfigV2>,

    /// User profile configuration (optional for backward compatibility).
    #[serde(default)]
    pub user_profile: Option<UserProfileDTO>,
}

/// Root configuration structure for personas (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRootV1 {
    #[serde(rename = "persona")]
    pub personas: Vec<PersonaConfigV1>,

    /// Workspaces managed by the system (added in workspace feature).
    #[serde(default)]
    pub workspaces: Vec<WorkspaceV1>,
}
