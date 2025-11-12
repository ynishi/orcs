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

mod app_state;
mod config_root;
mod persona;
mod secret;
mod session;
mod slash_command;
mod task;
mod uploaded_file;
mod user_profile;
mod workspace;

// Re-export app_state DTOs and migrator
pub use app_state::{AppStateDTO, AppStateV1_0, AppStateV1_1, create_app_state_migrator};

// Re-export config_root DTOs and migrator
pub use config_root::{
    ConfigRoot, ConfigRootV1_0_0, ConfigRootV1_1_0, ConfigRootV2_0_0, create_config_root_migrator,
};

// Re-export persona DTOs and migrator
pub use persona::{
    PersonaBackendDTO, PersonaConfigV1_0_0, PersonaConfigV1_1_0, PersonaSourceDTO,
    create_persona_migrator,
};

// Re-export secret DTOs and migrator
pub use secret::{SecretConfigV1_0_0, create_secret_migrator};

// Re-export session DTOs and migrator
pub use session::{SessionV1_0_0, SessionV1_1_0, SessionV2_0_0, create_session_migrator};

// Re-export slash_command DTOs and migrator
pub use slash_command::{SlashCommandV1, SlashCommandV1_1, create_slash_command_migrator};

// Re-export task DTOs and migrator
pub use task::{TaskV1_0_0, create_task_migrator};

// Re-export uploaded_file DTOs and migrator
pub use uploaded_file::{UploadedFileV1_0_0, UploadedFileV1_1_0, create_uploaded_file_migrator};

// Re-export user_profile DTOs and migrator
pub use user_profile::{
    UserProfileDTO, UserProfileV1_0, UserProfileV1_1, create_user_profile_migrator,
};

// Re-export workspace DTOs and migrators
pub use workspace::{
    ProjectContextV1, SessionWorkspaceV1, TempFileV1, WorkspaceResourcesV1, WorkspaceV1,
    WorkspaceV1_1_0, create_project_context_migrator, create_session_workspace_migrator,
    create_temp_file_migrator, create_workspace_migrator, create_workspace_resources_migrator,
};
