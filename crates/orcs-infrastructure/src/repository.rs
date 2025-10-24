use orcs_core::config::PersonaConfig;
use orcs_core::repository::PersonaRepository;
use crate::toml_storage;

/// A repository implementation for storing persona configurations in a TOML file.
pub struct TomlPersonaRepository;

impl PersonaRepository for TomlPersonaRepository {
    fn get_all(&self) -> Result<Vec<PersonaConfig>, String> {
        toml_storage::load_personas()
    }

    fn save_all(&self, configs: &[PersonaConfig]) -> Result<(), String> {
        toml_storage::save_personas(configs)
    }
}
