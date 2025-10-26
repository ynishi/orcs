# Schema Migration Guide

## Overview

This guide describes how to properly evolve DTOs and Domain models using the **`version-migrate`** crate with standardized patterns.

## 🎯 Core Principles

### 1. Always Use Standard `version-migrate` Patterns

**✅ Correct Implementation**

```rust
use version_migrate::{Versioned, MigratesTo, IntoDomain};
use serde::{Deserialize, Serialize};

/// V1.0.0: Initial schema
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct PersonaConfigV1_0_0 {
    pub id: String,
    pub name: String,
}

/// V1.1.0: Added backend field
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct PersonaConfigV1_1_0 {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub backend: PersonaBackend,
}

/// Migration: V1.0.0 → V1.1.0
impl MigratesTo<PersonaConfigV1_1_0> for PersonaConfigV1_0_0 {
    fn migrate(self) -> PersonaConfigV1_1_0 {
        PersonaConfigV1_1_0 {
            id: self.id,
            name: self.name,
            backend: Default::default(),
        }
    }
}

/// Domain conversion: V1.1.0 → Domain
impl IntoDomain<Persona> for PersonaConfigV1_1_0 {
    fn into_domain(self) -> Persona {
        Persona {
            id: self.id,
            name: self.name,
            backend: self.backend.into(),
        }
    }
}

/// Domain → DTO (for saving)
impl From<&Persona> for PersonaConfigV1_1_0 {
    fn from(persona: &Persona) -> Self {
        PersonaConfigV1_1_0 {
            id: persona.id.clone(),
            name: persona.name.clone(),
            backend: persona.backend.clone().into(),
        }
    }
}
```

### 2. Standard Migrator Setup

**✅ Correct: Use default version key**

```rust
// crates/orcs-infrastructure/src/storage/persona_migrator.rs
use version_migrate::Migrator;

pub fn create_persona_migrator() -> Migrator {
    let mut migrator = Migrator::builder()
        .build();  // No custom version_key needed!

    let persona_path = Migrator::define("persona")
        .from::<PersonaConfigV1_0_0>()
        .step::<PersonaConfigV1_1_0>()
        .into::<Persona>();

    migrator.register(persona_path)
        .expect("Failed to register persona migration path");

    migrator
}
```

**❌ Wrong: Custom version key**

```rust
// ❌ DEPRECATED
let mut migrator = Migrator::builder()
    .default_version_key("schema_version")  // Don't use this!
    .build();
```

### 3. Repository Layer Pattern

**✅ Correct: Clean separation of concerns**

```rust
// crates/orcs-infrastructure/src/toml_persona_repository.rs
use version_migrate::IntoDomain;

pub fn load_personas() -> Result<Vec<Persona>, String> {
    let migrator = create_persona_migrator();
    let config = load_config()?;

    let personas: Vec<Persona> = config.personas
        .into_iter()
        .map(|toml_value| {
            migrator.load_flat_from("persona", toml_value)
                .map_err(|e| format!("Migration failed: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(personas)
}

pub fn save_personas(personas: &[Persona]) -> Result<(), String> {
    let mut config = load_config()?;

    let persona_dtos: Vec<toml::Value> = personas
        .iter()
        .map(|p| {
            let dto: PersonaConfigV1_1_0 = p.into();
            toml::to_string(&dto)
                .and_then(|s| toml::from_str(&s))
                .map_err(|e| format!("Serialization failed: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    config.personas = persona_dtos;
    save_config(&config)
}
```

## 📋 Adding a New Entity

### Step 1: Define Domain Model

```rust
// crates/orcs-core/src/workspace.rs
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: String,
}
```

### Step 2: Define DTO with Versioned

```rust
// crates/orcs-infrastructure/src/dto/workspace.rs
use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, Versioned};

/// V1.0.0: Initial workspace schema
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct WorkspaceV1_0_0 {
    pub id: String,
    pub name: String,
    pub created_at: String,
}
```

### Step 3: Implement Domain Conversions

```rust
// Domain conversion: DTO → Domain
impl IntoDomain<Workspace> for WorkspaceV1_0_0 {
    fn into_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            created_at: self.created_at,
        }
    }
}

// Domain → DTO (for saving)
impl From<&Workspace> for WorkspaceV1_0_0 {
    fn from(workspace: &Workspace) -> Self {
        Self {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            created_at: workspace.created_at.clone(),
        }
    }
}
```

### Step 4: Create Migrator

```rust
// crates/orcs-infrastructure/src/storage/workspace_migrator.rs
use version_migrate::Migrator;

pub fn create_workspace_migrator() -> Migrator {
    let mut migrator = Migrator::builder().build();

    let workspace_path = Migrator::define("workspace")
        .from::<WorkspaceV1_0_0>()
        .into::<Workspace>();

    migrator.register(workspace_path)
        .expect("Failed to register workspace migration path");

    migrator
}
```

### Step 5: Implement Repository

```rust
// crates/orcs-infrastructure/src/toml_workspace_repository.rs
use version_migrate::IntoDomain;

pub struct TomlWorkspaceRepository {
    // ...
}

impl TomlWorkspaceRepository {
    pub fn load_workspaces(&self) -> Result<Vec<Workspace>, String> {
        let migrator = create_workspace_migrator();
        let config = self.load_config()?;

        config.workspaces
            .into_iter()
            .map(|toml_value| {
                migrator.load_flat_from("workspace", toml_value)
                    .map_err(|e| format!("Migration failed: {}", e))
            })
            .collect()
    }

    pub fn save_workspaces(&self, workspaces: &[Workspace]) -> Result<(), String> {
        let mut config = self.load_config()?;

        let workspace_dtos: Vec<toml::Value> = workspaces
            .iter()
            .map(|w| {
                let dto: WorkspaceV1_0_0 = w.into();
                toml::to_string(&dto)
                    .and_then(|s| toml::from_str(&s))
                    .map_err(|e| format!("Serialization failed: {}", e))
            })
            .collect::<Result<Vec<_>, _>>()?;

        config.workspaces = workspace_dtos;
        self.save_config(&config)
    }
}
```

### Step 6: Add to ConfigRoot

```rust
// crates/orcs-infrastructure/src/dto/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRoot {
    #[serde(rename = "persona", default)]
    pub personas: Vec<toml::Value>,

    #[serde(default)]
    pub user_profile: Option<UserProfileDTO>,

    // Add new entity
    #[serde(default)]
    pub workspaces: Vec<toml::Value>,
}
```

### Step 7: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use version_migrate::Versioned;

    #[test]
    fn test_workspace_version() {
        assert_eq!(WorkspaceV1_0_0::VERSION, "1.0.0");
    }

    #[test]
    fn test_workspace_domain_conversion() {
        let dto = WorkspaceV1_0_0 {
            version: WorkspaceV1_0_0::VERSION.to_string(),
            id: "test-id".to_string(),
            name: "Test Workspace".to_string(),
            created_at: "2025-10-26T00:00:00Z".to_string(),
        };

        let domain: Workspace = dto.into_domain();
        assert_eq!(domain.id, "test-id");
        assert_eq!(domain.name, "Test Workspace");
    }
}
```

## 📋 Updating an Existing Entity

### Decision Tree

```
What kind of change is it?
├─ Adding a new optional field
│  └─→ MINOR version bump (1.0.0 → 1.1.0)
│      └─ Add field with #[serde(default)]
│
└─ Breaking change (removal, rename, type change)
   └─→ MAJOR version bump (1.0.0 → 2.0.0)
       └─ Create new DTO struct V2
           └─ Implement MigratesTo<V2> for V1
```

### Case A: Adding a Field (Minor Bump)

**Example: Add `backend` field to Persona**

#### 1. Create New DTO Version

```rust
/// V1.1.0: Added backend field
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct PersonaConfigV1_1_0 {
    pub id: String,
    pub name: String,

    /// New field
    #[serde(default)]
    pub backend: PersonaBackend,
}
```

#### 2. Implement Migration

```rust
impl MigratesTo<PersonaConfigV1_1_0> for PersonaConfigV1_0_0 {
    fn migrate(self) -> PersonaConfigV1_1_0 {
        PersonaConfigV1_1_0 {
            version: PersonaConfigV1_1_0::default_version(),
            id: self.id,
            name: self.name,
            backend: Default::default(),
        }
    }
}
```

#### 3. Update Migrator

```rust
pub fn create_persona_migrator() -> Migrator {
    let mut migrator = Migrator::builder().build();

    let persona_path = Migrator::define("persona")
        .from::<PersonaConfigV1_0_0>()
        .step::<PersonaConfigV1_1_0>()  // Add migration step
        .into::<Persona>();

    migrator.register(persona_path)
        .expect("Failed to register persona migration path");

    migrator
}
```

#### 4. Update Domain Conversion

```rust
impl IntoDomain<Persona> for PersonaConfigV1_1_0 {
    fn into_domain(self) -> Persona {
        Persona {
            id: self.id,
            name: self.name,
            backend: self.backend.into(),
        }
    }
}

impl From<&Persona> for PersonaConfigV1_1_0 {
    fn from(persona: &Persona) -> Self {
        Self {
            id: persona.id.clone(),
            name: persona.name.clone(),
            backend: persona.backend.clone().into(),
        }
    }
}
```

#### 5. Add Tests

```rust
#[test]
fn test_persona_migration_v1_0_to_v1_1() {
    let migrator = create_persona_migrator();

    let toml_str = r#"
version = "1.0.0"
id = "test-id"
name = "Test"
"#;
    let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

    let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

    assert!(result.is_ok());
    let persona = result.unwrap();
    assert_eq!(persona.name, "Test");
    assert_eq!(persona.backend, PersonaBackend::ClaudeCli); // Default
}
```

### Case B: Breaking Change (Major Bump)

**Example: Change Session field name from `name` to `title`**

#### 1. Create New DTO Version

```rust
/// V2.0.0: Renamed 'name' to 'title'
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct SessionV2_0_0 {
    pub id: String,
    pub title: String,  // Renamed from 'name'
    pub created_at: String,
    pub updated_at: String,
}
```

#### 2. Implement Migration

```rust
impl MigratesTo<SessionV2_0_0> for SessionV1_0_0 {
    fn migrate(self) -> SessionV2_0_0 {
        SessionV2_0_0 {
            id: self.id,
            title: self.name,  // name → title
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
```

#### 3. Update Migrator

```rust
pub fn create_session_migrator() -> Migrator {
    let mut migrator = Migrator::builder().build();

    let session_path = Migrator::define("session")
        .from::<SessionV1_0_0>()
        .step::<SessionV2_0_0>()
        .into::<Session>();

    migrator.register(session_path)
        .expect("Failed to register session migration path");

    migrator
}
```

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use version_migrate::{MigratesTo, Versioned};

    #[test]
    fn test_dto_version_constant() {
        assert_eq!(PersonaConfigV1_0_0::VERSION, "1.0.0");
        assert_eq!(PersonaConfigV1_1_0::VERSION, "1.1.0");
    }

    #[test]
    fn test_migration() {
        let v1 = PersonaConfigV1_0_0 {
            version: "1.0.0".to_string(),
            id: "test".to_string(),
            name: "Test".to_string(),
        };

        let v2: PersonaConfigV1_1_0 = v1.migrate();

        assert_eq!(v2.version, "1.1.0");
        assert_eq!(v2.backend, PersonaBackend::ClaudeCli);
    }

    #[test]
    fn test_domain_conversion() {
        let dto = PersonaConfigV1_1_0 {
            version: "1.1.0".to_string(),
            id: "test".to_string(),
            name: "Test".to_string(),
            backend: PersonaBackend::GeminiCli,
        };

        let domain: Persona = dto.into_domain();

        assert_eq!(domain.id, "test");
        assert_eq!(domain.backend, PersonaBackend::GeminiCli);
    }

    #[test]
    fn test_round_trip() {
        let original = Persona {
            id: "test".to_string(),
            name: "Test".to_string(),
            backend: PersonaBackend::GeminiApi,
        };

        let dto: PersonaConfigV1_1_0 = (&original).into();
        let restored: Persona = dto.into_domain();

        assert_eq!(original.id, restored.id);
        assert_eq!(original.backend, restored.backend);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_migrator_load_old_version() {
    let migrator = create_persona_migrator();

    // Old V1.0.0 format
    let toml_str = r#"
version = "1.0.0"
id = "old-id"
name = "Old Persona"
"#;
    let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

    let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

    assert!(result.is_ok());
    let persona = result.unwrap();
    assert_eq!(persona.name, "Old Persona");
}

#[test]
fn test_migrator_load_new_version() {
    let migrator = create_persona_migrator();

    // New V1.1.0 format
    let toml_str = r#"
version = "1.1.0"
id = "new-id"
name = "New Persona"
backend = "gemini_cli"
"#;
    let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

    let result: Result<Persona, _> = migrator.load_flat_from("persona", toml_value);

    assert!(result.is_ok());
    let persona = result.unwrap();
    assert_eq!(persona.backend, PersonaBackend::GeminiCli);
}
```

## 📋 Checklist Templates

### Adding a New Entity

```markdown
## Add New Entity: [Entity Name]

### Domain Layer
- [ ] Define domain model (`crates/orcs-core/src/`)
  - [ ] Add required fields
  - [ ] Derive Debug, Clone, Serialize, Deserialize

### DTO Layer
- [ ] Create DTO (`crates/orcs-infrastructure/src/dto/`)
  - [ ] Add `#[derive(Versioned)]`
  - [ ] Add `#[versioned(version = "1.0.0")]`

  - [ ] Implement `IntoDomain<Entity>`
  - [ ] Implement `From<&Entity> for DTO`

### Migration Layer
- [ ] Create migrator (`crates/orcs-infrastructure/src/storage/`)
  - [ ] Create `create_<entity>_migrator()` function
  - [ ] Register migration path

### Repository Layer
- [ ] Implement repository (`crates/orcs-infrastructure/src/`)
  - [ ] Implement `load_<entities>()`
  - [ ] Implement `save_<entities>()`
  - [ ] Use migrator for loading

### Configuration
- [ ] Add to `ConfigRoot` with `#[serde(default)]`

### Tests
- [ ] Unit tests for DTO
- [ ] Unit tests for migration
- [ ] Unit tests for domain conversion
- [ ] Integration tests for migrator
- [ ] `cargo check` passes
- [ ] `cargo test` passes
```

### Updating an Existing Entity

```markdown
## Update Entity: [Entity Name] - [Change Description]

### Version Decision
- [ ] Determine version bump type
  - [ ] MINOR: Adding optional field (1.0.0 → 1.1.0)
  - [ ] MAJOR: Breaking change (1.0.0 → 2.0.0)

### Domain Layer
- [ ] Update domain model

### DTO Layer (Minor)
- [ ] Create new DTO version
- [ ] Add `version` field with default
- [ ] Implement `MigratesTo<V1_1> for V1_0`
- [ ] Update `IntoDomain` for new version
- [ ] Update `From<&Domain>` for new version

### DTO Layer (Major)
- [ ] Create new DTO version
- [ ] Add `version` field with default
- [ ] Implement `MigratesTo<V2> for V1`
- [ ] Implement `IntoDomain<Domain> for V2`
- [ ] Implement `From<&Domain> for V2`

### Migration Layer
- [ ] Update migrator to include new step

### Tests
- [ ] Test migration V1 → V2
- [ ] Test loading old version
- [ ] Test loading new version
- [ ] Test round-trip conversion
- [ ] `cargo check` passes
- [ ] `cargo test` passes

### Impact Analysis
- [ ] Search for old field names (`rg "old_field"`)
- [ ] Update all usages
- [ ] Update tests
```

## 🎓 Best Practices

### 1. Use Standard Migrator Pattern

```rust
// ✅ Correct: No custom configuration
pub fn create_entity_migrator() -> Migrator {
    let mut migrator = Migrator::builder().build();

    let path = Migrator::define("entity")
        .from::<EntityV1_0_0>()
        .step::<EntityV1_1_0>()
        .into::<Entity>();

    migrator.register(path).expect("Registration failed");
    migrator
}
```

### 2. Test All Migration Paths

```rust
#[test]
fn test_all_versions() {
    let migrator = create_entity_migrator();

    // Test V1.0.0 → Domain
    let v1_toml = toml::from_str("version = \"1.0.0\"\n...").unwrap();
    assert!(migrator.load_flat_from::<Entity>("entity", v1_toml).is_ok());

    // Test V1.1.0 → Domain
    let v1_1_toml = toml::from_str("version = \"1.1.0\"\n...").unwrap();
    assert!(migrator.load_flat_from::<Entity>("entity", v1_1_toml).is_ok());
}
```

### 3. Co-locate Related Code

Keep in the same file:
- DTO struct definitions
- Migration implementations (`MigratesTo`)
- Domain conversions (`IntoDomain`, `From`)

### 5. Document Version History

```rust
/// Session DTOs and migrations
///
/// Version History:
/// - **1.0.0**: Initial schema with `name` field
/// - **1.1.0**: Renamed `name` to `title`
/// - **2.0.0**: Added `workspace_id` for workspace association
```

## 🔍 Troubleshooting

### Q: Compilation error "missing field `version`"

**A**: Add the `version` field to your DTO struct:

```rust
#[derive(Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct EntityV1_0_0 {
    #[serde(default = "EntityV1_0_0::default_version")]
    pub version: String,  // Add this!
    // ... other fields
}
```

### Q: Migration fails with version mismatch

**A**: Check that:
1. TOML files use `version = "..."` (not `schema_version`)
2. Migrator includes all version steps
3. `default_version()` returns correct version string

### Q: Old TOML files won't load

**A**: Verify migration chain is complete:

```rust
let path = Migrator::define("entity")
    .from::<V1_0_0>()
    .step::<V1_1_0>()  // Don't skip steps!
    .step::<V2_0_0>()
    .into::<Domain>();
```

### Q: Need to migrate existing files

**A**: Use `sed` to update field names:

```bash
# Update all session files
find ~/.orcs/sessions -name "*.toml" -exec sed -i '' 's/^schema_version = /version = /' {} \;
```

## 📚 References

### Version-Migrate Crate
- **Location**: `version-migrate/`
- **Docs**: See crate documentation

### Real Examples
- **Persona**: `crates/orcs-infrastructure/src/dto/persona.rs`
- **Session**: `crates/orcs-infrastructure/src/dto/session.rs`
- **Migrators**: `crates/orcs-infrastructure/src/storage/*_migrator.rs`

### Architecture
- Clean Architecture: Domain → Application → Infrastructure → UI
- DTOs are in Infrastructure layer only
- Migrations are type-safe and testable
