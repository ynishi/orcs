//! TOML-based PersonaRepository implementation

use orcs_core::persona::Persona;
use orcs_core::repository::PersonaRepository;

/// A repository implementation for storing persona configurations in a TOML file.
pub struct TomlPersonaRepository;

impl PersonaRepository for TomlPersonaRepository {
    fn get_all(&self) -> Result<Vec<Persona>, String> {
        crate::toml_storage::load_personas()
    }

    fn save_all(&self, configs: &[Persona]) -> Result<(), String> {
        crate::toml_storage::save_personas(configs)
    }
}
