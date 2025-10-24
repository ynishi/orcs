//! Migration and conversion logic for session DTOs.
//!
//! This module handles the conversion between different versions of session DTOs
//! and the domain model. It encapsulates the migration logic for evolving
//! storage schemas over time.

use orcs_core::session::Session;
use semver::Version;
use crate::dto::{SessionV1, SESSION_V1_VERSION};

/// Convert SessionV1 DTO to domain model.
///
/// Handles backward compatibility across V1.x versions:
/// - V1.0.0: `created_at` is None → fallback to `updated_at`
/// - V1.1.0+: `created_at` is Some
impl From<SessionV1> for Session {
    fn from(dto: SessionV1) -> Self {
        // Parse version for future migration logic
        let _version = Version::parse(&dto.schema_version)
            .unwrap_or_else(|_| Version::new(1, 0, 0));

        Session {
            id: dto.id,
            title: dto.title,
            // For backward compatibility: if created_at is None (V1.0.0),
            // use updated_at as a fallback
            created_at: dto.created_at.unwrap_or_else(|| dto.updated_at.clone()),
            updated_at: dto.updated_at,
            current_persona_id: dto.current_persona_id,
            persona_histories: dto.persona_histories,
            app_mode: dto.app_mode,
        }
    }
}

/// Convert domain model to SessionV1 DTO for persistence.
///
/// Always saves with the current schema version (1.1.0).
impl From<&Session> for SessionV1 {
    fn from(session: &Session) -> Self {
        SessionV1 {
            schema_version: SESSION_V1_VERSION.to_string(),
            id: session.id.clone(),
            title: session.title.clone(),
            created_at: Some(session.created_at.clone()),
            updated_at: session.updated_at.clone(),
            current_persona_id: session.current_persona_id.clone(),
            persona_histories: session.persona_histories.clone(),
            app_mode: session.app_mode.clone(),
        }
    }
}

// ============================================================================
// PersonaConfig Migrations
// ============================================================================

use orcs_core::config::{PersonaConfig, PersonaSource};
use crate::dto::{PersonaConfigV1, PersonaConfigV2, PersonaSourceDTO, PERSONA_CONFIG_V1_VERSION, PERSONA_CONFIG_V2_VERSION};
use uuid::Uuid;

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
/// Handles backward compatibility across V1.x versions.
/// Migrates through V2 to ensure UUID conversion.
impl From<PersonaConfigV1> for PersonaConfig {
    fn from(dto: PersonaConfigV1) -> Self {
        // Migrate V1 -> V2 -> PersonaConfig to ensure UUID conversion
        let v2_dto: PersonaConfigV2 = dto.into();
        v2_dto.into()
    }
}

/// Convert domain model to PersonaConfigV1 DTO for persistence.
///
/// Always saves with the current schema version (1.0.0).
impl From<&PersonaConfig> for PersonaConfigV1 {
    fn from(config: &PersonaConfig) -> Self {
        PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: config.id.clone(),
            name: config.name.clone(),
            role: config.role.clone(),
            background: config.background.clone(),
            communication_style: config.communication_style.clone(),
            default_participant: config.default_participant,
            source: config.source.clone().into(),
        }
    }
}

/// Convert PersonaConfigV1 to PersonaConfigV2 (migration).
///
/// If the ID is not a valid UUID, generates a new UUID based on the name.
impl From<PersonaConfigV1> for PersonaConfigV2 {
    fn from(v1: PersonaConfigV1) -> Self {
        // Check if ID is already a valid UUID
        let id = if Uuid::parse_str(&v1.id).is_ok() {
            v1.id
        } else {
            // Not a valid UUID - generate a new one
            // Use name-based UUID (v5) for deterministic conversion
            let namespace = Uuid::NAMESPACE_OID;
            Uuid::new_v5(&namespace, v1.name.as_bytes()).to_string()
        };

        PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id,
            name: v1.name,
            role: v1.role,
            background: v1.background,
            communication_style: v1.communication_style,
            default_participant: v1.default_participant,
            source: v1.source,
        }
    }
}

/// Convert PersonaConfigV2 DTO to domain model.
///
/// If the ID is not a valid UUID (legacy data), converts it to UUID using V1→V2 logic.
impl From<PersonaConfigV2> for PersonaConfig {
    fn from(dto: PersonaConfigV2) -> Self {
        // Validate and fix ID if needed
        let id = if Uuid::parse_str(&dto.id).is_ok() {
            dto.id
        } else {
            // Legacy data: V2 schema but non-UUID ID
            // Convert using same logic as V1→V2
            let namespace = Uuid::NAMESPACE_OID;
            Uuid::new_v5(&namespace, dto.name.as_bytes()).to_string()
        };

        PersonaConfig {
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
impl From<&PersonaConfig> for PersonaConfigV2 {
    fn from(config: &PersonaConfig) -> Self {
        PersonaConfigV2 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id: config.id.clone(),
            name: config.name.clone(),
            role: config.role.clone(),
            background: config.background.clone(),
            communication_style: config.communication_style.clone(),
            default_participant: config.default_participant,
            source: config.source.clone().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_config_v1_to_v2_migration_with_non_uuid_id() {
        // Create a V1 persona with non-UUID ID
        let v1 = PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: "mai".to_string(),
            name: "Mai".to_string(),
            role: "UX Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        // Convert to V2
        let v2: PersonaConfigV2 = v1.into();

        // ID should be converted to UUID
        assert!(Uuid::parse_str(&v2.id).is_ok(), "ID should be a valid UUID, got: {}", v2.id);
        assert_ne!(v2.id, "mai", "ID should not be 'mai' anymore");
        assert_eq!(v2.schema_version, PERSONA_CONFIG_V2_VERSION);
        assert_eq!(v2.name, "Mai");
    }

    #[test]
    fn test_persona_config_v1_to_v2_migration_with_existing_uuid() {
        let existing_uuid = "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c";

        // Create a V1 persona with UUID ID
        let v1 = PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: existing_uuid.to_string(),
            name: "Mai".to_string(),
            role: "UX Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Friendly".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        // Convert to V2
        let v2: PersonaConfigV2 = v1.into();

        // ID should remain the same
        assert_eq!(v2.id, existing_uuid, "UUID should be preserved");
        assert_eq!(v2.schema_version, PERSONA_CONFIG_V2_VERSION);
    }

    #[test]
    fn test_persona_config_v1_to_domain_model_converts_uuid() {
        // Create a V1 persona with non-UUID ID
        let v1 = PersonaConfigV1 {
            schema_version: PERSONA_CONFIG_V1_VERSION.to_string(),
            id: "yui".to_string(),
            name: "Yui".to_string(),
            role: "Engineer".to_string(),
            background: "Background".to_string(),
            communication_style: "Professional".to_string(),
            default_participant: true,
            source: PersonaSourceDTO::System,
        };

        // Convert to domain model
        let persona: PersonaConfig = v1.into();

        // ID should be converted to UUID
        assert!(Uuid::parse_str(&persona.id).is_ok(), "ID should be a valid UUID, got: {}", persona.id);
        assert_ne!(persona.id, "yui", "ID should not be 'yui' anymore");
        assert_eq!(persona.name, "Yui");
    }
}
