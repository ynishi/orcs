//! Configuration domain models.
//!
//! Contains domain models for various configuration structures.

use serde::{Deserialize, Serialize};
use version_migrate::Queryable;

use crate::user::UserProfile;

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

impl Queryable for SecretConfig {
    const ENTITY_NAME: &'static str = "secret";
}

/// Claude API secret configuration.
///
/// Contains only sensitive data (API key).
/// Model settings are stored separately in ModelSettings (config.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// API key for Claude API
    pub api_key: String,
}

/// Gemini API secret configuration.
///
/// Contains only sensitive data (API key).
/// Model settings are stored separately in ModelSettings (config.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// API key for Gemini API
    pub api_key: String,
}

/// OpenAI API secret configuration.
///
/// Contains only sensitive data (API key).
/// Model settings are stored separately in ModelSettings (config.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// API key for OpenAI API
    pub api_key: String,
}

// ============================================================================
// Model configuration models
// ============================================================================

/// LLM model settings for each provider.
///
/// This is non-sensitive configuration and belongs in config.toml, not secret.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    /// Claude model configuration
    #[serde(default)]
    pub claude: Option<ClaudeModelConfig>,
    /// Gemini model configuration
    #[serde(default)]
    pub gemini: Option<GeminiModelConfig>,
    /// OpenAI model configuration
    #[serde(default)]
    pub openai: Option<OpenAIModelConfig>,
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            claude: Some(ClaudeModelConfig::default()),
            gemini: Some(GeminiModelConfig::default()),
            openai: Some(OpenAIModelConfig::default()),
        }
    }
}

/// Claude model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeModelConfig {
    /// Model name (e.g., "claude-sonnet-4-20250514")
    pub model_name: String,
}

impl Default for ClaudeModelConfig {
    fn default() -> Self {
        Self {
            model_name: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

/// Gemini model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiModelConfig {
    /// Model name (e.g., "gemini-2.5-flash")
    pub model_name: String,
}

impl Default for GeminiModelConfig {
    fn default() -> Self {
        Self {
            model_name: "gemini-2.5-flash".to_string(),
        }
    }
}

/// OpenAI model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModelConfig {
    /// Model name (e.g., "gpt-4o")
    pub model_name: String,
}

impl Default for OpenAIModelConfig {
    fn default() -> Self {
        Self {
            model_name: "gpt-4o".to_string(),
        }
    }
}

// ============================================================================
// Root configuration model (Domain layer)
// ============================================================================

/// Root configuration structure for the application (Domain model).
///
/// This is the domain representation of application configuration stored in config.toml.
/// Contains static configuration that changes infrequently.
///
/// Infrastructure layer DTOs (ConfigRootV1_x_x) are converted to/from this type.
///
/// # File Location
///
/// - macOS: `~/Library/Preferences/com.orcs-app/config.toml`
/// - Linux: `~/.config/com.orcs-app/config.toml`
/// - Windows: `%APPDATA%\com.orcs-app\config.toml`
///
/// # Design Notes
///
/// - **Secrets (API Keys)**: Managed separately in `secret.json`
/// - **Model Settings**: Stored here (non-sensitive configuration)
/// - **Personas**: Managed separately in `DataDir/personas/` directory
/// - **AppState**: Managed separately in `PrefDir/state.toml` (frequently updated)
/// - **Workspaces**: Managed separately in `DataDir/content/workspaces/` directory
///
/// # Usage
///
/// Always use this type in application and domain logic.
/// Infrastructure layer handles versioning and migration internally.
///
/// ```ignore
/// let config = RootConfig::default();
/// let nickname = config.user_profile.nickname;
/// let claude_model = config.model_settings.claude.unwrap().model_name;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    /// User profile configuration (name, background, etc.).
    /// Changes infrequently.
    pub user_profile: UserProfile,
    /// LLM model settings for each provider.
    /// Non-sensitive configuration.
    #[serde(default)]
    pub model_settings: ModelSettings,
}

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            user_profile: UserProfile::default(),
            model_settings: ModelSettings::default(),
        }
    }
}
