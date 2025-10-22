use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A shared error type for the entire Orcs application.
///
/// Each crate will have its own specific error enum, but this type can be used
/// for common errors or as a top-level error container if needed.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum OrcsError {
    #[error("An unknown error has occurred.")]
    Unknown,
}

/// Represents the current status of a task in the orchestration system.
///
/// Tasks progress through these states as they are processed by the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// The task has been created but is not yet running.
    Pending,
    /// The task is currently being executed by a worker.
    Running,
    /// The task completed successfully.
    Completed,
    /// The task failed during execution.
    Failed,
}

/// Contains the context and state information for a task.
///
/// This structure holds all the necessary information about a task as it
/// moves through the orchestration system, including its identifier, current
/// status, and the original user request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskContext {
    /// A unique identifier for the task.
    pub id: String,
    /// The current status of the task.
    pub status: TaskStatus,
    /// The user's original prompt or request.
    pub request: String,
    // Future fields to be implemented:
    // /// The execution strategy for this task.
    // pub strategy: Option<Strategy>,
    // /// The result of the task execution.
    // pub result: Option<TaskResult>,
}

/// Represents user input to the system.
///
/// User input can be either a direct command or natural language dialogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    /// A direct command from the user.
    Command(String),
    /// Natural language dialogue from the user.
    Dialogue(String),
}

/// Represents a proposed plan of action for user confirmation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// The individual steps that make up this plan.
    pub steps: Vec<String>,
}

/// Represents the current interaction mode of the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppMode {
    /// The application is waiting for user input in a normal chat mode.
    Idle,
    /// The application has proposed a plan and is waiting for user confirmation.
    AwaitingConfirmation {
        /// The plan awaiting confirmation.
        plan: Plan,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_context_serialization() {
        // Create a TaskContext instance with sample data
        let original = TaskContext {
            id: "task_123".to_string(),
            status: TaskStatus::Pending,
            request: "Analyze data".to_string(),
        };

        // Serialize to JSON string
        let json_string = serde_json::to_string(&original).unwrap();

        // Deserialize back to TaskContext
        let deserialized: TaskContext = serde_json::from_str(&json_string).unwrap();

        // Verify that the original and deserialized instances are identical
        assert_eq!(original, deserialized);
    }
}
