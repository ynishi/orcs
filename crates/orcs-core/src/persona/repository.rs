//! Persona repository trait.
//!
//! Defines the interface for persona persistence operations.

use super::model::Persona;

/// An abstract repository for managing persona persistence.
///
/// This trait defines the contract for persisting and retrieving personas,
/// decoupling the application's core logic from the specific storage mechanism
/// (e.g., TOML file, database, remote API).
///
/// # Implementation Notes
///
/// Implementations should handle:
/// - Schema versioning and migrations
/// - UUID validation
/// - Concurrent access if needed
pub trait PersonaRepository: Send + Sync {
    /// Retrieves all personas from storage.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Persona>)`: All stored personas
    /// - `Err(String)`: Error message if retrieval fails
    fn get_all(&self) -> Result<Vec<Persona>, String>;

    /// Saves all personas to storage, replacing existing ones.
    ///
    /// # Arguments
    ///
    /// * `personas` - The personas to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Personas saved successfully
    /// - `Err(String)`: Error message if save fails
    fn save_all(&self, personas: &[Persona]) -> Result<(), String>;
}
