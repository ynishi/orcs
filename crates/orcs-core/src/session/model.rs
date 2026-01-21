//! Session domain model.
//!
//! This module contains the core Session entity that represents
//! a user session in the application's domain layer.

use super::app_mode::{AppMode, ConversationMode};
use super::message::ConversationMessage;
use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use schema_bridge::SchemaBridge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Placeholder for workspace ID before it's initialized.
/// This will be replaced with the actual default workspace ID during bootstrap.
pub const PLACEHOLDER_WORKSPACE_ID: &str = "___workspace_placeholder___";

/// Configuration for AutoChat mode.
///
/// AutoChat enables automatic multi-round dialogue where agents continue
/// discussing until a stop condition is met.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SchemaBridge)]
pub struct AutoChatConfig {
    /// Maximum number of dialogue iterations
    pub max_iterations: i32,
    /// Stop condition strategy
    pub stop_condition: StopCondition,
    /// Enable WebSearch during auto-chat
    pub web_search_enabled: bool,
}

impl Default for AutoChatConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            stop_condition: StopCondition::IterationCount,
            web_search_enabled: true,
        }
    }
}

/// Stop condition for AutoChat mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum StopCondition {
    /// Stop after reaching max_iterations
    IterationCount,
    /// Continue until user manually stops
    UserInterrupt,
    // Future: ConsensusReached - detect when agents reach consensus
}

/// Sandbox state for git worktree-based isolated development.
///
/// When enabled, the session operates in a separate git worktree,
/// allowing experimentation without affecting the main workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
pub struct SandboxState {
    /// Absolute path to the worktree directory
    pub worktree_path: String,
    /// Original branch name before entering sandbox
    pub original_branch: String,
    /// Sandbox branch name (e.g., "sandbox-{session_id}")
    pub sandbox_branch: String,
    /// Base directory for sandbox worktrees (e.g., "../" or "./.orcs-sandboxes")
    /// Defaults to "../" if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_root: Option<String>,
}

/// Context mode for controlling AI context injection.
///
/// Controls the amount of system context provided to AI agents:
/// - Rich: Full context with all system extensions (SlashCommands, TalkStyle, etc.)
/// - Clean: Minimal context with Expertise only, no system extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ContextMode {
    /// Full context: all system extensions enabled (default)
    #[default]
    Rich,
    /// Clean context: expertise only, no system extensions
    Clean,
}

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
///
/// # Type System Architecture
///
/// - **Domain Model**: `Session` (this type) - Full entity with all fields including external types
/// - **DTO Layer**: `SessionV4_2_0` - Persistence format in `orcs_infrastructure::dto::session`
/// - **Schema Type**: `SessionType` - TypeScript generation in `crate::schema`
///
/// # External Type Dependencies
///
/// This struct currently uses external types from `llm_toolkit`:
/// - `execution_strategy: ExecutionModel` - Multi-agent execution strategy
/// - `talk_style: Option<TalkStyle>` - Dialogue context style
///
/// **Future Migration**: These will be migrated to internal wrapper types
/// (`ExecutionModelType`, `TalkStyleType`) to enable full TypeScript type generation.
/// See `crate::schema::SessionType` for the migration marker implementation.
///
/// # TypeScript Type Generation
///
/// Use `SessionType` from `crate::schema` for TypeScript type generation.
/// This domain model cannot directly use `SchemaBridge` derive due to:
/// 1. Complex nested type: `persona_histories: HashMap<String, Vec<ConversationMessage>>`
/// 2. External types: `ExecutionModel` and `TalkStyle` from llm-toolkit
///
/// The DTO layer (`SessionV4_2_0`) handles conversion between Session and persistence format,
/// isolating the domain model from external crate changes.
///
/// # JSON Serialization Format
///
/// This domain model uses `#[serde(rename_all = "camelCase")]` for Tauri IPC communication.
/// **IMPORTANT**: The DTO layer (`SessionV4_2_0`) does NOT use camelCase and remains snake_case
/// for backward compatibility with existing saved session files (`~/.orcs/sessions/*.json`).
///
/// - **Tauri IPC** (this struct): Serialized as camelCase for TypeScript frontend
/// - **Disk persistence** (DTO): Serialized as snake_case for file compatibility
/// - **DTO version**: No version bump needed when changing domain serialization format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    /// AutoChat configuration (None means AutoChat is disabled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_chat_config: Option<AutoChatConfig>,
    /// Whether this session is muted (AI won't respond to messages)
    #[serde(default)]
    pub is_muted: bool,
    /// Context mode for AI interactions (Rich = full context, Clean = expertise only)
    #[serde(default)]
    pub context_mode: ContextMode,
    /// Sandbox state (None = normal mode, Some = sandbox mode with git worktree)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_state: Option<SandboxState>,
    /// Timestamp of the last successful memory sync (ISO 8601 format)
    /// Used for differential sync - only messages after this timestamp are synced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_memory_sync_at: Option<String>,
}

fn default_execution_strategy() -> ExecutionModel {
    ExecutionModel::Broadcast
}
