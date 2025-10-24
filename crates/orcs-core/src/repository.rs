use crate::config::PersonaConfig;

/// An abstract repository for managing persona configurations.
///
/// This trait defines the contract for persisting and retrieving personas,
/// decoupling the application's core logic from the specific storage mechanism (e.g., TOML file, database).
pub trait PersonaRepository: Send + Sync {
    /// Retrieves all persona configurations.
    fn get_all(&self) -> Result<Vec<PersonaConfig>, String>;

    /// Saves all persona configurations, overwriting any existing ones.
    fn save_all(&self, configs: &[PersonaConfig]) -> Result<(), String>;
}
