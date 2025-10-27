use orcs_core::persona::{Persona, PersonaBackend, PersonaSource};
use orcs_core::repository::PersonaRepository;
use orcs_infrastructure::AsyncDirPersonaRepository;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread")]
async fn test_get_all_personas_empty() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let repo = AsyncDirPersonaRepository::new(temp_dir.path())
        .await
        .unwrap();

    // Should return empty vec for non-existent personas
    let personas = repo.get_all().expect("Should load personas");
    assert!(personas.is_empty(), "Should have no personas initially");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_save_and_load_personas() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let repo = AsyncDirPersonaRepository::new(temp_dir.path())
        .await
        .unwrap();

    // Create test personas
    let test_personas = vec![
        Persona {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Alice".to_string(),
            role: "Developer".to_string(),
            background: "Expert in Rust".to_string(),
            communication_style: "Direct and clear".to_string(),
            default_participant: true,
            source: PersonaSource::User,
            backend: PersonaBackend::ClaudeCli,
        },
        Persona {
            id: uuid::Uuid::new_v4().to_string(),
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

#[tokio::test(flavor = "multi_thread")]
async fn test_persona_fields() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let repo = AsyncDirPersonaRepository::new(temp_dir.path())
        .await
        .unwrap();

    // Create a test persona
    let test_persona = Persona {
        id: uuid::Uuid::new_v4().to_string(),
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

    // Load and verify fields
    let personas = repo.get_all().expect("Should load personas");
    assert_eq!(personas.len(), 1);

    let persona = &personas[0];
    assert!(!persona.name.is_empty(), "Persona should have a name");
    assert!(!persona.role.is_empty(), "Persona should have a role");
    assert!(!persona.background.is_empty(), "Persona should have a background");
    assert!(!persona.communication_style.is_empty(), "Persona should have a communication style");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_personas_stored_separately() {
    // Use temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let repo = AsyncDirPersonaRepository::new(temp_dir.path())
        .await
        .unwrap();

    // Create two personas
    let persona1 = Persona {
        id: uuid::Uuid::new_v4().to_string(),
        name: "First".to_string(),
        role: "First Role".to_string(),
        background: "First background".to_string(),
        communication_style: "First style".to_string(),
        default_participant: true,
        source: PersonaSource::User,
        backend: PersonaBackend::ClaudeCli,
    };

    let persona2 = Persona {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Second".to_string(),
        role: "Second Role".to_string(),
        background: "Second background".to_string(),
        communication_style: "Second style".to_string(),
        default_participant: false,
        source: PersonaSource::System,
        backend: PersonaBackend::GeminiCli,
    };

    // Save first persona
    repo.save_all(&[persona1.clone()])
        .expect("Should save first persona");

    // Save second persona
    repo.save_all(&[persona2.clone()])
        .expect("Should save second persona");

    // Load all personas
    let personas = repo.get_all().expect("Should load personas");
    assert_eq!(personas.len(), 2, "Should have 2 personas");

    // Verify both personas exist
    let names: Vec<String> = personas.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"First".to_string()));
    assert!(names.contains(&"Second".to_string()));
}
