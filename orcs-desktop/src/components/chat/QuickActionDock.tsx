/**
 * QuickActionDock - Quick Action icons for the floating toolbar
 *
 * Displays customizable quick action buttons that execute SlashCommands.
 * Configuration is stored per-workspace.
 */
import { useState, useEffect, useCallback } from 'react';
import {
  ActionIcon,
  Tooltip,
  Group,
  Menu,
  Select,
  Text,
  Divider,
  Stack,
  Badge,
} from '@mantine/core';
import { IconSettings, IconX } from '@tabler/icons-react';
import { notifications } from '@mantine/notifications';
import type { QuickActionConfig, QuickActionSlot } from '../../types/quick_action';
import { createDefaultQuickActionConfig, DEFAULT_SLOT_LABELS } from '../../types/quick_action';
import type { SlashCommand, CommandType } from '../../types/slash_command';

// Command type labels (same as SlashCommandList)
const COMMAND_TYPE_LABELS: Record<CommandType, string> = {
  prompt: 'PROMPT',
  shell: 'SHELL',
  task: 'TASK',
  action: 'ACTION',
};

interface QuickActionDockProps {
  /** Current workspace ID */
  workspaceId: string | null;
  /** Available slash commands to choose from */
  slashCommands: SlashCommand[];
  /** Callback when a quick action is executed */
  onExecuteCommand: (commandName: string) => Promise<void>;
}

/**
 * Get display icon for a slot (command icon or slot label)
 */
function getSlotIcon(slot: QuickActionSlot, commands: SlashCommand[]): string {
  if (slot.icon) return slot.icon;
  if (slot.commandName) {
    const cmd = commands.find((c) => c.name === slot.commandName);
    if (cmd?.icon) return cmd.icon;
  }
  return slot.slotId;
}

/**
 * Get tooltip label for a slot
 */
function getSlotTooltip(slot: QuickActionSlot, commands: SlashCommand[]): string {
  if (!slot.commandName) return `Slot ${slot.slotId} (empty)`;
  const cmd = commands.find((c) => c.name === slot.commandName);
  if (cmd) return `/${cmd.name}: ${cmd.description}`;
  return `/${slot.commandName}`;
}

export function QuickActionDock({
  workspaceId,
  slashCommands,
  onExecuteCommand,
}: QuickActionDockProps) {
  const [config, setConfig] = useState<QuickActionConfig>(createDefaultQuickActionConfig());
  const [isLoading, setIsLoading] = useState(false);
  const [configMenuOpened, setConfigMenuOpened] = useState(false);

  // Load config when workspace changes
  useEffect(() => {
    if (!workspaceId) return;

    const loadConfig = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const loadedConfig = await invoke<QuickActionConfig>('get_quick_actions', {
          workspaceId,
        });
        setConfig(loadedConfig);
      } catch (error) {
        console.error('[QuickActionDock] Failed to load config:', error);
        setConfig(createDefaultQuickActionConfig());
      }
    };

    loadConfig();
  }, [workspaceId]);

  // Handle slot command change
  const handleSlotChange = useCallback(
    async (slotId: string, commandName: string | null) => {
      if (!workspaceId) return;

      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const updatedConfig = await invoke<QuickActionConfig>('update_quick_action_slot', {
          workspaceId,
          slotId,
          commandName,
        });
        setConfig(updatedConfig);
      } catch (error) {
        console.error('[QuickActionDock] Failed to update slot:', error);
        notifications.show({
          title: 'Error',
          message: 'Failed to update quick action',
          color: 'red',
        });
      }
    },
    [workspaceId]
  );

  // Handle quick action click
  const handleQuickAction = useCallback(
    async (slot: QuickActionSlot) => {
      if (!slot.commandName) return;

      setIsLoading(true);
      try {
        await onExecuteCommand(slot.commandName);
      } finally {
        setIsLoading(false);
      }
    },
    [onExecuteCommand]
  );

  // Get configured (non-empty) slots
  const configuredSlots = config.slots.filter((s) => s.commandName);

  // Don't render if no workspace
  if (!workspaceId) return null;

  return (
    <Group gap={4}>
      {/* Settings gear icon with configuration menu */}
      <Menu
        opened={configMenuOpened}
        onChange={setConfigMenuOpened}
        position="top"
        withArrow
        closeOnItemClick={false}
        width={300}
      >
        <Menu.Target>
          <Tooltip label="Configure Quick Actions" withArrow>
            <ActionIcon
              variant="transparent"
              size="lg"
              style={{
                color: 'var(--mantine-color-gray-6)',
                borderRadius: '6px',
                transition: 'all 0.15s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              <IconSettings size={18} />
            </ActionIcon>
          </Tooltip>
        </Menu.Target>

        <Menu.Dropdown>
          <Menu.Label>Quick Action Configuration</Menu.Label>
          <Stack gap="xs" p="xs">
            {DEFAULT_SLOT_LABELS.map((slotId) => {
              const slot = config.slots.find((s) => s.slotId === slotId);
              const currentValue = slot?.commandName || null;

              return (
                <Group key={slotId} gap="xs" wrap="nowrap">
                  <Badge size="sm" variant="light" w={28}>
                    {slotId}
                  </Badge>
                  <Select
                    size="xs"
                    placeholder="Select command..."
                    data={[
                      { value: '__empty__', label: '(None)' },
                      ...slashCommands.map((cmd) => ({
                        value: cmd.name,
                        label: `${cmd.icon} /${cmd.name} [${COMMAND_TYPE_LABELS[cmd.type]}]`,
                      })),
                    ]}
                    value={currentValue || '__empty__'}
                    onChange={(value) =>
                      handleSlotChange(slotId, value === '__empty__' ? null : value)
                    }
                    style={{ flex: 1 }}
                    clearable={false}
                  />
                  {currentValue && (
                    <ActionIcon
                      size="xs"
                      variant="subtle"
                      color="gray"
                      onClick={() => handleSlotChange(slotId, null)}
                    >
                      <IconX size={12} />
                    </ActionIcon>
                  )}
                </Group>
              );
            })}
          </Stack>
        </Menu.Dropdown>
      </Menu>

      {/* Configured quick action buttons */}
      {configuredSlots.length > 0 && (
        <>
          <Divider orientation="vertical" />
          {configuredSlots.map((slot) => (
            <Tooltip key={slot.slotId} label={getSlotTooltip(slot, slashCommands)} withArrow>
              <ActionIcon
                variant="transparent"
                size="lg"
                onClick={() => handleQuickAction(slot)}
                disabled={isLoading}
                style={{
                  color: slot.color || 'var(--mantine-color-blue-6)',
                  borderRadius: '6px',
                  transition: 'all 0.15s ease',
                  fontWeight: 600,
                  fontSize: '14px',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = 'transparent';
                }}
              >
                <Text size="sm">{getSlotIcon(slot, slashCommands)}</Text>
              </ActionIcon>
            </Tooltip>
          ))}
        </>
      )}

      <Divider orientation="vertical" />
    </Group>
  );
}
