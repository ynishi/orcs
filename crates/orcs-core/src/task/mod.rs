//! Task domain module.
//!
//! This module contains all task-related domain models, repository interfaces,
//! and management logic for task execution (e.g., Coding, batch processing).
//!
//! # Module Structure
//!
//! - `model`: Core task domain models (`TaskStatus`, `TaskContext`, etc.)
//!   and domain messaging types
//! - `repository`: Task repository trait for persistence
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::task::{TaskStatus, TaskContext, SerializableOrchestrationResult};
//! use orcs_core::task::{DomainMessage, TaskManagerMessage, ExecutionMessage};
//! use orcs_core::task::TaskRepository;
//! ```

mod model;
pub mod repository;

// Re-export public API
pub use model::{
    DomainMessage, ExecutionDetails, ExecutionMessage, SerializableOrchestrationResult, StepInfo,
    StepStatus, Task, TaskContext, TaskManagerMessage, TaskStatus,
};

pub use repository::TaskRepository;
