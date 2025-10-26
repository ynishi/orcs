//! User domain module.
//!
//! This module contains user-related services and domain models.
//!
//! # Module Structure
//!
//! - `model`: User profile domain model
//! - `service`: User service trait and implementations
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::user::{UserProfile, UserService, DefaultUserService};
//! ```

mod model;
mod service;

// Re-export public API
pub use model::UserProfile;
pub use service::{DefaultUserService, UserService};
