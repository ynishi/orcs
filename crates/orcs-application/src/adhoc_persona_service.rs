//! Adhoc Persona Service
//!
//! This service handles the creation and management of adhoc expert personas.
//! Adhoc personas are temporary, session-specific experts generated from
//! expertise descriptions.

use anyhow::Result;
use llm_toolkit::agent::Agent;
use llm_toolkit::ToPrompt;
use orcs_core::persona::{Persona, PersonaBackend, PersonaSource};
use orcs_core::repository::PersonaRepository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Generate expert persona definitions with appropriate characteristics and communication styles",
    output = "PersonaDefinition"
)]
struct PersonaGeneratorAgent;

/// Service for managing adhoc expert personas
pub struct AdhocPersonaService {
    persona_repository: Arc<dyn PersonaRepository>,
}

impl AdhocPersonaService {
    /// Create a new AdhocPersonaService
    pub fn new(persona_repository: Arc<dyn PersonaRepository>) -> Self {
        Self {
            persona_repository,
        }
    }

    /// Generate an adhoc expert persona from expertise description
    ///
    /// # Arguments
    ///
    /// * `expertise` - Description of the expertise area (e.g., "映画制作プロセス")
    ///
    /// # Returns
    ///
    /// * `Ok(Persona)` - Generated adhoc persona
    /// * `Err` - If generation or parsing fails
    pub async fn generate_expert(&self, expertise: String) -> Result<Persona> {
        // Use typed agent to generate persona definition
        let agent = PersonaGeneratorAgent;
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
            .map_err(|e| anyhow::anyhow!(e))?;
        all_personas.push(persona.clone());
        self.persona_repository
            .save_all(&all_personas)
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
