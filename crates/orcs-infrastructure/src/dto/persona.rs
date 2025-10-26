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
    GeminiCli,
    GeminiApi,
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
            PersonaBackendDTO::GeminiCli => PersonaBackend::GeminiCli,
            PersonaBackendDTO::GeminiApi => PersonaBackend::GeminiApi,
        }
    }
}

impl From<PersonaBackend> for PersonaBackendDTO {
    fn from(backend: PersonaBackend) -> Self {
        match backend {
            PersonaBackend::ClaudeCli => PersonaBackendDTO::ClaudeCli,
            PersonaBackend::GeminiCli => PersonaBackendDTO::GeminiCli,
            PersonaBackend::GeminiApi => PersonaBackendDTO::GeminiApi,
        }
    }
}

/// Convert PersonaConfigV1_1_0 DTO to domain model.
impl IntoDomain<Persona> for PersonaConfigV1_1_0 {
    fn into_domain(self) -> Persona {
        // Validate and fix ID if needed
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Legacy data: V2 schema but non-UUID ID
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
        }
    }
}

/// Convert domain model to PersonaConfigV1_1_0 DTO for persistence.
impl From<&Persona> for PersonaConfigV1_1_0 {
    fn from(persona: &Persona) -> Self {
        PersonaConfigV1_1_0 {
            id: persona.id.clone(),
            name: persona.name.clone(),
            role: persona.role.clone(),
            background: persona.background.clone(),
            communication_style: persona.communication_style.clone(),
            default_participant: persona.default_participant,
            source: persona.source.clone().into(),
            backend: persona.backend.clone().into(),
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for Persona entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V1.1.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `backend` field with default value
/// - V1.1.0 → Persona: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_persona_migrator();
/// let personas: Vec<Persona> = migrator.load_vec_from("persona", toml_personas)?;
/// ```
pub fn create_persona_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> V1.1.0 -> Persona
    let persona_path = version_migrate::Migrator::define("persona")
        .from::<PersonaConfigV1_0_0>()
        .step::<PersonaConfigV1_1_0>()
        .into::<Persona>();

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
