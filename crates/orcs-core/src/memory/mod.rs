//! Memory synchronization module for RAG integration.
//!
//! This module provides traits and types for synchronizing conversation
//! memories to external RAG systems like Kaiba.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents a message to be synced to the memory system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMessage {
    /// Unique identifier for the message
    pub id: String,
    /// Session ID this message belongs to
    pub session_id: String,
    /// Workspace ID this message belongs to
    pub workspace_id: String,
    /// Role of the message author (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Timestamp of the message (ISO 8601)
    pub timestamp: String,
    /// Optional persona ID if message is from an assistant
    pub persona_id: Option<String>,
    /// Optional tags for categorization
    pub tags: Vec<String>,
}

/// Result of a memory sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of messages successfully synced
    pub synced_count: usize,
    /// Number of messages that failed to sync
    pub failed_count: usize,
    /// Optional error message if sync partially failed
    pub error: Option<String>,
}

impl SyncResult {
    /// Creates a successful sync result.
    pub fn success(count: usize) -> Self {
        Self {
            synced_count: count,
            failed_count: 0,
            error: None,
        }
    }

    /// Creates a failed sync result.
    pub fn failure(error: String) -> Self {
        Self {
            synced_count: 0,
            failed_count: 0,
            error: Some(error),
        }
    }
}

/// Trait for synchronizing conversation memories to external systems.
///
/// This trait abstracts the memory synchronization logic, allowing
/// different implementations (e.g., Kaiba, local vector DB, etc.).
#[async_trait]
pub trait MemorySyncService: Send + Sync {
    /// Ensures a Rei exists with the given ID, creating it if necessary.
    ///
    /// # Arguments
    ///
    /// * `rei_id` - The predetermined Rei ID
    /// * `workspace_name` - The workspace name for Rei metadata
    ///
    /// # Returns
    ///
    /// Ok(()) if the Rei exists or was created successfully.
    async fn ensure_rei_exists(&self, rei_id: &str, workspace_name: &str) -> Result<(), String>;

    /// Syncs a batch of messages to the memory system.
    ///
    /// # Arguments
    ///
    /// * `rei_id` - The Rei (persona) ID to associate memories with
    /// * `messages` - The messages to sync
    ///
    /// # Returns
    ///
    /// A `SyncResult` indicating the outcome of the sync operation.
    async fn sync_messages(&self, rei_id: &str, messages: Vec<MemoryMessage>) -> SyncResult;

    /// Searches memories for relevant context.
    ///
    /// # Arguments
    ///
    /// * `rei_id` - The Rei (persona) ID to search within
    /// * `query` - The search query
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A vector of relevant memory messages.
    async fn search_memories(
        &self,
        rei_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryMessage>, String>;

    /// Creates a new Rei for a workspace if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    /// * `workspace_name` - The workspace name for the Rei
    ///
    /// # Returns
    ///
    /// The Rei ID (either existing or newly created).
    async fn get_or_create_rei(
        &self,
        workspace_id: &str,
        workspace_name: &str,
    ) -> Result<String, String>;
}

/// A no-op implementation of MemorySyncService for when no sync is configured.
pub struct NoOpMemorySyncService;

#[async_trait]
impl MemorySyncService for NoOpMemorySyncService {
    async fn ensure_rei_exists(&self, _rei_id: &str, _workspace_name: &str) -> Result<(), String> {
        // No-op: always succeed
        Ok(())
    }

    async fn sync_messages(&self, _rei_id: &str, messages: Vec<MemoryMessage>) -> SyncResult {
        // No-op: just report success without actually syncing
        SyncResult::success(messages.len())
    }

    async fn search_memories(
        &self,
        _rei_id: &str,
        _query: &str,
        _limit: usize,
    ) -> Result<Vec<MemoryMessage>, String> {
        // No-op: return empty results
        Ok(vec![])
    }

    async fn get_or_create_rei(
        &self,
        workspace_id: &str,
        _workspace_name: &str,
    ) -> Result<String, String> {
        // No-op: just return the workspace_id as a fake rei_id
        Ok(format!("noop-rei-{}", workspace_id))
    }
}
