# Checklist for Schema/Domain Changes

## Overview

This is a practical guide for identifying the scope of impact and preventing omissions when changing the Domain Model or DTO Schema.

This project uses the **`version-migrate`** crate for type-safe schema versioning.

**Common Failure Patterns**:
- Committing changes to the Domain model without updating related components.
- Forgetting to update DTOs or implement necessary data migrations.
- Neglecting to update frontend type definitions and UI components.
- Encountering `undefined`/`null` errors at runtime.
- Inconsistent naming conventions across layers.

## 🎯 Core Principles

### 1. Always Use `version-migrate`

**✅ Employ a unified migration strategy for all Entities and DTOs.**

```rust
// ✅ Correct Implementation
use version_migrate::{Versioned, MigratesTo, IntoDomain};

#[derive(Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct PersonaConfigV1_0_0 {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct PersonaConfigV1_1_0 {
    pub id: String,  // UUID format
    pub name: String,
}

// Migration: V1 → V2
impl MigratesTo<PersonaConfigV1_1_0> for PersonaConfigV1_0_0 {
    fn migrate(self) -> PersonaConfigV1_1_0 {
        PersonaConfigV1_1_0 {
            id: generate_uuid_from_name(&self.name),
            name: self.name,
        }
    }
}

// Domain conversion: V2 → Domain
impl IntoDomain<Persona> for PersonaConfigV1_1_0 {
    fn into_domain(self) -> Persona {
        Persona {
            id: self.id,
            name: self.name,
            // ...
        }
    }
}
```

**❌ Strictly Prohibited: Ad-hoc Version Management**

```rust
// ❌❌❌ WRONG: Ad-hoc version management with if statements
if profile.schema_version != USER_PROFILE_VERSION {
    profile.schema_version = USER_PROFILE_VERSION.to_string();
    save_config(&config)?;
}
```

#### Why is this approach problematic?

1.  **Inconsistent Implementation**: Leads to different versioning patterns for each entity.
2.  **Scattered Logic**: Data conversion logic (`if schema_version`) spreads throughout the codebase.
3.  **Untestable**: Migrations cannot be verified with unit tests.
4.  **Unmaintainable**: The number of `if` statements grows with each new version (v1.1 → v1.2 → v1.3...).
5.  **Lacks Type Safety**: Version compatibility cannot be checked at compile time.

### 2. Handle Boundary Logic in the Infrastructure/Repository Layer

**✅ Keep the domain logic clean.**

The Repository layer is responsible for converting between DTOs (Data Transfer Objects) and Domain models.

```rust
// ✅ Correct: Convert DTO to Domain in the Repository layer
pub fn load_personas() -> Result<Vec<Persona>, String> {
    use version_migrate::IntoDomain;

    let config = load_config()?;
    let personas = config.personas.into_iter()
        .map(|dto| dto.into_domain())  // DTO → Domain
        .collect();
    Ok(personas)
}
```

**❌ Incorrect: Leaking DTOs into the Domain Layer**

```rust
// ❌ Business logic, like SessionManager, should only operate on pure Domain models.
//    It should not be aware of DTOs.
```

### 3. Config Operations: Partial vs. Full Updates

**🔑 Important: `load_config` and `save_config` operate on the entire configuration.**

To update a part of the configuration, you must follow a "read-modify-write" pattern on the whole configuration object.

```rust
// ✅ Correct Pattern: Load the whole config, update a part, and save the whole config.
pub fn save_personas(personas: &[Persona]) -> Result<(), String> {
    // 1. Load the entire config
    let mut config = load_config()?;

    // 2. Update a portion of it
    let persona_dtos: Vec<PersonaConfigV1_1_0> = personas.iter()
        .map(|p| p.into()) // Domain -> DTO
        .collect();
    config.personas = persona_dtos;

    // 3. Save the entire config
    save_config(&config)
}
```

**❌ Incorrect: Attempting to save only a partial configuration.**

```rust
// ❌ This will cause data loss for other fields (e.g., user_profile).
let persona_dtos = personas.iter().map(|p| p.into()).collect();
let config = ConfigRootV2 {
    personas: persona_dtos,
    user_profile: None,  // 😱 The existing user_profile will be erased!
};
save_config(&config)
```

## 📋 Flow for Adding a New Entity

### Step 1: Define the Domain Model

```rust
// crates/orcs-core/src/your_entity.rs
pub struct YourEntity {
    pub id: String,
    pub name: String,
    // ...
}
```

### Step 2: Define the DTO (with `Versioned` derive)

```rust
// crates/orcs-infrastructure/src/dto.rs

/// Version constant
pub const YOUR_ENTITY_V1_VERSION: &str = "1.0.0";

/// V1.0.0: Initial version
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct YourEntityV1 {
    /// The schema version of this data structure.
    #[serde(default = "default_your_entity_v1_version")]
    pub schema_version: String,

    pub id: String,
    pub name: String,
}

fn default_your_entity_v1_version() -> String {
    YOUR_ENTITY_V1_VERSION.to_string()
}
```

### Step 3: Implement Domain Conversions

```rust
// crates/orcs-infrastructure/src/dto.rs (continued)

use version_migrate::IntoDomain;
use orcs_core::your_entity::YourEntity;

/// DTO → Domain conversion
impl IntoDomain<YourEntity> for YourEntityV1 {
    fn into_domain(self) -> YourEntity {
        YourEntity {
            id: self.id,
            name: self.name,
        }
    }
}

/// Domain → DTO conversion (for saving)
impl From<&YourEntity> for YourEntityV1 {
    fn from(entity: &YourEntity) -> Self {
        Self {
            schema_version: YOUR_ENTITY_V1_VERSION.to_string(),
            id: entity.id.clone(),
            name: entity.name.clone(),
        }
    }
}
```

### Step 4: Implement the Repository

```rust
// crates/orcs-infrastructure/src/repository.rs or a dedicated file

use version_migrate::IntoDomain;

pub fn load_your_entities() -> Result<Vec<YourEntity>, String> {
    let config = load_config()?;
    let entities = config.your_entities.into_iter()
        .map(|dto| dto.into_domain())
        .collect();
    Ok(entities)
}

pub fn save_your_entities(entities: &[YourEntity]) -> Result<(), String> {
    // Load the entire config
    let mut config = load_config()?;

    // Update a portion of it
    let entity_dtos: Vec<YourEntityV1> = entities.iter()
        .map(|e| e.into())
        .collect();
    config.your_entities = entity_dtos;

    // Save the entire config
    save_config(&config)
}
```

### Step 5: Add to `ConfigRoot`

```rust
// crates/orcs-infrastructure/src/dto.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRootV2 {
    #[serde(rename = "persona")]
    pub personas: Vec<PersonaConfigV1_1_0>,

    #[serde(default)]
    pub user_profile: Option<UserProfileDTO>,

    // 🆕 Add the new entity
    #[serde(default)]  // `default` is crucial for backward compatibility
    pub your_entities: Vec<YourEntityV1>,
}
```

## 📋 Flow for Updating an Existing Entity

### Versioning Flowchart

```
What kind of change is it?
├─ Only adding a new field
│  └─→ MINOR version bump (1.0.0 → 1.1.0)
│      └─ Add `#[serde(default)] pub new_field: Option<T>` to the existing DTO.
│
└─ Removing a field, changing a type, or renaming a field
   └─→ MAJOR version bump (1.0.0 → 2.0.0)
       └─ Create a new DTO struct (V2).
           └─ Implement `MigratesTo<V2> for V1`.
```

### Case A: Adding a Field (Minor Version Bump)

**Example: Add a `background` field to `UserProfile`**

#### 1. Define New Version Constants

```rust
// crates/orcs-infrastructure/src/dto.rs

// Before
pub const USER_PROFILE_V1_0_VERSION: &str = "1.0.0";

// After
pub const USER_PROFILE_V1_1_VERSION: &str = "1.1.0";  // 🆕 Added
```

#### 2. Create a New DTO Struct

```rust
/// V1.0.0: Initial version (nickname only)
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct UserProfileV1_0 {
    #[serde(default = "default_user_profile_v1_0_version")]
    pub schema_version: String,
    pub nickname: String,
}

/// V1.1.0: Added `background` field
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct UserProfileV1_1 {
    #[serde(default = "default_user_profile_v1_1_version")]
    pub schema_version: String,
    pub nickname: String,

    /// 🆕 New field
    #[serde(default)]
    pub background: String,
}

/// Type alias for the latest version
pub type UserProfileDTO = UserProfileV1_1;
```

#### 3. Implement the Migration

```rust
use version_migrate::MigratesTo;

/// Migration from V1.0 → V1.1
impl MigratesTo<UserProfileV1_1> for UserProfileV1_0 {
    fn migrate(self) -> UserProfileV1_1 {
        UserProfileV1_1 {
            schema_version: USER_PROFILE_V1_1_VERSION.to_string(),
            nickname: self.nickname,
            background: String::new(),  // Provide a default value
        }
    }
}
```

#### 4. Implement `IntoDomain` for the Latest Version

```rust
use version_migrate::IntoDomain;

impl IntoDomain<UserProfile> for UserProfileV1_1 {
    fn into_domain(self) -> UserProfile {
        UserProfile {
            nickname: self.nickname,
            background: self.background,
        }
    }
}
```

### Case B: Breaking Change (Major Version Bump)

**Example: Change Persona `id` from `String` to `UUID` format**

#### 1. Create a New DTO Struct

```rust
pub const PERSONA_CONFIG_V2_VERSION: &str = "2.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
pub struct PersonaConfigV1_1_0 {
    #[serde(default = "default_persona_v2_version")]
    pub schema_version: String,

    pub id: String,  // Now in UUID format
    pub name: String,
    // ...
}
```

#### 2. Implement the Migration

```rust
use version_migrate::MigratesTo;

impl MigratesTo<PersonaConfigV1_1_0> for PersonaConfigV1_0_0 {
    fn migrate(self) -> PersonaConfigV1_1_0 {
        // Check if ID is already a valid UUID
        let id = if Uuid::parse_str(&self.id).is_ok() {
            self.id
        } else {
            // Generate a new UUID from the name for old formats
            generate_uuid_from_name(&self.name)
        };

        PersonaConfigV1_1_0 {
            schema_version: PERSONA_CONFIG_V2_VERSION.to_string(),
            id,
            name: self.name,
            // ...
        }
    }
}
```

#### 3. Implement `IntoDomain` for the New Version

```rust
impl IntoDomain<Persona> for PersonaConfigV1_1_0 {
    fn into_domain(self) -> Persona {
        Persona {
            id: self.id,
            name: self.name,
            // ...
        }
    }
}
```

#### 4. Handle Old Versions in the Repository

```rust
fn load_config() -> Result<ConfigRootV2, String> {
    let content = fs::read_to_string(&config_path)?;

    // Try to deserialize as V2 first
    if let Ok(root_dto) = toml::from_str::<ConfigRootV2>(&content) {
        return Ok(root_dto);
    }

    // Fallback to V1 format and migrate
    if let Ok(root_v1) = toml::from_str::<ConfigRootV1>(&content) {
        use version_migrate::MigratesTo;

        let personas = root_v1.personas.into_iter()
            .map(|v1_dto| v1_dto.migrate())  // V1 → V2
            .collect();

        return Ok(ConfigRootV2 {
            personas,
            user_profile: None, // or migrate if user_profile also changed
        });
    }

    Err("Unsupported schema version".to_string())
}
```

## 🔐 Complex Migrations in the Repository Layer

Some migrations may require external resources (like another repository). These should be handled within the repository layer.

**Example: Convert `persona_histories` keys from names to UUIDs in the `Session` entity.**

```rust
// crates/orcs-infrastructure/src/repository.rs

impl TomlSessionRepository {
    /// Migrates persona_histories keys from old IDs/names to UUIDs.
    fn migrate_persona_history_keys(
        &self,
        histories: HashMap<String, Vec<ConversationMessage>>,
    ) -> Result<HashMap<String, Vec<ConversationMessage>>> {
        let personas = self.persona_repository.get_all()
            .map_err(|e| anyhow::anyhow!("Failed to load personas: {}", e))?;

        let mut migrated = HashMap::new();

        for (key, messages) in histories {
            // If key is already a valid UUID, keep it.
            if uuid::Uuid::parse_str(&key).is_ok() {
                migrated.insert(key, messages);
                continue;
            }

            // Try to find a matching persona by name.
            let key_lower = key.to_lowercase();
            let matching_persona = personas.iter().find(|p| {
                p.name.to_lowercase() == key_lower || p.name == key
            });

            let final_key = if let Some(persona) = matching_persona {
                persona.id.clone() // Use the persona's UUID
            } else {
                key // Keep as-is (e.g., for "user" or other special keys)
            };

            migrated.insert(final_key, messages);
        }

        Ok(migrated)
    }

    fn load_session_from_path(&self, path: &Path) -> Result<Session> {
        // ... version detection logic ...

        let dto: SessionV1 = if version.major == 1 && version.minor == 0 {
            let v0: SessionV0 = toml::from_str(&toml_content)?;

            // Standard migration using MigratesTo
            let mut v1 = v0.migrate();

            // Additional complex migration
            v1.persona_histories = self.migrate_persona_history_keys(v1.persona_histories)?;
            v1.current_persona_id = self.migrate_persona_id(&v1.current_persona_id)?;

            v1
        } else {
            toml::from_str(&toml_content)?
        };

        // Convert final DTO to domain model
        Ok(dto.into_domain())
    }
}
```

## 📋 Checklist Templates

### When Adding a New Entity

```markdown
## Add New Entity: [Entity Name]

### Infrastructure Layer
- [ ] **Define Domain Model** (`crates/orcs-core/src/your_entity.rs`)
  - [ ] Define required fields.
  - [ ] Add `Derive` macros (Debug, Clone, Serialize, Deserialize).
- [ ] **Define DTO** (`crates/orcs-infrastructure/src/dto.rs`)
  - [ ] Define version constant (`YOUR_ENTITY_V1_VERSION`).
  - [ ] Add `#[derive(Versioned)]` and `#[versioned(version = "1.0.0")]`.
  - [ ] Add `schema_version` field for TOML compatibility.
  - [ ] Implement `IntoDomain<YourEntity>`.
  - [ ] Implement `From<&YourEntity> for YourEntityV1`.
- [ ] **Add to `ConfigRoot`**
  - [ ] Add the new field with `#[serde(default)]`.
- [ ] **Implement Repository** (`repository.rs` or a dedicated file)
  - [ ] Implement `load_your_entities()`.
  - [ ] Implement `save_your_entities()`.
  - [ ] Ensure the "load entire -> modify partial -> save entire" pattern is used.
- [ ] **Add Tests**
  - [ ] Test DTO → Domain conversion.
  - [ ] Test Domain → DTO conversion.
  - [ ] Test save/load round-trip.
- [ ] `cargo check` passes.
- [ ] `cargo test` passes.

### Application Layer
- [ ] Implement Manager/Service class if needed.
- [ ] Add Tauri commands (`orcs-desktop/src-tauri/src/main.rs`).

### Frontend Layer
- [ ] Add type definition (`src/types/your_entity.ts`).
- [ ] Implement React hook (`src/hooks/useYourEntities.ts`).
- [ ] Implement UI components (`src/components/your_entities/*.tsx`).
- [ ] `npx tsc --noEmit` passes.
- [ ] Verify functionality in the dev environment.
```

### When Updating an Existing Entity

```markdown
## Update Entity: [Entity Name] - [Description of Change]

### 🔢 Versioning Decision
- [ ] **Identify Change Type**
  - [ ] Field addition only → MINOR bump (e.g., 1.0.0 → 1.1.0).
  - [ ] Breaking change (removal, type change) → MAJOR bump (e.g., 1.0.0 → 2.0.0).

### Infrastructure Layer (Minor Bump)
- [ ] **Update Domain Model** (`crates/orcs-core/src/`).
  - [ ] Add the new field.
- [ ] **Create New DTO Version** (`crates/orcs-infrastructure/src/dto.rs`)
  - [ ] Add new version constant (`YOUR_ENTITY_V1_1_VERSION`).
  - [ ] Create new DTO struct (`YourEntityV1_1`).
  - [ ] Add `#[serde(default)]` to the new field.
- [ ] **Implement Migration**
  - [ ] Implement `MigratesTo<YourEntityV1_1> for YourEntityV1_0`.
  - [ ] Set an appropriate default value for the new field.
- [ ] **Implement `IntoDomain` for the Latest Version**
  - [ ] Implement `IntoDomain<YourEntity> for YourEntityV1_1`.
- [ ] **Update Repository** (if necessary).
- [ ] **Update Tests**
  - [ ] Add a test for the new migration.
  - [ ] Add a test for loading data from the old version.
- [ ] `cargo check` passes.
- [ ] `cargo test` passes.

### Infrastructure Layer (Major Bump)
- [ ] **Update Domain Model**.
- [ ] **Create New DTO Struct** (`YourEntityV2`)
  - [ ] Add new version constant (`YOUR_ENTITY_V2_VERSION`).
  - [ ] Add `#[derive(Versioned)]` and `#[versioned(version = "2.0.0")]`.
- [ ] **Implement Migration**
  - [ ] Implement `MigratesTo<YourEntityV2> for YourEntityV1`.
  - [ ] Implement complex conversion logic (e.g., UUID generation) if needed.
- [ ] **Implement `IntoDomain`**
  - [ ] Implement `IntoDomain<YourEntity> for YourEntityV2`.
- [ ] **Update Repository**
  - [ ] Update `load_config()` to handle fallback from the old version.
- [ ] **Update Tests**.
- [ ] `cargo check` passes.
- [ ] `cargo test` passes.

### Application/Frontend Layers
- [ ] Update business logic (e.g., `SessionManager`).
- [ ] Update Tauri commands (`src-tauri/src/main.rs`).
- [ ] Update TypeScript types (`src/types/*.ts`).
- [ ] Update React hooks (`src/hooks/*.ts`).
- [ ] Update UI components (`src/components/**/*.tsx`).
- [ ] `npx tsc --noEmit` passes.
- [ ] Verify functionality in the dev environment.

### Impact Analysis
- [ ] Search the codebase for the old field name (e.g., `rg "old_field_name"`).
- [ ] Review all struct instantiations.
- [ ] Check instantiations within test code.
- [ ] Double-check for any missed spots.
```

## 🎓 Best Practices

### 1. Retain the `schema_version` Field for Compatibility

While the `version-migrate` `Versioned` trait provides a `const VERSION`, we retain the `schema_version` field in our DTOs for backward compatibility with existing TOML files.

```rust
#[derive(Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct YourEntityV1 {
    /// Retained for TOML compatibility
    #[serde(default = "default_version")]
    pub schema_version: String,

    // Actual data fields
    pub id: String,
}
```

### 2. Co-locate DTOs and Conversion Logic

Keep the following in the same file (e.g., `dto.rs`):
- DTO struct definitions.
- Migration implementations (`MigratesTo`).
- Domain conversion implementations (`IntoDomain`, `From`).

This centralizes all versioning-related code.

### 3. Handle Complex Logic in the Repository Layer

Migrations that require external resources (like other repositories) should be implemented as methods within the repository layer.

### 4. Testing is Mandatory

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use version_migrate::MigratesTo;

    #[test]
    fn test_v1_to_v2_migration() {
        let v1 = YourEntityV1 {
            schema_version: "1.0.0".to_string(),
            id: "old-id".to_string(),
        };

        let v2: YourEntityV2 = v1.migrate();

        assert_eq!(v2.schema_version, "2.0.0");
        // Assert that the ID was correctly transformed, e.g., into a UUID.
    }
}
```

## 🔍 Troubleshooting

### Q: Can't load old configuration files.

**A**: Check the fallback logic in `load_config()`.

```rust
// Try to load as V2 first
if let Ok(root_v2) = toml::from_str::<ConfigRootV2>(&content) {
    return Ok(root_v2);
}

// If that fails, try to load as V1 and migrate
if let Ok(root_v1) = toml::from_str::<ConfigRootV1>(&content) {
    // Migrate V1 → V2
    // ...
}
```

### Q: Compile error during migration.

**A**: Ensure `MigratesTo` and `IntoDomain` are correctly implemented.

```rust
// Check that all required traits are implemented
impl MigratesTo<V2> for V1 { /* ... */ }
impl IntoDomain<Domain> for V2 { /* ... */ }
```

### Q: Existing data disappears after saving.

**A**: Confirm that you are using the "load entire -> modify -> save entire" pattern.

```rust
// ✅ Correct
let mut config = load_config()?;  // Load entire config
config.personas = new_personas;   // Modify a part
save_config(&config)?;            // Save entire config

// ❌ Incorrect
let config = ConfigRootV2 {
    personas: new_personas,
    user_profile: None,  // 😱 This erases existing data
};
```

## 📚 References

### `version-migrate` Crate
- **Location**: `/Users/yutakanishimura/projects/orcs/version-migrate/`
- **Integration Tests**: `version-migrate/version-migrate/tests/integration_test.rs`

### Architectural Principles
- **Clean Architecture**: Domain → Application → Infrastructure → Presentation
- **DTO Versioning**: Schema evolution is managed in the Infrastructure layer.
- **Type-Safe Migrations**: `version-migrate` ensures implementation consistency.
