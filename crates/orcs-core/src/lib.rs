pub mod error;
pub mod session;
pub mod persona;
pub mod user;
pub mod task;
pub mod repository;
pub mod workspace;
pub mod config;
pub mod slash_command;

// Re-export common error type
pub use error::OrcsError;

// Deprecated: Use orcs_core::user instead
#[deprecated(since = "0.2.0", note = "Use orcs_core::user instead")]
pub mod user_service {
    pub use crate::user::{DefaultUserService, UserService};
}

use crate::task::{TaskContext, TaskStatus};
use uuid::Uuid;

/// The central state manager for tasks in the Orcs orchestration system.
///
/// `TaskManager` is responsible for creating, storing, and managing the lifecycle
/// of tasks. It maintains a collection of all tasks and provides methods to
/// interact with them.
#[derive(Default)]
pub struct TaskManager {
    /// A vector containing all tasks managed by this instance.
    pub tasks: Vec<TaskContext>,
}

impl TaskManager {
    /// Creates a new `TaskManager` instance with an empty task list.
    ///
    /// # Examples
    ///
    /// ```
    /// use orcs_core::TaskManager;
    ///
    /// let manager = TaskManager::new();
    /// assert_eq!(manager.tasks.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new task with the given request and adds it to the task list.
    ///
    /// The newly created task will have:
    /// - A unique UUID-based identifier
    /// - Status set to `Pending`
    /// - The provided request string
    ///
    /// # Arguments
    ///
    /// * `request` - A string slice containing the user's request or prompt
    ///
    /// # Returns
    ///
    /// A reference to the newly created `TaskContext`
    ///
    /// # Examples
    ///
    /// ```
    /// use orcs_core::TaskManager;
    ///
    /// let mut manager = TaskManager::new();
    /// let task = manager.create_task("Analyze the data");
    /// assert_eq!(task.request, "Analyze the data");
    /// ```
    pub fn create_task(&mut self, request: &str) -> &TaskContext {
        let task = TaskContext {
            id: Uuid::new_v4().to_string(),
            status: TaskStatus::Pending,
            request: request.to_string(),
            strategy: None,
            result: None,
        };

        self.tasks.push(task);
        // Safe to unwrap because we just pushed an element
        self.tasks.last().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task_manager() {
        let manager = TaskManager::new();
        assert_eq!(manager.tasks.len(), 0);
    }

    #[test]
    fn test_create_task() {
        let mut manager = TaskManager::new();
        let request = "Process user data";

        let task = manager.create_task(request);

        assert_eq!(task.request, request);
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(!task.id.is_empty());
        assert_eq!(manager.tasks.len(), 1);
    }

    #[test]
    fn test_create_multiple_tasks() {
        let mut manager = TaskManager::new();

        manager.create_task("First task");
        manager.create_task("Second task");

        assert_eq!(manager.tasks.len(), 2);
        // Verify that the two tasks have unique IDs
        assert_ne!(manager.tasks[0].id, manager.tasks[1].id);
    }
}
