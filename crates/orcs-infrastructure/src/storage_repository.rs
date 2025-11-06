use std::path::Path;

use orcs_core::OrcsError;
use version_migrate::AsyncDirStorage;

pub trait StorageRepository {
    const SERVICE_TYPE: crate::paths::ServiceType;
    const ENTITY_NAME: &'static str;

    fn storage(&self) -> &AsyncDirStorage;

    fn base_dir(&self) -> &Path {
        self.storage().base_path()
    }
}

pub fn is_not_found(err: &version_migrate::errors::MigrationError) -> bool {
    matches!(err, version_migrate::MigrationError::EntityNotFound(_))
}