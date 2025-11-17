//! Dialogue preset repository trait.
//!
//! Defines the interface for dialogue preset persistence operations.

use super::preset::DialoguePreset;
use crate::error::Result;

/// An abstract repository for managing dialogue preset persistence.
///
/// This trait defines the contract for persisting and retrieving dialogue presets,
/// decoupling the application's core logic from the specific storage mechanism
/// (e.g., TOML files, database, remote API).
///
/// # Implementation Notes
///
/// Implementations should handle:
/// - Schema versioning and migrations
/// - UUID validation
/// - Concurrent access if needed
/// - System presets (read-only) vs User presets (read-write)
#[async_trait::async_trait]
pub trait DialoguePresetRepository: Send + Sync {
    /// Finds a dialogue preset by its ID.
    ///
    /// # Arguments
    ///
    /// * `preset_id` - The ID of the preset to find
    ///
    /// # Returns
    ///
    /// - `Ok(Some(DialoguePreset))`: Preset found
    /// - `Ok(None)`: Preset not found
    /// - `Err(OrcsError)`: Error occurred during retrieval
    async fn find_by_id(&self, preset_id: &str) -> Result<Option<DialoguePreset>>;

    /// Saves a dialogue preset to storage.
    ///
    /// # Arguments
    ///
    /// * `preset` - The preset to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Preset saved successfully
    /// - `Err(OrcsError)`: Error occurred during save
    ///
    /// # Notes
    ///
    /// System presets should not be saved/modified. Implementations should
    /// return an error if attempting to save a System preset.
    async fn save(&self, preset: &DialoguePreset) -> Result<()>;

    /// Deletes a dialogue preset from storage.
    ///
    /// # Arguments
    ///
    /// * `preset_id` - The ID of the preset to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Preset deleted successfully (or didn't exist)
    /// - `Err(OrcsError)`: Error occurred during deletion
    ///
    /// # Notes
    ///
    /// System presets should not be deleted. Implementations should
    /// return an error if attempting to delete a System preset.
    async fn delete(&self, preset_id: &str) -> Result<()>;

    /// Retrieves all dialogue presets from storage.
    ///
    /// This includes both system-provided default presets and user-created
    /// custom presets.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<DialoguePreset>)`: All stored presets
    /// - `Err(OrcsError)`: Error if retrieval fails
    async fn get_all(&self) -> Result<Vec<DialoguePreset>>;
}
