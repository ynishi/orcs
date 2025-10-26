//! Configuration file management for ORCS.
//!
//! Supports reading secrets from `~/.config/orcs/secret.json`.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Root configuration structure for secret.json
#[derive(Debug, Clone, Deserialize)]
pub struct SecretConfig {
    #[serde(default)]
    pub gemini: Option<GeminiConfig>,
}

/// Gemini API configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    #[serde(default)]
    pub model_name: Option<String>,
}

/// Loads the secret configuration file from ~/.config/orcs/secret.json
pub fn load_secret_config() -> Result<SecretConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(format!(
            "Configuration file not found at: {}",
            config_path.display()
        ));
    }

    let content = fs::read_to_string(&config_path).map_err(|e| {
        format!(
            "Failed to read configuration file at {}: {}",
            config_path.display(),
            e
        )
    })?;

    serde_json::from_str(&content).map_err(|e| {
        format!(
            "Failed to parse configuration file at {}: {}",
            config_path.display(),
            e
        )
    })
}

/// Returns the path to the configuration file: ~/.config/orcs/secret.json
fn get_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    Ok(home.join(".config").join("orcs").join("secret.json"))
}
