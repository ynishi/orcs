use orcs_core::config::{ConfigRoot, PersonaConfig};
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

            let root: ConfigRoot = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse TOML from {:?}: {}", config_path, e))?;

            Ok(root.personas)
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

            let root = ConfigRoot { personas: personas.to_vec() };

            // Serialize personas to TOML
            let toml_string = toml::to_string_pretty(&root)
                .map_err(|e| format!("Failed to serialize personas to TOML: {}", e))?;

            // Write to file
            fs::write(&config_path, toml_string)
                .map_err(|e| format!("Failed to write config file at {:?}: {}", config_path, e))?;

            Ok(())
        }
        None => Err("Cannot find config directory".to_string()),
    }
}
