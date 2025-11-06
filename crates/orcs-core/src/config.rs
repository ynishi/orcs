//! Configuration domain models.
//!
//! Contains domain models for various configuration structures.

use serde::{Deserialize, Serialize};

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
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    /// User profile configuration (name, background, etc.).
    /// Changes infrequently.
    pub user_profile: UserProfile,
}

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            user_profile: UserProfile::default(),
        }
    }
}
