//! Secret configuration DTOs and migrator.

use orcs_core::config::{ClaudeConfig, GeminiConfig, OpenAIConfig, SecretConfig};
use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, Versioned};

/// Secret configuration schema V1.0.0.
///
/// Stores provider API keys in `secret.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct SecretConfigV1_0_0 {
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

impl Default for SecretConfigV1_0_0 {
    fn default() -> Self {
        let default = SecretConfig::default();
        Self {
            claude: default.claude,
            gemini: default.gemini,
            openai: default.openai,
        }
    }
}

// ============================================================================
// Domain conversions
// ============================================================================

impl IntoDomain<SecretConfig> for SecretConfigV1_0_0 {
    fn into_domain(self) -> SecretConfig {
        SecretConfig {
            claude: self.claude,
            gemini: self.gemini,
            openai: self.openai,
        }
    }
}

impl version_migrate::FromDomain<SecretConfig> for SecretConfigV1_0_0 {
    fn from_domain(config: SecretConfig) -> Self {
        SecretConfigV1_0_0 {
            claude: config.claude,
            gemini: config.gemini,
            openai: config.openai,
        }
    }
}

// ============================================================================
// Migrator
// ============================================================================

/// Creates a migrator for secret configuration entities.
pub fn create_secret_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("secret" => [SecretConfigV1_0_0, SecretConfig], save = true)
        .expect("Failed to create secret migrator")
}
