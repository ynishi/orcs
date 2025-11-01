//! Session domain module.
//!
//! This module contains all session-related domain models, repository interfaces,
//! and management logic.
//!
//! # Module Structure
//!
//! - `model`: Core session domain model (`Session`)
//! - `message`: Conversation message types (`MessageRole`, `ConversationMessage`)
//! - `app_mode`: Session state types (`AppMode`, `Plan`)
//! - `user_input`: User input types (`UserInput`)
//! - `repository`: Repository trait for session persistence
//! - `manager`: Session lifecycle management (`SessionManager`)
//!
//! # Usage
//!
//! ```ignore
//! use orcs_core::session::{Session, SessionRepository, SessionManager};
//! use orcs_core::session::{MessageRole, ConversationMessage};
//! use orcs_core::session::{AppMode, Plan, UserInput};
//! ```

mod app_mode;
mod manager;
mod message;
mod model;
mod repository;
mod user_input;

// Re-export public API
pub use app_mode::{AppMode, ConversationMode, Plan};
pub use manager::{InteractionManagerTrait, SessionManager};
pub use message::{ConversationMessage, ErrorSeverity, MessageMetadata, MessageRole, SystemEventType};
pub use model::Session;
pub use repository::SessionRepository;
pub use user_input::UserInput;
