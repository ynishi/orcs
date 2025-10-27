//! WorkspaceMetadata DTOs and migrations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use version_migrate::{IntoDomain, Versioned};

use orcs_core::workspace::WorkspaceMetadata;

/// Workspace metadata V1.0.0 (initial version).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct WorkspaceMetadataV1_0 {
    /// Unique identifier for the workspace.
    pub id: String,
    /// Name of the workspace.
    pub name: String,
    /// Root directory path of the project.
    pub root_path: PathBuf,
    /// Last accessed timestamp (UNIX timestamp in seconds).
    pub last_accessed: i64,
    /// Whether this workspace is marked as favorite.
    #[serde(default)]
    pub is_favorite: bool,
}

/// Type alias for the latest WorkspaceMetadata version.
pub type WorkspaceMetadataDTO = WorkspaceMetadataV1_0;

impl Default for WorkspaceMetadataV1_0 {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            root_path: PathBuf::new(),
            last_accessed: 0,
            is_favorite: false,
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert WorkspaceMetadataV1_0 DTO to domain model.
impl IntoDomain<WorkspaceMetadata> for WorkspaceMetadataV1_0 {
    fn into_domain(self) -> WorkspaceMetadata {
        WorkspaceMetadata {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            last_accessed: self.last_accessed,
            is_favorite: self.is_favorite,
        }
    }
}

/// Convert domain model to WorkspaceMetadataV1_0 DTO for persistence.
impl version_migrate::FromDomain<WorkspaceMetadata> for WorkspaceMetadataV1_0 {
    fn from_domain(metadata: WorkspaceMetadata) -> Self {
        WorkspaceMetadataV1_0 {
            id: metadata.id,
            name: metadata.name,
            root_path: metadata.root_path,
            last_accessed: metadata.last_accessed,
            is_favorite: metadata.is_favorite,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for WorkspaceMetadata entities.
///
/// The migrator handles automatic schema migration and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0 → WorkspaceMetadata: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_workspace_metadata_migrator();
/// let metadata: WorkspaceMetadata = migrator.load_flat_from("workspace_metadata", toml_value)?;
/// ```
pub fn create_workspace_metadata_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0 -> WorkspaceMetadata
    let metadata_path = version_migrate::Migrator::define("workspace_metadata")
        .from::<WorkspaceMetadataV1_0>()
        .into_with_save::<WorkspaceMetadata>();

    migrator
        .register(metadata_path)
        .expect("Failed to register workspace_metadata migration path");

    migrator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_metadata_migrator_creation() {
        let _migrator = create_workspace_metadata_migrator();
        // Migrator should be created successfully
    }

    #[test]
    fn test_workspace_metadata_migration_v1_0_to_domain() {
        let migrator = create_workspace_metadata_migrator();

        // Simulate TOML structure with version V1.0
        let toml_str = r#"
version = "1.0.0"
id = "test-workspace-id"
name = "Test Workspace"
root_path = "/path/to/project"
last_accessed = 1234567890
is_favorite = true
"#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();

        // Migrate to domain model using flat format
        let result: Result<WorkspaceMetadata, _> =
            migrator.load_flat_from("workspace_metadata", toml_value);

        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
        let metadata = result.unwrap();
        assert_eq!(metadata.id, "test-workspace-id");
        assert_eq!(metadata.name, "Test Workspace");
        assert_eq!(metadata.root_path, PathBuf::from("/path/to/project"));
        assert_eq!(metadata.last_accessed, 1234567890);
        assert_eq!(metadata.is_favorite, true);
    }

    #[test]
    fn test_workspace_metadata_save() {
        let migrator = create_workspace_metadata_migrator();

        let metadata = WorkspaceMetadata {
            id: "ws-123".to_string(),
            name: "My Project".to_string(),
            root_path: PathBuf::from("/home/user/project"),
            last_accessed: 9876543210,
            is_favorite: false,
        };

        // Save domain model to JSON
        let result = migrator.save_domain_flat("workspace_metadata", &metadata);

        assert!(result.is_ok());
        let json_str = result.unwrap();

        // Should contain version field
        assert!(json_str.contains("\"version\":\"1.0.0\""));
        assert!(json_str.contains("\"id\":\"ws-123\""));
        assert!(json_str.contains("\"name\":\"My Project\""));
    }
}
