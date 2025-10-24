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
impl From<PersonaConfigV1> for PersonaConfig {
    fn from(dto: PersonaConfigV1) -> Self {
        // Parse version for future migration logic
        let _version = Version::parse(&dto.schema_version)
            .unwrap_or_else(|_| Version::new(1, 0, 0));

        PersonaConfig {
            id: dto.id,
            name: dto.name,
            role: dto.role,
            background: dto.background,
            communication_style: dto.communication_style,
            default_participant: dto.default_participant,
            source: dto.source.into(),
        }
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
impl From<PersonaConfigV2> for PersonaConfig {
    fn from(dto: PersonaConfigV2) -> Self {
        PersonaConfig {
            id: dto.id,
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
