//! Quick Action DTOs and migrations
//!
//! ## Version History
//! - **1.0.0**: Initial schema with slots and max_visible_slots

use serde::{Deserialize, Serialize};
use version_migrate::{FromDomain, IntoDomain, Versioned};

use orcs_core::quick_action::{QuickActionConfig, QuickActionSlot};

// ============================================================================
// QuickActionSlot DTOs
// ============================================================================

/// Quick action slot DTO V1.0.0
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionSlotV1_0_0 {
    /// Slot identifier (A, B, C, ...)
    pub slot_id: String,
    /// Name of the SlashCommand to execute
    pub command_name: Option<String>,
    /// Custom icon override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Custom color override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl From<&QuickActionSlot> for QuickActionSlotV1_0_0 {
    fn from(slot: &QuickActionSlot) -> Self {
        QuickActionSlotV1_0_0 {
            slot_id: slot.slot_id.clone(),
            command_name: slot.command_name.clone(),
            icon: slot.icon.clone(),
            color: slot.color.clone(),
        }
    }
}

impl From<QuickActionSlotV1_0_0> for QuickActionSlot {
    fn from(dto: QuickActionSlotV1_0_0) -> Self {
        QuickActionSlot {
            slot_id: dto.slot_id,
            command_name: dto.command_name,
            icon: dto.icon,
            color: dto.color,
        }
    }
}

// ============================================================================
// QuickActionConfig DTOs
// ============================================================================

/// Quick action config DTO V1.0.0
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
#[serde(rename_all = "camelCase")]
pub struct QuickActionConfigV1_0_0 {
    /// List of configured slots
    pub slots: Vec<QuickActionSlotV1_0_0>,
    /// Maximum number of slots to display
    #[serde(default = "default_max_visible_slots")]
    pub max_visible_slots: usize,
}

fn default_max_visible_slots() -> usize {
    10
}

/// Convert QuickActionConfigV1_0_0 DTO to domain model
impl IntoDomain<QuickActionConfig> for QuickActionConfigV1_0_0 {
    fn into_domain(self) -> QuickActionConfig {
        QuickActionConfig {
            slots: self.slots.into_iter().map(|s| s.into()).collect(),
            max_visible_slots: self.max_visible_slots,
        }
    }
}

/// Convert domain model to QuickActionConfigV1_0_0 DTO for persistence
impl From<&QuickActionConfig> for QuickActionConfigV1_0_0 {
    fn from(config: &QuickActionConfig) -> Self {
        QuickActionConfigV1_0_0 {
            slots: config.slots.iter().map(|s| s.into()).collect(),
            max_visible_slots: config.max_visible_slots,
        }
    }
}

/// Convert domain model to QuickActionConfigV1_0_0 DTO (for version-migrate save support)
impl FromDomain<QuickActionConfig> for QuickActionConfigV1_0_0 {
    fn from_domain(config: QuickActionConfig) -> Self {
        QuickActionConfigV1_0_0 {
            slots: config.slots.iter().map(|s| s.into()).collect(),
            max_visible_slots: config.max_visible_slots,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates a Migrator for QuickActionConfig entities.
pub fn create_quick_action_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("quick_action" => [
        QuickActionConfigV1_0_0,
        QuickActionConfig
    ], save = true)
    .expect("Failed to create quick_action migrator")
}
