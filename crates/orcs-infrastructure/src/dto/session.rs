//! Session DTOs and migrations

use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use version_migrate::{FromDomain, IntoDomain, MigratesTo, Versioned};

use orcs_core::session::{
    AppMode, AutoChatConfig, ConversationMessage, ConversationMode, MessageRole,
    PLACEHOLDER_WORKSPACE_ID, Session,
};

// ============================================================================
// ExecutionStrategy DTO (Anti-Corruption Layer)
// ============================================================================

/// V1.0.0: String-based execution strategy(llm-toolkit v0.52 compatible)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStrategyV1_0_0 {
    Sequential,
    Broadcast,
    Mentioned,
}

impl From<String> for ExecutionStrategyV1_0_0 {
    fn from(value: String) -> Self {
        let lowercase = value.to_lowercase();
        match lowercase.as_str() {
            "sequential" => Self::Sequential,
            "Mentioned" => Self::Mentioned,
            // fallback to broadcast
            _ => Self::Broadcast,
        }
    }
}

fn default_execution_strategy_v1_0_0() -> ExecutionStrategyV1_0_0 {
    ExecutionStrategyV1_0_0::Broadcast
}

/// V2.0.0: Enum-based execution strategy (isolated from llm-toolkit changes)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStrategyV2_0_0 {
    Sequential,
    Broadcast,
    Mentioned {
        /// JSON-serialized StrategyMap from llm-toolkit
        #[serde(skip_serializing_if = "Option::is_none")]
        strategy: Option<String>,
    },
}

fn default_execution_strategy_v2_0_0() -> ExecutionStrategyV2_0_0 {
    ExecutionStrategyV2_0_0::Broadcast
}

/// Migration from string-based (V1) to enum-based (V2) execution strategy
impl MigratesTo<ExecutionStrategyV2_0_0> for ExecutionStrategyV1_0_0 {
    fn migrate(self) -> ExecutionStrategyV2_0_0 {
        match self {
            ExecutionStrategyV1_0_0::Sequential => ExecutionStrategyV2_0_0::Sequential,
            ExecutionStrategyV1_0_0::Broadcast => ExecutionStrategyV2_0_0::Broadcast,
            ExecutionStrategyV1_0_0::Mentioned => {
                ExecutionStrategyV2_0_0::Mentioned { strategy: None }
            }
        }
    }
}

/// Convert DTO to domain model (ExecutionModel from llm-toolkit)
impl IntoDomain<ExecutionModel> for ExecutionStrategyV2_0_0 {
    fn into_domain(self) -> ExecutionModel {
        match self {
            ExecutionStrategyV2_0_0::Sequential => ExecutionModel::Sequential,
            ExecutionStrategyV2_0_0::Broadcast => ExecutionModel::Broadcast,
            ExecutionStrategyV2_0_0::Mentioned { strategy } => ExecutionModel::Mentioned {
                strategy: strategy
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
            },
        }
    }
}

/// Convert domain model to DTO
impl FromDomain<ExecutionModel> for ExecutionStrategyV2_0_0 {
    fn from_domain(model: ExecutionModel) -> Self {
        match model {
            ExecutionModel::Sequential => ExecutionStrategyV2_0_0::Sequential,
            ExecutionModel::Broadcast => ExecutionStrategyV2_0_0::Broadcast,
            ExecutionModel::Mentioned { strategy } => ExecutionStrategyV2_0_0::Mentioned {
                strategy: serde_json::to_string(&strategy).ok(),
            },
        }
    }
}

// ============================================================================
// Session DTOs
// ============================================================================

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

/// Represents V2.4.0 of the session data schema.
/// Added conversation_mode field for controlling multi-agent dialogue verbosity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.4.0")]
pub struct SessionV2_4_0 {
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
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
}

/// Represents V2.5.0 of the session data schema.
/// Added talk_style field for dialogue context (Brainstorm, Debate, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.5.0")]
pub struct SessionV2_5_0 {
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
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
}

/// Represents V2.6.0 of the session data schema.
/// Adds metadata preservation for system message types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.6.0")]
pub struct SessionV2_6_0 {
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
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
}

/// Represents V2.7.0 of the session data schema.
/// Changes execution_strategy from String to ExecutionStrategyV2_0_0 DTO for type safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.7.0")]
pub struct SessionV2_7_0 {
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
    /// Execution strategy (using DTO to isolate from llm-toolkit changes)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
    /// System messages (join/leave notifications, etc.)
    #[serde(default)]
    pub system_messages: Vec<ConversationMessage>,
    /// Participant persona ID to name mapping for display
    #[serde(default)]
    pub participants: HashMap<String, String>,
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.8.0")]
pub struct SessionV2_8_0 {
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
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
    /// System messages (join/leave notifications, etc.)
    #[serde(default)]
    pub system_messages: Vec<ConversationMessage>,
    /// Participant persona ID to name mapping for display
    #[serde(default)]
    pub participants: HashMap<String, String>,
    /// Participant persona ID to icon mapping for display
    #[serde(default)]
    pub participant_icons: HashMap<String, String>,
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
}

/// V2.9.0: Added participant_colors for UI theming
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.9.0")]
pub struct SessionV2_9_0 {
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
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
}

/// V3.0.0: Made workspace_id required
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "3.0.0")]
pub struct SessionV3_0_0 {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
}

/// V3.1.0: Added is_favorite and is_archived for session organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "3.1.0")]
pub struct SessionV3_1_0 {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
    /// Whether this session is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
}

/// V3.2.0: Added sort_order for manual session ordering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "3.2.0")]
pub struct SessionV3_2_0 {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
    /// Whether this session is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
}

/// V3.3.0: Added auto_chat_config for AutoChat mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "3.3.0")]
pub struct SessionV3_3_0 {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
    /// Whether this session is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    /// AutoChat configuration (None means AutoChat is disabled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_chat_config: Option<AutoChatConfig>,
}

/// V3.4.0: Added participant_backends and participant_models for API type display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "3.4.0")]
pub struct SessionV3_4_0 {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v1_0_0")]
    pub execution_strategy: ExecutionStrategyV1_0_0,
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
    /// Participant persona ID to backend mapping (e.g., "claude_api", "gemini_cli")
    #[serde(default)]
    pub participant_backends: HashMap<String, String>,
    /// Participant persona ID to model name mapping (e.g., "claude-sonnet-4-5-20250929")
    #[serde(default)]
    pub participant_models: HashMap<String, Option<String>>,
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
    /// Whether this session is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    /// AutoChat configuration (None means AutoChat is disabled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_chat_config: Option<AutoChatConfig>,
}

/// V4.0.0: Update execution_strategy V1_0_0 to V2_0_0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Versioned)]
#[versioned(version = "4.0.0")]
pub struct SessionV4_0_0 {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    #[serde(default)]
    pub active_participant_ids: Vec<String>,
    /// Execution strategy (now using ExecutionModel enum)
    #[serde(default = "default_execution_strategy_v2_0_0")]
    pub execution_strategy: ExecutionStrategyV2_0_0,
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
    /// Participant persona ID to backend mapping (e.g., "claude_api", "gemini_cli")
    #[serde(default)]
    pub participant_backends: HashMap<String, String>,
    /// Participant persona ID to model name mapping (e.g., "claude-sonnet-4-5-20250929")
    #[serde(default)]
    pub participant_models: HashMap<String, Option<String>>,
    /// Conversation mode (controls verbosity and style)
    #[serde(default)]
    pub conversation_mode: ConversationMode,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
    /// Whether this session is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    /// AutoChat configuration (None means AutoChat is disabled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_chat_config: Option<AutoChatConfig>,
}

fn default_execution_strategy() -> String {
    "broadcast".to_string()
}

fn normalize_conversation_messages(messages: Vec<ConversationMessage>) -> Vec<ConversationMessage> {
    messages
        .into_iter()
        .map(|mut message| {
            if message.metadata.system_message_type.is_none() && message.role == MessageRole::System
            {
                message.metadata.system_message_type = Some("system".to_string());
            }
            message
        })
        .collect()
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

/// Migration from SessionV2_3_0 to SessionV2_4_0.
/// Added conversation_mode field for controlling multi-agent dialogue verbosity.
impl MigratesTo<SessionV2_4_0> for SessionV2_3_0 {
    fn migrate(self) -> SessionV2_4_0 {
        SessionV2_4_0 {
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
            conversation_mode: ConversationMode::default(), // Default to Normal mode
        }
    }
}

/// Migration from SessionV2_4_0 to SessionV2_5_0.
/// Added talk_style field for dialogue context.
impl MigratesTo<SessionV2_5_0> for SessionV2_4_0 {
    fn migrate(self) -> SessionV2_5_0 {
        SessionV2_5_0 {
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
            conversation_mode: self.conversation_mode,
            talk_style: None, // Default to no talk style set
        }
    }
}

/// Migration from SessionV2_5_0 to SessionV2_6_0.
/// Normalizes conversation metadata for UI reconstruction.
impl MigratesTo<SessionV2_6_0> for SessionV2_5_0 {
    fn migrate(self) -> SessionV2_6_0 {
        SessionV2_6_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self
                .persona_histories
                .into_iter()
                .map(|(persona_id, messages)| {
                    (persona_id, normalize_conversation_messages(messages))
                })
                .collect(),
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy,
            system_messages: normalize_conversation_messages(self.system_messages),
            participants: self.participants,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
        }
    }
}

/// Migration from SessionV2_6_0 to SessionV2_7_0.
/// Changes execution_strategy from String to ExecutionStrategyV2_0_0 DTO.
impl MigratesTo<SessionV2_7_0> for SessionV2_6_0 {
    fn migrate(self) -> SessionV2_7_0 {
        SessionV2_7_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy.into(),
            system_messages: self.system_messages,
            participants: self.participants,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
        }
    }
}

/// Migration from SessionV2_7_0 to SessionV2_8_0.
impl MigratesTo<SessionV2_8_0> for SessionV2_7_0 {
    fn migrate(self) -> SessionV2_8_0 {
        SessionV2_8_0 {
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
            participant_icons: HashMap::new(), // V2_7_0 doesn't have icon field
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
        }
    }
}

/// Migration from SessionV2_8_0 to SessionV2_9_0.
/// Added participant_colors for UI theming
impl MigratesTo<SessionV2_9_0> for SessionV2_8_0 {
    fn migrate(self) -> SessionV2_9_0 {
        SessionV2_9_0 {
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
            participant_icons: self.participant_icons,
            participant_colors: HashMap::new(), // V2_8_0 doesn't have color field
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
        }
    }
}

/// Migration from V2.9.0 to V3.0.0
/// Makes workspace_id required by setting placeholder if None
impl MigratesTo<SessionV3_0_0> for SessionV2_9_0 {
    fn migrate(self) -> SessionV3_0_0 {
        SessionV3_0_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self
                .workspace_id
                .unwrap_or_else(|| PLACEHOLDER_WORKSPACE_ID.to_string()),
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy,
            system_messages: self.system_messages,
            participants: self.participants,
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
        }
    }
}

/// Migration from V3.0.0 to V3.1.0
/// Adds is_favorite and is_archived fields (default to false)
impl MigratesTo<SessionV3_1_0> for SessionV3_0_0 {
    fn migrate(self) -> SessionV3_1_0 {
        SessionV3_1_0 {
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
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            is_favorite: false, // Existing sessions are not favorited by default
            is_archived: false, // Existing sessions are not archived by default
        }
    }
}

/// Migration from V3.1.0 to V3.2.0
impl MigratesTo<SessionV3_2_0> for SessionV3_1_0 {
    fn migrate(self) -> SessionV3_2_0 {
        SessionV3_2_0 {
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
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            is_favorite: self.is_favorite,
            is_archived: self.is_archived,
            sort_order: None, // Existing sessions have no manual sort order by default
        }
    }
}

/// Migration from V3.2.0 to V3.3.0
/// Adds auto_chat_config field (default to None)
impl MigratesTo<SessionV3_3_0> for SessionV3_2_0 {
    fn migrate(self) -> SessionV3_3_0 {
        SessionV3_3_0 {
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
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            is_favorite: self.is_favorite,
            is_archived: self.is_archived,
            sort_order: self.sort_order,
            auto_chat_config: None, // Existing sessions have AutoChat disabled by default
        }
    }
}

/// Migration from SessionV3_3_0 to SessionV3_4_0.
impl MigratesTo<SessionV3_4_0> for SessionV3_3_0 {
    fn migrate(self) -> SessionV3_4_0 {
        SessionV3_4_0 {
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
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            participant_backends: HashMap::new(), // Will be populated on next participant add/remove
            participant_models: HashMap::new(), // Will be populated on next participant add/remove
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            is_favorite: self.is_favorite,
            is_archived: self.is_archived,
            sort_order: self.sort_order,
            auto_chat_config: self.auto_chat_config,
        }
    }
}

/// Migration from SessionV3_3_0 to SessionV3_4_0.
impl MigratesTo<SessionV4_0_0> for SessionV3_4_0 {
    fn migrate(self) -> SessionV4_0_0 {
        SessionV4_0_0 {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            current_persona_id: self.current_persona_id,
            persona_histories: self.persona_histories,
            app_mode: self.app_mode,
            workspace_id: self.workspace_id,
            active_participant_ids: self.active_participant_ids,
            execution_strategy: self.execution_strategy.migrate(),
            system_messages: self.system_messages,
            participants: self.participants,
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            participant_backends: HashMap::new(), // Will be populated on next participant add/remove
            participant_models: HashMap::new(), // Will be populated on next participant add/remove
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            is_favorite: self.is_favorite,
            is_archived: self.is_archived,
            sort_order: self.sort_order,
            auto_chat_config: self.auto_chat_config,
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert SessionV3_4_0 DTO to domain model.
impl IntoDomain<Session> for SessionV4_0_0 {
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
            execution_strategy: self.execution_strategy.into_domain(), // DTO → Domain
            system_messages: self.system_messages,
            participants: self.participants,
            participant_icons: self.participant_icons,
            participant_colors: self.participant_colors,
            participant_backends: self.participant_backends,
            participant_models: self.participant_models,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            is_favorite: self.is_favorite,
            is_archived: self.is_archived,
            sort_order: self.sort_order,
            auto_chat_config: self.auto_chat_config,
        }
    }
}

/// Convert domain model to SessionV3_4_0 DTO for persistence.
impl version_migrate::FromDomain<Session> for SessionV4_0_0 {
    fn from_domain(session: Session) -> Self {
        let Session {
            id,
            title,
            created_at,
            updated_at,
            current_persona_id,
            persona_histories,
            app_mode,
            workspace_id,
            active_participant_ids,
            execution_strategy,
            system_messages,
            participants,
            participant_icons,
            participant_colors,
            participant_backends,
            participant_models,
            conversation_mode,
            talk_style,
            is_favorite,
            is_archived,
            sort_order,
            auto_chat_config,
        } = session;

        SessionV4_0_0 {
            id,
            title,
            created_at,
            updated_at,
            current_persona_id,
            persona_histories,
            app_mode,
            workspace_id,
            active_participant_ids,
            execution_strategy: ExecutionStrategyV2_0_0::from_domain(execution_strategy), // Domain → DTO
            system_messages,
            participants,
            participant_icons,
            participant_colors,
            participant_backends,
            participant_models,
            conversation_mode,
            talk_style,
            is_favorite,
            is_archived,
            sort_order,
            auto_chat_config,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for Session entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V4.0.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Renames `name` field to `title`
/// - V1.1.0 → V2.0.0: Adds `workspace_id` field with default value None
/// - V2.0.0 → V2.1.0: Adds `active_participant_ids` and `execution_strategy` fields
/// - V2.1.0 → V2.2.0: Adds `system_messages` field for system notifications
/// - V2.2.0 → V2.3.0: Adds `participants` field for persona ID to name mapping
/// - V2.3.0 → V2.4.0: Adds `conversation_mode` field for controlling dialogue verbosity
/// - V2.4.0 → V2.5.0: Adds `talk_style` field for dialogue context
/// - V2.5.0 → V2.6.0: Normalizes conversation metadata for system message types
/// - V2.6.0 → V2.7.0: Changes execution_strategy from String to ExecutionModel enum
/// - V2.7.0 → V2.8.0: Adds `participant_icons` field for persona icons
/// - V2.8.0 → V2.9.0: Adds `participant_colors` field for UI theming
/// - V2.9.0 → V3.0.0: Makes `workspace_id` required
/// - V3.0.0 → V3.1.0: Adds `is_favorite` and `is_archived` fields for session organization
/// - V3.1.0 → V3.2.0: Adds `sort_order` field for manual session ordering
/// - V3.2.0 → V3.3.0: Adds `auto_chat_config` field for AutoChat mode
/// - V3.3.0 → V4.0.0: Changes execution_strategy from execution_model enum
/// - V4.0.0 → Session: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_session_migrator();
/// let session: Session = migrator.load_flat_from("session", toml_value)?;
/// ```
pub fn create_session_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> ... -> V3.2.0 -> V3.3.0 -> Session
    let session_path = version_migrate::Migrator::define("session")
        .from::<SessionV1_0_0>()
        .step::<SessionV1_1_0>()
        .step::<SessionV2_0_0>()
        .step::<SessionV2_1_0>()
        .step::<SessionV2_2_0>()
        .step::<SessionV2_3_0>()
        .step::<SessionV2_4_0>()
        .step::<SessionV2_5_0>()
        .step::<SessionV2_6_0>()
        .step::<SessionV2_7_0>()
        .step::<SessionV2_8_0>()
        .step::<SessionV2_9_0>()
        .step::<SessionV3_0_0>()
        .step::<SessionV3_1_0>()
        .step::<SessionV3_2_0>()
        .step::<SessionV3_3_0>()
        .step::<SessionV3_4_0>()
        .step::<SessionV4_0_0>()
        .into_with_save::<Session>();

    migrator
        .register(session_path)
        .expect("Failed to register session migration path");

    migrator
}
