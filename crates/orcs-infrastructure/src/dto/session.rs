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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

/// Represents V2.1.0 of the session data schema.
/// Added active_participant_ids and execution_strategy for persisting conversation state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.1.0")]
pub struct SessionV2_1_0 {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy ("broadcast" or "sequential")
    #[serde(default = "default_execution_strategy")]
    pub execution_strategy: String,
}

/// Represents V2.2.0 of the session data schema.
/// Added system_messages for persisting system notifications (join/leave events, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.2.0")]
pub struct SessionV2_2_0 {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy ("broadcast" or "sequential")
    #[serde(default = "default_execution_strategy")]
    pub execution_strategy: String,
    /// System messages (join/leave notifications, etc.)
    #[serde(default)]
    pub system_messages: Vec<ConversationMessage>,
}

/// Represents V2.3.0 of the session data schema.
/// Added participants field for persona ID to name mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.3.0")]
pub struct SessionV2_3_0 {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy ("broadcast" or "sequential")
    #[serde(default = "default_execution_strategy")]
    pub execution_strategy: String,
    /// System messages (join/leave notifications, etc.)
    #[serde(default)]
    pub system_messages: Vec<ConversationMessage>,
    /// Participant persona ID to name mapping for display
    #[serde(default)]
    pub participants: HashMap<String, String>,
}

fn default_execution_strategy() -> String {
    "broadcast".to_string()
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
            title: self.name, // name → title
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
            workspace_id: None, // Existing sessions have no workspace association
        }
    }
}

/// Migration from SessionV2_0_0 to SessionV2_1_0.
/// Added active_participant_ids and execution_strategy fields.
impl MigratesTo<SessionV2_1_0> for SessionV2_0_0 {
    fn migrate(self) -> SessionV2_1_0 {
        SessionV2_1_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
            active_participant_ids: Vec::new(), // No active participants in old sessions
            execution_strategy: default_execution_strategy(), // Default to broadcast
        }
    }
}

/// Migration from SessionV2_1_0 to SessionV2_2_0.
/// Added system_messages field for system notifications.
impl MigratesTo<SessionV2_2_0> for SessionV2_1_0 {
    fn migrate(self) -> SessionV2_2_0 {
        SessionV2_2_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy,
            system_messages: Vec::new(), // No system messages in old sessions
        }
    }
}

/// Migration from SessionV2_2_0 to SessionV2_3_0.
/// Added participants field for persona ID to name mapping.
impl MigratesTo<SessionV2_3_0> for SessionV2_2_0 {
    fn migrate(self) -> SessionV2_3_0 {
        SessionV2_3_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy,
            system_messages: self.system_messages,
            participants: HashMap::new(), // Will be populated on save
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert SessionV2_3_0 DTO to domain model.
impl IntoDomain<Session> for SessionV2_3_0 {
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
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy,
            system_messages: self.system_messages,
            participants: self.participants,
        }
    }
}

/// Convert domain model to SessionV2_3_0 DTO for persistence.
impl version_migrate::FromDomain<Session> for SessionV2_3_0 {
    fn from_domain(session: Session) -> Self {
        SessionV2_3_0 {
            id: session.id,
            title: session.title,
            created_at: session.created_at,
            updated_at: session.updated_at,
            current_persona_id: session.current_persona_id,
            persona_histories: session.persona_histories,
            app_mode: session.app_mode,
            workspace_id: session.workspace_id,
            active_participant_ids: session.active_participant_ids,
            execution_strategy: session.execution_strategy,
            system_messages: session.system_messages,
            participants: session.participants,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for Session entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V2.3.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Renames `name` field to `title`
/// - V1.1.0 → V2.0.0: Adds `workspace_id` field with default value None
/// - V2.0.0 → V2.1.0: Adds `active_participant_ids` and `execution_strategy` fields
/// - V2.1.0 → V2.2.0: Adds `system_messages` field for system notifications
/// - V2.2.0 → V2.3.0: Adds `participants` field for persona ID to name mapping
/// - V2.3.0 → Session: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_session_migrator();
/// let session: Session = migrator.load_flat_from("session", toml_value)?;
/// ```
pub fn create_session_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> V1.1.0 -> V2.0.0 -> V2.1.0 -> V2.2.0 -> V2.3.0 -> Session
    let session_path = version_migrate::Migrator::define("session")
        .from::<SessionV1_0_0>()
        .step::<SessionV1_1_0>()
        .step::<SessionV2_0_0>()
        .step::<SessionV2_1_0>()
        .step::<SessionV2_2_0>()
        .step::<SessionV2_3_0>()
        .into_with_save::<Session>();

    migrator
        .register(session_path)
        .expect("Failed to register session migration path");

    migrator
}
