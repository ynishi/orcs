//! Central manager for all entity migrations.
//!
//! This module provides `MigrationManager`, which coordinates migrations across
//! all entities in the system. When adding a new entity:
//!
//! 1. Add a field to `MigrationManager` for the entity's registry
//! 2. Add a corresponding getter method
//! 3. Add a `with_*_registry()` method to `MigrationManagerBuilder`
//! 4. Add validation logic in `MigrationManager::validate()`
//! 5. Add the field check in `MigrationManagerBuilder::build()`
//!
//! The compiler will enforce these steps through exhaustive matching and
//! unused field warnings.

use super::entity::Entity;
use super::registry::MigrationRegistry;
use crate::dto::{PersonaConfigV2, SessionV1};
use anyhow::Result;

/// Central coordinator for all entity migrations.
///
/// This struct holds migration registries for each entity type. Adding a new
/// entity requires adding a new field here, which will cause compile errors
/// in the builder until all necessary code is updated.
///
/// # Example
///
/// ```ignore
/// let manager = MigrationManager::builder()
///     .with_session_registry(session_migrations)
///     .with_persona_registry(persona_migrations)
///     .build()?;
///
/// // Use entity-specific registries
/// let session_data = manager.session_registry().migrate_to_latest(...)?;
/// ```
#[derive(Debug)]
pub struct MigrationManager {
    session_registry: MigrationRegistry<SessionV1>,
    persona_registry: MigrationRegistry<PersonaConfigV2>,
    // Adding a new entity? Add a field here:
    // dialogue_registry: MigrationRegistry<DialogueHistoryV1>,
}

impl MigrationManager {
    /// Creates a new builder for constructing a `MigrationManager`.
    ///
    /// Use the builder pattern to ensure all entity registries are provided.
    pub fn builder() -> MigrationManagerBuilder {
        MigrationManagerBuilder::new()
    }

    /// Returns the migration registry for Session entities.
    pub fn session_registry(&self) -> &MigrationRegistry<SessionV1> {
        &self.session_registry
    }

    /// Returns the migration registry for PersonaConfig entities.
    pub fn persona_registry(&self) -> &MigrationRegistry<PersonaConfigV2> {
        &self.persona_registry
    }

    // Adding a new entity? Add a getter here:
    // pub fn dialogue_registry(&self) -> &MigrationRegistry<DialogueHistoryV1> { ... }

    /// Validates that all entity registries are properly configured.
    ///
    /// This method uses exhaustive matching on `Entity::all()` to ensure
    /// that every entity has a corresponding registry. Adding a new entity
    /// to the `Entity` enum will cause a compile error here until you add
    /// the validation logic.
    pub fn validate(&self) -> Result<()> {
        for entity in Entity::all() {
            match entity {
                Entity::Session => {
                    if self.session_registry.is_empty() {
                        tracing::warn!("Session migration registry is empty");
                    } else {
                        tracing::debug!(
                            "Session registry: {} migrations registered",
                            self.session_registry.len()
                        );
                    }
                }
                Entity::PersonaConfig => {
                    if self.persona_registry.is_empty() {
                        tracing::warn!("PersonaConfig migration registry is empty");
                    } else {
                        tracing::debug!(
                            "PersonaConfig registry: {} migrations registered",
                            self.persona_registry.len()
                        );
                    }
                }
                // Adding a new Entity variant will cause a compile error here
                // until you add a match arm for it
            }
        }

        Ok(())
    }
}

/// Builder for constructing a `MigrationManager`.
///
/// The builder ensures that all entity registries are provided before
/// allowing construction. Missing registries will cause runtime errors
/// in `build()`.
///
/// # Example
///
/// ```ignore
/// let manager = MigrationManagerBuilder::new()
///     .with_session_registry(session_registry)
///     .with_persona_registry(persona_registry)
///     .build()?;
/// ```
pub struct MigrationManagerBuilder {
    session_registry: Option<MigrationRegistry<SessionV1>>,
    persona_registry: Option<MigrationRegistry<PersonaConfigV2>>,
    // Adding a new entity? Add an Option field here:
    // dialogue_registry: Option<MigrationRegistry<DialogueHistoryV1>>,
}

impl MigrationManagerBuilder {
    /// Creates a new builder with all registries unset.
    pub fn new() -> Self {
        Self {
            session_registry: None,
            persona_registry: None,
            // New entity fields start as None
        }
    }

    /// Sets the migration registry for Session entities.
    pub fn with_session_registry(mut self, registry: MigrationRegistry<SessionV1>) -> Self {
        self.session_registry = Some(registry);
        self
    }

    /// Sets the migration registry for PersonaConfig entities.
    pub fn with_persona_registry(mut self, registry: MigrationRegistry<PersonaConfigV2>) -> Self {
        self.persona_registry = Some(registry);
        self
    }

    // Adding a new entity? Add a with_* method here:
    // pub fn with_dialogue_registry(mut self, registry: MigrationRegistry<DialogueHistoryV1>) -> Self {
    //     self.dialogue_registry = Some(registry);
    //     self
    // }

    /// Builds the `MigrationManager`, ensuring all registries are set.
    ///
    /// # Errors
    ///
    /// Returns an error if any registry is missing. This happens if you forget
    /// to call the corresponding `with_*_registry()` method.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Missing persona_registry - will error
    /// let result = MigrationManagerBuilder::new()
    ///     .with_session_registry(session_reg)
    ///     .build();
    /// assert!(result.is_err());
    /// ```
    pub fn build(self) -> Result<MigrationManager> {
        // Extract all registries, returning errors for missing ones
        let session_registry = self
            .session_registry
            .ok_or_else(|| anyhow::anyhow!("Session migration registry not set"))?;

        let persona_registry = self
            .persona_registry
            .ok_or_else(|| anyhow::anyhow!("PersonaConfig migration registry not set"))?;

        // Adding a new entity? Extract the registry here (or get unused field warning):
        // let dialogue_registry = self.dialogue_registry
        //     .ok_or_else(|| anyhow::anyhow!("DialogueHistory migration registry not set"))?;

        let manager = MigrationManager {
            session_registry,
            persona_registry,
            // dialogue_registry,
        };

        // Validate that all entities are properly configured
        manager.validate()?;

        Ok(manager)
    }
}

impl Default for MigrationManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn test_builder_requires_all_registries() {
        // Only Session registry set - should fail
        let result = MigrationManagerBuilder::new()
            .with_session_registry(MigrationRegistry::new(Version::parse("1.1.0").unwrap()))
            .build();

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("PersonaConfig"),
            "Error should mention missing PersonaConfig registry"
        );
    }

    #[test]
    fn test_builder_success_with_all_registries() {
        let result = MigrationManagerBuilder::new()
            .with_session_registry(MigrationRegistry::new(Version::parse("1.1.0").unwrap()))
            .with_persona_registry(MigrationRegistry::new(Version::parse("2.0.0").unwrap()))
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_checks_all_entities() {
        let manager = MigrationManagerBuilder::new()
            .with_session_registry(MigrationRegistry::new(Version::parse("1.1.0").unwrap()))
            .with_persona_registry(MigrationRegistry::new(Version::parse("2.0.0").unwrap()))
            .build()
            .unwrap();

        // Validation should succeed even with empty registries
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_entity_all_matches_manager_fields() {
        // This test ensures that Entity::all() and MigrationManager fields are in sync
        let all_entities = Entity::all();

        // If you add a new Entity variant but forget to add it to MigrationManager,
        // this test will remind you
        assert!(
            all_entities.len() >= 2,
            "Expected at least 2 entities in Entity::all()"
        );

        // Ensure we can create a manager (which validates exhaustive matching)
        let manager = MigrationManagerBuilder::new()
            .with_session_registry(MigrationRegistry::new(Version::parse("1.1.0").unwrap()))
            .with_persona_registry(MigrationRegistry::new(Version::parse("2.0.0").unwrap()))
            .build()
            .unwrap();

        manager.validate().unwrap();
    }
}
