//! Migration and conversion logic for session DTOs.
//!
//! This module handles the conversion between different versions of session DTOs
//! and the domain model. It encapsulates the migration logic for evolving
//! storage schemas over time.

use orcs_core::session::Session;
use crate::dto::SessionV1;

/// Convert SessionV1 DTO to domain model.
impl From<SessionV1> for Session {
    fn from(dto: SessionV1) -> Self {
        Session {
            id: dto.id,
            title: dto.title,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            current_persona_id: dto.current_persona_id,
            persona_histories: dto.persona_histories,
            app_mode: dto.app_mode,
        }
    }
}

/// Convert domain model to SessionV1 DTO for persistence.
impl From<&Session> for SessionV1 {
    fn from(session: &Session) -> Self {
        SessionV1 {
            schema_version: "1".to_string(),
            id: session.id.clone(),
            title: session.title.clone(),
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
            current_persona_id: session.current_persona_id.clone(),
            persona_histories: session.persona_histories.clone(),
            app_mode: session.app_mode.clone(),
        }
    }
}
