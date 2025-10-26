//! Session DTOs and migrations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use orcs_core::session::{AppMode, ConversationMessage, Session};

/// Represents V1.0.0 of the session data schema.
/// Legacy schema with 'name' field instead of 'title'.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct SessionV1_0_0 {
     /// Unique session identifier
    pub id: String,
    /// Human-readable session name (renamed to 'title' in V1.1.0)
    pub name: String,
    /// Timestamp when the session was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when the session was last updated (ISO 8601 format)
    pub updated_at: String,
    /// The currently active persona ID
    pub current_persona_id: String,
    /// Conversation history for each persona
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode
    pub app_mode: AppMode,
}

/// Represents V1.1.0 of the session data schema.
/// Renamed 'name' to 'title'.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct SessionV1_1_0 {
    /// Unique session identifier
    pub id: String,
    /// Human-readable session title
    pub title: String,
    /// Timestamp when the session was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when the session was last updated (ISO 8601 format)
    pub updated_at: String,
    /// The currently active persona ID
    pub current_persona_id: String,
    /// Conversation history for each persona
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode
    pub app_mode: AppMode,
}

/// Represents V2.0.0 of the session data schema.
/// Added workspace_id for workspace-based session filtering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct SessionV2_0_0 {
    /// Unique session identifier
    pub id: String,
    /// Human-readable session title
    pub title: String,
    /// Timestamp when the session was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when the session was last updated (ISO 8601 format)
    pub updated_at: String,
    /// The currently active persona ID
    pub current_persona_id: String,
    /// Conversation history for each persona
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode
    pub app_mode: AppMode,
    /// Workspace ID if this session is associated with a workspace
    pub workspace_id: Option<String>,
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from SessionV1_0_0 to SessionV1_1_0.
/// Changes: 'name' → 'title'
impl MigratesTo<SessionV1_1_0> for SessionV1_0_0 {
    fn migrate(self) -> SessionV1_1_0 {
        SessionV1_1_0 {
            id: self.id,
            title: self.name,  // name → title
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
        }
    }
}

/// Migration from SessionV1_1_0 to SessionV2_0_0.
/// Added workspace_id field (defaults to None for existing sessions).
impl MigratesTo<SessionV2_0_0> for SessionV1_1_0 {
    fn migrate(self) -> SessionV2_0_0 {
        SessionV2_0_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: None,  // Existing sessions have no workspace association
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert SessionV2_0_0 DTO to domain model.
impl IntoDomain<Session> for SessionV2_0_0 {
    fn into_domain(self) -> Session {
        Session {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
        }
    }
}

/// Convert domain model to SessionV2_0_0 DTO for persistence.
impl From<&Session> for SessionV2_0_0 {
    fn from(session: &Session) -> Self {
        SessionV2_0_0 {
            id: session.id.clone(),
            title: session.title.clone(),
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
            current_persona_id: session.current_persona_id.clone(),
            persona_histories: session.persona_histories.clone(),
            app_mode: session.app_mode.clone(),
            workspace_id: session.workspace_id.clone(),
        }
    }
}
