//! Quick Action domain models.

use serde::{Deserialize, Serialize};

/// Default slot labels (A-J for 10 slots).
pub const DEFAULT_SLOT_LABELS: [&str; 10] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];

/// A single quick action slot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionSlot {
    /// Slot identifier (A, B, C, ...).
    pub slot_id: String,
    /// Name of the SlashCommand to execute (e.g., "summary", "review").
    /// None means the slot is empty.
    pub command_name: Option<String>,
    /// Custom icon override (optional, uses command icon if not set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Custom color override (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl QuickActionSlot {
    /// Creates a new empty slot.
    pub fn new_empty(slot_id: impl Into<String>) -> Self {
        Self {
            slot_id: slot_id.into(),
            command_name: None,
            icon: None,
            color: None,
        }
    }

    /// Creates a new slot with a command assigned.
    pub fn new_with_command(slot_id: impl Into<String>, command_name: impl Into<String>) -> Self {
        Self {
            slot_id: slot_id.into(),
            command_name: Some(command_name.into()),
            icon: None,
            color: None,
        }
    }
}

/// Quick Action Dock configuration for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionConfig {
    /// List of configured slots.
    pub slots: Vec<QuickActionSlot>,
    /// Maximum number of slots to display (default: 10).
    #[serde(default = "default_max_visible_slots")]
    pub max_visible_slots: usize,
}

fn default_max_visible_slots() -> usize {
    10
}

impl Default for QuickActionConfig {
    fn default() -> Self {
        Self {
            slots: DEFAULT_SLOT_LABELS
                .iter()
                .map(|&label| QuickActionSlot::new_empty(label))
                .collect(),
            max_visible_slots: default_max_visible_slots(),
        }
    }
}

impl QuickActionConfig {
    /// Creates a new default config with empty slots.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a slot by its ID.
    pub fn get_slot(&self, slot_id: &str) -> Option<&QuickActionSlot> {
        self.slots.iter().find(|s| s.slot_id == slot_id)
    }

    /// Gets a mutable slot by its ID.
    pub fn get_slot_mut(&mut self, slot_id: &str) -> Option<&mut QuickActionSlot> {
        self.slots.iter_mut().find(|s| s.slot_id == slot_id)
    }

    /// Sets a command for a slot.
    pub fn set_slot_command(&mut self, slot_id: &str, command_name: Option<String>) {
        if let Some(slot) = self.get_slot_mut(slot_id) {
            slot.command_name = command_name;
        }
    }

    /// Returns only the configured (non-empty) slots.
    pub fn configured_slots(&self) -> Vec<&QuickActionSlot> {
        self.slots
            .iter()
            .filter(|s| s.command_name.is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QuickActionConfig::default();
        assert_eq!(config.slots.len(), 10);
        assert_eq!(config.max_visible_slots, 10);
        assert!(config.slots.iter().all(|s| s.command_name.is_none()));
    }

    #[test]
    fn test_set_slot_command() {
        let mut config = QuickActionConfig::default();
        config.set_slot_command("A", Some("summary".to_string()));

        let slot = config.get_slot("A").unwrap();
        assert_eq!(slot.command_name, Some("summary".to_string()));
    }

    #[test]
    fn test_configured_slots() {
        let mut config = QuickActionConfig::default();
        config.set_slot_command("A", Some("summary".to_string()));
        config.set_slot_command("C", Some("review".to_string()));

        let configured = config.configured_slots();
        assert_eq!(configured.len(), 2);
    }
}
