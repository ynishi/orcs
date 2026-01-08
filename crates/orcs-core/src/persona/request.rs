//! Persona creation and update request models.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{GeminiOptions, KaibaOptions, Persona, PersonaBackend, PersonaSource};

/// Request to create a new persona.
///
/// This is the unified request model used by both:
/// - SlashCommand `/create-persona` (from JSON)
/// - UI Form (from PersonaEditorModal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersonaRequest {
    /// Display name (required)
    pub name: String,

    /// Role or title (required)
    pub role: String,

    /// Background description (required, min 10 chars recommended)
    pub background: String,

    /// Communication style (required, min 10 chars recommended)
    pub communication_style: String,

    /// Whether to include in new sessions by default
    #[serde(default)]
    pub default_participant: bool,

    /// LLM backend to use
    pub backend: PersonaBackend,

    /// Optional specific model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,

    /// Optional visual icon/emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Optional base color for UI theming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color: Option<String>,

    /// Gemini-specific options (thinking level, Google Search)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini_options: Option<GeminiOptions>,

    /// Kaiba-specific options (Rei ID for persistent memory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kaiba_options: Option<KaibaOptions>,
}

impl CreatePersonaRequest {
    /// Validate the request and return errors if any.
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err("Name is required and cannot be empty".to_string());
        }

        // Validate role
        if self.role.trim().is_empty() {
            return Err("Role is required and cannot be empty".to_string());
        }

        // Validate background (min 10 chars)
        if self.background.trim().len() < 10 {
            return Err("Background must be at least 10 characters long".to_string());
        }

        // Validate communication_style (min 10 chars)
        if self.communication_style.trim().len() < 10 {
            return Err("Communication style must be at least 10 characters long".to_string());
        }

        Ok(())
    }

    /// Convert this request into a Persona, always generating a new UUID.
    pub fn into_persona(self) -> Persona {
        let id = Uuid::new_v4().to_string();

        Persona {
            id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: PersonaSource::User,
            backend: self.backend,
            model_name: self.model_name,
            icon: self.icon,
            base_color: self.base_color,
            gemini_options: self.gemini_options,
            kaiba_options: self.kaiba_options,
        }
    }

    /// Create a request from an existing Persona (for editing).
    pub fn from_persona(persona: &Persona) -> Self {
        Self {
            name: persona.name.clone(),
            role: persona.role.clone(),
            background: persona.background.clone(),
            communication_style: persona.communication_style.clone(),
            default_participant: persona.default_participant,
            backend: persona.backend.clone(),
            model_name: persona.model_name.clone(),
            icon: persona.icon.clone(),
            base_color: persona.base_color.clone(),
            gemini_options: persona.gemini_options.clone(),
            kaiba_options: persona.kaiba_options.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_success() {
        let req = CreatePersonaRequest {
            name: "Test Persona".to_string(),
            role: "Tester".to_string(),
            background: "This is a test background with enough characters".to_string(),
            communication_style: "Clear and concise communication".to_string(),
            default_participant: false,
            backend: PersonaBackend::ClaudeCli,
            model_name: None,
            icon: None,
            base_color: None,
            gemini_options: None,
            kaiba_options: None,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let req = CreatePersonaRequest {
            name: "".to_string(),
            role: "Tester".to_string(),
            background: "Valid background".to_string(),
            communication_style: "Valid style".to_string(),
            default_participant: false,
            backend: PersonaBackend::ClaudeCli,
            model_name: None,
            icon: None,
            base_color: None,
            gemini_options: None,
            kaiba_options: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_short_background() {
        let req = CreatePersonaRequest {
            name: "Test".to_string(),
            role: "Tester".to_string(),
            background: "Short".to_string(),
            communication_style: "Valid style here".to_string(),
            default_participant: false,
            backend: PersonaBackend::ClaudeCli,
            model_name: None,
            icon: None,
            base_color: None,
            gemini_options: None,
            kaiba_options: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_into_persona_generates_uuid() {
        let req = CreatePersonaRequest {
            name: "Test".to_string(),
            role: "Tester".to_string(),
            background: "Valid background".to_string(),
            communication_style: "Valid style".to_string(),
            default_participant: false,
            backend: PersonaBackend::ClaudeCli,
            model_name: None,
            icon: None,
            base_color: None,
            gemini_options: None,
            kaiba_options: None,
        };

        let persona = req.into_persona();
        assert!(Uuid::parse_str(&persona.id).is_ok());
    }

    #[test]
    fn test_from_persona() {
        let persona = Persona {
            id: Uuid::new_v4().to_string(),
            name: "Test".to_string(),
            role: "Tester".to_string(),
            background: "Background".to_string(),
            communication_style: "Style".to_string(),
            default_participant: true,
            source: PersonaSource::User,
            backend: PersonaBackend::ClaudeApi,
            model_name: Some("claude-sonnet-4-5".to_string()),
            icon: Some("🎨".to_string()),
            base_color: Some("#FF5733".to_string()),
            gemini_options: None,
            kaiba_options: None,
        };

        let req = CreatePersonaRequest::from_persona(&persona);
        assert_eq!(req.name, persona.name);
        assert_eq!(req.backend, persona.backend);
    }
}
