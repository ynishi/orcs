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
//! - **1.0.0**: Initial V1 schema with `title` field (renamed from `name`)
//! - **1.1.0**: Added optional `created_at` field for session creation timestamp
//!
//! ### PersonaConfig Version History
//! - **1.0.0**: Initial V1 schema (string-based ID)
//! - **2.0.0**: V2 schema (UUID-based ID)

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use orcs_core::session::{AppMode, ConversationMessage};
use version_migrate::{Versioned, MigratesTo, IntoDomain};
use uuid::Uuid;

/// Current schema version for SessionV1.
pub const SESSION_V1_VERSION: &str = "1.1.0";

/// Represents V0 of the session data schema for serialization.
/// This is the legacy schema with the 'name' field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct SessionV0 {
    /// The schema version of this data structure.
    pub schema_version: String,

    /// Unique session identifier.
    pub id: String,
    /// Human-readable session name.
    pub name: String,
    /// Timestamp when the session was created (ISO 8601 format).
    pub created_at: String,
    /// Timestamp when the session was last updated (ISO 8601 format).
    pub updated_at: String,
    /// The currently active persona ID.
    pub current_persona_id: String,
    /// Conversation history for each persona.
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode.
    pub app_mode: AppMode,
}

/// Represents V1 of the session data schema for serialization.
/// This struct is what is actually written to and read from storage (e.g., a TOML file).
/// The main change from V0 is renaming 'name' to 'title'.
///
/// Version 1.1: Added optional `created_at` field for backward compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct SessionV1 {
    /// The schema version of this data structure.
    pub schema_version: String,

    /// Unique session identifier.
    pub id: String,
    /// Human-readable session title.
    pub title: String,
    /// Timestamp when the session was created (ISO 8601 format).
    /// Added in V1.1 - will be None for sessions created before this change.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Timestamp when the session was last updated (ISO 8601 format).
    pub updated_at: String,
    /// The currently active persona ID.
    pub current_persona_id: String,
    /// Conversation history for each persona.
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode.
    pub app_mode: AppMode,
}

// ============================================================================
// PersonaConfig DTOs
// ============================================================================

/// Current schema version for PersonaConfigV1.
pub const PERSONA_CONFIG_V1_VERSION: &str = "1.0.0";

/// Current schema version for PersonaConfigV2.
pub const PERSONA_CONFIG_V2_VERSION: &str = "2.0.0";

/// Current schema version for UserProfile V1.0.
pub const USER_PROFILE_V1_0_VERSION: &str = "1.0.0";

/// Current schema version for UserProfile V1.1.
pub const USER_PROFILE_V1_1_VERSION: &str = "1.1.0";

// ============================================================================
// Workspace DTOs
// ============================================================================

/// Current schema version for WorkspaceV1.
pub const WORKSPACE_V1_VERSION: &str = "1.0.0";

/// Current schema version for WorkspaceResourcesV1.
pub const WORKSPACE_RESOURCES_V1_VERSION: &str = "1.0.0";

/// Current schema version for UploadedFileV1.
pub const UPLOADED_FILE_V1_VERSION: &str = "1.0.0";

/// Current schema version for GeneratedDocV1.
pub const GENERATED_DOC_V1_VERSION: &str = "1.0.0";

/// Current schema version for ProjectContextV1.
pub const PROJECT_CONTEXT_V1_VERSION: &str = "1.0.0";

/// Current schema version for SessionWorkspaceV1.
pub const SESSION_WORKSPACE_V1_VERSION: &str = "1.0.0";

/// Current schema version for TempFileV1.
pub const TEMP_FILE_V1_VERSION: &str = "1.0.0";

/// Represents a temporary file created during operations (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct TempFileV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_temp_file_v1_version")]
    pub schema_version: String,

    /// Unique identifier for the temp file
    pub id: String,
    /// Path to the temporary file
    pub path: PathBuf,
    /// Purpose or description of the temp file
    pub purpose: String,
    /// Timestamp when the file was created
    pub created_at: i64,
    /// Whether the file should be deleted after session ends
    pub auto_delete: bool,
}

fn default_temp_file_v1_version() -> String {
    TEMP_FILE_V1_VERSION.to_string()
}

/// Represents a file uploaded to the workspace (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct UploadedFileV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_uploaded_file_v1_version")]
    pub schema_version: String,

    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
}

fn default_uploaded_file_v1_version() -> String {
    UPLOADED_FILE_V1_VERSION.to_string()
}

/// Represents an AI-generated document or artifact (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct GeneratedDocV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_generated_doc_v1_version")]
    pub schema_version: String,

    /// Unique identifier for the generated document
    pub id: String,
    /// Title or name of the document
    pub title: String,
    /// Path to the stored document
    pub path: PathBuf,
    /// Type of document (e.g., "summary", "analysis", "diagram")
    pub doc_type: String,
    /// ID of the session that generated this document
    pub session_id: String,
    /// Timestamp when the document was generated
    pub generated_at: i64,
}

fn default_generated_doc_v1_version() -> String {
    GENERATED_DOC_V1_VERSION.to_string()
}

/// Project-specific context and metadata (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct ProjectContextV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_project_context_v1_version")]
    pub schema_version: String,

    /// Programming languages detected in the project
    #[serde(default)]
    pub languages: Vec<String>,
    /// Build system or framework (e.g., "cargo", "npm", "maven")
    #[serde(default)]
    pub build_system: Option<String>,
    /// Project description or purpose
    #[serde(default)]
    pub description: Option<String>,
    /// Git repository URL if available
    #[serde(default)]
    pub repository_url: Option<String>,
    /// Additional metadata as key-value pairs
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_project_context_v1_version() -> String {
    PROJECT_CONTEXT_V1_VERSION.to_string()
}

/// Collection of all resources managed within a workspace (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct WorkspaceResourcesV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_workspace_resources_v1_version")]
    pub schema_version: String,

    /// Files uploaded by the user or system
    #[serde(default)]
    pub uploaded_files: Vec<UploadedFileV1>,
    /// AI-generated documentation and artifacts
    #[serde(default)]
    pub generated_docs: Vec<GeneratedDocV1>,
    /// Temporary files created during session operations
    #[serde(default)]
    pub temp_files: Vec<TempFileV1>,
}

fn default_workspace_resources_v1_version() -> String {
    WORKSPACE_RESOURCES_V1_VERSION.to_string()
}

/// Represents a project-level workspace (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct WorkspaceV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_workspace_v1_version")]
    pub schema_version: String,

    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Collection of all workspace resources
    pub resources: WorkspaceResourcesV1,
    /// Project-specific context and metadata
    pub project_context: ProjectContextV1,
}

fn default_workspace_v1_version() -> String {
    WORKSPACE_V1_VERSION.to_string()
}

/// Session-specific workspace view (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct SessionWorkspaceV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_session_workspace_v1_version")]
    pub schema_version: String,

    /// ID of the parent workspace
    pub workspace_id: String,
    /// ID of the current session
    pub session_id: String,
    /// Temporary files specific to this session
    #[serde(default)]
    pub session_temp_files: Vec<TempFileV1>,
}

fn default_session_workspace_v1_version() -> String {
    SESSION_WORKSPACE_V1_VERSION.to_string()
}

/// Represents the source of a persona.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaSourceDTO {
    System,
    User,
}

impl Default for PersonaSourceDTO {
    fn default() -> Self {
        PersonaSourceDTO::User
    }
}

/// Represents V1 of the persona config schema for serialization.
///
/// This struct is what is actually written to and read from storage (e.g., config.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct PersonaConfigV1 {
    /// Unique persona identifier.
    pub id: String,
    /// Display name of the persona.
    pub name: String,
    /// Role or title of the persona.
    pub role: String,
    /// Background description of the persona.
    pub background: String,
    /// Communication style of the persona.
    pub communication_style: String,
    /// Whether this persona is a default participant in new sessions.
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User).
    #[serde(default)]
    pub source: PersonaSourceDTO,
}

/// Represents V2 of the persona config schema for serialization.
///
/// V2 introduces UUID-based IDs for better internationalization and future extensibility.
/// This struct is the current version used for new writes.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct PersonaConfigV2 {
    /// The schema version of this data structure.
    #[serde(default = "default_persona_v2_version")]
    pub schema_version: String,

    /// Unique persona identifier (UUID format).
    pub id: String,
    /// Display name of the persona.
    pub name: String,
    /// Role or title of the persona.
    pub role: String,
    /// Background description of the persona.
    pub background: String,
    /// Communication style of the persona.
    pub communication_style: String,
    /// Whether this persona is a default participant in new sessions.
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User).
    #[serde(default)]
    pub source: PersonaSourceDTO,
}

fn default_persona_v2_version() -> String {
    PERSONA_CONFIG_V2_VERSION.to_string()
}

/// User profile configuration V1.0.0 (initial version with nickname only).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct UserProfileV1_0 {
    /// The schema version of this data structure.
    #[serde(default = "default_user_profile_v1_0_version")]
    pub schema_version: String,

    /// User's display nickname.
    pub nickname: String,
}

fn default_user_profile_v1_0_version() -> String {
    USER_PROFILE_V1_0_VERSION.to_string()
}

/// User profile configuration V1.1.0 (added background field).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct UserProfileV1_1 {
    /// The schema version of this data structure.
    #[serde(default = "default_user_profile_v1_1_version")]
    pub schema_version: String,

    /// User's display nickname.
    pub nickname: String,

    /// User's background or bio.
    #[serde(default)]
    pub background: String,
}

fn default_user_profile_v1_1_version() -> String {
    USER_PROFILE_V1_1_VERSION.to_string()
}

/// Type alias for the latest UserProfile version.
pub type UserProfileDTO = UserProfileV1_1;

impl Default for UserProfileV1_1 {
    fn default() -> Self {
        Self {
            schema_version: USER_PROFILE_V1_1_VERSION.to_string(),
            nickname: "You".to_string(),
            background: String::new(),
        }
    }
}

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

// ============================================================================
// Migration implementations
// ============================================================================

/// Generates a deterministic UUID from a persona name.
///
/// Uses UUID v5 with NAMESPACE_OID to ensure the same name always
/// produces the same UUID.
fn generate_uuid_from_name(name: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()).to_string()
}

/// Migration from PersonaConfigV1 to PersonaConfigV2.
///
/// Converts string-based IDs to UUID format using deterministic generation.
impl MigratesTo<PersonaConfigV2> for PersonaConfigV1 {
    fn migrate(self) -> PersonaConfigV2 {
        // Check if ID is already a valid UUID
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Not a valid UUID - generate a new one from the name
            generate_uuid_from_name(&self.name)
        };

        PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source,
        }
    }
}

/// Migration from SessionV0 to SessionV1.
///
/// Changes:
/// - Rename `name` to `title`
/// - Add `created_at` field (copies from V0's created_at)
impl MigratesTo<SessionV1> for SessionV0 {
    fn migrate(self) -> SessionV1 {
        SessionV1 {
            schema_version: SESSION_V1_VERSION.to_string(),
            id: self.id,
            title: self.name, // name → title
            created_at: Some(self.created_at.clone()),
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
        }
    }
}

/// Migration from UserProfileV1_0 to UserProfileV1_1.
///
/// Adds the `background` field with a default empty value.
impl MigratesTo<UserProfileV1_1> for UserProfileV1_0 {
    fn migrate(self) -> UserProfileV1_1 {
        UserProfileV1_1 {
            schema_version: USER_PROFILE_V1_1_VERSION.to_string(),
            nickname: self.nickname,
            background: String::new(),
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

use orcs_core::persona::{Persona, PersonaSource};
use orcs_core::session::Session;
use orcs_core::workspace::{
    Workspace, WorkspaceResources, UploadedFile, GeneratedDoc,
    ProjectContext, SessionWorkspace, TempFile,
};

/// Convert PersonaSourceDTO to domain model.
impl From<PersonaSourceDTO> for PersonaSource {
    fn from(dto: PersonaSourceDTO) -> Self {
        match dto {
            PersonaSourceDTO::System => PersonaSource::System,
            PersonaSourceDTO::User => PersonaSource::User,
        }
    }
}

/// Convert domain model to PersonaSourceDTO.
impl From<PersonaSource> for PersonaSourceDTO {
    fn from(source: PersonaSource) -> Self {
        match source {
            PersonaSource::System => PersonaSourceDTO::System,
            PersonaSource::User => PersonaSourceDTO::User,
        }
    }
}

/// Convert PersonaConfigV2 DTO to domain model.
impl IntoDomain<Persona> for PersonaConfigV2 {
    fn into_domain(self) -> Persona {
        // Validate and fix ID if needed
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Legacy data: V2 schema but non-UUID ID
            // Convert using same logic as V1→V2
            generate_uuid_from_name(&self.name)
        };

        Persona {
            id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source.into(),
        }
    }
}

/// Convert domain model to PersonaConfigV2 DTO for persistence.
impl From<&Persona> for PersonaConfigV2 {
    fn from(persona: &Persona) -> Self {
        PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id: persona.id.clone(),
            name: persona.name.clone(),
            role: persona.role.clone(),
            background: persona.background.clone(),
            communication_style: persona.communication_style.clone(),
            default_participant: persona.default_participant,
            source: persona.source.clone().into(),
        }
    }
}

/// Convert SessionV1 DTO to domain model.
impl IntoDomain<Session> for SessionV1 {
    fn into_domain(self) -> Session {
        Session {
            id: self.id,
            title: self.title,
            // For backward compatibility: if created_at is None (V1.0.0),
            // use updated_at as a fallback
            created_at: self.created_at.unwrap_or_else(|| self.updated_at.clone()),
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
        }
    }
}

/// Convert domain model to SessionV1 DTO for persistence.
impl From<&Session> for SessionV1 {
    fn from(session: &Session) -> Self {
        SessionV1 {
            schema_version: SESSION_V1_VERSION.to_string(),
            id: session.id.clone(),
            title: session.title.clone(),
            created_at: Some(session.created_at.clone()),
            updated_at: session.updated_at.clone(),
            current_persona_id: session.current_persona_id.clone(),
            persona_histories: session.persona_histories.clone(),
            app_mode: session.app_mode.clone(),
        }
    }
}

// ============================================================================
// Workspace Domain model conversions
// ============================================================================

/// Convert TempFileV1 DTO to domain model.
impl IntoDomain<TempFile> for TempFileV1 {
    fn into_domain(self) -> TempFile {
        TempFile {
            id: self.id,
            path: self.path,
            purpose: self.purpose,
            created_at: self.created_at,
            auto_delete: self.auto_delete,
        }
    }
}

/// Convert domain model to TempFileV1 DTO for persistence.
impl From<&TempFile> for TempFileV1 {
    fn from(temp_file: &TempFile) -> Self {
        TempFileV1 {
            schema_version: TEMP_FILE_V1_VERSION.to_string(),
            id: temp_file.id.clone(),
            path: temp_file.path.clone(),
            purpose: temp_file.purpose.clone(),
            created_at: temp_file.created_at,
            auto_delete: temp_file.auto_delete,
        }
    }
}

/// Convert UploadedFileV1 DTO to domain model.
impl IntoDomain<UploadedFile> for UploadedFileV1 {
    fn into_domain(self) -> UploadedFile {
        UploadedFile {
            id: self.id,
            name: self.name,
            path: self.path,
            mime_type: self.mime_type,
            size: self.size,
            uploaded_at: self.uploaded_at,
        }
    }
}

/// Convert domain model to UploadedFileV1 DTO for persistence.
impl From<&UploadedFile> for UploadedFileV1 {
    fn from(uploaded_file: &UploadedFile) -> Self {
        UploadedFileV1 {
            schema_version: UPLOADED_FILE_V1_VERSION.to_string(),
            id: uploaded_file.id.clone(),
            name: uploaded_file.name.clone(),
            path: uploaded_file.path.clone(),
            mime_type: uploaded_file.mime_type.clone(),
            size: uploaded_file.size,
            uploaded_at: uploaded_file.uploaded_at,
        }
    }
}

/// Convert GeneratedDocV1 DTO to domain model.
impl IntoDomain<GeneratedDoc> for GeneratedDocV1 {
    fn into_domain(self) -> GeneratedDoc {
        GeneratedDoc {
            id: self.id,
            title: self.title,
            path: self.path,
            doc_type: self.doc_type,
            session_id: self.session_id,
            generated_at: self.generated_at,
        }
    }
}

/// Convert domain model to GeneratedDocV1 DTO for persistence.
impl From<&GeneratedDoc> for GeneratedDocV1 {
    fn from(generated_doc: &GeneratedDoc) -> Self {
        GeneratedDocV1 {
            schema_version: GENERATED_DOC_V1_VERSION.to_string(),
            id: generated_doc.id.clone(),
            title: generated_doc.title.clone(),
            path: generated_doc.path.clone(),
            doc_type: generated_doc.doc_type.clone(),
            session_id: generated_doc.session_id.clone(),
            generated_at: generated_doc.generated_at,
        }
    }
}

/// Convert ProjectContextV1 DTO to domain model.
impl IntoDomain<ProjectContext> for ProjectContextV1 {
    fn into_domain(self) -> ProjectContext {
        ProjectContext {
            languages: self.languages,
            build_system: self.build_system,
            description: self.description,
            repository_url: self.repository_url,
            metadata: self.metadata,
        }
    }
}

/// Convert domain model to ProjectContextV1 DTO for persistence.
impl From<&ProjectContext> for ProjectContextV1 {
    fn from(project_context: &ProjectContext) -> Self {
        ProjectContextV1 {
            schema_version: PROJECT_CONTEXT_V1_VERSION.to_string(),
            languages: project_context.languages.clone(),
            build_system: project_context.build_system.clone(),
            description: project_context.description.clone(),
            repository_url: project_context.repository_url.clone(),
            metadata: project_context.metadata.clone(),
        }
    }
}

/// Convert WorkspaceResourcesV1 DTO to domain model.
impl IntoDomain<WorkspaceResources> for WorkspaceResourcesV1 {
    fn into_domain(self) -> WorkspaceResources {
        WorkspaceResources {
            uploaded_files: self.uploaded_files.into_iter()
                .map(|f| f.into_domain())
                .collect(),
            generated_docs: self.generated_docs.into_iter()
                .map(|d| d.into_domain())
                .collect(),
            temp_files: self.temp_files.into_iter()
                .map(|t| t.into_domain())
                .collect(),
        }
    }
}

/// Convert domain model to WorkspaceResourcesV1 DTO for persistence.
impl From<&WorkspaceResources> for WorkspaceResourcesV1 {
    fn from(resources: &WorkspaceResources) -> Self {
        WorkspaceResourcesV1 {
            schema_version: WORKSPACE_RESOURCES_V1_VERSION.to_string(),
            uploaded_files: resources.uploaded_files.iter()
                .map(UploadedFileV1::from)
                .collect(),
            generated_docs: resources.generated_docs.iter()
                .map(GeneratedDocV1::from)
                .collect(),
            temp_files: resources.temp_files.iter()
                .map(TempFileV1::from)
                .collect(),
        }
    }
}

/// Convert WorkspaceV1 DTO to domain model.
impl IntoDomain<Workspace> for WorkspaceV1 {
    fn into_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            resources: self.resources.into_domain(),
            project_context: self.project_context.into_domain(),
        }
    }
}

/// Convert domain model to WorkspaceV1 DTO for persistence.
impl From<&Workspace> for WorkspaceV1 {
    fn from(workspace: &Workspace) -> Self {
        WorkspaceV1 {
            schema_version: WORKSPACE_V1_VERSION.to_string(),
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            root_path: workspace.root_path.clone(),
            resources: WorkspaceResourcesV1::from(&workspace.resources),
            project_context: ProjectContextV1::from(&workspace.project_context),
        }
    }
}

/// Convert SessionWorkspaceV1 DTO to domain model.
impl IntoDomain<SessionWorkspace> for SessionWorkspaceV1 {
    fn into_domain(self) -> SessionWorkspace {
        SessionWorkspace {
            workspace_id: self.workspace_id,
            session_id: self.session_id,
            session_temp_files: self.session_temp_files.into_iter()
                .map(|t| t.into_domain())
                .collect(),
        }
    }
}

/// Convert domain model to SessionWorkspaceV1 DTO for persistence.
impl From<&SessionWorkspace> for SessionWorkspaceV1 {
    fn from(session_workspace: &SessionWorkspace) -> Self {
        SessionWorkspaceV1 {
            schema_version: SESSION_WORKSPACE_V1_VERSION.to_string(),
            workspace_id: session_workspace.workspace_id.clone(),
            session_id: session_workspace.session_id.clone(),
            session_temp_files: session_workspace.session_temp_files.iter()
                .map(TempFileV1::from)
                .collect(),
        }
    }
}
