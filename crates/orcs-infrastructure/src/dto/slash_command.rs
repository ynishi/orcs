//! Slash command DTOs and migrations

use serde::{Deserialize, Serialize};
use version_migrate::{FromDomain, IntoDomain, MigratesTo, Versioned};

use orcs_core::slash_command::{ActionConfig, CommandType, SlashCommand};

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

/// Slash command DTO V1.3.0 (adds action_config for Action type)
/// Action type uses content field as prompt template with variables like {session_all}, {session_recent}
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.3.0")]
pub struct SlashCommandV1_3 {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_config: Option<ActionConfig>,
}

/// Migration from SlashCommandV1_2 to SlashCommandV1_3.
/// Adds action_config field (defaults to None for existing commands).
impl MigratesTo<SlashCommandV1_3> for SlashCommandV1_2 {
    fn migrate(self) -> SlashCommandV1_3 {
        SlashCommandV1_3 {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: self.args_description,
            task_blueprint: self.task_blueprint,
            action_config: None,
        }
    }
}

/// Migration from SlashCommandV1_3 to SlashCommandV1_4.
/// Adds include_in_system_prompt field (defaults based on command_type).
impl MigratesTo<SlashCommandV1_4> for SlashCommandV1_3 {
    fn migrate(self) -> SlashCommandV1_4 {
        // Task commands default to false, all others default to true
        let include_in_system_prompt = self.command_type != CommandType::Task;

        SlashCommandV1_4 {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: self.args_description,
            task_blueprint: self.task_blueprint,
            action_config: self.action_config,
            include_in_system_prompt,
        }
    }
}

/// Slash command DTO V1.4.0 (adds include_in_system_prompt)
/// Controls whether the command is included in system prompts for personas.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.4.0")]
pub struct SlashCommandV1_4 {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_config: Option<ActionConfig>,
    /// Whether to include this command in system prompts for personas.
    /// Default: true for Prompt/Shell/Action, false for Task
    #[serde(default = "default_include_in_system_prompt")]
    pub include_in_system_prompt: bool,
}

/// Default value for include_in_system_prompt during deserialization.
fn default_include_in_system_prompt() -> bool {
    true
}

/// Migration from SlashCommandV1_4 to SlashCommandV1_5.
/// Adds is_favorite and sort_order fields.
impl MigratesTo<SlashCommandV1_5> for SlashCommandV1_4 {
    fn migrate(self) -> SlashCommandV1_5 {
        SlashCommandV1_5 {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: self.args_description,
            task_blueprint: self.task_blueprint,
            action_config: self.action_config,
            include_in_system_prompt: self.include_in_system_prompt,
            is_favorite: false,
            sort_order: None,
        }
    }
}

/// Slash command DTO V1.5.0 (adds is_favorite, sort_order)
/// Supports favorites and custom sorting within favorites.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.5.0")]
pub struct SlashCommandV1_5 {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_config: Option<ActionConfig>,
    #[serde(default = "default_include_in_system_prompt")]
    pub include_in_system_prompt: bool,
    /// Whether this command is marked as favorite.
    #[serde(default)]
    pub is_favorite: bool,
    /// Sort order within favorites (lower = higher priority).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<u32>,
}

/// Convert SlashCommandV1_5 DTO to domain model
impl IntoDomain<SlashCommand> for SlashCommandV1_5 {
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
            action_config: self.action_config,
            include_in_system_prompt: self.include_in_system_prompt,
            is_favorite: self.is_favorite,
            sort_order: self.sort_order,
        }
    }
}

/// Convert domain model to SlashCommandV1_5 DTO for persistence
impl From<&SlashCommand> for SlashCommandV1_5 {
    fn from(cmd: &SlashCommand) -> Self {
        SlashCommandV1_5 {
            name: cmd.name.clone(),
            icon: cmd.icon.clone(),
            description: cmd.description.clone(),
            command_type: cmd.command_type.clone(),
            content: cmd.content.clone(),
            working_dir: cmd.working_dir.clone(),
            args_description: cmd.args_description.clone(),
            task_blueprint: cmd.task_blueprint.clone(),
            action_config: cmd.action_config.clone(),
            include_in_system_prompt: cmd.include_in_system_prompt,
            is_favorite: cmd.is_favorite,
            sort_order: cmd.sort_order,
        }
    }
}

/// Convert domain model to SlashCommandV1_5 DTO (for version-migrate save support)
impl FromDomain<SlashCommand> for SlashCommandV1_5 {
    fn from_domain(cmd: SlashCommand) -> Self {
        SlashCommandV1_5 {
            name: cmd.name,
            icon: cmd.icon,
            description: cmd.description,
            command_type: cmd.command_type,
            content: cmd.content,
            working_dir: cmd.working_dir,
            args_description: cmd.args_description,
            task_blueprint: cmd.task_blueprint,
            action_config: cmd.action_config,
            include_in_system_prompt: cmd.include_in_system_prompt,
            is_favorite: cmd.is_favorite,
            sort_order: cmd.sort_order,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates a Migrator for SlashCommand entities.
pub fn create_slash_command_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("slash_command" => [
        SlashCommandV1,
        SlashCommandV1_1,
        SlashCommandV1_2,
        SlashCommandV1_3,
        SlashCommandV1_4,
        SlashCommandV1_5,
        SlashCommand
    ], save = true)
    .expect("Failed to create slash_command migrator")
}
