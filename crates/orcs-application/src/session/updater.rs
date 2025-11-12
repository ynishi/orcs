//! Session updater helper for common update patterns.
//!
//! This module provides `SessionUpdater` which abstracts the common
//! "find → update → save" pattern used across session metadata operations.

use orcs_core::error::{OrcsError, Result};
use orcs_core::session::{Session, SessionRepository};
use std::sync::Arc;

/// Helper struct for updating sessions with a common pattern.
///
/// `SessionUpdater` encapsulates the common pattern of:
/// 1. Loading a session from storage
/// 2. Applying updates
/// 3. Updating the timestamp
/// 4. Saving back to storage
pub struct SessionUpdater {
    repository: Arc<dyn SessionRepository>,
}

impl SessionUpdater {
    /// Creates a new `SessionUpdater` with the given repository.
    pub fn new(repository: Arc<dyn SessionRepository>) -> Self {
        Self { repository }
    }

    /// Updates a session by applying the given updater function.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to update
    /// * `updater` - A function that modifies the session
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The session doesn't exist
    /// - The updater function returns an error
    /// - Saving to storage fails
    pub async fn update<F>(&self, session_id: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut Session) -> Result<()>,
    {
        // Load the session from storage
        let mut session = self
            .repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::NotFound {
                entity_type: "Session",
                id: session_id.to_string(),
            })?;

        // Apply the updater function
        updater(&mut session)?;

        // Update timestamp
        session.updated_at = chrono::Utc::now().to_rfc3339();

        // Save back to storage
        self.repository.save(&session).await?;

        Ok(())
    }
}
