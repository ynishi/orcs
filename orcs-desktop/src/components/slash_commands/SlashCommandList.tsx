import { useState, useEffect } from 'react';
import { Stack, ScrollArea, Group, Text, Box, ActionIcon, Tooltip, Badge, Modal, Textarea, Button } from '@mantine/core';
import { IconPlus, IconPencil, IconTrash } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { SlashCommand } from '../../types/slash_command';
import { SlashCommandEditorModal } from './SlashCommandEditorModal';

const COMMAND_TYPE_LABELS: Record<SlashCommand['type'], string> = {
  prompt: 'Prompt',
  shell: 'Shell',
  task: 'Task',
};

const COMMAND_TYPE_COLORS: Record<SlashCommand['type'], string> = {
  prompt: 'blue',
  shell: 'violet',
  task: 'orange',
};

interface SlashCommandListProps {
  onMessage?: (type: 'system' | 'error', author: string, text: string) => void;
  onCommandsUpdated?: (commands: SlashCommand[]) => void;
  onRunCommand?: (command: SlashCommand, args: string) => void | Promise<void>;
}

export function SlashCommandList({ onMessage, onCommandsUpdated, onRunCommand }: SlashCommandListProps) {
  const [commands, setCommands] = useState<SlashCommand[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingCommand, setEditingCommand] = useState<Partial<SlashCommand> | null>(null);
  const [runningCommand, setRunningCommand] = useState<SlashCommand | null>(null);
  const [runArgs, setRunArgs] = useState('');
  const [isRunning, setIsRunning] = useState(false);

  // Fetch slash commands from backend
  const fetchCommands = async () => {
    try {
      const loadedCommands = await invoke<SlashCommand[]>('list_slash_commands');
      setCommands(loadedCommands);
      onCommandsUpdated?.(loadedCommands);
    } catch (error) {
      console.error('Failed to fetch slash commands:', error);
      onMessage?.('error', 'SYSTEM', `Failed to load slash commands: ${error}`);
    }
  };

  useEffect(() => {
    fetchCommands();
    // We intentionally ignore exhaustive-deps to avoid recreating fetchCommands
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [onCommandsUpdated]);

  // Handler to open modal for creating or editing
  const handleOpenModal = (command?: SlashCommand) => {
    setEditingCommand(command || { type: 'prompt' });
    setIsModalOpen(true);
  };

  // Handler to close modal
  const handleCloseModal = () => {
    setIsModalOpen(false);
    setEditingCommand(null);
  };

  // Handler for saving a command (create or update)
  const handleSaveCommand = async (commandToSave: SlashCommand) => {
    try {
      await invoke('save_slash_command', { command: commandToSave });
      await fetchCommands();
      handleCloseModal();

      const action = editingCommand?.name ? 'Updated' : 'Created';
      onMessage?.('system', 'SYSTEM', `${action} slash command: /${commandToSave.name}`);
    } catch (error) {
      console.error('Failed to save slash command:', error);
      onMessage?.('error', 'SYSTEM', `Failed to save command: ${error}`);
    }
  };

  // Handler for deleting a command
  const handleDeleteCommand = async (name: string) => {
    if (!window.confirm(`Are you sure you want to delete the command "/${name}"?`)) {
      return;
    }

    try {
      await invoke('remove_slash_command', { name });
      await fetchCommands();
      onMessage?.('system', 'SYSTEM', `Deleted slash command: /${name}`);
    } catch (error) {
      console.error('Failed to delete slash command:', error);
      onMessage?.('error', 'SYSTEM', `Failed to delete command: ${error}`);
    }
  };

  const handleRunCommand = (command: SlashCommand) => {
    setRunningCommand(command);
    setRunArgs('');
    setIsRunning(false);
  };

  const handleCloseRunModal = () => {
    if (isRunning) {
      return;
    }
    setRunningCommand(null);
    setRunArgs('');
  };

  const handleConfirmRun = async () => {
    if (!runningCommand) {
      return;
    }

    if (!onRunCommand) {
      setRunningCommand(null);
      setRunArgs('');
      return;
    }

    // Close modal immediately after starting execution
    const commandToRun = runningCommand;
    const argsToRun = runArgs;
    setRunningCommand(null);
    setRunArgs('');
    setIsRunning(false);

    // Execute command in background (tab will show progress)
    try {
      await onRunCommand(commandToRun, argsToRun);
    } catch (error) {
      console.error('Failed to run slash command:', error);
      onMessage?.('error', 'SYSTEM', `Failed to run command: ${error}`);
    }
  };

  const renderCommand = (command: SlashCommand) => {
    return (
      <Group
        key={command.name}
        gap="sm"
        wrap="nowrap"
        p="xs"
        style={{
          borderRadius: '8px',
          backgroundColor: 'transparent',
          transition: 'background-color 0.15s ease',
          cursor: 'pointer',
        }}
        onClick={() => handleRunCommand(command)}
      >
        {/* Command icon */}
        <Text size="lg">{command.icon}</Text>

        {/* Command info */}
        <Box style={{ flex: 1, minWidth: 0 }}>
          <Group gap="xs" mb={4}>
            <Text size="sm" fw={600} truncate>
              /{command.name}
            </Text>
            <Badge size="xs" color={COMMAND_TYPE_COLORS[command.type]}>
              {COMMAND_TYPE_LABELS[command.type]}
            </Badge>
          </Group>
          <Text size="xs" c="dimmed" truncate>
            {command.description}
          </Text>
        </Box>

        {/* Action buttons */}
        <Group gap={4}>
          <Tooltip label="Edit" withArrow>
            <ActionIcon
              variant="subtle"
              color="blue"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                handleOpenModal(command);
              }}
            >
              <IconPencil size={14} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Delete" withArrow>
            <ActionIcon
              variant="subtle"
              color="red"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                handleDeleteCommand(command.name);
              }}
            >
              <IconTrash size={14} />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>
    );
  };

  const requiresArgs = runningCommand
    ? runningCommand.content.includes('{args}') || runningCommand.workingDir?.includes('{args}')
    : false;

  return (
    <Stack gap="md" h="100%">
      {/* Header */}
      <Stack gap="xs" px="md" pt="md">
        <Group justify="space-between">
          <Group gap="xs">
            <Text size="lg" fw={700}>
              Slash Commands
            </Text>
            <Tooltip label="Add command" withArrow>
              <ActionIcon variant="subtle" color="blue" onClick={() => handleOpenModal()}>
                <IconPlus size={16} />
              </ActionIcon>
            </Tooltip>
          </Group>
          <Text size="sm" c="dimmed">
            {commands.length} commands
          </Text>
        </Group>
      </Stack>

      {/* Command list */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {commands.length > 0 && (
            <Box>
              <Stack gap={4}>
                {commands.map(renderCommand)}
              </Stack>
            </Box>
          )}

          {/* Empty state */}
          {commands.length === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                No custom commands yet
              </Text>
              <Text size="xs" c="dimmed" mt="xs">
                Click the + button to create your first slash command
              </Text>
            </Box>
          )}
        </Stack>
      </ScrollArea>

      {/* Footer */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          Custom commands execute transparently in chat
        </Text>
      </Box>

      {/* Modal */}
      <SlashCommandEditorModal
        opened={isModalOpen}
        onClose={handleCloseModal}
        command={editingCommand}
        onSave={handleSaveCommand}
      />

      <Modal
        opened={!!runningCommand}
        onClose={handleCloseRunModal}
        title={runningCommand ? `Run /${runningCommand.name}` : 'Run Command'}
        centered
      >
        <Stack gap="md">
          {runningCommand && (
            <>
              <Text size="sm" c="dimmed">
                {runningCommand.description}
              </Text>
              {runningCommand.argsDescription && (
                <Text size="sm" c="dimmed">
                  Args: {runningCommand.argsDescription}
                </Text>
              )}
              <Textarea
                label="Arguments"
                placeholder="Optional arguments (leave blank if not needed)"
                description={
                  requiresArgs
                    ? 'This command uses {args}. Provide the replacement text below.'
                    : 'Arguments will be appended and available as {args} if used.'
                }
                value={runArgs}
                onChange={(e) => setRunArgs(e.currentTarget.value)}
                minRows={3}
                autosize
                disabled={isRunning}
              />
            </>
          )}
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={handleCloseRunModal} disabled={isRunning}>
              Cancel
            </Button>
            <Button
              onClick={handleConfirmRun}
              loading={isRunning}
              disabled={
                isRunning ||
                (requiresArgs && runArgs.trim().length === 0)
              }
            >
              Run Command
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  );
}
