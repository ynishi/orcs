//! Migration registry for managing linear migration chains.
//!
//! This module implements a simple, linear migration chain where each version
//! must migrate through all intermediate versions. This approach prioritizes
//! safety and debuggability over performance.

use super::traits::{MigrationChain, TypedMigration};
use anyhow::{Context, Result};
use semver::Version;
use std::sync::Arc;

/// Registry for managing a linear chain of migrations.
///
/// Migrations are stored in order and must form a continuous chain:
/// V1.0.0 → V1.1.0 → V2.0.0 → ...
///
/// When adding migrations via `register()`, the registry validates that each
/// new migration's `from_version()` matches the previous migration's `to_version()`.
///
/// # Example
///
/// ```ignore
/// let mut registry = MigrationRegistry::new(Version::parse("2.0.0")?);
/// registry.register(Arc::new(V1_0_to_V1_1));  // 1.0.0 → 1.1.0
/// registry.register(Arc::new(V1_1_to_V2_0));  // 1.1.0 → 2.0.0
///
/// // Automatically migrates through all steps: 1.0.0 → 1.1.0 → 2.0.0
/// let migrated = registry.migrate_to_latest(old_data, &Version::parse("1.0.0")?)?;
/// ```
#[derive(Debug)]
pub struct MigrationRegistry<T> {
    /// Migrations in order, forming a linear chain.
    migrations: Vec<Arc<dyn TypedMigration<T, T>>>,
    /// The latest version this registry can migrate to.
    latest_version: Version,
}

impl<T> MigrationRegistry<T> {
    /// Creates a new migration registry with the specified latest version.
    ///
    /// # Arguments
    ///
    /// * `latest_version` - The target version for all migrations in this registry
    pub fn new(latest_version: Version) -> Self {
        Self {
            migrations: Vec::new(),
            latest_version,
        }
    }

    /// Registers a single migration, validating chain continuity.
    ///
    /// # Panics
    ///
    /// Panics if the migration doesn't connect to the existing chain.
    /// Specifically, if there are existing migrations, the new migration's
    /// `from_version()` must equal the last migration's `to_version()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register(Arc::new(V1_0_to_V1_1));  // OK
    /// registry.register(Arc::new(V1_1_to_V2_0));  // OK (connects to previous)
    /// registry.register(Arc::new(V1_5_to_V2_0));  // PANIC (chain broken)
    /// ```
    pub fn register(&mut self, migration: Arc<dyn TypedMigration<T, T>>) {
        if let Some(last) = self.migrations.last() {
            assert_eq!(
                last.to_version(),
                migration.from_version(),
                "Migration chain broken: expected migration from {} (previous to_version), but got migration from {}. \
                 Description: '{}' (previous) -> '{}' (current)",
                last.to_version(),
                migration.from_version(),
                last.description(),
                migration.description()
            );
        }

        // Validate that the final migration reaches the latest version
        if migration.to_version() > self.latest_version {
            panic!(
                "Migration target version {} exceeds registry's latest version {}",
                migration.to_version(),
                self.latest_version
            );
        }

        self.migrations.push(migration);
    }

    /// Registers multiple migrations at once.
    ///
    /// The migrations must be provided in order and form a continuous chain.
    ///
    /// # Panics
    ///
    /// Panics if any migration breaks the chain continuity.
    pub fn register_all(&mut self, migrations: Vec<Arc<dyn TypedMigration<T, T>>>) {
        for migration in migrations {
            self.register(migration);
        }
    }

    /// Returns the starting version of the first migration, if any.
    pub fn start_version(&self) -> Option<Version> {
        self.migrations.first().map(|m| m.from_version())
    }

    /// Returns true if no migrations are registered.
    pub fn is_empty(&self) -> bool {
        self.migrations.is_empty()
    }

    /// Returns the number of registered migrations.
    pub fn len(&self) -> usize {
        self.migrations.len()
    }

    /// Finds the index of the first migration that starts from the given version.
    fn find_start_index(&self, from_version: &Version) -> Option<usize> {
        self.migrations
            .iter()
            .position(|m| &m.from_version() == from_version)
    }
}

impl<T> MigrationChain<T> for MigrationRegistry<T>
where
    T: Clone,
{
    fn migrate_to_latest(&self, mut data: T, current_version: &Version) -> Result<T> {
        // Already at the latest version
        if current_version == &self.latest_version {
            tracing::debug!(
                "Data is already at the latest version ({}), no migration needed",
                current_version
            );
            return Ok(data);
        }

        // Data is newer than the latest version (shouldn't happen in normal operation)
        if current_version > &self.latest_version {
            anyhow::bail!(
                "Data version ({}) is newer than the latest supported version ({})",
                current_version,
                self.latest_version
            );
        }

        // Find the starting point in the migration chain
        let start_idx = self.find_start_index(current_version).ok_or_else(|| {
            let available: Vec<String> = self
                .migrations
                .iter()
                .map(|m| format!("{} -> {}", m.from_version(), m.to_version()))
                .collect();
            anyhow::anyhow!(
                "No migration found starting from version {}. Available migrations: [{}]",
                current_version,
                available.join(", ")
            )
        })?;

        // Execute all migrations in sequence
        tracing::info!(
            "Starting migration from {} to {} ({} steps)",
            current_version,
            self.latest_version,
            self.migrations.len() - start_idx
        );

        for (i, migration) in self.migrations[start_idx..].iter().enumerate() {
            tracing::info!(
                "Migration step {}/{}: {} -> {} ({})",
                i + 1,
                self.migrations.len() - start_idx,
                migration.from_version(),
                migration.to_version(),
                migration.description()
            );

            data = migration
                .migrate(data)
                .with_context(|| {
                    format!(
                        "Migration failed at step {}: {} -> {}",
                        i + 1,
                        migration.from_version(),
                        migration.to_version()
                    )
                })?;
        }

        tracing::info!(
            "Migration completed successfully: {} -> {}",
            current_version,
            self.latest_version
        );

        Ok(data)
    }

    fn available_paths(&self, from: &Version) -> Vec<Vec<Version>> {
        // Linear chain: only one path exists
        if let Some(start_idx) = self.find_start_index(from) {
            let mut path = vec![from.clone()];
            for migration in &self.migrations[start_idx..] {
                path.push(migration.to_version());
            }
            vec![path]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::traits::Migration;

    // Mock migration for testing
    #[derive(Debug)]
    struct MockMigration {
        from: Version,
        to: Version,
        desc: &'static str,
    }

    impl Migration for MockMigration {
        fn from_version(&self) -> Version {
            self.from.clone()
        }

        fn to_version(&self) -> Version {
            self.to.clone()
        }

        fn description(&self) -> &str {
            self.desc
        }
    }

    impl TypedMigration<String, String> for MockMigration {
        fn migrate(&self, from: String) -> Result<String> {
            Ok(format!("{} -> {}", from, self.to))
        }
    }

    #[test]
    fn test_empty_registry() {
        let registry: MigrationRegistry<String> =
            MigrationRegistry::new(Version::parse("1.0.0").unwrap());
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_single_migration() {
        let mut registry = MigrationRegistry::new(Version::parse("1.1.0").unwrap());
        registry.register(Arc::new(MockMigration {
            from: Version::parse("1.0.0").unwrap(),
            to: Version::parse("1.1.0").unwrap(),
            desc: "Test migration",
        }));

        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry.start_version(),
            Some(Version::parse("1.0.0").unwrap())
        );
    }

    #[test]
    #[should_panic(expected = "Migration chain broken")]
    fn test_register_broken_chain() {
        let mut registry = MigrationRegistry::new(Version::parse("2.0.0").unwrap());

        // Register first migration: 1.0.0 -> 1.1.0
        registry.register(Arc::new(MockMigration {
            from: Version::parse("1.0.0").unwrap(),
            to: Version::parse("1.1.0").unwrap(),
            desc: "First",
        }));

        // Try to register non-connecting migration: 1.5.0 -> 2.0.0
        // This should panic because 1.1.0 != 1.5.0
        registry.register(Arc::new(MockMigration {
            from: Version::parse("1.5.0").unwrap(),
            to: Version::parse("2.0.0").unwrap(),
            desc: "Second (broken)",
        }));
    }

    #[test]
    fn test_migrate_through_all_steps() {
        let mut registry = MigrationRegistry::new(Version::parse("3.0.0").unwrap());

        registry.register_all(vec![
            Arc::new(MockMigration {
                from: Version::parse("1.0.0").unwrap(),
                to: Version::parse("2.0.0").unwrap(),
                desc: "V1 to V2",
            }),
            Arc::new(MockMigration {
                from: Version::parse("2.0.0").unwrap(),
                to: Version::parse("3.0.0").unwrap(),
                desc: "V2 to V3",
            }),
        ]);

        let result = registry
            .migrate_to_latest("start".to_string(), &Version::parse("1.0.0").unwrap())
            .unwrap();

        // Should pass through both migrations
        assert!(result.contains("2.0.0"));
        assert!(result.contains("3.0.0"));
    }

    #[test]
    fn test_already_at_latest_version() {
        let registry: MigrationRegistry<String> =
            MigrationRegistry::new(Version::parse("1.0.0").unwrap());

        let result = registry
            .migrate_to_latest("data".to_string(), &Version::parse("1.0.0").unwrap())
            .unwrap();

        assert_eq!(result, "data");
    }

    #[test]
    fn test_available_paths() {
        let mut registry = MigrationRegistry::new(Version::parse("2.0.0").unwrap());

        registry.register_all(vec![
            Arc::new(MockMigration {
                from: Version::parse("1.0.0").unwrap(),
                to: Version::parse("1.5.0").unwrap(),
                desc: "Step 1",
            }),
            Arc::new(MockMigration {
                from: Version::parse("1.5.0").unwrap(),
                to: Version::parse("2.0.0").unwrap(),
                desc: "Step 2",
            }),
        ]);

        let paths = registry.available_paths(&Version::parse("1.0.0").unwrap());

        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0],
            vec![
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.5.0").unwrap(),
                Version::parse("2.0.0").unwrap(),
            ]
        );
    }
}
