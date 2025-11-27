use anyhow::{Context, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    println!("🔨 Building ORCS Desktop...");

    let status = Command::new("npm")
        .args(["run", "tauri", "build"])
        .current_dir("orcs-desktop")
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
