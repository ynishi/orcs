//! Schema migration framework for ORCS.
//!
//! This module provides a type-safe, linear migration chain system for evolving
//! data schemas over time. The framework ensures that:
//!
//! - All migrations are executed in order (no skipping)
//! - Each entity has its own migration registry
//! - Adding new entities triggers compile errors until all required code is updated
//! - Migration paths are transparent and debuggable
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    MigrationManager                          │
//! │  (Coordinates all entity migrations)                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  - Session Registry    (V0 → V1 → ...)                       │
//! │  - Persona Registry (V1 → V2 → ...)                    │
//! │  - [Future entities...]                                      │
//! └─────────────────────────────────────────────────────────────┘
//!          │                    │
//!          V                    V
//!   MigrationRegistry    MigrationRegistry
//!   (Linear chain)       (Linear chain)
//!          │                    │
//!          V                    V
//!   SessionV0ToV1         PersonaV1ToV2
//!   Migration             Migration
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use orcs_infrastructure::migration;
//!
//! // Build the migration manager
//! let manager = migration::build_migration_manager(persona_repository)?;
//!
//! // Migrate Session data
//! let session_dto = load_session_dto();
//! let current_version = Version::parse(&session_dto.schema_version)?;
//! let migrated = manager.session_registry()
//!     .migrate_to_latest(session_dto, &current_version)?;
//! ```
//!
//! # Adding a New Entity
//!
//! When adding a new entity type (e.g., `DialogueHistory`):
//!
//! 1. **Add to `Entity` enum** (`entity.rs`)
//!    - Add variant: `DialogueHistory`
//!    - Add to `Entity::all()`
//!    - Add to `Entity::name()` match
//!
//! 2. **Implement `MigratedEntity` trait**
//!    ```ignore
//!    impl MigratedEntity for DialogueHistory {
//!        type Dto = DialogueHistoryV1;
//!        const ENTITY: Entity = Entity::DialogueHistory;
//!        fn latest_version() -> Version { ... }
//!    }
//!    ```
//!
//! 3. **Create migration implementations** (`dialogue.rs`)
//!    - Implement migration structs
//!    - Implement `Migration` and `TypedMigration` traits
//!
//! 4. **Update `MigrationManager`** (`manager.rs`)
//!    - Add `dialogue_registry` field
//!    - Add `dialogue_registry()` getter
//!    - Add `with_dialogue_registry()` to builder
//!    - Add validation in `validate()` match
//!    - Add extraction in `build()`
//!
//! 5. **Update `build_migration_manager()`** (this file)
//!    - Create dialogue registry
//!    - Call `with_dialogue_registry()` in builder
//!
//! The compiler will guide you through these steps with errors.

mod entity;
mod manager;
mod persona;
mod registry;
mod session;
mod traits;

// Public API
pub use entity::{Entity, MigratedEntity};
pub use manager::{MigrationManager, MigrationManagerBuilder};
pub use registry::MigrationRegistry;
pub use traits::{Migration, MigrationChain, TypedMigration};

// Re-export specific migrations for advanced use cases
pub use persona::PersonaV1ToV2Migration;
pub use session::SessionV0ToV1Migration;

use anyhow::Result;
use orcs_core::persona::Persona;
use orcs_core::repository::PersonaRepository;
use orcs_core::session::Session;
use std::sync::Arc;

/// Builds a fully-configured `MigrationManager` with all entity registries.
///
/// This is the primary entry point for setting up migrations. It creates
/// migration registries for all supported entities and wires them together
/// into a `MigrationManager`.
///
/// # Arguments
///
/// * `persona_repository` - Required for Session migrations to resolve persona names to UUIDs
///
/// # Errors
///
/// Returns an error if:
/// - Any migration registration fails (e.g., broken chain)
/// - The `MigrationManager` validation fails
///
/// # Example
///
/// ```ignore
/// let persona_repo = Arc::new(TomlPersonaRepository);
/// let migration_manager = build_migration_manager(persona_repo)?;
///
/// // Use in repository
/// repository.set_migration_manager(migration_manager);
/// ```
pub fn build_migration_manager(
    _persona_repository: Arc<dyn PersonaRepository>,
) -> Result<MigrationManager> {
    // ========================================================================
    // Session Migrations
    // ========================================================================
    // Note: SessionV1 (1.1.0) is currently the latest version.
    // SessionV0→V1 migration is handled directly in the repository layer
    // during load, not through the registry (different DTO types).
    // When SessionV2 is introduced, add SessionV1ToV2Migration here.
    let session_registry = {
        let registry = MigrationRegistry::new(Session::latest_version());

        // Future migrations:
        // registry.register_all(vec![
        //     Arc::new(SessionV1ToV2Migration::new(...)),
        // ]);

        registry
    };

    // ========================================================================
    // Persona Migrations
    // ========================================================================
    // Note: PersonaConfigV2 (2.0.0) is currently the latest version.
    // PersonaV1→V2 migration is handled during load in the repository layer.
    // When PersonaConfigV3 is introduced, add PersonaV2ToV3Migration here.
    let persona_registry = {
        let registry = MigrationRegistry::new(Persona::latest_version());

        // Future migrations:
        // registry.register_all(vec![
        //     Arc::new(PersonaV2ToV3Migration),
        // ]);

        registry
    };

    // ========================================================================
    // Build MigrationManager
    // ========================================================================
    // The builder ensures all registries are set via compile-time checks
    MigrationManager::builder()
        .with_session_registry(session_registry)
        .with_persona_registry(persona_registry)
        // Future entities:
        // .with_dialogue_registry(dialogue_registry)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::persona::{Persona, PersonaSource};
    use std::sync::Mutex;

    // Mock PersonaRepository for testing
    struct MockPersonaRepository {
        personas: Mutex<Vec<Persona>>,
    }

    impl MockPersonaRepository {
        fn new() -> Self {
            Self {
                personas: Mutex::new(vec![
                    Persona {
                        id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
                        name: "Mai".to_string(),
                        role: "Engineer".to_string(),
                        background: "".to_string(),
                        communication_style: "".to_string(),
                        default_participant: true,
                        source: PersonaSource::System,
                    },
                ]),
            }
        }
    }

    impl PersonaRepository for MockPersonaRepository {
        fn get_all(&self) -> Result<Vec<Persona>, String> {
            Ok(self.personas.lock().unwrap().clone())
        }

        fn save_all(&self, _configs: &[Persona]) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_build_migration_manager() {
        let repo = Arc::new(MockPersonaRepository::new());
        let result = build_migration_manager(repo);

        assert!(result.is_ok(), "Failed to build migration manager");

        let _manager = result.unwrap();

        // Note: Registries may be empty if no migrations are needed yet
        // (e.g., SessionV1 and PersonaConfigV2 are already the latest versions)
        // The important thing is that the manager was created successfully
    }

    #[test]
    fn test_all_entities_have_registries() {
        let repo = Arc::new(MockPersonaRepository::new());
        let manager = build_migration_manager(repo).unwrap();

        // This test verifies that Entity::all() and MigrationManager are in sync
        // by ensuring we can access registries for all entities
        for entity in Entity::all() {
            match entity {
                Entity::Session => {
                    // Registry exists (may be empty if already at latest version)
                    let _ = manager.session_registry();
                }
                Entity::Persona => {
                    // Registry exists (may be empty if already at latest version)
                    let _ = manager.persona_registry();
                }
                // If you add a new Entity but forget to add its registry,
                // this match will fail to compile
            }
        }
    }
}
