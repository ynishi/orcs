//! Session metadata service for managing session metadata operations.
//!
//! This module provides `SessionMetadataService` which handles all
//! session metadata updates such as renaming, favoriting, archiving, etc.

use super::updater::SessionUpdater;
use crate::error::Result;

/// Service for managing session metadata operations.
///
/// `SessionMetadataService` provides a clean interface for updating
/// session metadata without exposing the underlying storage details.
pub struct SessionMetadataService {
    updater: SessionUpdater,
}

impl SessionMetadataService {
    /// Creates a new `SessionMetadataService` with the given updater.
    pub fn new(updater: SessionUpdater) -> Self {
        Self { updater }
    }

    /// Renames a session by updating its title.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to rename
    /// * `new_title` - The new title for the session
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or cannot be saved.
    pub async fn rename(&self, session_id: &str, new_title: String) -> Result<()> {
        self.updater
            .update(session_id, |session| {
                session.title = new_title;
                Ok(())
            })
            .await
    }

    /// Toggles the favorite status of a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to toggle
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or cannot be saved.
    pub async fn toggle_favorite(&self, session_id: &str) -> Result<()> {
        self.updater
            .update(session_id, |session| {
                session.is_favorite = !session.is_favorite;
                Ok(())
            })
            .await
    }

    /// Toggles the archive status of a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to toggle
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or cannot be saved.
    pub async fn toggle_archive(&self, session_id: &str) -> Result<()> {
        self.updater
            .update(session_id, |session| {
                session.is_archived = !session.is_archived;
                Ok(())
            })
            .await
    }

    /// Updates the manual sort order of a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to update
    /// * `sort_order` - The new sort order (None to clear)
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or cannot be saved.
    pub async fn update_sort_order(&self, session_id: &str, sort_order: Option<i32>) -> Result<()> {
        self.updater
            .update(session_id, |session| {
                session.sort_order = sort_order;
                Ok(())
            })
            .await
    }
}

