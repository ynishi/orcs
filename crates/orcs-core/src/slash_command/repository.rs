//! Slash command repository trait.

use async_trait::async_trait;

use crate::error::Result;
use crate::slash_command::SlashCommand;

/// Repository for managing slash commands.
#[async_trait]
pub trait SlashCommandRepository: Send + Sync {
    /// Lists all available slash commands.
    async fn list_commands(&self) -> Result<Vec<SlashCommand>>;

    /// Gets a specific command by name.
    async fn get_command(&self, name: &str) -> Result<Option<SlashCommand>>;

    /// Adds or updates a slash command.
    async fn save_command(&self, command: SlashCommand) -> Result<()>;

    /// Removes a slash command by name.
    async fn remove_command(&self, name: &str) -> Result<()>;
}
