use crate::dto::{ConfigRoot, UserProfileDTO};
use crate::storage::{AtomicTomlFile, create_persona_migrator};
use orcs_core::persona::Persona;
use std::path::PathBuf;

// ============================================================================
// Low-level Config I/O - AtomicTomlFileを使用してACID保証
// ============================================================================

/// Gets the path to the config file.
fn get_config_path() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|dir| dir.join("orcs").join("config.toml"))
        .ok_or_else(|| "Cannot find config directory".to_string())
}

/// Gets an AtomicTomlFile handle for the config.
fn get_config_file() -> Result<AtomicTomlFile<ConfigRoot>, String> {
    let path = get_config_path()?;
    Ok(AtomicTomlFile::new(path))
}

// ============================================================================
// High-level API - AtomicTomlFileによるACID保証された操作
// ============================================================================

/// Loads persona configurations from the default config file path.
///
/// Uses version-migrate to automatically handle schema migration from any version
/// (V1.0.0, V1.1.0, etc.) to the latest domain model.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Returns
///
/// - `Ok(Vec<Persona>)`: A vector of persona configs.
///   If the file does not exist or is empty, returns an empty vector.
/// - `Err(String)`: An error message if the file exists but cannot be read or parsed.
pub fn load_personas() -> Result<Vec<Persona>, String> {
    let config_file = get_config_file()?;
    let config = config_file
        .load()
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

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

/// Saves persona configurations to the default config file path.
///
/// Uses atomic update with file locking to prevent data loss.
/// Personas are saved in the latest schema version (V1.1.0).
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Arguments
///
/// * `personas` - A slice of Persona structs to save.
///
/// # Returns
///
/// - `Ok(())`: If the file was successfully written.
/// - `Err(String)`: An error message if the operation failed.
pub fn save_personas(personas: &[Persona]) -> Result<(), String> {
    use crate::dto::PersonaConfigV1_1_0;

    let config_file = get_config_file()?;

    // Convert Persona domain models to latest DTO version (V1.1.0)
    let persona_dtos: Vec<PersonaConfigV1_1_0> = personas.iter().map(|p| p.into()).collect();

    // Convert DTOs to TOML values
    let persona_values: Vec<toml::Value> = persona_dtos
        .iter()
        .map(|dto| {
            let toml_string = toml::to_string(dto)
                .map_err(|e| format!("Failed to serialize persona: {}", e))?;
            toml::from_str(&toml_string)
                .map_err(|e| format!("Failed to parse persona TOML: {}", e))
        })
        .collect::<Result<Vec<_>, String>>()?;

    config_file
        .update(ConfigRoot::default(), |config| {
            config.personas = persona_values.clone();
            Ok(())
        })
        .map_err(|e| e.to_string())
}

/// Ensures user profile exists in config and is migrated to the latest version.
///
/// This should be called on application startup, similar to persona initialization.
///
/// # Returns
///
/// - `Ok(())`: If the profile exists or was successfully initialized.
/// - `Err(String)`: An error message if the operation failed.
pub fn ensure_user_profile_initialized() -> Result<(), String> {
    let config_file = get_config_file()?;

    config_file
        .update(ConfigRoot::default(), |config| {
            if config.user_profile.is_none() {
                config.user_profile = Some(UserProfileDTO::default());
            }
            Ok(())
        })
        .map_err(|e| e.to_string())
}

/// Loads user profile configuration from the default config file path.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Returns
///
/// - `Ok(String)`: The user's nickname. Returns "You" if no profile exists.
/// - `Err(String)`: An error message if the file cannot be read.
pub fn load_user_nickname() -> Result<String, String> {
    let config_file = get_config_file()?;
    let config = config_file
        .load()
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    Ok(config
        .user_profile
        .map(|up| up.nickname)
        .unwrap_or_else(|| "You".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{PersonaBackendDTO, PersonaSourceDTO};

    #[test]
    fn test_persona_serialization() {
        use crate::dto::PersonaConfigV1_1_0;

        let persona_dto = PersonaConfigV1_1_0 {
            id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
            name: "Mai".to_string(),
            role: "UX Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
            backend: PersonaBackendDTO::ClaudeCli,
        };

        // Convert DTO to TOML value
        let toml_value: toml::Value = toml::from_str(&toml::to_string(&persona_dto).unwrap()).unwrap();

        let config = ConfigRoot {
            personas: vec![toml_value],
            user_profile: None,
            workspaces: Vec::new(),
        };

        let toml_string = toml::to_string_pretty(&config).unwrap();

        assert!(
            toml_string.contains("id = \"8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c\""),
            "Serialized TOML should contain UUID"
        );
        assert!(
            toml_string.contains("name = \"Mai\""),
            "Serialized TOML should contain name"
        );
    }

    #[test]
    fn test_atomic_save_and_load() {
        use crate::dto::PersonaConfigV1_1_0;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let atomic_file = AtomicTomlFile::<ConfigRoot>::new(config_path);

        // Create test persona
        let persona_dto = PersonaConfigV1_1_0 {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Test".to_string(),
            role: "Tester".to_string(),
            background: "Test background".to_string(),
            communication_style: "Test style".to_string(),
            default_participant: false,
            source: PersonaSourceDTO::User,
            backend: PersonaBackendDTO::ClaudeCli,
        };

        // Convert DTO to TOML value
        let toml_value: toml::Value = toml::from_str(&toml::to_string(&persona_dto).unwrap()).unwrap();

        let config = ConfigRoot {
            personas: vec![toml_value],
            user_profile: Some(UserProfileDTO {
                nickname: "TestUser".to_string(),
                background: "Test background".to_string(),
            }),
            workspaces: Vec::new(),
        };

        // Save
        atomic_file.save(&config).unwrap();

        // Load
        let loaded = atomic_file.load().unwrap().unwrap();
        assert_eq!(loaded.personas.len(), 1);

        // Parse persona back from TOML value
        let loaded_persona: PersonaConfigV1_1_0 =
            toml::from_str(&toml::to_string(&loaded.personas[0]).unwrap()).unwrap();
        assert_eq!(loaded_persona.name, "Test");
        assert_eq!(loaded.user_profile.as_ref().unwrap().nickname, "TestUser");
    }
}
