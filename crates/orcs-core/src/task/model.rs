//! Task domain model.
//!
//! This module contains the core Task entities and value objects that represent
//! task execution (e.g., Coding, batch processing) in the application's domain layer.

use llm_toolkit::orchestrator::StrategyMap;
use serde::{Deserialize, Serialize};

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

// ============================================================================
// Domain messaging types
// ============================================================================

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
