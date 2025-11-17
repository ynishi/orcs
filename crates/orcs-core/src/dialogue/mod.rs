//! Dialogue configuration and presets module.
//!
//! This module manages dialogue configurations that control multi-agent
//! conversation behavior, including execution strategy, conversation mode,
//! and talk style.
//!
//! # Module Structure
//!
//! - `preset`: Dialogue preset models and default configurations
//! - `repository`: Repository trait for dialogue preset persistence
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::dialogue::{DialoguePreset, DialoguePresetRepository, get_default_presets};
//! ```

pub mod preset;
pub mod repository;

// Re-export public API
pub use preset::{get_default_presets, DialoguePreset, PresetSource};
pub use repository::DialoguePresetRepository;
