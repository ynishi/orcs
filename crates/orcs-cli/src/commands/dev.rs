use anyhow::{Context, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    println!("🚀 Starting ORCS Desktop in development mode...");

    let status = Command::new("npm")
        .args(["run", "tauri", "dev"])
        .current_dir("orcs-desktop")
        .status()
        .context("Failed to execute npm run tauri dev")?;

    if status.success() {
        println!("✅ Development server stopped successfully!");
    } else {
        anyhow::bail!("Dev server failed with exit code: {:?}", status.code());
    }

    Ok(())
}
