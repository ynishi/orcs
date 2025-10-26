//! TOML-based PersonaRepository implementation

use crate::dto::create_persona_migrator;
use crate::paths::OrcsPaths;
use crate::storage::ConfigStorage;
use orcs_core::persona::Persona;
use orcs_core::repository::PersonaRepository;
use std::path::PathBuf;
use version_migrate::ConfigMigrator;

/// A repository implementation for storing persona configurations in a TOML file.
///
/// Responsibilities:
/// - Load/save personas from ConfigStorage
/// - Execute migrations (V1.0.0 → V1.1.0 → Persona)
/// - Convert between DTOs and domain models
///
/// Does NOT:
/// - Handle file locking (delegated to ConfigStorage)
/// - Know about TOML format (ConfigStorage returns serde_json::Value)
pub struct TomlPersonaRepository {
    storage: ConfigStorage,
}

impl TomlPersonaRepository {
    /// Creates a new repository with the default config path (~/.config/orcs/config.toml)
    pub fn new() -> Result<Self, String> {
        let config_path = Self::get_default_config_path()?;
        Ok(Self {
            storage: ConfigStorage::new(config_path),
        })
    }

    /// Creates a new repository with a custom config path (for testing)
    pub fn with_path(config_path: PathBuf) -> Self {
        Self {
            storage: ConfigStorage::new(config_path),
        }
    }

    /// Gets the default config path (~/.config/orcs/config.toml)
    fn get_default_config_path() -> Result<PathBuf, String> {
        OrcsPaths::config_file().map_err(|e| e.to_string())
    }
}

impl PersonaRepository for TomlPersonaRepository {
    fn get_all(&self) -> Result<Vec<Persona>, String> {
        // Load config as serde_json::Value
        let json_value = self
            .storage
            .load()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| serde_json::json!({"version": "1.0.0"}));

        // Convert to JSON string for ConfigMigrator
        let json_str = serde_json::to_string(&json_value)
            .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;

        // Use ConfigMigrator to query personas
        let config = ConfigMigrator::from(&json_str, create_persona_migrator())
            .map_err(|e| format!("Failed to create ConfigMigrator: {}", e))?;

        // Query personas - automatically migrates from any version to Persona domain model
        let personas: Vec<Persona> = config
            .query("persona")
            .map_err(|e| format!("Failed to query personas: {}", e))?;

        Ok(personas)
    }

    fn save_all(&self, personas: &[Persona]) -> Result<(), String> {
        // Load existing config to preserve other fields
        let json_value = self
            .storage
            .load()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| serde_json::json!({"version": "1.0.0"}));

        // Convert to JSON string for ConfigMigrator
        let json_str = serde_json::to_string(&json_value)
            .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;

        // Use ConfigMigrator to update personas
        let mut config = ConfigMigrator::from(&json_str, create_persona_migrator())
            .map_err(|e| format!("Failed to create ConfigMigrator: {}", e))?;

        // Update personas - automatically serializes to latest version
        config
            .update("persona", personas.to_vec())
            .map_err(|e| format!("Failed to update personas: {}", e))?;

        // Convert back to serde_json::Value and save
        let config_json_str = config
            .to_string()
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        let config_json_value: serde_json::Value = serde_json::from_str(&config_json_str)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

        // Save via ConfigStorage
        self.storage
            .save(&config_json_value)
            .map_err(|e| e.to_string())
    }
}
