//! Unified search functionality for sessions and workspace files.
//!
//! This module provides a unified search interface that searches across:
//! - Session histories (conversation logs)
//! - Workspace files (uploaded files, etc.)
//! - Project files (source code in workspace.root_path)

pub mod model;
pub mod service;

pub use model::{SearchFilters, SearchOptions, SearchResult, SearchResultItem};
pub use service::SearchService;
