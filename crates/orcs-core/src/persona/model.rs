//! Persona domain model.
//!
//! Represents AI personas that participate in conversations with users.
//! Each persona has unique characteristics, roles, and communication styles.

use serde::{Deserialize, Serialize};

/// Represents the source of a persona (system-provided or user-created).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum PersonaSource {
    /// System-provided default personas
    System,
    /// User-created custom personas
    User,
}

impl Default for PersonaSource {
    fn default() -> Self {
        PersonaSource::User
    }
}

/// A persona representing an AI agent with specific characteristics and expertise.
///
/// Personas define the behavior, expertise, and communication style of AI agents
/// participating in conversations. Each persona has a unique UUID identifier.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Persona {
    /// Unique identifier (UUID format)
    pub id: String,
    /// Display name of the persona
    pub name: String,
    /// Role or title describing the persona's expertise
    pub role: String,
    /// Background description of the persona's capabilities
    pub background: String,
    /// Communication style characteristics
    pub communication_style: String,
    /// Whether this persona is included in new sessions by default
    #[serde(default)]
    pub default_participant: bool,
    /// Source of the persona (System or User)
    #[serde(default)]
    pub source: PersonaSource,
}
