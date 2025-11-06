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

pub fn is_not_found(err: version_migrate::errors::MigrationError) -> bool {
    matches!(err, version_migrate::MigrationError::EntityNotFound(_))
}

pub fn version_error_to_result<T>(err: version_migrate::errors::MigrationError) -> Result<T, OrcsError> {
    match err {
        version_migrate::MigrationError::DeserializationError(_) => OrcsError::Serialization(err),
        version_migrate::MigrationError::SerializationError(_) => OrcsError::Serialization(err),
        version_migrate::MigrationError::EntityNotFound(_) => OrcsError::Io(format!("Not found", )),
        version_migrate::MigrationError::MigrationPathNotDefined { entity, version } => todo!(),
        version_migrate::MigrationError::MigrationStepFailed { from, to, error } => todo!(),
        version_migrate::MigrationError::CircularMigrationPath { entity, path } => todo!(),
        version_migrate::MigrationError::InvalidVersionOrder { entity, from, to } => todo!(),
        version_migrate::MigrationError::IoError { operation, path, context, error } => todo!(),
        version_migrate::MigrationError::LockError { path, error } => todo!(),
        version_migrate::MigrationError::TomlParseError(_) => todo!(),
        version_migrate::MigrationError::TomlSerializeError(_) => todo!(),
        version_migrate::MigrationError::HomeDirNotFound => todo!(),
        version_migrate::MigrationError::PathResolution(_) => todo!(),
        version_migrate::MigrationError::FilenameEncoding { id, reason } => todo!(),
        _ => OrcsError::Unknown(err.to_string())
    }
}