use orcs_core::persona::{Persona, PersonaBackend, PersonaSource};
use orcs_core::repository::PersonaRepository;
use orcs_infrastructure::TomlPersonaRepository;
use tempfile::TempDir;

#[test]
fn test_get_all_personas_empty() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let repo = TomlPersonaRepository::with_path(config_path);

    // Should return empty vec for non-existent file
    let personas = repo.get_all().expect("Should load personas");
    assert!(personas.is_empty(), "Should have no personas initially");
}

#[test]
fn test_save_and_load_personas() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let repo = TomlPersonaRepository::with_path(config_path);

    // Create test personas
    let test_personas = vec![
        Persona {
            id: "test-id-1".to_string(),
            name: "Alice".to_string(),
            role: "Developer".to_string(),
            background: "Expert in Rust".to_string(),
            communication_style: "Direct and clear".to_string(),
            default_participant: true,
            source: PersonaSource::User,
            backend: PersonaBackend::ClaudeCli,
        },
        Persona {
            id: "test-id-2".to_string(),
            name: "Bob".to_string(),
            role: "Designer".to_string(),
            background: "UI/UX specialist".to_string(),
            communication_style: "Visual and creative".to_string(),
            default_participant: false,
            source: PersonaSource::System,
            backend: PersonaBackend::GeminiCli,
        },
    ];

    // Save personas
    repo.save_all(&test_personas)
        .expect("Should save personas");

    // Load personas
    let loaded_personas = repo.get_all().expect("Should load personas");

    // Verify
    assert_eq!(loaded_personas.len(), 2, "Should load 2 personas");

    let alice = loaded_personas.iter().find(|p| p.name == "Alice").unwrap();
    assert_eq!(alice.role, "Developer");
    assert_eq!(alice.backend, PersonaBackend::ClaudeCli);

    let bob = loaded_personas.iter().find(|p| p.name == "Bob").unwrap();
    assert_eq!(bob.role, "Designer");
    assert_eq!(bob.backend, PersonaBackend::GeminiCli);
}

#[test]
fn test_persona_fields() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let repo = TomlPersonaRepository::with_path(config_path.clone());

    // Create a test persona
    let test_persona = Persona {
        id: "test-id".to_string(),
        name: "Test Person".to_string(),
        role: "Tester".to_string(),
        background: "Testing expert".to_string(),
        communication_style: "Methodical".to_string(),
        default_participant: true,
        source: PersonaSource::User,
        backend: PersonaBackend::ClaudeCli,
    };

    // Save
    repo.save_all(&[test_persona])
        .expect("Should save persona");

    // Debug: Print the saved TOML
    let toml_content = std::fs::read_to_string(&config_path).expect("Should read file");
    println!("Saved TOML:\n{}", toml_content);

    // Load and verify fields
    let personas = repo.get_all().expect("Should load personas");
    assert_eq!(personas.len(), 1);

    let persona = &personas[0];
    assert!(!persona.name.is_empty(), "Persona should have a name");
    assert!(!persona.role.is_empty(), "Persona should have a role");
    assert!(!persona.background.is_empty(), "Persona should have a background");
    assert!(!persona.communication_style.is_empty(), "Persona should have a communication style");
}
