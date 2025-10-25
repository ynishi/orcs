use orcs_core::persona::Persona;
use crate::dto::{ConfigRootV1, ConfigRootV2, PersonaConfigV2, UserProfileDTO, USER_PROFILE_V1_1_VERSION};
use std::fs;

// ============================================================================
// Low-level Config I/O - これらが全体の読み書きを担当
// ============================================================================

/// Loads the entire config file as ConfigRootV2.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Returns
///
/// - `Ok(ConfigRootV2)`: The complete configuration.
///   If the file doesn't exist or is empty, returns a default empty config.
/// - `Err(String)`: An error if the file exists but cannot be read or parsed.
fn load_config() -> Result<ConfigRootV2, String> {
    match dirs::config_dir() {
        Some(config_dir) => {
            let config_path = config_dir.join("orcs").join("config.toml");

            if !config_path.exists() {
                // No config file - return empty config
                return Ok(ConfigRootV2 {
                    personas: Vec::new(),
                    user_profile: None,
                });
            }

            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

            if content.trim().is_empty() {
                // Empty file - return empty config
                return Ok(ConfigRootV2 {
                    personas: Vec::new(),
                    user_profile: None,
                });
            }

            // Try V2 format first
            if let Ok(root_dto) = toml::from_str::<ConfigRootV2>(&content) {
                return Ok(root_dto);
            }

            // Fallback to V1 format and migrate
            if let Ok(root_v1) = toml::from_str::<ConfigRootV1>(&content) {
                use version_migrate::MigratesTo;

                let personas: Vec<PersonaConfigV2> = root_v1.personas.into_iter()
                    .map(|v1_dto| v1_dto.migrate())  // PersonaConfigV1 -> PersonaConfigV2
                    .collect();

                return Ok(ConfigRootV2 {
                    personas,
                    user_profile: None, // V1 didn't have user_profile
                });
            }

            Err(format!("Failed to parse TOML from {:?}: unsupported schema version", config_path))
        }
        None => {
            // Cannot find config dir - return empty config
            Ok(ConfigRootV2 {
                personas: Vec::new(),
                user_profile: None,
            })
        }
    }
}

/// Saves the entire config file from ConfigRootV2.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Arguments
///
/// * `config` - The complete ConfigRootV2 to save.
///
/// # Returns
///
/// - `Ok(())`: If the file was successfully written.
/// - `Err(String)`: An error message if the operation failed.
fn save_config(config: &ConfigRootV2) -> Result<(), String> {
    match dirs::config_dir() {
        Some(config_dir) => {
            let orcs_config_dir = config_dir.join("orcs");
            let config_path = orcs_config_dir.join("config.toml");

            // Create the directory if it doesn't exist
            if !orcs_config_dir.exists() {
                fs::create_dir_all(&orcs_config_dir)
                    .map_err(|e| format!("Failed to create config directory at {:?}: {}", orcs_config_dir, e))?;
            }

            // Serialize to TOML
            let toml_string = toml::to_string_pretty(config)
                .map_err(|e| format!("Failed to serialize config to TOML: {}", e))?;

            // Write to file
            fs::write(&config_path, toml_string)
                .map_err(|e| format!("Failed to write config file at {:?}: {}", config_path, e))?;

            Ok(())
        }
        None => Err("Cannot find config directory".to_string()),
    }
}

// ============================================================================
// High-level API - これらが load_config/save_config を使って部分的な操作を提供
// ============================================================================

/// Loads persona configurations from the default config file path.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Returns
///
/// - `Ok(Vec<Persona>)`: A vector of persona configs.
///   If the file does not exist or is empty, returns an empty vector.
/// - `Err(String)`: An error message if the file exists but cannot be read or parsed.
pub fn load_personas() -> Result<Vec<Persona>, String> {
    use version_migrate::IntoDomain;

    let config = load_config()?;
    let personas: Vec<Persona> = config.personas.into_iter()
        .map(|dto| dto.into_domain())
        .collect();
    Ok(personas)
}

/// Saves persona configurations to the default config file path.
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
/// - `Err(String)`: An error message if the config directory cannot be found,
///   the directory cannot be created, or the file cannot be written.
pub fn save_personas(personas: &[Persona]) -> Result<(), String> {
    // Load the entire config
    let mut config = load_config()?;

    // Update only the personas part
    let persona_dtos: Vec<PersonaConfigV2> = personas.iter()
        .map(|p| p.into())
        .collect();
    config.personas = persona_dtos;

    // Save the entire config back
    save_config(&config)
}

/// Ensures user profile exists in config and is migrated to the latest version.
///
/// This should be called on application startup, similar to persona initialization.
///
/// # Returns
///
/// - `Ok(())`: If the profile exists or was successfully initialized and migrated.
/// - `Err(String)`: An error message if the operation failed.
pub fn ensure_user_profile_initialized() -> Result<(), String> {
    let mut config = load_config()?;
    let mut needs_save = false;

    // Initialize if missing
    if config.user_profile.is_none() {
        config.user_profile = Some(UserProfileDTO::default());
        needs_save = true;
    } else {
        // Auto-migrate: check if schema_version needs update
        if let Some(ref profile) = config.user_profile {
            if profile.schema_version != USER_PROFILE_V1_1_VERSION {
                // Update to latest version
                let mut updated_profile = profile.clone();
                updated_profile.schema_version = USER_PROFILE_V1_1_VERSION.to_string();
                config.user_profile = Some(updated_profile);
                needs_save = true;
            }
        }
    }

    if needs_save {
        save_config(&config)?;
    }

    Ok(())
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
    let config = load_config()?;
    Ok(config.user_profile
        .map(|up| up.nickname)
        .unwrap_or_else(|| "You".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_and_save_v1_config_migrates_to_v2() {
        // Create a temporary V1 config file
        let v1_toml = r#"
[[persona]]
schema_version = "1.0.0"
id = "mai"
name = "Mai"
role = "UX Engineer"
background = "Background"
communication_style = "Friendly"
default_participant = true
source = "System"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(v1_toml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Read the V1 config
        let content = fs::read_to_string(temp_file.path()).unwrap();

        // Try to load as V1
        use version_migrate::{MigratesTo, IntoDomain};

        let root_dto = toml::from_str::<ConfigRootV1>(&content).unwrap();
        let personas: Vec<Persona> = root_dto.personas.into_iter()
            .map(|v1_dto| {
                let v2_dto = v1_dto.migrate();  // PersonaConfigV1 -> PersonaConfigV2
                v2_dto.into_domain()
            })
            .collect();

        // Verify UUID conversion happened
        assert!(uuid::Uuid::parse_str(&personas[0].id).is_ok(),
                "ID should be UUID, got: {}", personas[0].id);
        assert_ne!(personas[0].id, "mai", "ID should not be 'mai' anymore");

        // Convert back to V2 DTO and serialize
        let persona_dtos: Vec<PersonaConfigV2> = personas.iter()
            .map(|p| p.into())
            .collect();
        let root_dto = ConfigRootV2 {
            personas: persona_dtos,
            user_profile: None,
        };
        let toml_string = toml::to_string_pretty(&root_dto).unwrap();

        // Verify V2 format in TOML output
        assert!(toml_string.contains("schema_version = \"2.0.0\""),
                "TOML should contain V2 schema version");
        assert!(toml_string.contains("name = \"Mai\""),
                "TOML should contain persona name");

        // Verify it's not the old ID
        assert!(!toml_string.contains("id = \"mai\""),
                "TOML should not contain old ID 'mai'");
    }

    #[test]
    fn test_v2_persona_serialization_includes_schema_version() {
        use crate::dto::{PersonaConfigV2, PersonaSourceDTO, PERSONA_CONFIG_V2_VERSION};

        let persona_v2 = PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
            name: "Mai".to_string(),
            role: "UX Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        let toml_string = toml::to_string_pretty(&persona_v2).unwrap();

        assert!(toml_string.contains("schema_version = \"2.0.0\""),
                "Serialized TOML should contain schema_version");
        assert!(toml_string.contains("id = \"8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c\""),
                "Serialized TOML should contain UUID");
    }
}
