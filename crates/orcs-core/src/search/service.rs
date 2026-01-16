//! Search service trait definition.

use async_trait::async_trait;
use std::path::PathBuf;

use crate::error::Result;
use crate::search::{SearchFilters, SearchOptions, SearchResult};

/// Service for executing unified searches.
#[async_trait]
pub trait SearchService: Send + Sync {
    /// Executes a search with the given query and paths.
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `options` - Search options (all_workspaces, include_project)
    /// * `search_paths` - Paths to search within
    /// * `filters` - Optional filters to refine search results
    ///
    /// # Returns
    /// A SearchResult containing matched items and metadata.
    async fn search(
        &self,
        query: &str,
        options: SearchOptions,
        search_paths: Vec<PathBuf>,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResult>;
}
