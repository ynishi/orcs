//! User domain module.
//!
//! This module contains user-related services and future user domain models.
//!
//! # Module Structure
//!
//! - `service`: User service trait and implementations
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::user::{UserService, DefaultUserService};
//! ```

mod service;

// Re-export public API
pub use service::{DefaultUserService, UserService};
