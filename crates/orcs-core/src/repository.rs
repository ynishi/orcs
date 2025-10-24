//! Repository trait re-exports.
//!
//! This module provides centralized access to all repository traits
//! for backward compatibility.

// Re-export SessionRepository from session module
pub use crate::session::SessionRepository;

// Re-export PersonaRepository from persona module
pub use crate::persona::PersonaRepository;
