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
