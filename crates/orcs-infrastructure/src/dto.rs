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
use orcs_types::{ConversationMessage, AppMode};

/// Current schema version for SessionV1.
pub const SESSION_V1_VERSION: &str = "1.1.0";

/// Represents V0 of the session data schema for serialization.
/// This is the legacy schema with the 'name' field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfigV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_persona_version")]
    pub schema_version: String,

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

fn default_persona_version() -> String {
    PERSONA_CONFIG_V1_VERSION.to_string()
}

/// Represents V2 of the persona config schema for serialization.
///
/// V2 introduces UUID-based IDs for better internationalization and future extensibility.
/// This struct is the current version used for new writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Root configuration structure for personas (DTO V2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRootV2 {
    #[serde(rename = "persona")]
    pub personas: Vec<PersonaConfigV2>,
}

/// Root configuration structure for personas (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRootV1 {
    #[serde(rename = "persona")]
    pub personas: Vec<PersonaConfigV1>,
}
