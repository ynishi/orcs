//! Persona domain module.
//!
//! This module contains all persona-related domain models, repository interfaces,
//! and preset configurations.
//!
//! # Module Structure
//!
//! - `model`: Core persona domain models (`Persona`, `PersonaSource`, `PersonaBackend`)
//! - `repository`: Repository trait for persona persistence
//! - `preset`: Default system personas
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::persona::{Persona, PersonaSource, PersonaRepository, get_default_presets};
//! ```

mod model;
mod preset;
mod repository;
pub mod request;

// Re-export public API
pub use model::{GeminiOptions, KaibaOptions, Persona, PersonaBackend, PersonaSource};
pub use preset::get_default_presets;
pub use repository::PersonaRepository;
pub use request::CreatePersonaRequest;
