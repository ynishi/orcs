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
    /// Executes a long-running orchestration task workflow
    Task,
    /// Creates or updates an entity (Persona, Workspace, SlashCommand, etc.)
    Entity,
}

/// Type of entity that can be created or managed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    /// Persona configuration
    Persona,
    /// Workspace configuration
    Workspace,
    /// Slash command definition
    SlashCommand,
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
    /// Type of command (prompt, shell, task, or entity)
    #[serde(rename = "type")]
    pub command_type: CommandType,
    /// Command content (prompt template, shell command, task description, or empty for entity)
    pub content: String,
    /// Working directory for shell commands (supports variables like {workspace_path})
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "workingDir")]
    pub working_dir: Option<String>,
    /// Optional description of expected arguments (displayed in UI)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "argsDescription")]
    pub args_description: Option<String>,
    /// Task execution strategy blueprint (JSON serialized StrategyMap) for Task type commands
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "taskBlueprint")]
    pub task_blueprint: Option<String>,
    /// Entity type for Entity commands
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "entityType")]
    pub entity_type: Option<EntityType>,
    /// JSON Schema for validation (Entity commands only)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "entitySchema")]
    pub entity_schema: Option<String>,
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
            args_description: None,
            task_blueprint: None,
            entity_type: None,
            entity_schema: None,
        }
    }

    /// Creates a new prompt-type slash command with an argument description.
    pub fn new_prompt_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_prompt(name, icon, description, content);
        cmd.args_description = args_description;
        cmd
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
            args_description: None,
            task_blueprint: None,
            entity_type: None,
            entity_schema: None,
        }
    }

    /// Creates a new shell-type slash command with an argument description.
    pub fn new_shell_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        working_dir: Option<String>,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_shell(name, icon, description, content, working_dir);
        cmd.args_description = args_description;
        cmd
    }

    /// Creates a new task-type slash command.
    pub fn new_task(
        name: String,
        icon: String,
        description: String,
        content: String,
        task_blueprint: Option<String>,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Task,
            content,
            working_dir: None,
            args_description: None,
            task_blueprint,
            entity_type: None,
            entity_schema: None,
        }
    }

    /// Creates a new entity-type slash command.
    pub fn new_entity(
        name: String,
        icon: String,
        description: String,
        entity_type: EntityType,
        entity_schema: Option<String>,
        args_description: Option<String>,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Entity,
            content: String::new(), // Empty for entity commands
            working_dir: None,
            args_description,
            task_blueprint: None,
            entity_type: Some(entity_type),
            entity_schema,
        }
    }

    /// Creates a new task-type slash command with an argument description.
    pub fn new_task_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        task_blueprint: Option<String>,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_task(name, icon, description, content, task_blueprint);
        cmd.args_description = args_description;
        cmd
    }
}
