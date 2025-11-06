//! Task domain model.
//!
//! This module contains the core Task entities and value objects that represent
//! task execution (e.g., Coding, batch processing) in the application's domain layer.

use llm_toolkit::orchestrator::StrategyMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use version_migrate::DeriveQueryable as Queryable;

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

// ============================================================================
// Task execution history and details
// ============================================================================

/// Information about a single step in task execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepInfo {
    /// Step identifier (e.g., "step_1", "analysis")
    pub id: String,
    /// Step description
    pub description: String,
    /// Status of this step
    pub status: StepStatus,
    /// Agent that executed this step
    pub agent: String,
    /// Output from this step (if available)
    pub output: Option<serde_json::Value>,
    /// Error message if step failed
    pub error: Option<String>,
}

/// Status of an individual step in task execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step is waiting to be executed
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Completed,
    /// Step was skipped due to dependencies
    Skipped,
    /// Step failed with error
    Failed,
}

/// Detailed execution information for a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionDetails {
    /// Individual steps and their status
    pub steps: Vec<StepInfo>,
    /// Execution context values (intermediate results)
    pub context: HashMap<String, serde_json::Value>,
}

/// A task execution record for history and display.
///
/// This represents a completed or in-progress task execution that can be
/// persisted and displayed in the UI. Unlike `TaskContext` which is a
/// temporary execution context, `Task` is the permanent historical record.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[queryable(entity = "task")]
pub struct Task {
    /// Unique task identifier (UUID format)
    pub id: String,
    /// Session ID where this task was executed
    pub session_id: String,
    /// Task title (shortened from description)
    pub title: String,
    /// Full task description/request
    pub description: String,
    /// Current task status
    pub status: TaskStatus,
    /// Timestamp when task was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when task was last updated (ISO 8601 format)
    pub updated_at: String,
    /// Timestamp when task completed (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Number of steps executed
    pub steps_executed: u32,
    /// Number of steps skipped
    pub steps_skipped: u32,
    /// Number of context keys generated
    pub context_keys: u32,
    /// Error message if task failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Result summary text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Detailed execution information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_details: Option<ExecutionDetails>,
}
