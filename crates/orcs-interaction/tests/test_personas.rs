use orcs_core::repository::PersonaRepository;
use orcs_infrastructure::TomlPersonaRepository;

#[test]
fn test_get_all_personas() {
    let repo = TomlPersonaRepository;
    let personas = repo.get_all().expect("Should load personas");
    assert!(!personas.is_empty(), "Should have at least one persona");
}

#[test]
fn test_persona_fields() {
    let repo = TomlPersonaRepository;
    let personas = repo.get_all().expect("Should load personas");

    // Find a persona (we don't know which ones exist, so just check the first one)
    if let Some(persona) = personas.first() {
        assert!(!persona.name.is_empty(), "Persona should have a name");
        assert!(!persona.role.is_empty(), "Persona should have a role");
        assert!(!persona.background.is_empty(), "Persona should have a background");
        assert!(!persona.communication_style.is_empty(), "Persona should have a communication style");
    }
}
