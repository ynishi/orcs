use serde::{Deserialize, Serialize};
use thiserror::Error;
use llm_toolkit::orchestrator::StrategyMap;

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

/// A serializable representation of orchestration results.
///
/// This struct captures the essential information from the LLM toolkit's
/// orchestration results in a format suitable for storage and transmission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializableOrchestrationResult {
    /// Whether the orchestration completed successfully.
    pub success: bool,
    /// The number of steps that were executed.
    pub steps_executed: usize,
    /// The number of steps that were skipped.
    pub steps_skipped: usize,
    /// An error message if the orchestration failed.
    pub error: Option<String>,
    /// Whether the orchestration was terminated early.
    pub terminated: bool,
    /// The reason for early termination, if applicable.
    pub termination_reason: Option<String>,
}

/// Contains the context and state information for a task.
///
/// This structure holds all the necessary information about a task as it
/// moves through the orchestration system, including its identifier, current
/// status, and the original user request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// A unique identifier for the task.
    pub id: String,
    /// The current status of the task.
    pub status: TaskStatus,
    /// The user's original prompt or request.
    pub request: String,
    /// The execution strategy for this task.
    pub strategy: Option<StrategyMap>,
    /// The result of the task execution.
    pub result: Option<SerializableOrchestrationResult>,
}

impl PartialEq for TaskContext {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.status == other.status
            && self.request == other.request
            && self.result == other.result
        // strategy is intentionally excluded from equality check
    }
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

// NOTE: AppMode and Plan have been moved to orcs_core::session

/// Messages that can be sent between different domains in the application.
#[derive(Debug)]
pub enum DomainMessage {
    /// A message for the task manager domain.
    TaskManager(TaskManagerMessage),
    /// A message for the execution domain.
    Execution(ExecutionMessage),
}

/// Messages that can be sent to the task manager domain.
#[derive(Debug)]
pub enum TaskManagerMessage {
    /// Update the status of a task.
    TaskStatusUpdate {
        /// The ID of the task being updated.
        task_id: String,
        /// The new status of the task.
        status: TaskStatus,
    },
    /// Notify that a task has been completed.
    TaskCompleted {
        /// The ID of the completed task.
        task_id: String,
        /// The result of the task execution.
        result: SerializableOrchestrationResult,
    },
}

/// Messages that can be sent to the execution domain.
#[derive(Debug)]
pub enum ExecutionMessage {
    /// Request execution of a task.
    ExecuteTask {
        /// The ID of the task to execute.
        task_id: String,
        /// The execution strategy to use.
        strategy: StrategyMap,
    },
}

// NOTE: MessageRole and ConversationMessage have been moved to orcs_core::session
// NOTE: PersonaInfo has been removed (use orcs_core::persona::Persona instead)

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
            strategy: None,
            result: None,
        };

        // Serialize to JSON string
        let json_string = serde_json::to_string(&original).unwrap();

        // Deserialize back to TaskContext
        let deserialized: TaskContext = serde_json::from_str(&json_string).unwrap();

        // Verify that the original and deserialized instances are identical
        assert_eq!(original, deserialized);
    }
}
