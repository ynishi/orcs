//! Task domain module.
//!
//! This module contains all task-related domain models, repository interfaces,
//! and management logic for task execution (e.g., Coding, batch processing).
//!
//! # Module Structure
//!
//! - `model`: Core task domain models (`TaskStatus`, `TaskContext`, etc.)
//!   and domain messaging types
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::task::{TaskStatus, TaskContext, SerializableOrchestrationResult};
//! use orcs_core::task::{DomainMessage, TaskManagerMessage, ExecutionMessage};
//! ```

mod model;

// Re-export public API
pub use model::{
    DomainMessage, ExecutionMessage, SerializableOrchestrationResult, TaskContext,
    TaskManagerMessage, TaskStatus,
};
