//! PersonaConfig entity migrations.
//!
//! This module contains all migration implementations for the PersonaConfig entity.
//! The main migration (V1→V2) converts string-based IDs to UUID-based IDs for
//! better internationalization and future extensibility.

use super::traits::{Migration, TypedMigration};
use crate::dto::{
    PersonaConfigV1, PersonaConfigV2, PersonaSourceDTO, PERSONA_CONFIG_V1_VERSION,
    PERSONA_CONFIG_V2_VERSION,
};
use anyhow::Result;
use orcs_core::persona::{Persona, PersonaSource};
use semver::Version;
use uuid::Uuid;

/// Migration from PersonaConfigV1 (1.0.0) to PersonaConfigV2 (2.0.0).
///
/// Changes:
/// - Convert string-based IDs (e.g., "mai", "yui") to UUID format
/// - Use name-based UUID v5 for deterministic conversion
/// - Preserve existing UUIDs if already present
///
/// The UUID conversion is deterministic: the same persona name will always
/// generate the same UUID, which is important for data consistency across
/// multiple instances.
#[derive(Debug)]
pub struct PersonaV1ToV2Migration;

impl PersonaV1ToV2Migration {
    /// Generates a deterministic UUID from a persona name.
    ///
    /// Uses UUID v5 with NAMESPACE_OID to ensure the same name always
    /// produces the same UUID.
    fn generate_uuid_from_name(name: &str) -> String {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()).to_string()
    }
}

impl Migration for PersonaV1ToV2Migration {
    fn from_version(&self) -> Version {
        Version::parse(PERSONA_CONFIG_V1_VERSION).expect("Invalid PERSONA_CONFIG_V1_VERSION")
    }

    fn to_version(&self) -> Version {
        Version::parse(PERSONA_CONFIG_V2_VERSION).expect("Invalid PERSONA_CONFIG_V2_VERSION")
    }

    fn description(&self) -> &str {
        "Convert persona IDs from strings to UUIDs (name-based v5)"
    }
}

impl TypedMigration<PersonaConfigV1, PersonaConfigV2> for PersonaV1ToV2Migration {
    fn migrate(&self, v1: PersonaConfigV1) -> Result<PersonaConfigV2> {
        // Check if ID is already a valid UUID
        let id = if Uuid::parse_str(&v1.id).is_ok() {
            tracing::debug!(
                "Persona '{}' already has UUID ID: {}",
                v1.name,
                v1.id
            );
            v1.id
        } else {
            // Not a valid UUID - generate a new one from the name
            let new_id = Self::generate_uuid_from_name(&v1.name);
            tracing::info!(
                "Migrated persona '{}' ID: '{}' -> '{}' (UUID v5)",
                v1.name,
                v1.id,
                new_id
            );
            new_id
        };

        Ok(PersonaConfigV2 {
            schema_version: self.to_version().to_string(),
            id,
            name: v1.name,
            role: v1.role,
            background: v1.background,
            communication_style: v1.communication_style,
            default_participant: v1.default_participant,
            source: v1.source,
        })
    }
}

// ============================================================================
// Conversion traits for domain model interop
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

/// Convert PersonaConfigV1 DTO to domain model.
///
/// Migrates through V2 to ensure UUID conversion.
impl From<PersonaConfigV1> for Persona {
    fn from(dto: PersonaConfigV1) -> Self {
        // Migrate V1 -> V2 -> Persona to ensure UUID conversion
        let v2_dto: PersonaConfigV2 = PersonaV1ToV2Migration.migrate(dto)
            .expect("V1→V2 migration should not fail");
        v2_dto.into()
    }
}

/// Convert domain model to PersonaConfigV1 DTO for persistence.
///
/// Note: This is only used for backward compatibility. New code should
/// use V2 for persistence.
impl From<&Persona> for PersonaConfigV1 {
    fn from(persona: &Persona) -> Self {
        PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
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

/// Convert PersonaConfigV2 DTO to domain model.
///
/// If the ID is not a valid UUID (legacy data in V2 format), converts it
/// using the same logic as V1→V2 migration.
impl From<PersonaConfigV2> for Persona {
    fn from(dto: PersonaConfigV2) -> Self {
        // Validate and fix ID if needed
        let id = if Uuid::parse_str(&dto.id).is_ok() {
            dto.id
        } else {
            // Legacy data: V2 schema but non-UUID ID
            // Convert using same logic as V1→V2
            tracing::warn!(
                "PersonaConfigV2 has non-UUID ID '{}', converting to UUID",
                dto.id
            );
            PersonaV1ToV2Migration::generate_uuid_from_name(&dto.name)
        };

        Persona {
            id,
            name: dto.name,
            role: dto.role,
            background: dto.background,
            communication_style: dto.communication_style,
            default_participant: dto.default_participant,
            source: dto.source.into(),
        }
    }
}

/// Convert domain model to PersonaConfigV2 DTO for persistence.
///
/// Always saves with the current schema version (2.0.0).
impl From<&Persona> for PersonaConfigV2 {
    fn from(persona: &Persona) -> Self {
        PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1_to_v2_with_non_uuid_id() {
        let v1 = PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: "mai".to_string(),
            name: "Mai".to_string(),
            role: "Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        let migration = PersonaV1ToV2Migration;
        let v2 = migration.migrate(v1).unwrap();

        // ID should be converted to UUID
        assert!(
            Uuid::parse_str(&v2.id).is_ok(),
            "ID should be a valid UUID, got: {}",
            v2.id
        );
        assert_ne!(v2.id, "mai", "ID should not be 'mai' anymore");
        assert_eq!(v2.schema_version, PERSONA_CONFIG_V2_VERSION);
        assert_eq!(v2.name, "Mai");
    }

    #[test]
    fn test_v1_to_v2_with_existing_uuid() {
        let existing_uuid = "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c";

        let v1 = PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: existing_uuid.to_string(),
            name: "Mai".to_string(),
            role: "Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        let migration = PersonaV1ToV2Migration;
        let v2 = migration.migrate(v1).unwrap();

        // UUID should be preserved
        assert_eq!(v2.id, existing_uuid, "UUID should be preserved");
        assert_eq!(v2.schema_version, PERSONA_CONFIG_V2_VERSION);
    }

    #[test]
    fn test_v1_to_domain_model_converts_uuid() {
        let v1 = PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: "yui".to_string(),
            name: "Yui".to_string(),
            role: "Architect".to_string(),
            background: "Background".to_string(),
            communication_style: "Professional".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        let persona: Persona = v1.into();

        // ID should be converted to UUID
        assert!(
            Uuid::parse_str(&persona.id).is_ok(),
            "ID should be a valid UUID, got: {}",
            persona.id
        );
        assert_ne!(persona.id, "yui", "ID should not be 'yui' anymore");
        assert_eq!(persona.name, "Yui");
    }

    #[test]
    fn test_deterministic_uuid_generation() {
        // Same name should always generate the same UUID
        let uuid1 = PersonaV1ToV2Migration::generate_uuid_from_name("TestPersona");
        let uuid2 = PersonaV1ToV2Migration::generate_uuid_from_name("TestPersona");

        assert_eq!(uuid1, uuid2, "UUID generation should be deterministic");
        assert!(Uuid::parse_str(&uuid1).is_ok());
    }

    #[test]
    fn test_different_names_different_uuids() {
        let uuid_mai = PersonaV1ToV2Migration::generate_uuid_from_name("Mai");
        let uuid_yui = PersonaV1ToV2Migration::generate_uuid_from_name("Yui");

        assert_ne!(uuid_mai, uuid_yui, "Different names should produce different UUIDs");
    }

    #[test]
    fn test_v2_dto_to_domain_model() {
        let v2 = PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
            name: "Mai".to_string(),
            role: "Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        let persona: Persona = v2.into();

        assert_eq!(persona.id, "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c");
        assert_eq!(persona.name, "Mai");
    }
}
