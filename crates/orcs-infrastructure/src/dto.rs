//! Data Transfer Objects (DTOs) for session persistence.
//!
//! These DTOs represent the versioned schema for persisting session data.
//! They are private to the infrastructure layer and handle the evolution
//! of the storage format over time.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use orcs_types::{ConversationMessage, AppMode};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionV1 {
    /// The schema version of this data structure.
    pub schema_version: String,

    /// Unique session identifier.
    pub id: String,
    /// Human-readable session title.
    pub title: String,
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
