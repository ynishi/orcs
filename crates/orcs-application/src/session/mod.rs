//! Session application services.
//!
//! This module contains application-layer services for managing sessions,
//! including lifecycle management, metadata operations, and session updates.

mod manager;
mod metadata_service;
mod updater;

#[cfg(test)]
mod manager_test;

pub use manager::SessionManager;
pub use metadata_service::SessionMetadataService;
pub use updater::SessionUpdater;

