//! Agent execution environment utilities.

use std::path::{Path, PathBuf};

/// Builds an enhanced PATH environment variable that includes workspace-specific
/// tool directories and system binary paths.
///
/// # Priority Order
/// 1. Workspace-specific tool directories (node_modules/.bin, .venv/bin, etc.)
/// 2. System paths from /etc/paths and /etc/paths.d/*
/// 3. Common binary locations (/usr/local/bin, /usr/bin, etc.)
/// 4. User home bin directories (~/.local/bin, ~/bin)
/// 5. Existing PATH entries
///
/// # Arguments
/// * `workspace_root` - Root directory of the workspace
///
/// # Returns
/// Colon-separated PATH string ready for use in environment variables
///
/// # Example
/// ```
/// use orcs_core::agent::build_enhanced_path;
/// use std::path::PathBuf;
///
/// let workspace = PathBuf::from("/path/to/project");
/// let enhanced_path = build_enhanced_path(&workspace);
/// assert!(!enhanced_path.is_empty());
/// ```
pub fn build_enhanced_path(workspace_root: &Path) -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut path_components = Vec::new();

    // 1. Add workspace-specific tool directories (highest priority)
    let workspace_tool_dirs = vec![
        workspace_root.join("node_modules/.bin"), // npm/yarn
        workspace_root.join(".venv/bin"),         // Python venv
        workspace_root.join("target/debug"),      // Rust debug builds
        workspace_root.join("target/release"),    // Rust release builds
        workspace_root.join("bin"),               // Generic bin
    ];

    for dir in workspace_tool_dirs {
        if dir.exists() {
            if let Some(dir_str) = dir.to_str() {
                if !path_components.contains(&dir_str.to_string()) {
                    path_components.push(dir_str.to_string());
                }
            }
        }
    }

    // 2. Read system paths from /etc/paths (macOS/Linux)
    #[cfg(unix)]
    {
        if let Ok(contents) = std::fs::read_to_string("/etc/paths") {
            for line in contents.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !path_components.contains(&trimmed.to_string()) {
                    path_components.push(trimmed.to_string());
                }
            }
        }

        // Read from /etc/paths.d/*
        if let Ok(entries) = std::fs::read_dir("/etc/paths.d") {
            for entry in entries.flatten() {
                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                    for line in contents.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty()
                            && !path_components.contains(&trimmed.to_string())
                        {
                            path_components.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Add common binary locations
    let common_paths = vec![
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
        "/opt/homebrew/bin", // Apple Silicon Homebrew
        "/usr/local/opt",
    ];

    for path in common_paths {
        if !path_components.contains(&path.to_string()) {
            path_components.push(path.to_string());
        }
    }

    // 4. Add user's home bin directories
    if let Ok(home) = std::env::var("HOME") {
        let home_paths = vec![
            PathBuf::from(&home).join(".local/bin"),
            PathBuf::from(&home).join("bin"),
        ];

        for path in home_paths {
            if path.exists() {
                if let Some(path_str) = path.to_str() {
                    if !path_components.contains(&path_str.to_string()) {
                        path_components.push(path_str.to_string());
                    }
                }
            }
        }
    }

    // 5. Preserve any existing PATH entries that aren't already included
    if !current_path.is_empty() {
        for existing in current_path.split(':') {
            if !existing.is_empty() && !path_components.contains(&existing.to_string()) {
                path_components.push(existing.to_string());
            }
        }
    }

    path_components.join(":")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_enhanced_path_not_empty() {
        let workspace = PathBuf::from("/test/workspace");
        let path = build_enhanced_path(&workspace);

        // Should always produce a non-empty path
        assert!(!path.is_empty());
    }

    #[test]
    fn test_build_enhanced_path_includes_common_paths() {
        let workspace = PathBuf::from("/test/workspace");
        let path = build_enhanced_path(&workspace);

        // Should include at least one common binary path
        assert!(
            path.contains("/usr/bin") || path.contains("/usr/local/bin"),
            "PATH should include common binary directories"
        );
    }

    #[test]
    fn test_build_enhanced_path_preserves_existing_path() {
        // Save original PATH
        let original_path = std::env::var("PATH").ok();

        // SAFETY: This test temporarily modifies PATH in a controlled manner
        // and restores it afterwards. No other code runs during this test.
        unsafe {
            std::env::set_var("PATH", "/custom/bin:/another/path");
        }
        let workspace = PathBuf::from("/test/workspace");
        let path = build_enhanced_path(&workspace);

        assert!(path.contains("/custom/bin"));
        assert!(path.contains("/another/path"));

        // Restore original PATH
        if let Some(original) = original_path {
            // SAFETY: Restoring the original PATH value
            unsafe {
                std::env::set_var("PATH", original);
            }
        }
    }
}
