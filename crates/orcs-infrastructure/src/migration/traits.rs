//! Core traits for the migration framework.
//!
//! This module defines the fundamental abstractions for implementing
//! schema migrations in a type-safe and composable way.

use anyhow::Result;
use semver::Version;

/// Base trait for all migrations.
///
/// Provides version information and metadata about a migration step.
pub trait Migration: Send + Sync {
    /// Returns the source version this migration starts from.
    fn from_version(&self) -> Version;

    /// Returns the target version this migration produces.
    fn to_version(&self) -> Version;

    /// Checks if this migration can be applied to the given version.
    fn can_migrate(&self, version: &Version) -> bool {
        version == &self.from_version()
    }

    /// Returns a human-readable description of this migration.
    ///
    /// Used for logging and debugging purposes.
    fn description(&self) -> &str;
}

/// Typed migration that transforms data from one version to another.
///
/// This trait extends `Migration` with actual data transformation logic.
/// The `From` and `To` type parameters ensure type safety across the migration chain.
pub trait TypedMigration<From, To>: Migration + std::fmt::Debug {
    /// Executes the migration, transforming data from the source to target format.
    ///
    /// # Errors
    ///
    /// Returns an error if the migration cannot be completed successfully.
    fn migrate(&self, from: From) -> Result<To>;
}

/// A chain of migrations that can automatically upgrade data to the latest version.
///
/// Implementations should traverse all intermediate migration steps in order,
/// ensuring no migration is skipped. This guarantees data consistency and
/// makes debugging easier.
pub trait MigrationChain<T> {
    /// Migrates data from a specific version to the latest version.
    ///
    /// This method applies all necessary migrations in sequence, starting from
    /// `current_version` and proceeding through each intermediate version until
    /// reaching the latest version.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to migrate
    /// * `current_version` - The current version of the data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No migration path exists from the current version to the latest
    /// - Any migration in the chain fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let data = load_data_v1_0_0();
    /// let migrated = chain.migrate_to_latest(data, &Version::parse("1.0.0")?)?;
    /// // Data is now at the latest version (e.g., 2.0.0)
    /// ```
    fn migrate_to_latest(&self, data: T, current_version: &Version) -> Result<T>;

    /// Returns all available migration paths from a given version.
    ///
    /// For linear migration chains, this returns a single path.
    /// This method is primarily used for debugging and validation.
    fn available_paths(&self, from: &Version) -> Vec<Vec<Version>>;
}
