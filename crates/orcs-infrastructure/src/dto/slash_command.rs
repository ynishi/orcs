//! Slash command DTOs and migrations

use serde::{Deserialize, Serialize};
use version_migrate::{FromDomain, IntoDomain, MigratesTo, Versioned};

use orcs_core::slash_command::{CommandType, SlashCommand};

/// Slash command DTO V1.0.0
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct SlashCommandV1 {
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
    /// Working directory for shell commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

/// Slash command DTO V1.1.0 (adds args_description)
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct SlashCommandV1_1 {
    pub name: String,
    pub icon: String,
    pub description: String,
    #[serde(rename = "type")]
    pub command_type: CommandType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args_description: Option<String>,
}

/// Migration from SlashCommandV1 to SlashCommandV1_1.
/// Adds args_description field (defaults to None for existing commands).
impl MigratesTo<SlashCommandV1_1> for SlashCommandV1 {
    fn migrate(self) -> SlashCommandV1_1 {
        SlashCommandV1_1 {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: None, // Default: no args description for V1 commands
        }
    }
}

/// Slash command DTO V1.2.0 (adds task_blueprint)
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.2.0")]
pub struct SlashCommandV1_2 {
    pub name: String,
    pub icon: String,
    pub description: String,
    #[serde(rename = "type")]
    pub command_type: CommandType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_blueprint: Option<String>,
}

/// Migration from SlashCommandV1_1 to SlashCommandV1_2.
/// Adds task_blueprint field (defaults to None for existing commands).
impl MigratesTo<SlashCommandV1_2> for SlashCommandV1_1 {
    fn migrate(self) -> SlashCommandV1_2 {
        SlashCommandV1_2 {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: self.args_description,
            task_blueprint: None, // Default: no task blueprint for V1.1 commands
        }
    }
}

/// Convert SlashCommandV1_2 DTO to domain model
impl IntoDomain<SlashCommand> for SlashCommandV1_2 {
    fn into_domain(self) -> SlashCommand {
        SlashCommand {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: self.args_description,
            task_blueprint: self.task_blueprint,
        }
    }
}

/// Convert domain model to SlashCommandV1_2 DTO for persistence
impl From<&SlashCommand> for SlashCommandV1_2 {
    fn from(cmd: &SlashCommand) -> Self {
        SlashCommandV1_2 {
            name: cmd.name.clone(),
            icon: cmd.icon.clone(),
            description: cmd.description.clone(),
            command_type: cmd.command_type.clone(),
            content: cmd.content.clone(),
            working_dir: cmd.working_dir.clone(),
            args_description: cmd.args_description.clone(),
            task_blueprint: cmd.task_blueprint.clone(),
        }
    }
}

/// Convert domain model to SlashCommandV1_2 DTO (for version-migrate save support)
impl FromDomain<SlashCommand> for SlashCommandV1_2 {
    fn from_domain(cmd: SlashCommand) -> Self {
        SlashCommandV1_2 {
            name: cmd.name,
            icon: cmd.icon,
            description: cmd.description,
            command_type: cmd.command_type,
            content: cmd.content,
            working_dir: cmd.working_dir,
            args_description: cmd.args_description,
            task_blueprint: cmd.task_blueprint,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates a Migrator for SlashCommand entities.
pub fn create_slash_command_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();
    let path = version_migrate::Migrator::define("slash_command")
        .from::<SlashCommandV1>()
        .step::<SlashCommandV1_1>()
        .step::<SlashCommandV1_2>()
        .into_with_save::<SlashCommand>();
    migrator
        .register(path)
        .expect("Failed to register slash_command migration path");
    migrator
}
