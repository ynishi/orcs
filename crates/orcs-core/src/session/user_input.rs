//! User input types for session interaction.

/// Represents user input to the system within a session.
///
/// User input can be either a direct command or natural language dialogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    /// A direct command from the user.
    Command(String),
    /// Natural language dialogue from the user.
    Dialogue(String),
}
