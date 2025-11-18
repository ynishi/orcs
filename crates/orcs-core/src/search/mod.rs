//! Unified search functionality across workspace, local, and global scopes.
//!
//! This module provides a unified search interface that abstracts over different
//! search backends and scopes, allowing AI agents to transparently access information
//! from various sources.

pub mod model;
pub mod service;

pub use model::{SearchFilters, SearchResult, SearchResultItem, SearchScope};
pub use service::SearchService;
