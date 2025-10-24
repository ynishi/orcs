//! DTOs for session data persistence and the session domain model.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{ConversationMessage, AppMode};

// --- Domain Model ---

/// Represents the session concept in the application's core logic.
/// This is the "pure" model that the business logic layer operates on.
/// It is independent of any specific storage format or version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub current_persona_id: String,
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    pub app_mode: AppMode,
}


// --- Data Transfer Objects (DTOs) for Persistence ---

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

// --- Type Conversions ---

/// Convert SessionV1 DTO to domain model.
impl From<SessionV1> for Session {
    fn from(dto: SessionV1) -> Self {
        Session {
            id: dto.id,
            title: dto.title,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            current_persona_id: dto.current_persona_id,
            persona_histories: dto.persona_histories,
            app_mode: dto.app_mode,
        }
    }
}

/// Convert domain model to SessionV1 DTO for persistence.
impl From<&Session> for SessionV1 {
    fn from(session: &Session) -> Self {
        SessionV1 {
            schema_version: "1".to_string(),
            id: session.id.clone(),
            title: session.title.clone(),
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
            current_persona_id: session.current_persona_id.clone(),
            persona_histories: session.persona_histories.clone(),
            app_mode: session.app_mode.clone(),
        }
    }
}
