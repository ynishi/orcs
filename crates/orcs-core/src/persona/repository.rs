//! Persona repository trait.
//!
//! Defines the interface for persona persistence operations.

use super::model::Persona;
use crate::error::Result;

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
#[async_trait::async_trait]
pub trait PersonaRepository: Send + Sync {
    /// Finds a persona by its ID.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The ID of the persona to find
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Persona))`: Persona found
    /// - `Ok(None)`: Persona not found
    /// - `Err(OrcsError)`: Error occurred during retrieval
    async fn find_by_id(&self, persona_id: &str) -> Result<Option<Persona>>;

    /// Saves a persona to storage.
    ///
    /// # Arguments
    ///
    /// * `persona` - The persona to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Persona saved successfully
    /// - `Err(OrcsError)`: Error occurred during save
    async fn save(&self, persona: &Persona) -> Result<()>;

    /// Deletes a persona from storage.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The ID of the persona to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Persona deleted successfully (or didn't exist)
    /// - `Err(OrcsError)`: Error occurred during deletion
    async fn delete(&self, persona_id: &str) -> Result<()>;

    /// Retrieves all personas from storage.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Persona>)`: All stored personas
    /// - `Err(OrcsError)`: Error if retrieval fails
    async fn get_all(&self) -> Result<Vec<Persona>>;

    /// Saves all provided personas to storage.
    ///
    /// # Arguments
    ///
    /// * `personas` - The personas to save or update. Use
    ///   [`PersonaRepository::delete`] to remove personas that should no longer exist.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Personas saved successfully
    /// - `Err(OrcsError)`: Error if save fails
    async fn save_all(&self, personas: &[Persona]) -> Result<()>;
}
