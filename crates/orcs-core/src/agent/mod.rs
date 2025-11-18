//! Agent configuration and builder utilities for ORCS.
//!
//! This module provides common functionality for configuring and creating
//! llm-toolkit agents with workspace-aware settings.
//!
//! # Example
//! ```
//! use orcs_core::agent::AgentBuilder;
//! use std::path::PathBuf;
//!
//! // Create a workspace-aware agent configuration
//! let config = AgentBuilder::new()
//!     .with_workspace(PathBuf::from("/path/to/project"))
//!     .build();
//!
//! assert!(config.cwd.is_some());
//! assert!(config.env_vars.contains_key("PATH"));
//! ```

mod builder;
mod config;
mod env;
mod web_search;

pub use builder::AgentBuilder;
pub use config::{AgentConfig, WorkspaceConfig};
pub use env::build_enhanced_path;
pub use web_search::{WebSearchAgent, WebSearchReference, WebSearchResponse};
