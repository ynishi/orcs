//! Search domain models.

use serde::{Deserialize, Serialize};

/// Scope of the search operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchScope {
    /// Search within the current workspace (default)
    Workspace,
    /// Search across all workspaces (local RAG)
    Local,
    /// Search the web (global)
    Global,
}

impl Default for SearchScope {
    fn default() -> Self {
        Self::Workspace
    }
}

/// Filters to refine search results.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    /// File types to include (e.g., ["rs", "md"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_types: Option<Vec<String>>,

    /// Paths to exclude from search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_paths: Option<Vec<String>>,

    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<usize>,

    /// Context lines before match (for code search)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_before: Option<usize>,

    /// Context lines after match (for code search)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_after: Option<usize>,
}

/// A single search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// File path (relative to workspace root)
    pub path: String,

    /// Line number where the match was found
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,

    /// The matching line or snippet
    pub content: String,

    /// Context lines before the match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_before: Option<Vec<String>>,

    /// Context lines after the match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_after: Option<Vec<String>>,
}

/// Result of a search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The search query that was executed
    pub query: String,

    /// The scope in which the search was performed
    pub scope: SearchScope,

    /// Search result items
    pub items: Vec<SearchResultItem>,

    /// Optional summary/answer describing the results (used for global web search)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Total number of matches (may be larger than items.len() if limited)
    pub total_matches: usize,
}

impl SearchResult {
    /// Creates a new empty search result.
    pub fn empty(query: String, scope: SearchScope) -> Self {
        Self {
            query,
            scope,
            items: Vec::new(),
            summary: None,
            total_matches: 0,
        }
    }

    /// Creates a new search result with items.
    pub fn new(query: String, scope: SearchScope, items: Vec<SearchResultItem>) -> Self {
        let total_matches = items.len();
        Self {
            query,
            scope,
            items,
            summary: None,
            total_matches,
        }
    }
}
