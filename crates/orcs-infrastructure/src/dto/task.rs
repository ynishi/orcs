//! Task DTOs and migrations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use orcs_core::task::{ExecutionDetails, StepInfo, StepStatus, Task, TaskStatus};

/// Task status DTO matching domain model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatusDTO {
    Pending,
    Running,
    Completed,
    Failed,
}

impl From<TaskStatusDTO> for TaskStatus {
    fn from(dto: TaskStatusDTO) -> Self {
        match dto {
            TaskStatusDTO::Pending => TaskStatus::Pending,
            TaskStatusDTO::Running => TaskStatus::Running,
            TaskStatusDTO::Completed => TaskStatus::Completed,
            TaskStatusDTO::Failed => TaskStatus::Failed,
        }
    }
}

impl From<TaskStatus> for TaskStatusDTO {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => TaskStatusDTO::Pending,
            TaskStatus::Running => TaskStatusDTO::Running,
            TaskStatus::Completed => TaskStatusDTO::Completed,
            TaskStatus::Failed => TaskStatusDTO::Failed,
        }
    }
}

/// Step status DTO matching domain model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatusDTO {
    Pending,
    Running,
    Completed,
    Skipped,
    Failed,
}

impl From<StepStatusDTO> for StepStatus {
    fn from(dto: StepStatusDTO) -> Self {
        match dto {
            StepStatusDTO::Pending => StepStatus::Pending,
            StepStatusDTO::Running => StepStatus::Running,
            StepStatusDTO::Completed => StepStatus::Completed,
            StepStatusDTO::Skipped => StepStatus::Skipped,
            StepStatusDTO::Failed => StepStatus::Failed,
        }
    }
}

impl From<StepStatus> for StepStatusDTO {
    fn from(status: StepStatus) -> Self {
        match status {
            StepStatus::Pending => StepStatusDTO::Pending,
            StepStatus::Running => StepStatusDTO::Running,
            StepStatus::Completed => StepStatusDTO::Completed,
            StepStatus::Skipped => StepStatusDTO::Skipped,
            StepStatus::Failed => StepStatusDTO::Failed,
        }
    }
}

/// Step information DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepInfoDTO {
    pub id: String,
    pub description: String,
    pub status: StepStatusDTO,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<StepInfoDTO> for StepInfo {
    fn from(dto: StepInfoDTO) -> Self {
        StepInfo {
            id: dto.id,
            description: dto.description,
            status: dto.status.into(),
            agent: dto.agent,
            output: dto.output,
            error: dto.error,
        }
    }
}

impl From<StepInfo> for StepInfoDTO {
    fn from(step: StepInfo) -> Self {
        StepInfoDTO {
            id: step.id,
            description: step.description,
            status: step.status.into(),
            agent: step.agent,
            output: step.output,
            error: step.error,
        }
    }
}

/// Execution details DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionDetailsDTO {
    pub steps: Vec<StepInfoDTO>,
    pub context: HashMap<String, serde_json::Value>,
}

impl From<ExecutionDetailsDTO> for ExecutionDetails {
    fn from(dto: ExecutionDetailsDTO) -> Self {
        ExecutionDetails {
            steps: dto.steps.into_iter().map(Into::into).collect(),
            context: dto.context,
        }
    }
}

impl From<ExecutionDetails> for ExecutionDetailsDTO {
    fn from(details: ExecutionDetails) -> Self {
        ExecutionDetailsDTO {
            steps: details.steps.into_iter().map(Into::into).collect(),
            context: details.context,
        }
    }
}

/// V1.0.0: Initial task schema.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct TaskV1_0_0 {
    /// Unique task identifier (UUID format).
    pub id: String,
    /// Session ID where this task was executed.
    pub session_id: String,
    /// Task title.
    pub title: String,
    /// Full task description/request.
    pub description: String,
    /// Current task status.
    pub status: TaskStatusDTO,
    /// Timestamp when task was created (ISO 8601 format).
    pub created_at: String,
    /// Timestamp when task was last updated (ISO 8601 format).
    pub updated_at: String,
    /// Timestamp when task completed (ISO 8601 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Number of steps executed.
    pub steps_executed: u32,
    /// Number of steps skipped.
    pub steps_skipped: u32,
    /// Number of context keys generated.
    pub context_keys: u32,
    /// Error message if task failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Result summary text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Detailed execution information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_details: Option<ExecutionDetailsDTO>,
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Generates a deterministic UUID from a task title and timestamp.
fn generate_uuid_from_task(title: &str, timestamp: &str) -> String {
    let combined = format!("{}:{}", title, timestamp);
    Uuid::new_v5(&Uuid::NAMESPACE_OID, combined.as_bytes()).to_string()
}

/// Convert TaskV1_0_0 DTO to domain model.
impl IntoDomain<Task> for TaskV1_0_0 {
    fn into_domain(self) -> Task {
        // Validate and fix ID if needed
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Legacy data: non-UUID ID
            generate_uuid_from_task(&self.title, &self.created_at)
        };

        Task {
            id,
            session_id: self.session_id,
            title: self.title,
            description: self.description,
            status: self.status.into(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
            steps_executed: self.steps_executed,
            steps_skipped: self.steps_skipped,
            context_keys: self.context_keys,
            error: self.error,
            result: self.result,
            execution_details: self.execution_details.map(Into::into),
        }
    }
}

/// Convert domain model to TaskV1_0_0 DTO for persistence.
impl version_migrate::FromDomain<Task> for TaskV1_0_0 {
    fn from_domain(task: Task) -> Self {
        TaskV1_0_0 {
            id: task.id,
            session_id: task.session_id,
            title: task.title,
            description: task.description,
            status: task.status.into(),
            created_at: task.created_at,
            updated_at: task.updated_at,
            completed_at: task.completed_at,
            steps_executed: task.steps_executed,
            steps_skipped: task.steps_skipped,
            context_keys: task.context_keys,
            error: task.error,
            result: task.result,
            execution_details: task.execution_details.map(Into::into),
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for Task entities.
///
/// The migrator handles automatic schema migration and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → Task: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_task_migrator();
/// let tasks: Vec<Task> = migrator.load_vec_from("task", toml_tasks)?;
/// ```
pub fn create_task_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> Task
    let task_path = version_migrate::Migrator::define("task")
        .from::<TaskV1_0_0>()
        .into_with_save::<Task>();

    migrator
        .register(task_path)
        .expect("Failed to register task migration path");

    migrator
}

#[cfg(test)]
mod migrator_tests {
    use super::*;

    #[test]
    fn test_task_migrator_creation() {
        let _migrator = create_task_migrator();
        // Migrator should be created successfully
    }

    #[test]
    fn test_task_migration_v1_0_to_domain() {
        let migrator = create_task_migrator();

        // Simulate TOML structure with version V1.0.0
        let toml_str = r#"
version = "1.0.0"
id = "550e8400-e29b-41d4-a716-446655440000"
session_id = "660e8400-e29b-41d4-a716-446655440001"
title = "Test Task"
description = "Test task description"
status = "Completed"
created_at = "2025-01-01T00:00:00Z"
updated_at = "2025-01-01T00:01:00Z"
steps_executed = 5
steps_skipped = 0
context_keys = 6
"#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

        // Migrate to domain model using flat format
        let result: Result<Task, _> = migrator.load_flat_from("task", toml_value);

        if let Err(e) = &result {
            eprintln!("Migration error: {}", e);
        }
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
        let task = result.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.session_id, "660e8400-e29b-41d4-a716-446655440001");
        assert_eq!(task.steps_executed, 5);
        assert_eq!(task.steps_skipped, 0);
        assert_eq!(task.context_keys, 6);
    }
}
