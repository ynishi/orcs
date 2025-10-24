//! Session entity migrations.
//!
//! This module contains all migration implementations for the Session entity.
//! Each migration handles a specific version transition and includes any
//! necessary data transformations.

use super::traits::{Migration, TypedMigration};
use crate::dto::{SessionV0, SessionV1, SESSION_V1_VERSION};
use anyhow::{Context, Result};
use orcs_core::repository::PersonaRepository;
use orcs_core::session::ConversationMessage;
use semver::Version;
use std::collections::HashMap;
use std::sync::Arc;

/// Migration from SessionV0 (1.0.0) to SessionV1 (1.1.0).
///
/// Changes:
/// - Rename `name` field to `title`
/// - Add `created_at` field (copies from `updated_at` for V0 data)
/// - Migrate persona_histories keys from string IDs to UUIDs
///
/// The persona_histories migration is critical: old sessions used persona names
/// (e.g., "mai", "yui", "Mai", "Yui") as keys, but the new schema uses UUIDs.
/// This migration resolves those names to their corresponding persona UUIDs.
pub struct SessionV0ToV1Migration {
    persona_repository: Arc<dyn PersonaRepository>,
}

impl std::fmt::Debug for SessionV0ToV1Migration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionV0ToV1Migration")
            .field("persona_repository", &"<dyn PersonaRepository>")
            .finish()
    }
}

impl SessionV0ToV1Migration {
    /// Creates a new V0→V1 migration.
    ///
    /// # Arguments
    ///
    /// * `persona_repository` - Used to resolve persona names to UUIDs
    pub fn new(persona_repository: Arc<dyn PersonaRepository>) -> Self {
        Self { persona_repository }
    }

    /// Migrates persona_histories keys from old IDs/names to UUIDs.
    ///
    /// This handles several formats:
    /// - Old string IDs: "mai", "yui" (lowercase)
    /// - Display names: "Mai", "Yui" (capitalized)
    /// - Already UUIDs: pass through unchanged
    /// - Special keys: "user" (preserved as-is)
    ///
    /// The migration uses case-insensitive matching to handle various formats
    /// that may exist in legacy data.
    fn migrate_persona_history_keys(
        &self,
        histories: HashMap<String, Vec<ConversationMessage>>,
    ) -> Result<HashMap<String, Vec<ConversationMessage>>> {
        let personas = self
            .persona_repository
            .get_all()
            .map_err(|e| anyhow::anyhow!("Failed to load personas for migration: {}", e))?;

        let mut migrated = HashMap::new();

        for (key, messages) in histories {
            // Check if key is already a valid UUID
            if uuid::Uuid::parse_str(&key).is_ok() {
                migrated.insert(key, messages);
                continue;
            }

            // Try to find persona by old ID or name (case-insensitive)
            let key_lower = key.to_lowercase();
            let matching_persona = personas.iter().find(|p| {
                // Check old ID format (mai, yui, etc.)
                let old_id_matches = p.name.to_lowercase() == key_lower;
                // Check display name (Mai, Yui)
                let name_matches = p.name == key;
                old_id_matches || name_matches
            });

            let final_key = if let Some(persona) = matching_persona {
                // Found matching persona - use its UUID
                tracing::debug!(
                    "Migrated persona_histories key: '{}' -> '{}' ({})",
                    key,
                    persona.id,
                    persona.name
                );
                persona.id.clone()
            } else {
                // Unknown key - keep as is (might be "user" or other special keys)
                tracing::debug!(
                    "Preserved non-persona persona_histories key: '{}'",
                    key
                );
                key
            };

            migrated.insert(final_key, messages);
        }

        Ok(migrated)
    }
}

impl Migration for SessionV0ToV1Migration {
    fn from_version(&self) -> Version {
        Version::parse("1.0.0").expect("Invalid version 1.0.0")
    }

    fn to_version(&self) -> Version {
        Version::parse(SESSION_V1_VERSION).expect("Invalid SESSION_V1_VERSION")
    }

    fn description(&self) -> &str {
        "Rename 'name' to 'title', add 'created_at', migrate persona_histories keys to UUIDs"
    }
}

impl TypedMigration<SessionV0, SessionV1> for SessionV0ToV1Migration {
    fn migrate(&self, v0: SessionV0) -> Result<SessionV1> {
        // Migrate persona_histories keys
        let migrated_histories = self
            .migrate_persona_history_keys(v0.persona_histories)
            .with_context(|| format!("Failed to migrate persona_histories for session {}", v0.id))?;

        // Migrate current_persona_id if it's not a UUID
        let current_persona_id = if uuid::Uuid::parse_str(&v0.current_persona_id).is_ok() {
            v0.current_persona_id
        } else {
            // Try to resolve to UUID
            let personas = self
                .persona_repository
                .get_all()
                .map_err(|e| anyhow::anyhow!("Failed to load personas: {}", e))?;

            let id_lower = v0.current_persona_id.to_lowercase();
            personas
                .iter()
                .find(|p| p.name.to_lowercase() == id_lower || p.name == v0.current_persona_id)
                .map(|p| p.id.clone())
                .unwrap_or_else(|| {
                    tracing::warn!(
                        "Could not resolve current_persona_id '{}' to UUID, keeping as-is",
                        v0.current_persona_id
                    );
                    v0.current_persona_id.clone()
                })
        };

        Ok(SessionV1 {
            schema_version: self.to_version().to_string(),
            id: v0.id,
            title: v0.name, // name → title
            created_at: Some(v0.created_at.clone()), // Add created_at field
            updated_at: v0.updated_at,
            current_persona_id,
            persona_histories: migrated_histories,
            app_mode: v0.app_mode,
        })
    }
}

// ============================================================================
// Conversion traits for domain model interop
// ============================================================================

use orcs_core::session::Session;

/// Convert SessionV1 DTO to domain model.
///
/// Handles backward compatibility across V1.x versions:
/// - V1.0.0: `created_at` is None → fallback to `updated_at`
/// - V1.1.0+: `created_at` is Some
impl From<SessionV1> for Session {
    fn from(dto: SessionV1) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::persona::{Persona, PersonaSource};
    use orcs_core::session::{AppMode, MessageRole};
    use std::sync::Mutex;

    // Mock PersonaRepository for testing
    struct MockPersonaRepository {
        personas: Mutex<Vec<Persona>>,
    }

    impl MockPersonaRepository {
        fn new(personas: Vec<Persona>) -> Self {
            Self {
                personas: Mutex::new(personas),
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

    fn create_test_personas() -> Vec<Persona> {
        vec![
            Persona {
                id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
                name: "Mai".to_string(),
                role: "Engineer".to_string(),
                background: "".to_string(),
                communication_style: "".to_string(),
                default_participant: true,
                source: PersonaSource::System,
            },
            Persona {
                id: "2a9f5c3b-1e7d-5a4f-8b2c-6d3e9f1a7b4c".to_string(),
                name: "Yui".to_string(),
                role: "Architect".to_string(),
                background: "".to_string(),
                communication_style: "".to_string(),
                default_participant: true,
                source: PersonaSource::System,
            },
        ]
    }

    #[test]
    fn test_migrate_persona_histories_lowercase_keys() {
        let personas = create_test_personas();
        let repo = Arc::new(MockPersonaRepository::new(personas));
        let migration = SessionV0ToV1Migration::new(repo);

        let mut histories = HashMap::new();
        histories.insert(
            "mai".to_string(),
            vec![ConversationMessage {
                role: MessageRole::User,
                content: "test".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
            }],
        );

        let migrated = migration.migrate_persona_history_keys(histories).unwrap();

        // "mai" should be converted to Mai's UUID
        assert!(migrated.contains_key("8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c"));
        assert!(!migrated.contains_key("mai"));
    }

    #[test]
    fn test_migrate_v0_to_v1() {
        let personas = create_test_personas();
        let repo = Arc::new(MockPersonaRepository::new(personas));
        let migration = SessionV0ToV1Migration::new(repo);

        let v0 = SessionV0 {
            schema_version: "1.0.0".to_string(),
            id: "test-session".to_string(),
            name: "Test Session".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
            current_persona_id: "mai".to_string(),
            persona_histories: HashMap::new(),
            app_mode: AppMode::Idle,
        };

        let v1 = migration.migrate(v0).unwrap();

        assert_eq!(v1.title, "Test Session"); // name → title
        assert_eq!(v1.created_at, Some("2024-01-01T00:00:00Z".to_string()));
        // current_persona_id should be converted to UUID
        assert_eq!(
            v1.current_persona_id,
            "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c"
        );
    }

    #[test]
    fn test_preserve_existing_uuid_keys() {
        let personas = create_test_personas();
        let repo = Arc::new(MockPersonaRepository::new(personas));
        let migration = SessionV0ToV1Migration::new(repo);

        let uuid_key = "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c";
        let mut histories = HashMap::new();
        histories.insert(uuid_key.to_string(), vec![]);

        let migrated = migration.migrate_persona_history_keys(histories).unwrap();

        // UUID keys should be preserved
        assert!(migrated.contains_key(uuid_key));
    }

    #[test]
    fn test_preserve_special_keys() {
        let personas = create_test_personas();
        let repo = Arc::new(MockPersonaRepository::new(personas));
        let migration = SessionV0ToV1Migration::new(repo);

        let mut histories = HashMap::new();
        histories.insert("user".to_string(), vec![]);

        let migrated = migration.migrate_persona_history_keys(histories).unwrap();

        // Special keys like "user" should be preserved
        assert!(migrated.contains_key("user"));
    }
}
