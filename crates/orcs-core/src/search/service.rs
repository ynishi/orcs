//! Search service trait definition.

use async_trait::async_trait;
use std::path::PathBuf;

use crate::error::Result;
use crate::search::{SearchFilters, SearchResult, SearchScope};

/// Service for executing unified searches across different scopes.
#[async_trait]
pub trait SearchService: Send + Sync {
    /// Executes a search with the given query and scope.
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `scope` - The scope in which to search (workspace, local, global)
    /// * `workspace_path` - Path to the current workspace (required for workspace/local scopes)
    /// * `filters` - Optional filters to refine search results
    ///
    /// # Returns
    /// A SearchResult containing matched items and metadata.
    async fn search(
        &self,
        query: &str,
        scope: SearchScope,
        workspace_path: Option<PathBuf>,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResult>;
}
