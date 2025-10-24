use orcs_interaction::personas;

#[test]
fn test_get_all_personas() {
    let personas = personas::get_all_personas();
    assert!(!personas.is_empty(), "Should have at least one persona");
}

#[test]
fn test_get_persona_by_id() {
    let mai = personas::get_persona_by_id("mai");
    assert!(mai.is_some(), "Mai persona should exist");

    let yui = personas::get_persona_by_id("yui");
    assert!(yui.is_some(), "Yui persona should exist");

    let unknown = personas::get_persona_by_id("unknown");
    assert!(unknown.is_none(), "Unknown persona should not exist");
}

#[test]
fn test_get_default_participants() {
    let default_participants = personas::get_default_participants();
    assert!(!default_participants.is_empty(), "Should have at least one default participant");
}

#[test]
fn test_persona_fields() {
    let mai = personas::get_persona_by_id("mai").expect("Mai should exist");
    assert_eq!(mai.name, "Mai");
    assert_eq!(mai.role, "World-Class UX Engineer");
    assert!(!mai.background.is_empty());
    assert!(!mai.communication_style.is_empty());
}
