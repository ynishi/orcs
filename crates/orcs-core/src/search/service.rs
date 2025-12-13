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
    /// * `workspace_paths` - Paths to search within (e.g., project root and workspace storage)
    /// * `filters` - Optional filters to refine search results
    ///
    /// # Returns
    /// A SearchResult containing matched items and metadata.
    async fn search(
        &self,
        query: &str,
        scope: SearchScope,
        workspace_paths: Vec<PathBuf>,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResult>;
}
