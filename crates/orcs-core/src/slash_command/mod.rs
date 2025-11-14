//! Slash command definitions and management.
//!
//! This module provides custom slash commands that users can define and execute
//! from the chat interface. Commands can either expand into prompts or execute
//! shell commands, with results being transparently injected into the conversation.

pub mod model;
pub mod repository;

pub use model::{CommandType, EntityType, SlashCommand};
pub use repository::SlashCommandRepository;
