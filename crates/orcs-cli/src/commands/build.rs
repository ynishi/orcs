use anyhow::{Context, Result};
use std::process::Command;

use super::utils::find_workspace_root;

pub fn run() -> Result<()> {
    println!("🔨 Building ORCS Desktop...");

    // Find workspace root and navigate to orcs-desktop
    let workspace_root = find_workspace_root()?;
    let orcs_desktop_dir = workspace_root.join("orcs-desktop");

    let status = Command::new("npm")
        .args(["run", "tauri", "build"])
        .current_dir(&orcs_desktop_dir)
        .status()
        .context("Failed to execute npm run tauri build")?;

    if status.success() {
        println!("✅ Build completed successfully!");
        println!("📦 Bundle location: target/release/bundle/");
    } else {
        anyhow::bail!("Build failed with exit code: {:?}", status.code());
    }

    Ok(())
}
