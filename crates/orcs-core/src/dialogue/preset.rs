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
///
/// # JSON Serialization Format
///
/// Uses `#[serde(rename_all = "snake_case")]` to serialize as "system" or "user".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PresetSource {
    /// System-provided default presets
    System,
    /// User-created custom presets
    #[default]
    User,
}

/// A dialogue preset configuration.
///
/// Presets define the behavior of multi-agent conversations by bundling
/// execution strategy, conversation mode, and talk style into a single
/// named configuration that can be applied to sessions.
///
/// # JSON Serialization Format
///
/// This domain model uses `#[serde(rename_all = "camelCase")]` for Tauri IPC communication.
/// Dialogue presets are stored in `~/.orcs/dialogue_presets/*.json` with snake_case fields.
///
/// - **Tauri IPC** (this struct): Serialized as camelCase for TypeScript frontend
/// - **Disk persistence**: Currently no DTO layer, saved directly (needs migration if DTO added)
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[serde(rename_all = "camelCase")]
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

    /// Persona IDs to automatically add as participants when this preset is applied.
    /// Uses merge strategy: existing participants are kept, these are added (duplicates ignored).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_persona_ids: Vec<String>,
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
            name: "Brainstorm".to_string(),
            icon: Some("💡".to_string()),
            description: Some(
                "Broadcast + Concise + Brainstorm: Everyone freely shares ideas".to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Concise,
            talk_style: Some(TalkStyle::Brainstorm),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-code-review".to_string(),
            name: "Code Review".to_string(),
            icon: Some("🔍".to_string()),
            description: Some("Sequential + Brief + Review: Concise sequential review".to_string()),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::Review),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-discussion".to_string(),
            name: "Deep Discussion".to_string(),
            icon: Some("💭".to_string()),
            description: Some(
                "Broadcast + Discussion + Debate: Deep discussion with everyone".to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Discussion,
            talk_style: Some(TalkStyle::Debate),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-quick-decision".to_string(),
            name: "Quick Decision".to_string(),
            icon: Some("⚡".to_string()),
            description: Some(
                "Broadcast + Brief + DecisionMaking: Gather everyone's opinions concisely"
                    .to_string(),
            ),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::DecisionMaking),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-problem-solving".to_string(),
            name: "Problem Solving".to_string(),
            icon: Some("🔧".to_string()),
            description: Some(
                "Sequential + Concise + ProblemSolving: Sequential solution exploration"
                    .to_string(),
            ),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Concise,
            talk_style: Some(TalkStyle::ProblemSolving),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-planning".to_string(),
            name: "Planning".to_string(),
            icon: Some("📋".to_string()),
            description: Some("Sequential + Normal + Planning: Sequential planning".to_string()),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Normal,
            talk_style: Some(TalkStyle::Planning),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-casual-chat".to_string(),
            name: "Casual Chat".to_string(),
            icon: Some("☕".to_string()),
            description: Some("Broadcast + Normal + Casual: Relaxed conversation".to_string()),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Normal,
            talk_style: Some(TalkStyle::Casual),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
        DialoguePreset {
            id: "preset-research".to_string(),
            name: "Research".to_string(),
            icon: Some("🔬".to_string()),
            description: Some(
                "Sequential + Detailed + Research: Fact-focused deep investigation with WebSearch"
                    .to_string(),
            ),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Detailed,
            talk_style: Some(TalkStyle::Research),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
            default_persona_ids: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_presets_count() {
        let presets = get_default_presets();
        assert_eq!(presets.len(), 8, "Expected 8 system default presets");
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

        assert_eq!(brainstorm.name, "Brainstorm");
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

        assert_eq!(code_review.name, "Code Review");
        assert_eq!(code_review.icon, Some("🔍".to_string()));
        assert!(matches!(
            code_review.execution_strategy,
            ExecutionModel::Sequential
        ));
        assert_eq!(code_review.conversation_mode, ConversationMode::Brief);
        assert_eq!(code_review.talk_style, Some(TalkStyle::Review));
    }
}
