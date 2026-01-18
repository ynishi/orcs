//! Slash command definitions and management.
//!
//! This module provides custom slash commands that users can define and execute
//! from the chat interface. Commands can either expand into prompts or execute
//! shell commands, with results being transparently injected into the conversation.

pub mod builtin;
pub mod model;
pub mod repository;
pub mod request;

pub use builtin::{builtin_commands, find_builtin_command, BuiltinSlashCommand};
pub use model::{ActionConfig, CommandType, SlashCommand};
pub use repository::SlashCommandRepository;
pub use request::CreateSlashCommandRequest;
