use orcs_core::config::PersonaConfig;
use crate::dto::{ConfigRootV1, ConfigRootV2, PersonaConfigV2};
use std::fs;

/// Loads persona configurations from the default config file path.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// This function is purely responsible for reading the TOML file from disk
/// and does not contain any application-specific fallback logic.
///
/// # Returns
///
/// - `Ok(Vec<PersonaConfig>)`: A vector of persona configs if the file is found and parsed.
///   If the file does not exist, the config directory cannot be found, or the file is empty,
///   it returns an empty vector `Ok(vec![])`.
/// - `Err(String)`: An error message if the file exists but cannot be read or parsed.
pub fn load_personas() -> Result<Vec<PersonaConfig>, String> {
    match dirs::config_dir() {
        Some(config_dir) => {
            let config_path = config_dir.join("orcs").join("config.toml");
            if !config_path.exists() {
                return Ok(Vec::new()); // No config file, return empty vec
            }

            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

            // Return empty vec if file is empty
            if content.trim().is_empty() {
                return Ok(Vec::new());
            }

            // Try to deserialize as V2 first (current version)
            if let Ok(root_dto) = toml::from_str::<ConfigRootV2>(&content) {
                // V2 format - convert directly to domain models
                let personas = root_dto.personas.into_iter()
                    .map(|dto| dto.into())
                    .collect();
                return Ok(personas);
            }

            // Fallback to V1 format
            if let Ok(root_dto) = toml::from_str::<ConfigRootV1>(&content) {
                // V1 format - migrate to V2 then to domain models
                let personas = root_dto.personas.into_iter()
                    .map(|v1_dto| {
                        let v2_dto: PersonaConfigV2 = v1_dto.into();
                        v2_dto.into()
                    })
                    .collect();
                return Ok(personas);
            }

            Err(format!("Failed to parse TOML from {:?}: unsupported schema version", config_path))
        }
        None => Ok(Vec::new()), // Cannot find config dir, return empty vec
    }
}

/// Saves persona configurations to the default config file path.
///
/// The path is `~/.config/orcs/config.toml`.
///
/// # Arguments
///
/// * `personas` - A slice of PersonaConfig structs to save.
///
/// # Returns
///
/// - `Ok(())`: If the file was successfully written.
/// - `Err(String)`: An error message if the config directory cannot be found,
///   the directory cannot be created, or the file cannot be written.
pub fn save_personas(personas: &[PersonaConfig]) -> Result<(), String> {
    match dirs::config_dir() {
        Some(config_dir) => {
            let orcs_config_dir = config_dir.join("orcs");
            let config_path = orcs_config_dir.join("config.toml");

            // Create the directory if it doesn't exist
            if !orcs_config_dir.exists() {
                fs::create_dir_all(&orcs_config_dir)
                    .map_err(|e| format!("Failed to create config directory at {:?}: {}", orcs_config_dir, e))?;
            }

            // Convert domain models to V2 DTOs (always save as V2)
            let persona_dtos: Vec<PersonaConfigV2> = personas.iter()
                .map(|p| p.into())
                .collect();

            let root_dto = ConfigRootV2 { personas: persona_dtos };

            // Serialize V2 DTOs to TOML
            let toml_string = toml::to_string_pretty(&root_dto)
                .map_err(|e| format!("Failed to serialize personas to TOML: {}", e))?;

            // Write to file
            fs::write(&config_path, toml_string)
                .map_err(|e| format!("Failed to write config file at {:?}: {}", config_path, e))?;

            Ok(())
        }
        None => Err("Cannot find config directory".to_string()),
    }
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
        let root_dto = toml::from_str::<ConfigRootV1>(&content).unwrap();
        let personas: Vec<PersonaConfig> = root_dto.personas.into_iter()
            .map(|v1_dto| {
                let v2_dto: PersonaConfigV2 = v1_dto.into();
                v2_dto.into()
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
        let root_dto = ConfigRootV2 { personas: persona_dtos };
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
