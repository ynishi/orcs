//! Slash command DTOs and migrations

use serde::{Deserialize, Serialize};
use version_migrate::{FromDomain, IntoDomain, Versioned};

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

/// Convert SlashCommandV1 DTO to domain model
impl IntoDomain<SlashCommand> for SlashCommandV1 {
    fn into_domain(self) -> SlashCommand {
        SlashCommand {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
        }
    }
}

/// Convert domain model to SlashCommandV1 DTO for persistence
impl From<&SlashCommand> for SlashCommandV1 {
    fn from(cmd: &SlashCommand) -> Self {
        SlashCommandV1 {
            name: cmd.name.clone(),
            icon: cmd.icon.clone(),
            description: cmd.description.clone(),
            command_type: cmd.command_type.clone(),
            content: cmd.content.clone(),
            working_dir: cmd.working_dir.clone(),
        }
    }
}

/// Convert domain model to SlashCommandV1 DTO (for version-migrate save support)
impl FromDomain<SlashCommand> for SlashCommandV1 {
    fn from_domain(cmd: SlashCommand) -> Self {
        SlashCommandV1 {
            name: cmd.name,
            icon: cmd.icon,
            description: cmd.description,
            command_type: cmd.command_type,
            content: cmd.content,
            working_dir: cmd.working_dir,
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
        .into_with_save::<SlashCommand>();
    migrator
        .register(path)
        .expect("Failed to register slash_command migration path");
    migrator
}
