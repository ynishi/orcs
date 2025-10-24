//! Session repository trait.
//!
//! Defines the interface for session persistence operations.

use super::model::Session;
use anyhow::Result;
use async_trait::async_trait;

/// An abstract repository for managing session persistence.
///
/// This trait defines the contract for persisting and retrieving sessions,
/// decoupling the application's core logic from the specific storage mechanism
/// (e.g., TOML files, database, remote API).
///
/// # Implementation Notes
///
/// Implementations should handle:
/// - Session versioning and migrations
/// - Concurrent access if needed
/// - Active session tracking
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Finds a session by its ID.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to find
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Session))`: Session found
    /// - `Ok(None)`: Session not found
    /// - `Err(_)`: Error occurred during retrieval
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>>;

    /// Saves a session to storage.
    ///
    /// # Arguments
    ///
    /// * `session` - The session to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Session saved successfully
    /// - `Err(_)`: Error occurred during save
    async fn save(&self, session: &Session) -> Result<()>;

    /// Deletes a session from storage.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Session deleted successfully (or didn't exist)
    /// - `Err(_)`: Error occurred during deletion
    async fn delete(&self, session_id: &str) -> Result<()>;

    /// Lists all stored sessions.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Session>)`: All stored sessions
    /// - `Err(_)`: Error occurred during listing
    async fn list_all(&self) -> Result<Vec<Session>>;

    /// Gets the ID of the currently active session.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(session_id))`: Active session ID
    /// - `Ok(None)`: No active session set
    /// - `Err(_)`: Error occurred during retrieval
    async fn get_active_session_id(&self) -> Result<Option<String>>;

    /// Sets the ID of the currently active session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to set as active
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Active session set successfully
    /// - `Err(_)`: Error occurred during setting
    async fn set_active_session_id(&self, session_id: &str) -> Result<()>;
}
