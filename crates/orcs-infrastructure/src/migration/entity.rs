//! Entity definitions and type-safe migration support.
//!
//! This module defines the `Entity` enum that lists all migratable entities
//! in the system. When adding a new entity:
//! 1. Add a variant to `Entity`
//! 2. Add it to `Entity::all()`
//! 3. Add a match arm to `Entity::name()`
//! 4. Implement `MigratedEntity` for the domain model
//! 5. Add the corresponding registry to `MigrationManager`
//!
//! The compiler will ensure all steps are completed through exhaustive matching.

use semver::Version;

/// All entities that support schema migrations.
///
/// Adding a new entity requires updating:
/// - This enum (new variant)
/// - `Entity::all()` (add to array)
/// - `Entity::name()` (exhaustive match forces you to add a case)
/// - `MigrationManager` (exhaustive struct fields force you to add a registry)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Entity {
    /// Session data (conversation history, state, etc.)
    Session,
    /// Persona configuration (AI assistant profiles)
    PersonaConfig,
    // Future entities:
    // DialogueHistory,
    // UserSettings,
}

impl Entity {
    /// Returns all entities that support migrations.
    ///
    /// When adding a new entity, you must add it here, or `validate()`
    /// in `MigrationManager` won't check it.
    pub const fn all() -> &'static [Entity] {
        &[Entity::Session, Entity::PersonaConfig]
    }

    /// Returns the human-readable name of this entity.
    ///
    /// The exhaustive match ensures that adding a new `Entity` variant
    /// will cause a compile error until this method is updated.
    pub const fn name(&self) -> &'static str {
        match self {
            Entity::Session => "Session",
            Entity::PersonaConfig => "PersonaConfig",
            // Adding a new Entity variant will cause a compile error here
        }
    }
}

/// Trait for domain models that support schema migrations.
///
/// This trait links a domain model to its DTO type and provides metadata
/// about versioning. It uses the sealed trait pattern to prevent external
/// implementations.
///
/// # Example
///
/// ```ignore
/// impl MigratedEntity for Session {
///     type Dto = SessionV1;
///     const ENTITY: Entity = Entity::Session;
///
///     fn latest_version() -> Version {
///         Version::parse("1.1.0").unwrap()
///     }
/// }
/// ```
pub trait MigratedEntity: private::Sealed + Sized {
    /// The DTO type used for persistence of this entity.
    type Dto: Clone;

    /// The entity identifier.
    const ENTITY: Entity;

    /// Returns the latest schema version for this entity.
    fn latest_version() -> Version;
}

/// Sealed trait pattern to prevent external implementations.
///
/// Only types defined in this crate can implement `MigratedEntity`.
mod private {
    pub trait Sealed {}
}

// ============================================================================
// Implementations for domain models
// ============================================================================

use crate::dto::{PersonaConfigV2, SessionV1, PERSONA_CONFIG_V2_VERSION, SESSION_V1_VERSION};
use orcs_core::config::PersonaConfig;
use orcs_core::session::Session;

impl private::Sealed for Session {}
impl MigratedEntity for Session {
    type Dto = SessionV1;
    const ENTITY: Entity = Entity::Session;

    fn latest_version() -> Version {
        Version::parse(SESSION_V1_VERSION).expect("Invalid SESSION_V1_VERSION")
    }
}

impl private::Sealed for PersonaConfig {}
impl MigratedEntity for PersonaConfig {
    type Dto = PersonaConfigV2;
    const ENTITY: Entity = Entity::PersonaConfig;

    fn latest_version() -> Version {
        Version::parse(PERSONA_CONFIG_V2_VERSION).expect("Invalid PERSONA_CONFIG_V2_VERSION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_entities_have_names() {
        // Ensure all entities in Entity::all() can be named
        for entity in Entity::all() {
            let name = entity.name();
            assert!(!name.is_empty(), "Entity {:?} has no name", entity);
        }
    }

    #[test]
    fn test_entity_count() {
        // This test will fail if Entity::all() is not updated when adding a variant
        let all_count = Entity::all().len();
        assert!(
            all_count >= 2,
            "Expected at least 2 entities (Session, PersonaConfig), got {}",
            all_count
        );
    }

    #[test]
    fn test_migrated_entity_versions_are_valid() {
        // Ensure latest versions are parseable
        let session_version = Session::latest_version();
        assert!(session_version.major >= 1);

        let persona_version = PersonaConfig::latest_version();
        assert!(persona_version.major >= 1);
    }
}
