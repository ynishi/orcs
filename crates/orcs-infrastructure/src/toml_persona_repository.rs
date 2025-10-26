//! TOML-based PersonaRepository implementation

use crate::dto::{ConfigRoot, PersonaConfigV1_1_0};
use crate::storage::{create_persona_migrator, ConfigStorage};
use orcs_core::persona::Persona;
use orcs_core::repository::PersonaRepository;
use std::path::PathBuf;

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
        dirs::config_dir()
            .map(|dir| dir.join("orcs").join("config.toml"))
            .ok_or_else(|| "Cannot find config directory".to_string())
    }
}

impl PersonaRepository for TomlPersonaRepository {
    fn get_all(&self) -> Result<Vec<Persona>, String> {
        // Load config as serde_json::Value
        let json_value = self
            .storage
            .load()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| serde_json::json!({}));

        // Deserialize to ConfigRoot
        let config: ConfigRoot = serde_json::from_value(json_value)
            .map_err(|e| format!("Failed to deserialize config: {}", e))?;

        if config.personas.is_empty() {
            return Ok(Vec::new());
        }

        // Use version-migrate to handle automatic migration (flat format)
        let migrator = create_persona_migrator();
        let personas = migrator
            .load_vec_flat_from("persona", config.personas)
            .map_err(|e| format!("Failed to migrate personas: {}", e))?;

        Ok(personas)
    }

    fn save_all(&self, personas: &[Persona]) -> Result<(), String> {
        // Convert Persona domain models to latest DTO version (V1.1.0)
        let persona_dtos: Vec<PersonaConfigV1_1_0> =
            personas.iter().map(|p| p.into()).collect();

        // Use migrator to serialize with version field
        let migrator = create_persona_migrator();
        let json_str = migrator
            .save_vec_flat(persona_dtos)
            .map_err(|e| format!("Failed to serialize personas: {}", e))?;

        let persona_values: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse persona JSON: {}", e))?;

        // Create ConfigRoot with personas
        let config = ConfigRoot {
            personas: persona_values,
            user_profile: None, // TODO: preserve existing user_profile
            workspaces: Vec::new(), // TODO: preserve existing workspaces
        };

        // Serialize to serde_json::Value
        let json_value = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // Save via ConfigStorage
        self.storage
            .save(&json_value)
            .map_err(|e| e.to_string())
    }
}
