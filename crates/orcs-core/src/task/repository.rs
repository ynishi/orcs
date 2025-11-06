//! Task repository trait.
//!
//! Defines the interface for task persistence operations.

use super::model::Task;
use anyhow::Result;
use async_trait::async_trait;

/// An abstract repository for managing task persistence.
///
/// This trait defines the contract for persisting and retrieving task execution
/// records, decoupling the application's core logic from the specific storage
/// mechanism (e.g., TOML files, database, remote API).
///
/// # Implementation Notes
///
/// Implementations should handle:
/// - Task versioning and migrations
/// - Concurrent access if needed
/// - Session-based filtering
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Finds a task by its ID.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to find
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Task))`: Task found
    /// - `Ok(None)`: Task not found
    /// - `Err(_)`: Error occurred during retrieval
    async fn find_by_id(&self, task_id: &str) -> Result<Option<Task>>;

    /// Saves a task to storage.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Task saved successfully
    /// - `Err(_)`: Error occurred during save
    async fn save(&self, task: &Task) -> Result<()>;

    /// Deletes a task from storage.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Task deleted successfully (or didn't exist)
    /// - `Err(_)`: Error occurred during deletion
    async fn delete(&self, task_id: &str) -> Result<()>;

    /// Lists all stored tasks.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Task>)`: All stored tasks
    /// - `Err(_)`: Error occurred during listing
    async fn list_all(&self) -> Result<Vec<Task>>;

    /// Lists tasks for a specific session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to filter by
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Task>)`: Tasks belonging to the session
    /// - `Err(_)`: Error occurred during listing
    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Task>>;
}
