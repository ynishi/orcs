use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

pub fn bump(version: &str) -> Result<()> {
    println!("📝 Bumping version to {}...", version);

    // Update orcs-desktop/package.json
    update_package_json("orcs-desktop/package.json", version)?;

    // Update orcs-desktop/src-tauri/tauri.conf.json
    update_tauri_conf("orcs-desktop/src-tauri/tauri.conf.json", version)?;

    // Update workspace Cargo.toml
    update_cargo_toml("Cargo.toml", version)?;

    println!("✅ Version bumped to {} successfully!", version);
    println!("\n📋 Updated files:");
    println!("  - orcs-desktop/package.json");
    println!("  - orcs-desktop/src-tauri/tauri.conf.json");
    println!("  - Cargo.toml (workspace)");
    println!("\n💡 Next steps:");
    println!("  1. Run: cargo update (to update Cargo.lock)");
    println!("  2. Test the build: orcs build local");
    println!("  3. Commit changes: git add -A && git commit -m 'Bump version to {}'", version);
    println!("  4. Create tag: git tag v{}", version);
    println!("  5. Push: git push && git push --tags");

    Ok(())
}

fn update_package_json(path: &str, version: &str) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    let mut json: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {} as JSON", path))?;

    json["version"] = Value::String(version.to_string());

    let updated = serde_json::to_string_pretty(&json)?;
    fs::write(path, updated + "\n")
        .with_context(|| format!("Failed to write {}", path))?;

    println!("  ✓ Updated {}", path);
    Ok(())
}

fn update_tauri_conf(path: &str, version: &str) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    let mut json: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {} as JSON", path))?;

    json["version"] = Value::String(version.to_string());

    let updated = serde_json::to_string_pretty(&json)?;
    fs::write(path, updated + "\n")
        .with_context(|| format!("Failed to write {}", path))?;

    println!("  ✓ Updated {}", path);
    Ok(())
}

fn update_cargo_toml(path: &str, version: &str) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut updated_lines = Vec::new();
    let mut in_workspace_package = false;

    for line in lines {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
            updated_lines.push(line.to_string());
        } else if in_workspace_package && line.starts_with("version") {
            updated_lines.push(format!("version = \"{}\"", version));
            in_workspace_package = false;
        } else if line.starts_with('[') && line != "[workspace.package]" {
            in_workspace_package = false;
            updated_lines.push(line.to_string());
        } else {
            updated_lines.push(line.to_string());
        }
    }

    fs::write(path, updated_lines.join("\n") + "\n")
        .with_context(|| format!("Failed to write {}", path))?;

    println!("  ✓ Updated {}", path);
    Ok(())
}

pub fn show() -> Result<()> {
    let cargo_path = "Cargo.toml";
    let content = fs::read_to_string(cargo_path)
        .with_context(|| format!("Failed to read {}", cargo_path))?;

    for line in content.lines() {
        if line.trim().starts_with("version") && line.contains("\"") {
            let version = line.split('"').nth(1).unwrap_or("unknown");
            println!("Current version: {}", version);
            return Ok(());
        }
    }

    println!("Version not found in Cargo.toml");
    Ok(())
}
