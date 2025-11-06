//! Repository trait re-exports.
//!
//! This module provides centralized access to all repository traits
//! for backward compatibility.

// Re-export SessionRepository from session module
pub use crate::session::SessionRepository;

// Re-export PersonaRepository from persona module
pub use crate::persona::PersonaRepository;

// Re-export TaskRepository from task module
pub use crate::task::TaskRepository;

pub use crate::slash_command::SlashCommandRepository;

pub use crate::workspace::WorkspaceRepository;
