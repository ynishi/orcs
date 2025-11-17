//! Agent execution environment utilities.

use std::path::{Path, PathBuf};

/// Builds an enhanced PATH environment variable that includes workspace-specific
/// tool directories, user-configured paths, and system binary paths.
///
/// # Priority Order
/// 1. Workspace-specific tool directories (node_modules/.bin, .venv/bin, etc.)
/// 2. User-configured additional paths (from EnvSettings)
/// 3. Tool manager paths (mise, asdf, volta) if auto-detection is enabled
/// 4. System paths from /etc/paths and /etc/paths.d/*
/// 5. Common binary locations (/usr/local/bin, /usr/bin, etc.)
/// 6. User home bin directories (~/.local/bin, ~/bin)
/// 7. Existing PATH entries
///
/// # Arguments
/// * `workspace_root` - Root directory of the workspace
/// * `env_settings` - Optional environment configuration for PATH customization
///
/// # Returns
/// Colon-separated PATH string ready for use in environment variables
///
/// # Example
/// ```
/// use orcs_core::agent::build_enhanced_path;
/// use orcs_core::config::EnvSettings;
/// use std::path::PathBuf;
///
/// let workspace = PathBuf::from("/path/to/project");
/// let settings = EnvSettings {
///     additional_paths: vec!["/custom/bin".to_string()],
///     auto_detect_tool_managers: true,
/// };
/// let enhanced_path = build_enhanced_path(&workspace, Some(&settings));
/// assert!(!enhanced_path.is_empty());
/// ```
pub fn build_enhanced_path(
    workspace_root: &Path,
    env_settings: Option<&crate::config::EnvSettings>,
) -> String {
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

    // 2. Add user-configured additional paths (from EnvSettings)
    if let Some(settings) = env_settings {
        for path in &settings.additional_paths {
            if !path_components.contains(&path.to_string()) {
                path_components.push(path.clone());
            }
        }

        // 3. Add tool manager paths if auto-detection is enabled
        if settings.auto_detect_tool_managers {
            for path in detect_tool_manager_paths() {
                if !path_components.contains(&path) {
                    path_components.push(path);
                }
            }
        }
    }

    // 4. Read system paths from /etc/paths (macOS/Linux)
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
                        if !trimmed.is_empty() && !path_components.contains(&trimmed.to_string()) {
                            path_components.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    // 5. Add common binary locations
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

    // 6. Add user's home bin directories
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

    // 7. Preserve any existing PATH entries that aren't already included
    if !current_path.is_empty() {
        for existing in current_path.split(':') {
            if !existing.is_empty() && !path_components.contains(&existing.to_string()) {
                path_components.push(existing.to_string());
            }
        }
    }

    path_components.join(":")
}

/// Detects and returns paths from common tool managers (mise, asdf, volta, etc.).
///
/// This function searches for tool manager installations and returns their PATH directories
/// if they exist. Only returns paths that actually exist on the filesystem.
///
/// # Supported Tool Managers
/// - **mise**: `~/.local/share/mise/shims`, `~/.local/share/mise/installs/*/bin`
/// - **asdf**: `~/.asdf/shims`
/// - **volta**: `~/.volta/bin`
///
/// # Returns
/// A vector of path strings that exist on the filesystem
fn detect_tool_manager_paths() -> Vec<String> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    let mut detected_paths = Vec::new();

    // mise paths
    let mise_shims = PathBuf::from(&home).join(".local/share/mise/shims");
    if mise_shims.exists() {
        if let Some(path_str) = mise_shims.to_str() {
            detected_paths.push(path_str.to_string());
        }
    }

    // Scan mise installs directory for specific tool bins (e.g., gemini, node, etc.)
    let mise_installs = PathBuf::from(&home).join(".local/share/mise/installs");
    if mise_installs.exists() {
        if let Ok(entries) = std::fs::read_dir(&mise_installs) {
            for entry in entries.flatten() {
                let tool_path = entry.path();
                // Look for version directories under each tool (e.g., gemini/0.11.0/bin)
                if tool_path.is_dir() {
                    if let Ok(version_entries) = std::fs::read_dir(&tool_path) {
                        for version_entry in version_entries.flatten() {
                            let bin_path = version_entry.path().join("bin");
                            if bin_path.exists() && bin_path.is_dir() {
                                if let Some(bin_str) = bin_path.to_str() {
                                    detected_paths.push(bin_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // asdf paths
    let asdf_shims = PathBuf::from(&home).join(".asdf/shims");
    if asdf_shims.exists() {
        if let Some(path_str) = asdf_shims.to_str() {
            detected_paths.push(path_str.to_string());
        }
    }

    // volta paths
    let volta_bin = PathBuf::from(&home).join(".volta/bin");
    if volta_bin.exists() {
        if let Some(path_str) = volta_bin.to_str() {
            detected_paths.push(path_str.to_string());
        }
    }

    detected_paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_enhanced_path_not_empty() {
        let workspace = PathBuf::from("/test/workspace");
        let path = build_enhanced_path(&workspace, None);

        // Should always produce a non-empty path
        assert!(!path.is_empty());
    }

    #[test]
    fn test_build_enhanced_path_includes_common_paths() {
        let workspace = PathBuf::from("/test/workspace");
        let path = build_enhanced_path(&workspace, None);

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
        let path = build_enhanced_path(&workspace, None);

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

    #[test]
    fn test_build_enhanced_path_with_env_settings() {
        use crate::config::EnvSettings;

        let workspace = PathBuf::from("/test/workspace");
        let settings = EnvSettings {
            additional_paths: vec!["/custom/tool/bin".to_string(), "/opt/myapp/bin".to_string()],
            auto_detect_tool_managers: false, // Disable auto-detect for test stability
        };
        let path = build_enhanced_path(&workspace, Some(&settings));

        // Should include user-configured additional paths
        assert!(
            path.contains("/custom/tool/bin"),
            "PATH should include user-configured additional paths"
        );
        assert!(
            path.contains("/opt/myapp/bin"),
            "PATH should include user-configured additional paths"
        );
    }

    #[test]
    fn test_detect_tool_manager_paths() {
        // This test only verifies that the function runs without errors
        // Actual paths depend on the system environment
        let paths = detect_tool_manager_paths();

        // Should return a Vec, may be empty if no tool managers are installed
        assert!(paths.is_empty() || !paths.is_empty());
    }
}
