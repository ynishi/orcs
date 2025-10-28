//! AsyncDirStorage-based SlashCommandRepository implementation
//!
//! Uses version-migrate AsyncDirStorage for proper ACID guarantees and async I/O.
//! Benefits:
//! - No manual Migrator management
//! - Built-in ACID guarantees
//! - Fully async I/O (no spawn_blocking)
//! - 1 command = 1 file (scalable for large prompts)

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use version_migrate::{AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy};

use orcs_core::error::{OrcsError, Result};
use orcs_core::slash_command::{SlashCommand, SlashCommandRepository};

use crate::dto::create_slash_command_migrator;

/// AsyncDirStorage-based slash command repository.
///
/// Directory structure:
/// ```text
/// base_dir/
/// └── slash_commands/
///     ├── <command-name-1>.toml
///     ├── <command-name-2>.toml
///     └── <command-name-3>.toml
/// ```
pub struct AsyncDirSlashCommandRepository {
    storage: AsyncDirStorage,
    #[allow(dead_code)]
    base_dir: PathBuf,
}

impl AsyncDirSlashCommandRepository {
    /// Creates an AsyncDirSlashCommandRepository instance at the default location (~/.config/orcs).
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn new() -> Result<Self> {
        use crate::paths::OrcsPaths;
        let base_dir = OrcsPaths::config_dir()
            .map_err(|e| OrcsError::Io(format!("Failed to get config directory: {:?}", e)))?;
        Self::new_with_base(base_dir).await
    }

    /// Creates a new AsyncDirSlashCommandRepository.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for slash commands (e.g., ~/.config/orcs)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Directory creation fails
    /// - AsyncDirStorage initialization fails
    pub async fn new_with_base(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Ensure base directory exists
        fs::create_dir_all(&base_dir)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to create base directory: {}", e)))?;

        // Setup AppPaths with CustomBase strategy to use our base_dir
        let paths = AppPaths::new("orcs")
            .data_strategy(PathStrategy::CustomBase(base_dir.clone()));

        // Setup migrator
        let migrator = create_slash_command_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage
        let storage = AsyncDirStorage::new(
            paths,
            "slash_commands",
            migrator,
            strategy
        )
        .await
        .map_err(|e| OrcsError::Io(format!("Failed to create AsyncDirStorage: {}", e)))?;

        Ok(Self { storage, base_dir })
    }
}

#[async_trait]
impl SlashCommandRepository for AsyncDirSlashCommandRepository {
    async fn list_commands(&self) -> Result<Vec<SlashCommand>> {
        let all_commands = self.storage
            .load_all::<SlashCommand>("slash_command")
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to load all slash commands: {}", e)))?;

        // Extract values from Vec<(String, SlashCommand)>
        let commands: Vec<SlashCommand> = all_commands.into_iter().map(|(_, cmd)| cmd).collect();
        Ok(commands)
    }

    async fn get_command(&self, name: &str) -> Result<Option<SlashCommand>> {
        match self.storage.load::<SlashCommand>("slash_command", name).await {
            Ok(command) => Ok(Some(command)),
            Err(e) => {
                // Check if it's a "not found" error
                let error_msg = e.to_string();
                if error_msg.contains("No such file or directory") || error_msg.contains("not found") {
                    Ok(None)
                } else {
                    Err(OrcsError::Io(format!("Failed to load slash command '{}': {}", name, e)))
                }
            }
        }
    }

    async fn save_command(&self, command: SlashCommand) -> Result<()> {
        self.storage
            .save("slash_command", &command.name, &command)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to save slash command '{}': {}", command.name, e)))
    }

    async fn remove_command(&self, name: &str) -> Result<()> {
        self.storage
            .delete(name)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to delete slash command '{}': {}", name, e)))
    }
}
