//! Persona migration configuration using version-migrate.

use crate::dto::{PersonaConfigV1_0_0, PersonaConfigV1_1_0};
use orcs_core::persona::Persona;
use version_migrate::Migrator;

/// Creates and configures a Migrator instance for Persona entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V1.1.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `backend` field with default value
/// - V1.1.0 → Persona: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_persona_migrator();
/// let personas: Vec<Persona> = migrator.load_vec_from("persona", toml_personas)?;
/// ```
pub fn create_persona_migrator() -> Migrator {
    let mut migrator = Migrator::builder()
        .build();

    // Register migration path: V1.0.0 -> V1.1.0 -> Persona
    let persona_path = Migrator::define("persona")
        .from::<PersonaConfigV1_0_0>()
        .step::<PersonaConfigV1_1_0>()
        .into::<Persona>();

    migrator.register(persona_path)
        .expect("Failed to register persona migration path");

    migrator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_migrator_creation() {
        let _migrator = create_persona_migrator();
        // Migrator should be created successfully
    }

    #[test]
    fn test_persona_migration_v1_0_to_domain() {
        let migrator = create_persona_migrator();

        // Simulate TOML structure with version V1.0.0
        let toml_str = r#"
version = "1.0.0"
id = "test-id"
name = "Test"
role = "Tester"
background = "Test background"
communication_style = "Test style"
default_participant = true
source = "User"
"#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

        // Migrate to domain model using flat format
        let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

        if let Err(e) = &result {
            eprintln!("Migration error: {}", e);
        }
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
        let persona = result.unwrap();
        assert_eq!(persona.name, "Test");
        assert_eq!(persona.role, "Tester");
        // Backend should be default (ClaudeCli) after migration
        assert_eq!(persona.backend, orcs_core::persona::PersonaBackend::ClaudeCli);
    }

    #[test]
    fn test_persona_migration_v1_1_to_domain() {
        let migrator = create_persona_migrator();

        // Simulate TOML structure with version V1.1.0
        let toml_str = r#"
version = "1.1.0"
id = "test-id"
name = "Test"
role = "Tester"
background = "Test background"
communication_style = "Test style"
default_participant = true
source = "User"
backend = "gemini_cli"
"#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

        // Migrate to domain model using flat format
        let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

        if let Err(e) = &result {
            eprintln!("Migration error: {}", e);
        }
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
        let persona = result.unwrap();
        assert_eq!(persona.name, "Test");
        assert_eq!(persona.backend, orcs_core::persona::PersonaBackend::GeminiCli);
    }
}
