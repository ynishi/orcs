//! Configuration domain models.
//!
//! Contains domain models for various configuration structures.

use serde::{Deserialize, Serialize};

// Re-export from persona module for backward compatibility
#[deprecated(since = "0.2.0", note = "Use orcs_core::persona::Persona instead")]
pub use crate::persona::Persona as PersonaConfig;

#[deprecated(
    since = "0.2.0",
    note = "Use orcs_core::persona::PersonaSource instead"
)]
pub use crate::persona::PersonaSource;

// ============================================================================
// Secret configuration models
// ============================================================================

/// Root configuration structure for secret.json.
///
/// Contains sensitive configuration data such as API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    /// Claude API configuration
    #[serde(default)]
    pub claude: Option<ClaudeConfig>,
    /// Gemini API configuration
    #[serde(default)]
    pub gemini: Option<GeminiConfig>,
    /// OpenAI API configuration
    #[serde(default)]
    pub openai: Option<OpenAIConfig>,
}

impl Default for SecretConfig {
    fn default() -> Self {
        Self {
            claude: None,
            gemini: None,
            openai: None,
        }
    }
}

/// Claude API configuration.
///
/// Contains API key and optional model name for Anthropic's Claude API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// API key for Claude API
    pub api_key: String,
    /// Optional model name (e.g., "claude-sonnet-4-20250514")
    #[serde(default)]
    pub model_name: Option<String>,
}

/// Gemini API configuration.
///
/// Contains API key and optional model name for Google's Gemini API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// API key for Gemini API
    pub api_key: String,
    /// Optional model name (e.g., "gemini-pro")
    #[serde(default)]
    pub model_name: Option<String>,
}

/// OpenAI API configuration.
///
/// Contains API key and optional model name for OpenAI's GPT API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// API key for OpenAI API
    pub api_key: String,
    /// Optional model name (e.g., "gpt-4o")
    #[serde(default)]
    pub model_name: Option<String>,
}
