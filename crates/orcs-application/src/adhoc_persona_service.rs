//! Adhoc Persona Service
//!
//! This service handles the creation and management of adhoc expert personas.
//! Adhoc personas are temporary, session-specific experts generated from
//! expertise descriptions.

use anyhow::Result;
use llm_toolkit::ToPrompt;
use llm_toolkit::agent::Agent;
use llm_toolkit::agent::impls::ClaudeCodeAgent;
use orcs_core::persona::{Persona, PersonaBackend, PersonaSource};
use orcs_core::repository::PersonaRepository;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Builds an enhanced PATH environment variable that includes workspace-specific tool directories
/// and system binary paths.
fn build_enhanced_path(workspace_root: &Path) -> String {
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
                        if !trimmed.is_empty() && !path_components.contains(&trimmed.to_string()) {
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

/// Structured persona definition output from LLM
#[derive(Debug, Clone, Serialize, Deserialize, ToPrompt)]
#[prompt(mode = "full")]
pub struct PersonaDefinition {
    /// Professional name for the expert (e.g., "Film Production Specialist")
    #[serde(default)]
    pub name: String,
    /// Concise role title (e.g., "映画制作プロセスの専門家")
    #[serde(default)]
    pub role: String,
    /// 2-3 sentences about expertise and experience
    #[serde(default)]
    pub background: String,
    /// Communication style (e.g., "実践的で具体的なアドバイス")
    #[serde(default)]
    pub communication_style: String,
    /// Single emoji representing the expert (e.g., "🎬")
    #[serde(default)]
    pub icon: Option<String>,
}

/// Typed agent for generating persona definitions
#[llm_toolkit::agent(
    expertise = "Generate expert persona definitions with appropriate characteristics and communication styles",
    output = "PersonaDefinition",
    default_inner = "llm_toolkit::agent::impls::ClaudeCodeAgent"
)]
struct PersonaGeneratorAgent;

/// Service for managing adhoc expert personas
pub struct AdhocPersonaService {
    persona_repository: Arc<dyn PersonaRepository>,
}

impl AdhocPersonaService {
    /// Create a new AdhocPersonaService
    pub fn new(persona_repository: Arc<dyn PersonaRepository>) -> Self {
        Self { persona_repository }
    }

    /// Generate an adhoc expert persona from expertise description
    ///
    /// # Arguments
    ///
    /// * `expertise` - Description of the expertise area (e.g., "映画制作プロセス")
    /// * `workspace_root` - Optional workspace root path for agent execution context
    ///
    /// # Returns
    ///
    /// * `Ok(Persona)` - Generated adhoc persona
    /// * `Err` - If generation or parsing fails
    pub async fn generate_expert(
        &self,
        expertise: String,
        workspace_root: Option<PathBuf>,
    ) -> Result<Persona> {
        // Create ClaudeCodeAgent with workspace context and enhanced PATH
        let mut inner_agent = ClaudeCodeAgent::new();

        if let Some(workspace) = workspace_root {
            tracing::info!(
                "[AdhocPersonaService] Setting workspace root for persona generation: {:?}",
                workspace
            );
            let enhanced_path = build_enhanced_path(&workspace);
            inner_agent = inner_agent
                .with_cwd(workspace)
                .with_env("PATH", enhanced_path);
        }

        // Create PersonaGeneratorAgent with configured inner agent
        let agent = PersonaGeneratorAgent::new(inner_agent);

        let prompt = format!(
            "Create a detailed expert persona definition for: {}

Consider:
- Appropriate professional name that reflects the domain
- Concise but descriptive role title
- Background showing depth of expertise
- Communication style that fits the domain
- Representative emoji/icon

Generate a complete PersonaDefinition.",
            expertise
        );

        // Execute with typed output
        let definition = agent.execute(prompt.into()).await?;

        // Create Persona from definition
        let persona = Persona {
            id: uuid::Uuid::new_v4().to_string(),
            name: definition.name,
            role: definition.role,
            background: definition.background,
            communication_style: definition.communication_style,
            default_participant: false,
            source: PersonaSource::Adhoc,
            backend: PersonaBackend::ClaudeCli,
            model_name: None,
            icon: definition.icon,
            base_color: None,
        };

        // Save adhoc persona to repository (temporary)
        let mut all_personas = self
            .persona_repository
            .get_all()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        all_personas.push(persona.clone());
        self.persona_repository
            .save_all(&all_personas)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(persona)
    }

    /// Promote an adhoc persona to permanent user persona
    ///
    /// # Arguments
    ///
    /// * `persona_id` - ID of the adhoc persona to promote
    ///
    /// # Returns
    ///
    /// * `Ok(Persona)` - The promoted persona (now with source = User)
    /// * `Err` - If persona not found or not adhoc
    pub async fn promote_to_user(&self, persona_id: &str) -> Result<Persona> {
        // Get all personas
        let mut personas = self
            .persona_repository
            .get_all()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // Find and update the adhoc persona
        let mut found = false;
        for persona in &mut personas {
            if persona.id == persona_id && persona.source == PersonaSource::Adhoc {
                persona.source = PersonaSource::User;
                found = true;
                break;
            }
        }

        if !found {
            return Err(anyhow::anyhow!("Adhoc persona not found"));
        }

        // Save all personas
        self.persona_repository
            .save_all(&personas)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // Get the updated persona
        let saved_persona = personas
            .iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| anyhow::anyhow!("Persona not found after save"))?
            .clone();

        Ok(saved_persona)
    }
}
