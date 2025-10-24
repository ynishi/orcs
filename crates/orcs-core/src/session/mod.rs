//! Session domain module.
//!
//! This module contains all session-related domain models, repository interfaces,
//! and management logic.
//!
//! # Module Structure
//!
//! - `model`: Core session domain model (`Session`)
//! - `repository`: Repository trait for session persistence
//! - `manager`: Session lifecycle management (`SessionManager`)
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::session::{Session, SessionRepository, SessionManager};
//! ```

mod manager;
mod model;
mod repository;

// Re-export public API
pub use manager::{InteractionManagerTrait, SessionManager};
pub use model::Session;
pub use repository::SessionRepository;
