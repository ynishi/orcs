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
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

use orcs_core::error::{OrcsError, Result};
use orcs_core::slash_command::{SlashCommand, SlashCommandRepository};

use crate::ServiceType;
use crate::dto::create_slash_command_migrator;
use crate::storage_repository::StorageRepository;

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
}

impl StorageRepository for AsyncDirSlashCommandRepository {
    const SERVICE_TYPE: ServiceType = ServiceType::SlashCommand;
    const ENTITY_NAME: &'static str = "slash_command";

    fn storage(&self) -> &AsyncDirStorage {
        &self.storage
    }
}

impl AsyncDirSlashCommandRepository {

    pub async fn default() -> Result<Self> {
        Self::new(None).await
    }
    
    /// Creates an AsyncDirSlashCommandRepository instance at the default location.
    ///
    /// Uses centralized path management and storage creation via `OrcsPaths`.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage cannot be created.
    pub async fn new(base_dir: Option<&Path>) -> Result<Self> {
        use crate::paths::OrcsPaths;

        // Create AsyncDirStorage via centralized helper
        let migrator = create_slash_command_migrator();
        let orcs_paths = OrcsPaths::new(base_dir);
        let storage = orcs_paths
            .create_async_dir_storage(Self::SERVICE_TYPE, migrator)
            .await?;
        Ok(Self { storage })
    }
}

#[async_trait]
impl SlashCommandRepository for AsyncDirSlashCommandRepository {
    async fn list_commands(&self) -> Result<Vec<SlashCommand>> {
        let all_commands = self
            .storage
            .load_all::<SlashCommand>(Self::ENTITY_NAME)
            .await?;

        // Extract values from Vec<(String, SlashCommand)>
        let commands: Vec<SlashCommand> = all_commands.into_iter().map(|(_, cmd)| cmd).collect();
        Ok(commands)
    }

    async fn get_command(&self, name: &str) -> Result<Option<SlashCommand>> {
        match self
            .storage
            .load::<SlashCommand>(Self::ENTITY_NAME, name)
            .await
        {
            Ok(command) => Ok(Some(command)),
            Err(e) => {
                let orcs_err = e.into();
                if orcs_core::OrcsError::is_not_found(&orcs_err) {
                    Ok(None)
                } else {
                    Err(orcs_err)
                }
            }
        }
    }

    async fn save_command(&self, command: SlashCommand) -> Result<()> {
        self.storage
            .save(Self::ENTITY_NAME, &command.name, &command)
            .await?;
        Ok(())
    }

    async fn remove_command(&self, name: &str) -> Result<()> {
        self.storage
            .delete(name)
            .await?;
        Ok(())
    }
}
