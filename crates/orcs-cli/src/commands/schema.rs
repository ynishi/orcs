use anyhow::{Context, Result};
use orcs_core::schema::{
    ConversationModeType, ExecutionModelType, PresetSourceType, TalkStyleType,
};
use schema_bridge::{export_types, SchemaBridge};
use std::env;
use std::path::PathBuf;

pub fn generate() -> Result<()> {
    // Get the workspace root
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set. Run from cargo or set manually.")?;
    let manifest_path = PathBuf::from(&manifest_dir);

    // Navigate to workspace root (crates/orcs-cli -> crates -> root)
    let workspace_root = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .context("Failed to find workspace root")?;

    // Target path: orcs-desktop/src/types/generated/schema.ts
    let output_file = workspace_root
        .join("orcs-desktop")
        .join("src")
        .join("types")
        .join("generated")
        .join("schema.ts");

    println!("Generating TypeScript types...");
    println!("Output: {}", output_file.display());

    // Use schema-bridge's export_types! macro
    export_types!(
        output_file.to_str().unwrap(),
        TalkStyleType,
        ExecutionModelType,
        ConversationModeType,
        PresetSourceType
    )?;

    println!(" TypeScript types generated successfully!");

    Ok(())
}
