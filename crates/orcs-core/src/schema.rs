//! TypeScript schema bridge definitions.
//!
//! This module provides local enum types that mirror external domain enums
//! and enable automatic TypeScript type generation via the schema-bridge crate.
//!
//! These types solve the problem of keeping Rust and TypeScript type definitions
//! in sync. Due to Rust's orphan rule, we cannot implement external traits on
//! external types, so we define local enums and provide From/Into conversions.

use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use schema_bridge::SchemaBridge;
use serde::{Deserialize, Serialize};

use crate::dialogue::preset::PresetSource;
use crate::session::ConversationMode;

/// Talk style for dialogue context.
///
/// Mirrors `llm_toolkit::agent::dialogue::TalkStyle` for schema generation.
/// Enables automatic TypeScript generation:
/// `export type TalkStyleType = 'Brainstorm' | 'Casual' | 'DecisionMaking' | ...`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[schema_bridge(string_conversion)]
pub enum TalkStyleType {
    /// Brainstorming session - creative, exploratory, building on ideas.
    Brainstorm,
    /// Casual conversation - relaxed, friendly, conversational.
    Casual,
    /// Decision-making discussion - analytical, weighing options, reaching conclusion.
    DecisionMaking,
    /// Debate - challenging ideas, diverse perspectives, constructive argument.
    Debate,
    /// Problem-solving session - systematic, solution-focused, practical.
    ProblemSolving,
    /// Review/Critique - constructive feedback, detailed analysis.
    Review,
    /// Planning session - structured, forward-thinking, action-oriented.
    Planning,
}

impl From<TalkStyle> for TalkStyleType {
    fn from(value: TalkStyle) -> Self {
        match value {
            TalkStyle::Brainstorm => Self::Brainstorm,
            TalkStyle::Casual => Self::Casual,
            TalkStyle::DecisionMaking => Self::DecisionMaking,
            TalkStyle::Debate => Self::Debate,
            TalkStyle::ProblemSolving => Self::ProblemSolving,
            TalkStyle::Review => Self::Review,
            TalkStyle::Planning => Self::Planning,
        }
    }
}

impl From<TalkStyleType> for TalkStyle {
    fn from(value: TalkStyleType) -> Self {
        match value {
            TalkStyleType::Brainstorm => Self::Brainstorm,
            TalkStyleType::Casual => Self::Casual,
            TalkStyleType::DecisionMaking => Self::DecisionMaking,
            TalkStyleType::Debate => Self::Debate,
            TalkStyleType::ProblemSolving => Self::ProblemSolving,
            TalkStyleType::Review => Self::Review,
            TalkStyleType::Planning => Self::Planning,
        }
    }
}

/// Execution strategy for multi-agent dialogue.
///
/// Anti-Corruption Layer for `llm_toolkit::agent::dialogue::ExecutionModel`.
/// Enables automatic TypeScript generation:
/// `export type ExecutionModelType = 'broadcast' | 'sequential' | 'mentioned'`
///
/// Note: Complex variants like OrderedSequential, OrderedBroadcast, and Moderator
/// are mapped to their base types. Mentioned variant uses default strategy internally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionModelType {
    /// All participants respond in parallel to the same input.
    Broadcast,
    /// Participants execute sequentially, with output chained as input.
    Sequential,
    /// Only @mentioned participants respond to messages.
    Mentioned,
}

impl From<ExecutionModel> for ExecutionModelType {
    fn from(value: ExecutionModel) -> Self {
        match value {
            ExecutionModel::Broadcast => Self::Broadcast,
            ExecutionModel::OrderedBroadcast(_) => Self::Broadcast,
            ExecutionModel::Sequential => Self::Sequential,
            ExecutionModel::OrderedSequential(_) => Self::Sequential,
            ExecutionModel::Mentioned { .. } => Self::Mentioned,
            ExecutionModel::Moderator => Self::Broadcast,
        }
    }
}

impl From<ExecutionModelType> for ExecutionModel {
    fn from(value: ExecutionModelType) -> Self {
        match value {
            ExecutionModelType::Broadcast => Self::Broadcast,
            ExecutionModelType::Sequential => Self::Sequential,
            ExecutionModelType::Mentioned => Self::Mentioned {
                strategy: Default::default(),
            },
        }
    }
}

/// Conversation mode controlling verbosity and style.
///
/// Mirrors `crate::session::ConversationMode` for schema generation.
/// Enables automatic TypeScript generation:
/// `export type ConversationModeType = 'Detailed' | 'Normal' | 'Concise' | 'Brief' | 'Discussion'`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ConversationModeType {
    /// Detailed mode - comprehensive responses with thorough explanations.
    Detailed,
    /// Normal mode - no special constraints (default).
    Normal,
    /// Concise mode - responses should be under 300 characters.
    Concise,
    /// Brief mode - responses should be under 150 characters.
    Brief,
    /// Discussion mode - focus on new perspectives only.
    Discussion,
}

impl From<ConversationMode> for ConversationModeType {
    fn from(value: ConversationMode) -> Self {
        match value {
            ConversationMode::Detailed => Self::Detailed,
            ConversationMode::Normal => Self::Normal,
            ConversationMode::Concise => Self::Concise,
            ConversationMode::Brief => Self::Brief,
            ConversationMode::Discussion => Self::Discussion,
        }
    }
}

impl From<ConversationModeType> for ConversationMode {
    fn from(value: ConversationModeType) -> Self {
        match value {
            ConversationModeType::Detailed => Self::Detailed,
            ConversationModeType::Normal => Self::Normal,
            ConversationModeType::Concise => Self::Concise,
            ConversationModeType::Brief => Self::Brief,
            ConversationModeType::Discussion => Self::Discussion,
        }
    }
}

/// Source of a dialogue preset.
///
/// Mirrors `crate::dialogue::preset::PresetSource` for schema generation.
/// Enables automatic TypeScript generation:
/// `export type PresetSourceType = 'System' | 'User'`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum PresetSourceType {
    /// System-provided default presets
    System,
    /// User-created custom presets
    User,
}

impl From<PresetSource> for PresetSourceType {
    fn from(value: PresetSource) -> Self {
        match value {
            PresetSource::System => Self::System,
            PresetSource::User => Self::User,
        }
    }
}

impl From<PresetSourceType> for PresetSource {
    fn from(value: PresetSourceType) -> Self {
        match value {
            PresetSourceType::System => Self::System,
            PresetSourceType::User => Self::User,
        }
    }
}

/// Session metadata for TypeScript type generation.
///
/// This is a simplified view of `crate::session::Session` that excludes
/// `persona_histories` (HashMap<String, Vec<ConversationMessage>>) which
/// schema-bridge cannot yet fully support.
///
/// This type serves two purposes:
/// 1. **TypeScript Generation**: Enables automatic TypeScript type generation
///    for Session entity without HashMap<Vec> complexity.
/// 2. **Future Migration Marker**: The `From<SessionType> for Session` implementation
///    is intentionally unused but ensures compiler errors if Session structure changes,
///    guiding developers to update this type accordingly.
///
/// # Relationship with Domain Model
///
/// - **Domain Model**: `crate::session::Session` - Full entity with all fields
/// - **DTO Layer**: `crate::infrastructure::dto::session::SessionV4_2_0` - Persistence format
/// - **Schema Type**: `SessionType` - TypeScript generation (this type)
///
/// # Fields Excluded
///
/// - `persona_histories: HashMap<String, Vec<ConversationMessage>>` - Complex nested type
///
/// # TypeScript Output
///
/// Generates TypeScript interface with all metadata fields for frontend use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "camelCase")]
pub struct SessionType {
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
    /// Workspace ID - all sessions must be associated with a workspace
    pub workspace_id: String,
    /// Active participant persona IDs
    pub active_participant_ids: Vec<String>,
    /// Execution strategy for multi-agent dialogue
    pub execution_strategy: ExecutionModelType,
    /// Participant persona ID to name mapping for display
    pub participants: std::collections::HashMap<String, String>,
    /// Participant persona ID to icon mapping for display
    pub participant_icons: std::collections::HashMap<String, String>,
    /// Participant persona ID to base color mapping for UI theming
    pub participant_colors: std::collections::HashMap<String, String>,
    /// Participant persona ID to backend mapping (e.g., "claude_api", "gemini_cli")
    pub participant_backends: std::collections::HashMap<String, String>,
    /// Participant persona ID to model name mapping (e.g., "claude-sonnet-4-5-20250929")
    pub participant_models: std::collections::HashMap<String, String>,
    /// Conversation mode (controls verbosity and style)
    pub conversation_mode: ConversationModeType,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyleType>,
    /// Whether this session is marked as favorite (pinned to top)
    pub is_favorite: bool,
    /// Whether this session is archived (hidden by default)
    pub is_archived: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    /// Whether this session is muted (AI won't respond to messages)
    pub is_muted: bool,
}

/// Conversion from SessionType to Session domain model.
///
/// **NOTE**: This implementation is intentionally unused in current codebase.
/// Its purpose is to serve as a compiler-enforced marker for future migration
/// from `ExecutionModel`/`TalkStyle` (external types from llm-toolkit) to
/// `ExecutionModelType`/`TalkStyleType` (internal schema types).
///
/// When Session struct is migrated to use internal types, this implementation
/// will guide developers to update SessionType accordingly through compiler errors.
///
/// # Migration Plan
///
/// 1. Change Session.execution_strategy: ExecutionModel → ExecutionModelType
/// 2. Change Session.talk_style: Option<TalkStyle> → Option<TalkStyleType>
/// 3. Update this From implementation (compiler will flag outdated conversions)
/// 4. Update DTO layer conversions in infrastructure crate
///
/// For detailed migration plan, see workspace/ENTITY_SSOT_STATUS.md
impl From<SessionType> for crate::session::Session {
    fn from(value: SessionType) -> Self {
        use crate::session::Session;

        Session {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
            current_persona_id: value.current_persona_id,
            persona_histories: std::collections::HashMap::new(), // Excluded from SessionType
            app_mode: crate::session::AppMode::Idle,             // Default value
            workspace_id: value.workspace_id,
            active_participant_ids: value.active_participant_ids,
            execution_strategy: value.execution_strategy.into(), // ExecutionModelType → ExecutionModel
            system_messages: Vec::new(),                         // Excluded from SessionType
            participants: value.participants,
            participant_icons: value.participant_icons,
            participant_colors: value.participant_colors,
            participant_backends: value.participant_backends,
            participant_models: value
                .participant_models
                .into_iter()
                .map(|(k, v)| (k, Some(v)))
                .collect(),
            conversation_mode: value.conversation_mode.into(), // ConversationModeType → ConversationMode
            talk_style: value.talk_style.map(|ts| ts.into()),  // TalkStyleType → TalkStyle
            is_favorite: value.is_favorite,
            is_archived: value.is_archived,
            sort_order: value.sort_order,
            auto_chat_config: None, // Excluded from SessionType
            is_muted: value.is_muted,
            context_mode: crate::session::ContextMode::default(), // Default to Rich
            sandbox_state: None,                                  // Default to non-sandbox mode
        }
    }
}

/// Task metadata for TypeScript type generation.
///
/// This is a simplified view of `crate::task::Task` that excludes complex fields
/// like `execution_details` (contains serde_json::Value), `strategy`, and `journal_log`.
///
/// # Relationship with Domain Model
///
/// - **Domain Model**: `crate::task::Task` - Full entity with execution details
/// - **DTO Layer**: DTO types in `orcs_infrastructure::dto::task`
/// - **Schema Type**: `TaskType` - TypeScript generation (this type)
///
/// # Fields Excluded
///
/// - `execution_details: Option<ExecutionDetails>` - Contains serde_json::Value
/// - `strategy: Option<String>` - Large JSON string not needed in frontend
/// - `journal_log: Option<String>` - Execution trace not needed in frontend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "camelCase")]
pub struct TaskType {
    /// Unique task identifier (UUID format)
    pub id: String,
    /// Session ID where this task was executed
    pub session_id: String,
    /// Task title (shortened from description)
    pub title: String,
    /// Full task description/request
    pub description: String,
    /// Current task status
    pub status: TaskStatus,
    /// Timestamp when task was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when task was last updated (ISO 8601 format)
    pub updated_at: String,
    /// Timestamp when task completed (ISO 8601 format)
    pub completed_at: Option<String>,
    /// Number of steps executed
    pub steps_executed: i32,
    /// Number of steps skipped
    pub steps_skipped: i32,
    /// Number of context keys generated
    pub context_keys: i32,
    /// Error message if task failed
    pub error: Option<String>,
    /// Result summary text
    pub result: Option<String>,
}

// Re-export TaskStatus from task module for TypeScript generation
pub use crate::task::TaskStatus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_talk_style_type_to_ts() {
        // Should generate TypeScript string union type
        let ts_type = TalkStyleType::to_ts();
        assert!(!ts_type.is_empty());
        println!("TalkStyleType TS: {}", ts_type);
    }

    #[test]
    fn test_execution_model_type_to_ts() {
        let ts_type = ExecutionModelType::to_ts();
        assert!(!ts_type.is_empty());
        println!("ExecutionModelType TS: {}", ts_type);
    }

    #[test]
    fn test_conversation_mode_type_to_ts() {
        let ts_type = ConversationModeType::to_ts();
        assert!(!ts_type.is_empty());
        println!("ConversationModeType TS: {}", ts_type);
    }

    #[test]
    fn test_preset_source_type_to_ts() {
        let ts_type = PresetSourceType::to_ts();
        assert!(!ts_type.is_empty());
        println!("PresetSourceType TS: {}", ts_type);
    }

    #[test]
    fn test_talk_style_conversion() {
        let orig = TalkStyle::Brainstorm;
        let converted: TalkStyleType = orig.into();
        let back: TalkStyle = converted.into();
        assert_eq!(orig, back);
    }

    #[test]
    fn test_execution_model_conversion() {
        // Test basic types
        assert_eq!(
            ExecutionModelType::from(ExecutionModel::Broadcast),
            ExecutionModelType::Broadcast
        );
        assert_eq!(
            ExecutionModelType::from(ExecutionModel::Sequential),
            ExecutionModelType::Sequential
        );
    }

    #[test]
    fn test_conversation_mode_conversion() {
        let orig = ConversationMode::Concise;
        let converted: ConversationModeType = orig.clone().into();
        let back: ConversationMode = converted.into();
        assert_eq!(orig, back);
    }

    #[test]
    fn test_preset_source_conversion() {
        let orig = PresetSource::System;
        let converted: PresetSourceType = orig.clone().into();
        let back: PresetSource = converted.into();
        assert_eq!(orig, back);
    }

    #[test]
    fn test_talk_style_type_string_conversion() {
        use std::str::FromStr;

        // Test ToString
        assert_eq!(TalkStyleType::Brainstorm.to_string(), "Brainstorm");
        assert_eq!(TalkStyleType::Casual.to_string(), "Casual");
        assert_eq!(TalkStyleType::Planning.to_string(), "Planning");

        // Test FromStr
        assert_eq!(
            TalkStyleType::from_str("Brainstorm").unwrap(),
            TalkStyleType::Brainstorm
        );
        assert_eq!(
            TalkStyleType::from_str("Casual").unwrap(),
            TalkStyleType::Casual
        );
        assert_eq!(
            TalkStyleType::from_str("Planning").unwrap(),
            TalkStyleType::Planning
        );

        // Test roundtrip
        for variant in [
            TalkStyleType::Brainstorm,
            TalkStyleType::Casual,
            TalkStyleType::DecisionMaking,
            TalkStyleType::Debate,
            TalkStyleType::ProblemSolving,
            TalkStyleType::Review,
            TalkStyleType::Planning,
        ] {
            let s = variant.to_string();
            let parsed = TalkStyleType::from_str(&s).unwrap();
            assert_eq!(variant, parsed);
        }
    }
}
