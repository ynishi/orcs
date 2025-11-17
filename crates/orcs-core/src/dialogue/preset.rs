//! Dialogue preset models and default configurations.
//!
//! A DialoguePreset captures a specific combination of:
//! - ExecutionStrategy (Broadcast/Sequential/Mentioned)
//! - ConversationMode (Normal/Concise/Brief/Discussion)
//! - TalkStyle (Brainstorm/Casual/DecisionMaking/etc.)
//!
//! These presets allow users to quickly switch between common dialogue patterns
//! like "brainstorming session" or "code review" without manually configuring
//! each setting.

use crate::session::ConversationMode;
use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// Source of a dialogue preset (system-provided or user-created).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetSource {
    /// System-provided default presets
    System,
    /// User-created custom presets
    User,
}

impl Default for PresetSource {
    fn default() -> Self {
        PresetSource::User
    }
}

/// A dialogue preset configuration.
///
/// Presets define the behavior of multi-agent conversations by bundling
/// execution strategy, conversation mode, and talk style into a single
/// named configuration that can be applied to sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[queryable(entity = "dialogue_preset")]
pub struct DialoguePreset {
    /// Unique identifier (UUID format)
    pub id: String,

    /// Display name of the preset (e.g., "アイデア出し", "コードレビュー")
    pub name: String,

    /// Visual icon/emoji representing this preset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Description of this preset's purpose and behavior
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Execution strategy for this preset
    pub execution_strategy: ExecutionModel,

    /// Conversation mode for this preset
    pub conversation_mode: ConversationMode,

    /// Talk style for this preset (None = default/normal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,

    /// Timestamp when the preset was created (ISO 8601 format)
    pub created_at: String,

    /// Source of the preset (System or User)
    #[serde(default)]
    pub source: PresetSource,
}

/// Returns the system-defined default dialogue presets.
///
/// These presets cover common multi-agent conversation patterns:
/// - Brainstorming sessions (Broadcast + Concise + Brainstorm)
/// - Code reviews (Sequential + Brief + Review)
/// - Deep discussions (Broadcast + Discussion + Debate)
/// - Quick decision making (Broadcast + Brief + DecisionMaking)
/// - Problem solving (Sequential + Concise + ProblemSolving)
/// - Planning sessions (Sequential + Normal + Planning)
pub fn get_default_presets() -> Vec<DialoguePreset> {
    vec![
        DialoguePreset {
            id: "preset-brainstorm".to_string(),
            name: "アイデア出し".to_string(),
            icon: Some("💡".to_string()),
            description: Some(
                "Broadcast + 簡潔 + Brainstorm: 全員が自由にアイデアを出し合う".to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Concise,
            talk_style: Some(TalkStyle::Brainstorm),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
        DialoguePreset {
            id: "preset-code-review".to_string(),
            name: "コードレビュー".to_string(),
            icon: Some("🔍".to_string()),
            description: Some(
                "Sequential + 極簡潔 + Review: 順番に簡潔にレビュー".to_string(),
            ),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::Review),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
        DialoguePreset {
            id: "preset-discussion".to_string(),
            name: "深い議論".to_string(),
            icon: Some("💭".to_string()),
            description: Some(
                "Broadcast + 議論 + Debate: 全員で深く議論".to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Discussion,
            talk_style: Some(TalkStyle::Debate),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
        DialoguePreset {
            id: "preset-quick-decision".to_string(),
            name: "素早い意思決定".to_string(),
            icon: Some("⚡".to_string()),
            description: Some(
                "Broadcast + 極簡潔 + DecisionMaking: 簡潔に全員の意見を集める".to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::DecisionMaking),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
        DialoguePreset {
            id: "preset-problem-solving".to_string(),
            name: "問題解決".to_string(),
            icon: Some("🔧".to_string()),
            description: Some(
                "Sequential + 簡潔 + ProblemSolving: 順番に解決策を検討".to_string(),
            ),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Concise,
            talk_style: Some(TalkStyle::ProblemSolving),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
        DialoguePreset {
            id: "preset-planning".to_string(),
            name: "計画立案".to_string(),
            icon: Some("📋".to_string()),
            description: Some(
                "Sequential + 通常 + Planning: 順番に計画を立てる".to_string(),
            ),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Normal,
            talk_style: Some(TalkStyle::Planning),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
        DialoguePreset {
            id: "preset-casual-chat".to_string(),
            name: "カジュアル雑談".to_string(),
            icon: Some("☕".to_string()),
            description: Some(
                "Broadcast + 通常 + Casual: 気楽に会話".to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Normal,
            talk_style: Some(TalkStyle::Casual),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_presets_count() {
        let presets = get_default_presets();
        assert_eq!(presets.len(), 7, "Expected 7 system default presets");
    }

    #[test]
    fn test_default_presets_have_system_source() {
        let presets = get_default_presets();
        for preset in presets {
            assert_eq!(
                preset.source,
                PresetSource::System,
                "All default presets should have System source"
            );
        }
    }

    #[test]
    fn test_default_presets_have_unique_ids() {
        let presets = get_default_presets();
        let mut ids = std::collections::HashSet::new();
        for preset in presets {
            assert!(
                ids.insert(preset.id.clone()),
                "Preset IDs must be unique, found duplicate: {}",
                preset.id
            );
        }
    }

    #[test]
    fn test_brainstorm_preset_configuration() {
        let presets = get_default_presets();
        let brainstorm = presets
            .iter()
            .find(|p| p.id == "preset-brainstorm")
            .expect("Brainstorm preset should exist");

        assert_eq!(brainstorm.name, "アイデア出し");
        assert_eq!(brainstorm.icon, Some("💡".to_string()));
        assert!(matches!(
            brainstorm.execution_strategy,
            ExecutionModel::Broadcast
        ));
        assert_eq!(brainstorm.conversation_mode, ConversationMode::Concise);
        assert_eq!(brainstorm.talk_style, Some(TalkStyle::Brainstorm));
    }

    #[test]
    fn test_code_review_preset_configuration() {
        let presets = get_default_presets();
        let code_review = presets
            .iter()
            .find(|p| p.id == "preset-code-review")
            .expect("Code review preset should exist");

        assert_eq!(code_review.name, "コードレビュー");
        assert_eq!(code_review.icon, Some("🔍".to_string()));
        assert!(matches!(
            code_review.execution_strategy,
            ExecutionModel::Sequential
        ));
        assert_eq!(code_review.conversation_mode, ConversationMode::Brief);
        assert_eq!(code_review.talk_style, Some(TalkStyle::Review));
    }
}
