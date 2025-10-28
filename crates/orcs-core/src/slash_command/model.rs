//! Slash command domain models.

use serde::{Deserialize, Serialize};

/// Type of slash command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CommandType {
    /// Expands into a prompt template and sends to AI
    Prompt,
    /// Executes a shell command and shows output
    Shell,
}

/// A custom slash command definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    /// Command name (used as /name in chat)
    pub name: String,
    /// Icon to display in UI
    pub icon: String,
    /// Human-readable description
    pub description: String,
    /// Type of command (prompt or shell)
    #[serde(rename = "type")]
    pub command_type: CommandType,
    /// Command content (prompt template or shell command)
    pub content: String,
    /// Working directory for shell commands (supports variables like {workspace_path})
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

impl SlashCommand {
    /// Creates a new prompt-type slash command.
    pub fn new_prompt(name: String, icon: String, description: String, content: String) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Prompt,
            content,
            working_dir: None,
        }
    }

    /// Creates a new shell-type slash command.
    pub fn new_shell(
        name: String,
        icon: String,
        description: String,
        content: String,
        working_dir: Option<String>,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Shell,
            content,
            working_dir,
        }
    }
}
