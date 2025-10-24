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
