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
    pub session_id: Option<String>,
    /// Message timestamp if this file was saved from a chat message (ISO 8601)
    pub message_timestamp: Option<String>,
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
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert UploadedFileV1_1_0 DTO to domain model.
impl IntoDomain<UploadedFile> for UploadedFileV1_1_0 {
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
        }
    }
}

/// Convert domain model to UploadedFileV1_1_0 DTO for persistence.
impl From<&UploadedFile> for UploadedFileV1_1_0 {
    fn from(uploaded_file: &UploadedFile) -> Self {
        UploadedFileV1_1_0 {
            id: uploaded_file.id.clone(),
            name: uploaded_file.name.clone(),
            path: uploaded_file.path.clone(),
            mime_type: uploaded_file.mime_type.clone(),
            size: uploaded_file.size,
            uploaded_at: uploaded_file.uploaded_at,
            session_id: uploaded_file.session_id.clone(),
            message_timestamp: uploaded_file.message_timestamp.clone(),
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for UploadedFile entities.
///
/// The migrator handles automatic schema migration from V1.0.0 to V1.1.0
/// and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `session_id` and `message_timestamp` fields with default value None
/// - V1.1.0 → UploadedFile: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_uploaded_file_migrator();
/// let file: UploadedFile = migrator.load_flat_from("uploaded_file", toml_value)?;
/// ```
pub fn create_uploaded_file_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0.0 -> V1.1.0 -> UploadedFile
    let uploaded_file_path = version_migrate::Migrator::define("uploaded_file")
        .from::<UploadedFileV1_0_0>()
        .step::<UploadedFileV1_1_0>()
        .into::<UploadedFile>();

    migrator
        .register(uploaded_file_path)
        .expect("Failed to register uploaded_file migration path");

    migrator
}
