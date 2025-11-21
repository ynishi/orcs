//! UploadedFile DTOs and migrations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use orcs_core::workspace::UploadedFile;

/// Represents a file uploaded to the workspace (DTO V1.0.0).
/// This is the initial version without session tracking fields.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct UploadedFileV1_0_0 {
    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
}

/// Represents a file uploaded to the workspace (DTO V1.1.0).
/// Added session tracking fields for files saved from chat messages.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct UploadedFileV1_1_0 {
    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
    /// Session ID if this file was saved from a chat message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Message timestamp if this file was saved from a chat message (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_timestamp: Option<String>,
    /// Author of the file (user ID, persona ID, or "system")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Represents a file uploaded to the workspace (DTO V1.2.0).
/// Added is_archived for file organization.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.2.0")]
pub struct UploadedFileV1_2_0 {
    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
    /// Session ID if this file was saved from a chat message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Message timestamp if this file was saved from a chat message (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_timestamp: Option<String>,
    /// Author of the file (user ID, persona ID, or "system")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Whether this file is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
}

/// Represents a file uploaded to the workspace (DTO V1.3.0).
/// Added is_favorite for file organization.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.3.0")]
pub struct UploadedFileV1_3_0 {
    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
    /// Session ID if this file was saved from a chat message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Message timestamp if this file was saved from a chat message (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_timestamp: Option<String>,
    /// Author of the file (user ID, persona ID, or "system")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Whether this file is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Whether this file is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
}

/// Represents a file uploaded to the workspace (DTO V1.4.0).
/// Added sort_order for manual file ordering within favorites.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.4.0")]
pub struct UploadedFileV1_4_0 {
    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
    /// Session ID if this file was saved from a chat message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Message timestamp if this file was saved from a chat message (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_timestamp: Option<String>,
    /// Author of the file (user ID, persona ID, or "system")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Whether this file is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Whether this file is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from UploadedFileV1_0_0 to UploadedFileV1_1_0.
///
/// V1.0.0 files don't have session tracking, so we set those fields to None.
impl MigratesTo<UploadedFileV1_1_0> for UploadedFileV1_0_0 {
    fn migrate(self) -> UploadedFileV1_1_0 {
        UploadedFileV1_1_0 {
            id: self.id,
            name: self.name,
            path: self.path,
            mime_type: self.mime_type,
            size: self.size,
            uploaded_at: self.uploaded_at,
            session_id: None,
            message_timestamp: None,
            author: None,
        }
    }
}

/// Migration from UploadedFileV1_1_0 to UploadedFileV1_2_0.
///
/// V1.1.0 files don't have archive status, so we set is_archived to false.
impl MigratesTo<UploadedFileV1_2_0> for UploadedFileV1_1_0 {
    fn migrate(self) -> UploadedFileV1_2_0 {
        UploadedFileV1_2_0 {
            id: self.id,
            name: self.name,
            path: self.path,
            mime_type: self.mime_type,
            size: self.size,
            uploaded_at: self.uploaded_at,
            session_id: self.session_id,
            message_timestamp: self.message_timestamp,
            author: self.author,
            is_archived: false, // Existing files are not archived by default
        }
    }
}

/// Migration from UploadedFileV1_2_0 to UploadedFileV1_3_0.
///
/// V1.2.0 files don't have favorite status, so we set is_favorite to false.
impl MigratesTo<UploadedFileV1_3_0> for UploadedFileV1_2_0 {
    fn migrate(self) -> UploadedFileV1_3_0 {
        UploadedFileV1_3_0 {
            id: self.id,
            name: self.name,
            path: self.path,
            mime_type: self.mime_type,
            size: self.size,
            uploaded_at: self.uploaded_at,
            session_id: self.session_id,
            message_timestamp: self.message_timestamp,
            author: self.author,
            is_archived: self.is_archived,
            is_favorite: false, // Existing files are not favorited by default
        }
    }
}

/// Migration from UploadedFileV1_3_0 to UploadedFileV1_4_0.
///
/// V1.3.0 files don't have sort order, so we set sort_order to None.
impl MigratesTo<UploadedFileV1_4_0> for UploadedFileV1_3_0 {
    fn migrate(self) -> UploadedFileV1_4_0 {
        UploadedFileV1_4_0 {
            id: self.id,
            name: self.name,
            path: self.path,
            mime_type: self.mime_type,
            size: self.size,
            uploaded_at: self.uploaded_at,
            session_id: self.session_id,
            message_timestamp: self.message_timestamp,
            author: self.author,
            is_archived: self.is_archived,
            is_favorite: self.is_favorite,
            sort_order: None, // Existing files have no manual sort order by default
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert UploadedFileV1_4_0 DTO to domain model.
impl IntoDomain<UploadedFile> for UploadedFileV1_4_0 {
    fn into_domain(self) -> UploadedFile {
        UploadedFile {
            id: self.id,
            name: self.name,
            path: self.path,
            mime_type: self.mime_type,
            size: self.size,
            uploaded_at: self.uploaded_at,
            session_id: self.session_id,
            message_timestamp: self.message_timestamp,
            author: self.author,
            is_archived: self.is_archived,
            is_favorite: self.is_favorite,
            sort_order: self.sort_order,
        }
    }
}

/// Convert domain model to UploadedFileV1_4_0 DTO for persistence.
impl From<&UploadedFile> for UploadedFileV1_4_0 {
    fn from(uploaded_file: &UploadedFile) -> Self {
        UploadedFileV1_4_0 {
            id: uploaded_file.id.clone(),
            name: uploaded_file.name.clone(),
            path: uploaded_file.path.clone(),
            mime_type: uploaded_file.mime_type.clone(),
            size: uploaded_file.size,
            uploaded_at: uploaded_file.uploaded_at,
            session_id: uploaded_file.session_id.clone(),
            message_timestamp: uploaded_file.message_timestamp.clone(),
            author: uploaded_file.author.clone(),
            is_archived: uploaded_file.is_archived,
            is_favorite: uploaded_file.is_favorite,
            sort_order: uploaded_file.sort_order,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for UploadedFile entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V1.4.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `session_id`, `message_timestamp`, and `author` fields with default value None
/// - V1.1.0 → V1.2.0: Adds `is_archived` field with default value false
/// - V1.2.0 → V1.3.0: Adds `is_favorite` field with default value false
/// - V1.3.0 → V1.4.0: Adds `sort_order` field with default value None
/// - V1.4.0 → UploadedFile: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_uploaded_file_migrator();
/// let file: UploadedFile = migrator.load_flat_from("uploaded_file", toml_value)?;
/// ```
pub fn create_uploaded_file_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> V1.1.0 -> V1.2.0 -> V1.3.0 -> V1.4.0 -> UploadedFile
    let uploaded_file_path = version_migrate::Migrator::define("uploaded_file")
        .from::<UploadedFileV1_0_0>()
        .step::<UploadedFileV1_1_0>()
        .step::<UploadedFileV1_2_0>()
        .step::<UploadedFileV1_3_0>()
        .step::<UploadedFileV1_4_0>()
        .into::<UploadedFile>();

    migrator
        .register(uploaded_file_path)
        .expect("Failed to register uploaded_file migration path");

    migrator
}
