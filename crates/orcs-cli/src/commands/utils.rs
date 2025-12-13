use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

/// Find the workspace root directory.
///
/// This function tries multiple strategies to locate the workspace root:
/// 1. Uses CARGO_MANIFEST_DIR if running via `cargo run`
/// 2. Searches upward from current directory for workspace Cargo.toml
/// 3. Falls back to current directory and checks for orcs-desktop subdirectory
pub fn find_workspace_root() -> Result<PathBuf> {
    // Strategy 1: Use CARGO_MANIFEST_DIR if available (running via cargo run)
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(&manifest_dir);
        // Navigate up from crates/orcs-cli to workspace root
        if let Some(workspace_root) = manifest_path.parent().and_then(|p| p.parent()) {
            if workspace_root.join("orcs-desktop").exists() {
                return Ok(workspace_root.to_path_buf());
            }
        }
    }

    // Strategy 2: Search upward from current directory
    let mut current = env::current_dir().context("Failed to get current directory")?;
    loop {
        // Check if this directory has both Cargo.toml and orcs-desktop subdirectory
        let cargo_toml = current.join("Cargo.toml");
        let orcs_desktop = current.join("orcs-desktop");

        if cargo_toml.exists() && orcs_desktop.exists() {
            // Verify it's a workspace by checking for [workspace] section
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Ok(current);
                }
            }
        }

        // Move up one directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Strategy 3: Check if current directory is already the workspace root
    let cwd = env::current_dir().context("Failed to get current directory")?;
    if cwd.join("orcs-desktop").exists() && cwd.join("Cargo.toml").exists() {
        return Ok(cwd);
    }

    anyhow::bail!(
        "Could not find workspace root. Please run this command from the ORCS project directory."
    )
}
