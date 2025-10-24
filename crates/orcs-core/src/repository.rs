use crate::session::Session;
use anyhow::Result;
use async_trait::async_trait;

/// An abstract repository for managing session persistence.
///
/// This trait defines the contract for persisting and retrieving sessions,
/// decoupling the application's core logic from the specific storage mechanism.
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
    /// `Some(Session)` if the session exists, `None` otherwise.
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>>;

    /// Saves a session.
    ///
    /// # Arguments
    ///
    /// * `session` - The session to save
    async fn save(&self, session: &Session) -> Result<()>;

    /// Deletes a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to delete
    async fn delete(&self, session_id: &str) -> Result<()>;

    /// Lists all stored sessions.
    ///
    /// # Returns
    ///
    /// A vector of all stored sessions.
    async fn list_all(&self) -> Result<Vec<Session>>;

    /// Gets the ID of the currently active session.
    ///
    /// # Returns
    ///
    /// `Some(session_id)` if an active session is set, `None` otherwise.
    async fn get_active_session_id(&self) -> Result<Option<String>>;

    /// Sets the ID of the currently active session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to set as active
    async fn set_active_session_id(&self, session_id: &str) -> Result<()>;
}

// Re-export PersonaRepository from persona module
pub use crate::persona::PersonaRepository;
