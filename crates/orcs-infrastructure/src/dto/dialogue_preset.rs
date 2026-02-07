//! Dialogue preset DTOs and migrations

use orcs_core::dialogue::{DialoguePreset, PresetSource};
use orcs_core::session::ConversationMode;
use serde::{Deserialize, Serialize};
use version_migrate::{FromDomain, IntoDomain, Versioned};

/// Dialogue preset DTO V1.0.0
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct DialoguePresetV1_0_0 {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON-serialized ExecutionModel
    pub execution_strategy: String,
    pub conversation_mode: ConversationMode,
    /// JSON-serialized TalkStyle (None = no style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub source: PresetSource,
    /// Persona IDs to auto-add when preset is applied (empty = none)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_persona_ids: Vec<String>,
}

/// Convert DialoguePresetV1_0_0 DTO to domain model
impl IntoDomain<DialoguePreset> for DialoguePresetV1_0_0 {
    fn into_domain(self) -> DialoguePreset {
        DialoguePreset {
            id: self.id,
            name: self.name,
            icon: self.icon,
            description: self.description,
            execution_strategy: serde_json::from_str(&self.execution_strategy)
                .unwrap_or(llm_toolkit::agent::dialogue::ExecutionModel::Broadcast),
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style.and_then(|s| serde_json::from_str(&s).ok()),
            created_at: self.created_at,
            source: self.source,
            default_persona_ids: self.default_persona_ids,
        }
    }
}

/// Convert domain model to DialoguePresetV1_0_0 DTO for persistence
impl FromDomain<DialoguePreset> for DialoguePresetV1_0_0 {
    fn from_domain(preset: DialoguePreset) -> Self {
        DialoguePresetV1_0_0 {
            id: preset.id,
            name: preset.name,
            icon: preset.icon,
            description: preset.description,
            execution_strategy: serde_json::to_string(&preset.execution_strategy)
                .unwrap_or_else(|_| r#"{"type":"broadcast"}"#.to_string()),
            conversation_mode: preset.conversation_mode,
            talk_style: preset
                .talk_style
                .and_then(|s| serde_json::to_string(&s).ok()),
            created_at: preset.created_at,
            source: preset.source,
            default_persona_ids: preset.default_persona_ids,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates a Migrator for DialoguePreset entities.
pub fn create_dialogue_preset_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("dialogue_preset" => [DialoguePresetV1_0_0, DialoguePreset], save = true)
        .expect("Failed to create dialogue_preset migrator")
}
