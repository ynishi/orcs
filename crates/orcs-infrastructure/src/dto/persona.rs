//! PersonaConfig DTOs and migrations

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, MigratesTo, Versioned};
use uuid::Uuid;

use orcs_core::persona::{Persona, PersonaSource};

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

/// Represents V1 of the persona config schema for serialization.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct PersonaConfigV1 {
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

/// Represents V2 of the persona config schema for serialization.
///
/// V2 introduces UUID-based IDs for better internationalization and future extensibility.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct PersonaConfigV2 {
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
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Generates a deterministic UUID from a persona name.
fn generate_uuid_from_name(name: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()).to_string()
}

/// Migration from PersonaConfigV1 to PersonaConfigV2.
impl MigratesTo<PersonaConfigV2> for PersonaConfigV1 {
    fn migrate(self) -> PersonaConfigV2 {
        // Check if ID is already a valid UUID
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Not a valid UUID - generate a new one from the name
            generate_uuid_from_name(&self.name)
        };

        PersonaConfigV2 {
            id,
            name: self.name,
            role: self.role,
            background: self.background,
            communication_style: self.communication_style,
            default_participant: self.default_participant,
            source: self.source,
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

/// Convert PersonaConfigV2 DTO to domain model.
impl IntoDomain<Persona> for PersonaConfigV2 {
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
        }
    }
}

/// Convert domain model to PersonaConfigV2 DTO for persistence.
impl From<&Persona> for PersonaConfigV2 {
    fn from(persona: &Persona) -> Self {
        PersonaConfigV2 {
            id: persona.id.clone(),
            name: persona.name.clone(),
            role: persona.role.clone(),
            background: persona.background.clone(),
            communication_style: persona.communication_style.clone(),
            default_participant: persona.default_participant,
            source: persona.source.clone().into(),
        }
    }
}
