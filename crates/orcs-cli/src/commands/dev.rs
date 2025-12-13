use anyhow::{Context, Result};
use std::process::Command;

use super::utils::find_workspace_root;

pub fn run() -> Result<()> {
    println!("🚀 Starting ORCS Desktop in development mode...");

    // Find workspace root and navigate to orcs-desktop
    let workspace_root = find_workspace_root()?;
    let orcs_desktop_dir = workspace_root.join("orcs-desktop");

    let status = Command::new("npm")
        .args(["run", "tauri", "dev"])
        .current_dir(&orcs_desktop_dir)
        .status()
        .context("Failed to execute npm run tauri dev")?;

    if status.success() {
        println!("✅ Development server stopped successfully!");
    } else {
        anyhow::bail!("Dev server failed with exit code: {:?}", status.code());
    }

    Ok(())
}
