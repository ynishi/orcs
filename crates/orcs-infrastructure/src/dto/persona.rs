//! PersonaConfig DTOs and migrations

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use orcs_core::persona::{Persona, PersonaBackend, PersonaSource};

/// Represents the source of a persona.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaSourceDTO {
    System,
    User,
}

impl Default for PersonaSourceDTO {
    fn default() -> Self {
        PersonaSourceDTO::User
    }
}

/// Represents backend options for a persona.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaBackendDTO {
    ClaudeCli,
    ClaudeApi,
    GeminiCli,
    GeminiApi,
    OpenAiApi,
    CodexCli,
}

impl Default for PersonaBackendDTO {
    fn default() -> Self {
        PersonaBackendDTO::ClaudeCli
    }
}

/// Represents V1 of the persona config schema for serialization.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct PersonaConfigV1_0_0 {
    /// Unique persona identifier.
    pub id: String,
    /// Display name of the persona.
    pub name: String,
    /// Role or title of the persona.
    pub role: String,
    /// Background description of the persona.
    pub background: String,
    /// Communication style of the persona.
    pub communication_style: String,
    /// Whether this persona is a default participant in new sessions.
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User).
    #[serde(default)]
    pub source: PersonaSourceDTO,
}

#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct PersonaConfigV1_1_0 {
    /// Unique persona identifier (UUID format).
    pub id: String,
    /// Display name of the persona.
    pub name: String,
    /// Role or title of the persona.
    pub role: String,
    /// Background description of the persona.
    pub background: String,
    /// Communication style of the persona.
    pub communication_style: String,
    /// Whether this persona is a default participant in new sessions.
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User).
    #[serde(default)]
    pub source: PersonaSourceDTO,
    /// Backend to execute persona with.
    #[serde(default)]
    pub backend: PersonaBackendDTO,
}

#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.2.0")]
pub struct PersonaConfigV1_2_0 {
    /// Unique persona identifier (UUID format).
    pub id: String,
    /// Display name of the persona.
    pub name: String,
    /// Role or title of the persona.
    pub role: String,
    /// Background description of the persona.
    pub background: String,
    /// Communication style of the persona.
    pub communication_style: String,
    /// Whether this persona is a default participant in new sessions.
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User).
    #[serde(default)]
    pub source: PersonaSourceDTO,
    /// Backend to execute persona with (supports all 6 backends: ClaudeCli, ClaudeApi, GeminiCli, GeminiApi, OpenAiApi, CodexCli).
    #[serde(default)]
    pub backend: PersonaBackendDTO,
    /// Model name for the backend (e.g., "claude-sonnet-4-5-20250929", "gemini-2.5-flash")
    /// If None, uses the backend's default model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.3.0")]
pub struct PersonaConfigV1_3_0 {
    /// Unique persona identifier (UUID format).
    pub id: String,
    /// Display name of the persona.
    pub name: String,
    /// Role or title of the persona.
    pub role: String,
    /// Background description of the persona.
    pub background: String,
    /// Communication style of the persona.
    pub communication_style: String,
    /// Whether this persona is a default participant in new sessions.
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User).
    #[serde(default)]
    pub source: PersonaSourceDTO,
    /// Backend to execute persona with (supports all 6 backends: ClaudeCli, ClaudeApi, GeminiCli, GeminiApi, OpenAiApi, CodexCli).
    #[serde(default)]
    pub backend: PersonaBackendDTO,
    /// Model name for the backend (e.g., "claude-sonnet-4-5-20250929", "gemini-2.5-flash")
    /// If None, uses the backend's default model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    /// Visual icon/emoji representing this persona (e.g., "🎨", "🔧", "📊")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Generates a deterministic UUID from a persona name.
fn generate_uuid_from_name(name: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()).to_string()
}

/// Migration from PersonaConfigV1_0_0 to PersonaConfigV1_1_0.
impl MigratesTo<PersonaConfigV1_1_0> for PersonaConfigV1_0_0 {
    fn migrate(self) -> PersonaConfigV1_1_0 {
        // Check if ID is already a valid UUID
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Not a valid UUID - generate a new one from the name
            generate_uuid_from_name(&self.name)
        };

        PersonaConfigV1_1_0 {
            id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source,
            backend: Default::default(),
        }
    }
}

/// Migration from PersonaConfigV1_1_0 to PersonaConfigV1_2_0.
impl MigratesTo<PersonaConfigV1_2_0> for PersonaConfigV1_1_0 {
    fn migrate(self) -> PersonaConfigV1_2_0 {
        PersonaConfigV1_2_0 {
            id: self.id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source,
            backend: self.backend,
            model_name: None, // V1_1_0 doesn't have model_name field
        }
    }
}

/// Migration from PersonaConfigV1_2_0 to PersonaConfigV1_3_0.
impl MigratesTo<PersonaConfigV1_3_0> for PersonaConfigV1_2_0 {
    fn migrate(self) -> PersonaConfigV1_3_0 {
        PersonaConfigV1_3_0 {
            id: self.id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source,
            backend: self.backend,
            model_name: self.model_name,
            icon: None, // V1_2_0 doesn't have icon field
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert PersonaSourceDTO to domain model.
impl From<PersonaSourceDTO> for PersonaSource {
    fn from(dto: PersonaSourceDTO) -> Self {
        match dto {
            PersonaSourceDTO::System => PersonaSource::System,
            PersonaSourceDTO::User => PersonaSource::User,
        }
    }
}

/// Convert domain model to PersonaSourceDTO.
impl From<PersonaSource> for PersonaSourceDTO {
    fn from(source: PersonaSource) -> Self {
        match source {
            PersonaSource::System => PersonaSourceDTO::System,
            PersonaSource::User => PersonaSourceDTO::User,
        }
    }
}

impl From<PersonaBackendDTO> for PersonaBackend {
    fn from(dto: PersonaBackendDTO) -> Self {
        match dto {
            PersonaBackendDTO::ClaudeCli => PersonaBackend::ClaudeCli,
            PersonaBackendDTO::ClaudeApi => PersonaBackend::ClaudeApi,
            PersonaBackendDTO::GeminiCli => PersonaBackend::GeminiCli,
            PersonaBackendDTO::GeminiApi => PersonaBackend::GeminiApi,
            PersonaBackendDTO::OpenAiApi => PersonaBackend::OpenAiApi,
            PersonaBackendDTO::CodexCli => PersonaBackend::CodexCli,
        }
    }
}

impl From<PersonaBackend> for PersonaBackendDTO {
    fn from(backend: PersonaBackend) -> Self {
        match backend {
            PersonaBackend::ClaudeCli => PersonaBackendDTO::ClaudeCli,
            PersonaBackend::ClaudeApi => PersonaBackendDTO::ClaudeApi,
            PersonaBackend::GeminiCli => PersonaBackendDTO::GeminiCli,
            PersonaBackend::GeminiApi => PersonaBackendDTO::GeminiApi,
            PersonaBackend::OpenAiApi => PersonaBackendDTO::OpenAiApi,
            PersonaBackend::CodexCli => PersonaBackendDTO::CodexCli,
        }
    }
}

/// Convert PersonaConfigV1_3_0 DTO to domain model.
impl IntoDomain<Persona> for PersonaConfigV1_3_0 {
    fn into_domain(self) -> Persona {
        // Validate and fix ID if needed
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Legacy data: V1.3.0 schema but non-UUID ID
            generate_uuid_from_name(&self.name)
        };

        Persona {
            id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source.into(),
            backend: self.backend.into(),
            model_name: self.model_name,
            icon: self.icon,
            base_color: None, // V1.3.0 doesn't have base_color
        }
    }
}

/// Convert domain model to PersonaConfigV1_3_0 DTO for persistence.
impl version_migrate::FromDomain<Persona> for PersonaConfigV1_3_0 {
    fn from_domain(persona: Persona) -> Self {
        PersonaConfigV1_3_0 {
            id: persona.id,
            name: persona.name,
            role: persona.role,
            background: persona.background,
            communication_style: persona.communication_style,
            default_participant: persona.default_participant,
            source: persona.source.into(),
            backend: persona.backend.into(),
            model_name: persona.model_name,
            icon: persona.icon,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for Persona entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V1.3.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `backend` field with default value
/// - V1.1.0 → V1.2.0: Adds `model_name` field (optional)
/// - V1.2.0 → V1.3.0: Adds `icon` field (optional)
/// - V1.3.0 → Persona: Converts DTO to domain model (supports all 6 backends via enum expansion)
///
/// # Example
///
/// ```ignore
/// let migrator = create_persona_migrator();
/// let personas: Vec<Persona> = migrator.load_vec_from("persona", toml_personas)?;
/// ```
pub fn create_persona_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> V1.1.0 -> V1.2.0 -> V1.3.0 -> Persona
    let persona_path = version_migrate::Migrator::define("persona")
        .from::<PersonaConfigV1_0_0>()
        .step::<PersonaConfigV1_1_0>()
        .step::<PersonaConfigV1_2_0>()
        .step::<PersonaConfigV1_3_0>()
        .into_with_save::<Persona>();

    migrator
        .register(persona_path)
        .expect("Failed to register persona migration path");

    migrator
}

#[cfg(test)]
mod migrator_tests {
    use super::*;

    #[test]
    fn test_persona_migrator_creation() {
        let _migrator = create_persona_migrator();
        // Migrator should be created successfully
    }

    #[test]
    fn test_persona_migration_v1_0_to_domain() {
        let migrator = create_persona_migrator();

        // Simulate TOML structure with version V1.0.0
        let toml_str = r#"
version = "1.0.0"
id = "test-id"
name = "Test"
role = "Tester"
background = "Test background"
communication_style = "Test style"
default_participant = true
source = "User"
"#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

        // Migrate to domain model using flat format
        let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

        if let Err(e) = &result {
            eprintln!("Migration error: {}", e);
        }
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
        let persona = result.unwrap();
        assert_eq!(persona.name, "Test");
        assert_eq!(persona.role, "Tester");
        // Backend should be default (ClaudeCli) after migration
        assert_eq!(persona.backend, PersonaBackend::ClaudeCli);
    }

    #[test]
    fn test_persona_migration_v1_1_to_domain() {
        let migrator = create_persona_migrator();

        // Simulate TOML structure with version V1.1.0
        let toml_str = r#"
version = "1.1.0"
id = "test-id"
name = "Test"
role = "Tester"
background = "Test background"
communication_style = "Test style"
default_participant = true
source = "User"
backend = "gemini_cli"
"#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

        // Migrate to domain model using flat format
        let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

        if let Err(e) = &result {
            eprintln!("Migration error: {}", e);
        }
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
        let persona = result.unwrap();
        assert_eq!(persona.name, "Test");
        assert_eq!(persona.backend, PersonaBackend::GeminiCli);
    }
}
