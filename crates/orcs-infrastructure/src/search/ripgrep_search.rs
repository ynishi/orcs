//! Ripgrep-based search implementation.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

use orcs_core::error::{OrcsError, Result};
use orcs_core::search::model::SearchResultItem;
use orcs_core::search::{SearchFilters, SearchResult, SearchScope, SearchService};

/// Search service implementation using ripgrep.
pub struct RipgrepSearchService;

impl RipgrepSearchService {
    pub fn new() -> Self {
        Self
    }

    /// Executes ripgrep command with the given parameters.
    fn execute_ripgrep(
        &self,
        query: &str,
        search_path: &PathBuf,
        filters: &Option<SearchFilters>,
    ) -> Result<Vec<SearchResultItem>> {
        let mut cmd = Command::new("rg");

        // Basic flags
        cmd.arg("--line-number"); // Show line numbers
        cmd.arg("--no-heading"); // Don't group by file
        cmd.arg("--with-filename"); // Always show filename

        // Apply filters
        if let Some(f) = filters {
            // File type filtering
            if let Some(ref types) = f.file_types {
                for t in types {
                    cmd.arg("--type").arg(t);
                }
            }

            // Exclude paths
            if let Some(ref excludes) = f.exclude_paths {
                for exclude in excludes {
                    cmd.arg("--glob").arg(format!("!{}", exclude));
                }
            }

            // Max results (using --max-count)
            if let Some(max) = f.max_results {
                cmd.arg("--max-count").arg(max.to_string());
            }

            // Context lines
            if let Some(before) = f.context_before {
                cmd.arg("--before-context").arg(before.to_string());
            }
            if let Some(after) = f.context_after {
                cmd.arg("--after-context").arg(after.to_string());
            }
        }

        // Add query and search path
        cmd.arg(query);
        cmd.arg(search_path);

        tracing::debug!("Executing ripgrep command: {:?}", cmd);

        // Execute command
        let output = cmd
            .output()
            .map_err(|e| OrcsError::io(format!("Failed to execute ripgrep: {}", e)))?;

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let items = self.parse_ripgrep_output(&stdout)?;

        Ok(items)
    }

    /// Parses ripgrep output into SearchResultItems.
    fn parse_ripgrep_output(&self, output: &str) -> Result<Vec<SearchResultItem>> {
        let mut items = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse ripgrep output format: "path:line_number:content"
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() >= 3 {
                let path = parts[0].to_string();
                let line_number = parts[1].parse::<usize>().ok();
                let content = parts[2].to_string();

                items.push(SearchResultItem {
                    path,
                    line_number,
                    content,
                    context_before: None, // TODO: Parse context lines if needed
                    context_after: None,
                });
            }
        }

        Ok(items)
    }
}

impl Default for RipgrepSearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchService for RipgrepSearchService {
    async fn search(
        &self,
        query: &str,
        scope: SearchScope,
        workspace_path: Option<PathBuf>,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResult> {
        match scope {
            SearchScope::Workspace => {
                // Search in current workspace
                let path = workspace_path.ok_or_else(|| {
                    OrcsError::internal("Workspace path required for workspace search")
                })?;

                let items = self.execute_ripgrep(query, &path, &filters)?;
                Ok(SearchResult::new(query.to_string(), scope, items))
            }
            SearchScope::Local => {
                // TODO: Implement cross-workspace search
                // For now, just search in current workspace
                let path = workspace_path.ok_or_else(|| {
                    OrcsError::internal("Workspace path required for local search")
                })?;

                let items = self.execute_ripgrep(query, &path, &filters)?;
                Ok(SearchResult::new(query.to_string(), scope, items))
            }
            SearchScope::Global => {
                // Web search not implemented in ripgrep service
                Err(OrcsError::internal(
                    "Global (web) search not supported by RipgrepSearchService",
                ))
            }
        }
    }
}
