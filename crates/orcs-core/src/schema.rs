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
/// Simplified version of `llm_toolkit::agent::dialogue::ExecutionModel` for schema generation.
/// Enables automatic TypeScript generation:
/// `export type ExecutionModelType = 'broadcast' | 'sequential'`
///
/// Note: This is a simplified view. Complex variants like OrderedSequential, OrderedBroadcast,
/// Mentioned, and Moderator are mapped to their base types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionModelType {
    /// All participants respond in parallel to the same input.
    Broadcast,
    /// Participants execute sequentially, with output chained as input.
    Sequential,
}

impl From<ExecutionModel> for ExecutionModelType {
    fn from(value: ExecutionModel) -> Self {
        match value {
            ExecutionModel::Broadcast => Self::Broadcast,
            ExecutionModel::OrderedBroadcast(_) => Self::Broadcast,
            ExecutionModel::Sequential => Self::Sequential,
            ExecutionModel::OrderedSequential(_) => Self::Sequential,
            ExecutionModel::Mentioned { .. } => Self::Broadcast,
            ExecutionModel::Moderator => Self::Broadcast,
        }
    }
}

impl From<ExecutionModelType> for ExecutionModel {
    fn from(value: ExecutionModelType) -> Self {
        match value {
            ExecutionModelType::Broadcast => Self::Broadcast,
            ExecutionModelType::Sequential => Self::Sequential,
        }
    }
}

/// Conversation mode controlling verbosity and style.
///
/// Mirrors `crate::session::ConversationMode` for schema generation.
/// Enables automatic TypeScript generation:
/// `export type ConversationModeType = 'Normal' | 'Concise' | 'Brief' | 'Discussion'`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ConversationModeType {
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
}
