//! Search domain models.

use serde::{Deserialize, Serialize};

/// Search options to control what is searched.
///
/// Default: searches current workspace's sessions and files.
/// - `-p`: also search project files (root_path)
/// - `-a`: search all workspaces' sessions and files
/// - `-f` (or `-ap`): search everything (all workspaces + project files)
/// - `-m`: search Kaiba memory (RAG semantic search)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct SearchOptions {
    /// Search all workspaces instead of just current workspace
    #[serde(default)]
    pub all_workspaces: bool,

    /// Include project files (workspace.root_path) in search
    #[serde(default)]
    pub include_project: bool,

    /// Search Kaiba memory (RAG semantic search)
    #[serde(default)]
    pub search_memory: bool,
}

impl SearchOptions {
    /// Default: current workspace sessions + workspace files
    pub fn default_workspace() -> Self {
        Self {
            all_workspaces: false,
            include_project: false,
            search_memory: false,
        }
    }

    /// -p: current workspace + project files
    pub fn with_project() -> Self {
        Self {
            all_workspaces: false,
            include_project: true,
            search_memory: false,
        }
    }

    /// -a: all workspaces sessions + files
    pub fn all() -> Self {
        Self {
            all_workspaces: true,
            include_project: false,
            search_memory: false,
        }
    }

    /// -f: full search (all workspaces + project files)
    pub fn full() -> Self {
        Self {
            all_workspaces: true,
            include_project: true,
            search_memory: false,
        }
    }

    /// -m: search Kaiba memory only
    pub fn memory_only() -> Self {
        Self {
            all_workspaces: false,
            include_project: false,
            search_memory: true,
        }
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

    /// The options used for this search
    pub options: SearchOptions,

    /// Search result items
    pub items: Vec<SearchResultItem>,

    /// Optional summary/answer describing the results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Total number of matches (may be larger than items.len() if limited)
    pub total_matches: usize,
}

impl SearchResult {
    /// Creates a new empty search result.
    pub fn empty(query: String, options: SearchOptions) -> Self {
        Self {
            query,
            options,
            items: Vec::new(),
            summary: None,
            total_matches: 0,
        }
    }

    /// Creates a new search result with items.
    pub fn new(query: String, options: SearchOptions, items: Vec<SearchResultItem>) -> Self {
        let total_matches = items.len();
        Self {
            query,
            options,
            items,
            summary: None,
            total_matches,
        }
    }
}
