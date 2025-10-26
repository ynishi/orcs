//! Configuration domain models.
//!
//! Contains domain models for various configuration structures.

use serde::{Deserialize, Serialize};

// Re-export from persona module for backward compatibility
#[deprecated(since = "0.2.0", note = "Use orcs_core::persona::Persona instead")]
pub use crate::persona::Persona as PersonaConfig;

#[deprecated(since = "0.2.0", note = "Use orcs_core::persona::PersonaSource instead")]
pub use crate::persona::PersonaSource;

// ============================================================================
// Secret configuration models
// ============================================================================

/// Root configuration structure for secret.json.
///
/// Contains sensitive configuration data such as API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    /// Gemini API configuration
    #[serde(default)]
    pub gemini: Option<GeminiConfig>,
}

impl Default for SecretConfig {
    fn default() -> Self {
        Self { gemini: None }
    }
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
