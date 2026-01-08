//! Quick Action domain models and repository traits.
//!
//! Quick Actions are workspace-specific shortcuts to SlashCommands
//! displayed in the floating toolbar above the chat input.

mod model;
mod repository;

pub use model::{DEFAULT_SLOT_LABELS, QuickActionConfig, QuickActionSlot};
pub use repository::QuickActionRepository;
