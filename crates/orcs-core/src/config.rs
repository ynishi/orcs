//! Deprecated config module.
//!
//! This module is deprecated. Use `orcs_core::persona` instead.

// Re-export from persona module for backward compatibility
#[deprecated(since = "0.2.0", note = "Use orcs_core::persona::Persona instead")]
pub use crate::persona::Persona as PersonaConfig;

#[deprecated(since = "0.2.0", note = "Use orcs_core::persona::PersonaSource instead")]
pub use crate::persona::PersonaSource;
