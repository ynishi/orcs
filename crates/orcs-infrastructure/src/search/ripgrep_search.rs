//! Ripgrep-based search implementation.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

use orcs_core::agent::build_enhanced_path;
use orcs_core::error::{OrcsError, Result};
use orcs_core::search::model::SearchResultItem;
use orcs_core::search::{SearchFilters, SearchOptions, SearchResult, SearchService};

/// Search service implementation using ripgrep.
pub struct RipgrepSearchService;

impl RipgrepSearchService {
    pub fn new() -> Self {
        Self
    }

    /// Searches for files by filename matching the query.
    fn search_by_filename(
        &self,
        query: &str,
        search_paths: &[PathBuf],
        filters: &Option<SearchFilters>,
    ) -> Result<Vec<SearchResultItem>> {
        if search_paths.is_empty() {
            return Ok(Vec::new());
        }

        let mut cmd = Command::new("rg");

        // Set enhanced PATH
        let enhanced_path = build_enhanced_path(&search_paths[0], None);
        cmd.env("PATH", enhanced_path);

        // List files only
        cmd.arg("--files");

        // Apply file type filters if provided
        if let Some(f) = filters {
            if let Some(ref types) = f.file_types {
                for t in types {
                    cmd.arg("--type").arg(t);
                }
            }

            if let Some(ref excludes) = f.exclude_paths {
                for exclude in excludes {
                    cmd.arg("--glob").arg(format!("!{}", exclude));
                }
            }
        }

        // Add all search paths
        for path in search_paths {
            cmd.arg(path);
        }

        tracing::debug!("Executing ripgrep --files command: {:?}", cmd);

        // Execute command
        let output = cmd
            .output()
            .map_err(|e| OrcsError::io(format!("Failed to execute ripgrep --files: {}", e)))?;

        // Parse output and filter by query
        let stdout = String::from_utf8_lossy(&output.stdout);
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Check if filename contains the query (case-insensitive)
            let path_str = line.to_string();
            if let Some(file_name) = std::path::Path::new(&path_str).file_name() {
                let file_name_str = file_name.to_string_lossy().to_string();
                if file_name_str.to_lowercase().contains(&query_lower) {
                    items.push(SearchResultItem {
                        path: path_str,
                        line_number: None,
                        content: format!("[Filename match: {}]", file_name_str),
                        context_before: None,
                        context_after: None,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Executes ripgrep command with the given parameters.
    fn execute_ripgrep(
        &self,
        query: &str,
        search_paths: &[PathBuf],
        filters: &Option<SearchFilters>,
    ) -> Result<Vec<SearchResultItem>> {
        if search_paths.is_empty() {
            return Ok(Vec::new());
        }

        let mut cmd = Command::new("rg");

        // Set enhanced PATH to find ripgrep in system and workspace-specific locations
        // Use first path for PATH enhancement
        let enhanced_path = build_enhanced_path(&search_paths[0], None);
        cmd.env("PATH", enhanced_path);

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

        // Add query
        cmd.arg(query);

        // Add all search paths
        for path in search_paths {
            cmd.arg(path);
        }

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
        options: SearchOptions,
        search_paths: Vec<PathBuf>,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResult> {
        if search_paths.is_empty() {
            // No paths to search - return empty result
            return Ok(SearchResult::empty(query.to_string(), options));
        }

        // Search both file contents and filenames
        let content_items = self.execute_ripgrep(query, &search_paths, &filters)?;
        let filename_items = self.search_by_filename(query, &search_paths, &filters)?;

        // Merge results (filename matches first, then content matches)
        let mut all_items = filename_items;
        all_items.extend(content_items);

        Ok(SearchResult::new(query.to_string(), options, all_items))
    }
}
