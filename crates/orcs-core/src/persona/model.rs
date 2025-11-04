//! Persona domain model.
//!
//! Represents AI personas that participate in conversations with users.
//! Each persona has unique characteristics, roles, and communication styles.

use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// Supported LLM backends for personas.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaBackend {
    /// Anthropic Claude Code CLI backend
    ClaudeCli,
    /// Anthropic Claude API backend
    ClaudeApi,
    /// Google Gemini CLI backend
    GeminiCli,
    /// Google Gemini API backend
    GeminiApi,
    /// OpenAI API backend (GPT-4, GPT-3.5, etc.)
    OpenAiApi,
    /// Codex CLI backend
    CodexCli,
}

impl PersonaBackend {
    /// Returns all available backend variants with their display names.
    pub fn all_variants() -> Vec<(String, String)> {
        vec![
            ("claude_cli".to_string(), "Claude CLI".to_string()),
            ("claude_api".to_string(), "Claude API".to_string()),
            ("gemini_cli".to_string(), "Gemini CLI".to_string()),
            ("gemini_api".to_string(), "Gemini API".to_string()),
            ("open_ai_api".to_string(), "OpenAI API".to_string()),
            ("codex_cli".to_string(), "Codex CLI".to_string()),
        ]
    }
}

impl Default for PersonaBackend {
    fn default() -> Self {
        PersonaBackend::ClaudeCli
    }
}

/// Represents the source of a persona (system-provided or user-created).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum PersonaSource {
    /// System-provided default personas
    System,
    /// User-created custom personas
    User,
}

impl Default for PersonaSource {
    fn default() -> Self {
        PersonaSource::User
    }
}

/// A persona representing an AI agent with specific characteristics and expertise.
///
/// Personas define the behavior, expertise, and communication style of AI agents
/// participating in conversations. Each persona has a unique UUID identifier.
#[derive(Deserialize, Serialize, Debug, Clone, Queryable)]
#[queryable(entity = "persona")]
pub struct Persona {
    /// Unique identifier (UUID format)
    pub id: String,
    /// Display name of the persona
    pub name: String,
    /// Role or title describing the persona's expertise
    pub role: String,
    /// Background description of the persona's capabilities
    pub background: String,
    /// Communication style characteristics
    pub communication_style: String,
    /// Whether this persona is included in new sessions by default
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User)
    #[serde(default)]
    pub source: PersonaSource,
    /// Backend used to execute this persona
    #[serde(default)]
    pub backend: PersonaBackend,
    /// Model name for the backend (e.g., "claude-sonnet-4.5", "gemini-2.5-flash")
    /// If None, uses the backend's default model
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_backend_all_variants() {
        let variants = PersonaBackend::all_variants();

        // Should have exactly 6 backend options
        assert_eq!(variants.len(), 6);

        // Verify each variant exists and has correct snake_case format
        let keys: Vec<String> = variants.iter().map(|(k, _)| k.clone()).collect();
        assert!(keys.contains(&"claude_cli".to_string()));
        assert!(keys.contains(&"claude_api".to_string()));
        assert!(keys.contains(&"gemini_cli".to_string()));
        assert!(keys.contains(&"gemini_api".to_string()));
        assert!(keys.contains(&"open_ai_api".to_string())); // Note: two underscores
        assert!(keys.contains(&"codex_cli".to_string()));

        // Verify display names are present
        let labels: Vec<String> = variants.iter().map(|(_, v)| v.clone()).collect();
        assert!(labels.contains(&"Claude CLI".to_string()));
        assert!(labels.contains(&"OpenAI API".to_string()));
    }

    #[test]
    fn test_persona_backend_serialization() {
        // Test that OpenAiApi serializes to "open_ai_api" (with two underscores)
        let backend = PersonaBackend::OpenAiApi;
        let serialized = serde_json::to_string(&backend).unwrap();
        assert_eq!(serialized, r#""open_ai_api""#);

        // Test deserialization
        let deserialized: PersonaBackend = serde_json::from_str(r#""open_ai_api""#).unwrap();
        assert_eq!(deserialized, PersonaBackend::OpenAiApi);
    }

    #[test]
    fn test_all_variants_match_enum() {
        // Ensure all_variants() returns keys that can be deserialized
        let variants = PersonaBackend::all_variants();

        for (key, _label) in variants {
            let json = format!(r#""{}""#, key);
            let result: Result<PersonaBackend, _> = serde_json::from_str(&json);
            assert!(result.is_ok(), "Failed to deserialize variant key: {}", key);
        }
    }
}
