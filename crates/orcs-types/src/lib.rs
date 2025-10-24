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

// NOTE: The following types have been moved to orcs_core modules:
// - MessageRole, ConversationMessage → orcs_core::session
// - AppMode, Plan → orcs_core::session
// - TaskStatus, TaskContext, SerializableOrchestrationResult → orcs_core::task
// - DomainMessage, TaskManagerMessage, ExecutionMessage → orcs_core::task
// - PersonaInfo has been removed (use orcs_core::persona::Persona instead)
