use std::path::Path;

use version_migrate::AsyncDirStorage;

/// Common trait for repositories using AsyncDirStorage.
///
/// This trait provides common functionality for repositories that use
/// version-migrate's AsyncDirStorage for persistence.
pub trait StorageRepository {
    /// The service type for path resolution
    const SERVICE_TYPE: crate::paths::ServiceType;
    
    /// The entity name used in storage operations
    const ENTITY_NAME: &'static str;

    /// Returns a reference to the underlying storage
    fn storage(&self) -> &AsyncDirStorage;

    /// Returns the base directory path
    fn base_dir(&self) -> &Path {
        self.storage().base_path()
    }
}