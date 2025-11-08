//! Session domain model.
//!
//! This module contains the core Session entity that represents
//! a user session in the application's domain layer.

use super::app_mode::{AppMode, ConversationMode};
use super::message::ConversationMessage;
use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Placeholder for workspace ID before it's initialized.
/// This will be replaced with the actual default workspace ID during bootstrap.
pub const PLACEHOLDER_WORKSPACE_ID: &str = "___workspace_placeholder___";

/// Represents a user session in the application's domain layer.
///
/// A session contains:
/// - Conversation history for each participating persona
/// - System messages (participant join/leave notifications, etc.)
/// - The currently active persona
/// - Active participants (personas participating in the conversation)
/// - Execution strategy (broadcast or sequential)
/// - Application mode (Idle, Planning, etc.)
/// - Timestamps for creation and last update
/// - Optional workspace association for filtering
///
/// This is the "pure" domain model that business logic operates on,
/// independent of any specific storage format or version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier (UUID format)
    pub id: String,
    /// Human-readable session title
    pub title: String,
    /// Timestamp when the session was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when the session was last updated (ISO 8601 format)
    pub updated_at: String,
    /// The currently active persona ID (UUID format)
    pub current_persona_id: String,
    /// Conversation history for each persona (keyed by persona ID)
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode
    pub app_mode: AppMode,
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy
    #[serde(default = "default_execution_strategy")]
    pub execution_strategy: ExecutionModel,
    /// System messages (join/leave notifications, etc.)
    #[serde(default)]
    pub system_messages: Vec<ConversationMessage>,
    /// Participant persona ID to name mapping for display
    #[serde(default)]
    pub participants: HashMap<String, String>,
    /// Participant persona ID to icon mapping for display
    #[serde(default)]
    pub participant_icons: HashMap<String, String>,
    /// Participant persona ID to base color mapping for UI theming
    #[serde(default)]
    pub participant_colors: HashMap<String, String>,
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(default)]
    pub talk_style: Option<TalkStyle>,
    /// Whether this session is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default)]
    pub sort_order: Option<i32>,
}

fn default_execution_strategy() -> ExecutionModel {
    ExecutionModel::Broadcast
}
