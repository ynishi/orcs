//! Dialogue preset DTOs and migrations

use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use orcs_core::dialogue::{DialoguePreset, PresetSource};
use orcs_core::session::ConversationMode;
use serde::{Deserialize, Serialize};
use version_migrate::{FromDomain, IntoDomain, MigratesTo, Versioned};

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
}

/// Migration from V1.0.0 to V1.1.0:
/// - Add default_persona_ids field
/// - Convert execution_strategy from JSON String to typed ExecutionModel
/// - Convert talk_style from JSON String to typed Option<TalkStyle>
impl MigratesTo<DialoguePresetV1_1_0> for DialoguePresetV1_0_0 {
    fn migrate(self) -> DialoguePresetV1_1_0 {
        DialoguePresetV1_1_0 {
            id: self.id,
            name: self.name,
            icon: self.icon,
            description: self.description,
            execution_strategy: serde_json::from_str(&self.execution_strategy)
                .unwrap_or(ExecutionModel::Broadcast),
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style.and_then(|s| serde_json::from_str(&s).ok()),
            created_at: self.created_at,
            source: self.source,
            default_persona_ids: vec![],
        }
    }
}

/// Dialogue preset DTO V1.1.0
///
/// Changes from V1.0.0:
/// - `execution_strategy`: `String` (JSON) -> `ExecutionModel` (typed)
/// - `talk_style`: `Option<String>` (JSON) -> `Option<TalkStyle>` (typed)
/// - Added `default_persona_ids`: `Vec<String>`
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct DialoguePresetV1_1_0 {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub execution_strategy: ExecutionModel,
    pub conversation_mode: ConversationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<TalkStyle>,
    pub created_at: String,
    #[serde(default)]
    pub source: PresetSource,
    /// Persona IDs to auto-add when preset is applied (empty = none)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_persona_ids: Vec<String>,
}

/// Convert DialoguePresetV1_1_0 DTO to domain model
impl IntoDomain<DialoguePreset> for DialoguePresetV1_1_0 {
    fn into_domain(self) -> DialoguePreset {
        DialoguePreset {
            id: self.id,
            name: self.name,
            icon: self.icon,
            description: self.description,
            execution_strategy: self.execution_strategy,
            conversation_mode: self.conversation_mode,
            talk_style: self.talk_style,
            created_at: self.created_at,
            source: self.source,
            default_persona_ids: self.default_persona_ids,
        }
    }
}

/// Convert domain model to DialoguePresetV1_1_0 DTO for persistence
impl FromDomain<DialoguePreset> for DialoguePresetV1_1_0 {
    fn from_domain(preset: DialoguePreset) -> Self {
        DialoguePresetV1_1_0 {
            id: preset.id,
            name: preset.name,
            icon: preset.icon,
            description: preset.description,
            execution_strategy: preset.execution_strategy,
            conversation_mode: preset.conversation_mode,
            talk_style: preset.talk_style,
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
    version_migrate::migrator!("dialogue_preset" => [DialoguePresetV1_0_0, DialoguePresetV1_1_0, DialoguePreset], save = true)
        .expect("Failed to create dialogue_preset migrator")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_v1_0_0(execution_strategy: &str, talk_style: Option<&str>) -> DialoguePresetV1_0_0 {
        DialoguePresetV1_0_0 {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            icon: Some("T".to_string()),
            description: Some("desc".to_string()),
            execution_strategy: execution_strategy.to_string(),
            conversation_mode: ConversationMode::Normal,
            talk_style: talk_style.map(|s| s.to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            source: PresetSource::User,
        }
    }

    #[test]
    fn migrate_v1_0_0_to_v1_1_0_broadcast() {
        // ExecutionModel uses snake_case, TalkStyle uses PascalCase
        let v1 = make_v1_0_0(r#""broadcast""#, Some(r#""Brainstorm""#));
        let v1_1 = v1.migrate();

        assert!(matches!(v1_1.execution_strategy, ExecutionModel::Broadcast));
        assert_eq!(v1_1.talk_style, Some(TalkStyle::Brainstorm));
        assert!(v1_1.default_persona_ids.is_empty());
    }

    #[test]
    fn migrate_v1_0_0_to_v1_1_0_sequential() {
        let v1 = make_v1_0_0(r#""sequential""#, Some(r#""Review""#));
        let v1_1 = v1.migrate();

        assert!(matches!(
            v1_1.execution_strategy,
            ExecutionModel::Sequential
        ));
        assert_eq!(v1_1.talk_style, Some(TalkStyle::Review));
    }

    #[test]
    fn migrate_v1_0_0_to_v1_1_0_none_talk_style() {
        let v1 = make_v1_0_0(r#""broadcast""#, None);
        let v1_1 = v1.migrate();

        assert_eq!(v1_1.talk_style, None);
    }

    #[test]
    fn migrate_v1_0_0_to_v1_1_0_invalid_strategy_falls_back_to_broadcast() {
        let v1 = make_v1_0_0("invalid-json", Some(r#""Casual""#));
        let v1_1 = v1.migrate();

        assert!(matches!(v1_1.execution_strategy, ExecutionModel::Broadcast));
        assert_eq!(v1_1.talk_style, Some(TalkStyle::Casual));
    }

    #[test]
    fn migrate_v1_0_0_to_v1_1_0_invalid_talk_style_becomes_none() {
        let v1 = make_v1_0_0(r#""broadcast""#, Some("invalid-json"));
        let v1_1 = v1.migrate();

        assert_eq!(v1_1.talk_style, None);
    }

    #[test]
    fn into_domain_roundtrip() {
        let domain = DialoguePreset {
            id: "roundtrip".to_string(),
            name: "Roundtrip".to_string(),
            icon: Some("R".to_string()),
            description: Some("desc".to_string()),
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::Planning),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            source: PresetSource::User,
            default_persona_ids: vec!["p1".to_string(), "p2".to_string()],
        };

        let dto = DialoguePresetV1_1_0::from_domain(domain.clone());
        let restored = dto.into_domain();

        assert_eq!(restored.id, domain.id);
        assert_eq!(restored.name, domain.name);
        assert_eq!(restored.icon, domain.icon);
        assert_eq!(restored.description, domain.description);
        assert!(matches!(
            restored.execution_strategy,
            ExecutionModel::Sequential
        ));
        assert_eq!(restored.conversation_mode, domain.conversation_mode);
        assert_eq!(restored.talk_style, domain.talk_style);
        assert_eq!(restored.created_at, domain.created_at);
        assert_eq!(restored.source, domain.source);
        assert_eq!(restored.default_persona_ids, domain.default_persona_ids);
    }

    #[test]
    fn v1_1_0_serde_roundtrip() {
        let dto = DialoguePresetV1_1_0 {
            id: "serde-test".to_string(),
            name: "Serde".to_string(),
            icon: None,
            description: None,
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Concise,
            talk_style: Some(TalkStyle::Debate),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            source: PresetSource::System,
            default_persona_ids: vec!["persona-a".to_string()],
        };

        let json = serde_json::to_string(&dto).expect("serialize");
        let restored: DialoguePresetV1_1_0 = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.id, dto.id);
        assert!(matches!(
            restored.execution_strategy,
            ExecutionModel::Broadcast
        ));
        assert_eq!(restored.talk_style, Some(TalkStyle::Debate));
        assert_eq!(restored.default_persona_ids, vec!["persona-a"]);
    }

    #[test]
    fn v1_1_0_empty_persona_ids_skipped_in_json() {
        let dto = DialoguePresetV1_1_0 {
            id: "skip-test".to_string(),
            name: "Skip".to_string(),
            icon: None,
            description: None,
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Normal,
            talk_style: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            source: PresetSource::User,
            default_persona_ids: vec![],
        };

        let json = serde_json::to_string(&dto).expect("serialize");
        assert!(!json.contains("default_persona_ids"));

        // Deserialize back - default should kick in
        let restored: DialoguePresetV1_1_0 = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.default_persona_ids.is_empty());
    }
}
