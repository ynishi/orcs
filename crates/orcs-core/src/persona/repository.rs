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
    /// Retrieves all personas from storage.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Persona>)`: All stored personas
    /// - `Err(OrcsError)`: Error if retrieval fails
    async fn get_all(&self) -> Result<Vec<Persona>>;

    /// Saves all personas to storage, replacing existing ones.
    ///
    /// # Arguments
    ///
    /// * `personas` - The personas to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Personas saved successfully
    /// - `Err(OrcsError)`: Error if save fails
    async fn save_all(&self, personas: &[Persona]) -> Result<()>;
}
