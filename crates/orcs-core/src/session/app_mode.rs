//! Application mode types for session state management.

use serde::{Deserialize, Serialize};

/// Represents a proposed plan of action for user confirmation.
///
/// Plans are generated when the system needs to propose a series of steps
/// and wait for user approval before proceeding with execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// The individual steps that make up this plan.
    pub steps: Vec<String>,
}

/// Represents the current interaction mode within a session.
///
/// This tracks whether the session is in normal chat mode or waiting for
/// human-in-the-loop (HIL) confirmation of a proposed plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppMode {
    /// The application is waiting for user input in a normal chat mode.
    Idle,
    /// The application has proposed a plan and is waiting for user confirmation.
    AwaitingConfirmation {
        /// The plan awaiting confirmation.
        plan: Plan,
    },
}
