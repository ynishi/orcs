/**
 * Quick Action Dock configuration types
 *
 * Quick Actions are workspace-specific shortcuts to SlashCommands
 * displayed in the floating toolbar above the chat input.
 */

/**
 * A single quick action slot configuration
 */
export interface QuickActionSlot {
  /** Slot identifier (A, B, C, ...) */
  slotId: string;
  /** Name of the SlashCommand to execute (e.g., "summary", "review") */
  commandName: string | null;
  /** Custom icon override (optional, uses command icon if not set) */
  icon?: string;
  /** Custom color override (optional) */
  color?: string;
}

/**
 * Quick Action Dock configuration for a workspace
 */
export interface QuickActionConfig {
  /** List of configured slots */
  slots: QuickActionSlot[];
  /** Maximum number of slots to display (default: 10) */
  maxVisibleSlots: number;
}

/**
 * Default slot labels (A-J for 10 slots)
 */
export const DEFAULT_SLOT_LABELS = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'];

/**
 * Create default quick action config with empty slots
 */
export function createDefaultQuickActionConfig(): QuickActionConfig {
  return {
    slots: DEFAULT_SLOT_LABELS.map((label) => ({
      slotId: label,
      commandName: null,
    })),
    maxVisibleSlots: 10,
  };
}
