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
