import { useState, useEffect, useMemo, useCallback } from 'react';
import { Stack, ScrollArea, Group, Text, Box, ActionIcon, Tooltip, Badge, Modal, Textarea, Button, TextInput, Menu } from '@mantine/core';
import { IconPlus, IconSearch, IconDotsVertical, IconStar, IconStarFilled, IconPencil, IconTrash, IconArrowUp, IconArrowDown, IconBrain } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { SlashCommand, CommandType } from '../../types/slash_command';
import { SlashCommandEditorModal } from './SlashCommandEditorModal';

const COMMAND_TYPE_LABELS: Record<SlashCommand['type'], string> = {
  prompt: 'Prompt',
  shell: 'Shell',
  task: 'Task',
  action: 'Action',
  pipeline: 'Pipeline',
};

const COMMAND_TYPE_COLORS: Record<SlashCommand['type'], string> = {
  prompt: 'blue',
  shell: 'violet',
  task: 'orange',
  action: 'teal',
  pipeline: 'cyan',
};

type FilterType = 'all' | 'favorites' | CommandType;

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
  const [deletingCommand, setDeletingCommand] = useState<string | null>(null);

  // New states for filtering
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<FilterType>('all');

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [onCommandsUpdated]);

  // Filter and sort commands
  const { favoriteCommands, regularCommands, filteredCount } = useMemo(() => {
    let filtered = commands;

    // Apply search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (c) =>
          c.name.toLowerCase().includes(query) ||
          c.description.toLowerCase().includes(query)
      );
    }

    // Apply type filter
    if (filterType === 'favorites') {
      filtered = filtered.filter((c) => c.isFavorite);
    } else if (filterType !== 'all') {
      filtered = filtered.filter((c) => c.type === filterType);
    }

    // Separate favorites and regular commands
    const favorites = filtered
      .filter((c) => c.isFavorite)
      .sort((a, b) => (a.sortOrder ?? 999) - (b.sortOrder ?? 999));

    const regular = filtered
      .filter((c) => !c.isFavorite)
      .sort((a, b) => a.name.localeCompare(b.name));

    return {
      favoriteCommands: favorites,
      regularCommands: regular,
      filteredCount: filtered.length,
    };
  }, [commands, searchQuery, filterType]);

  // Handler to toggle favorite
  const handleToggleFavorite = useCallback(async (command: SlashCommand) => {
    try {
      const updated = await invoke<SlashCommand>('toggle_slash_command_favorite', {
        name: command.name,
        isFavorite: !command.isFavorite,
      });
      setCommands((prev) =>
        prev.map((c) => (c.name === updated.name ? updated : c))
      );
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
      onMessage?.('error', 'SYSTEM', `Failed to toggle favorite: ${error}`);
    }
  }, [onMessage]);

  // Handler to toggle includeInSystemPrompt
  const handleToggleIncludeInSystemPrompt = useCallback(async (command: SlashCommand) => {
    try {
      const updated = await invoke<SlashCommand>('toggle_slash_command_include_in_system_prompt', {
        name: command.name,
        includeInSystemPrompt: !command.includeInSystemPrompt,
      });
      setCommands((prev) =>
        prev.map((c) => (c.name === updated.name ? updated : c))
      );
    } catch (error) {
      console.error('Failed to toggle includeInSystemPrompt:', error);
      onMessage?.('error', 'SYSTEM', `Failed to toggle system prompt inclusion: ${error}`);
    }
  }, [onMessage]);

  // Handler to move favorite up/down
  const handleMoveFavorite = useCallback(async (command: SlashCommand, direction: 'up' | 'down') => {
    const currentIndex = favoriteCommands.findIndex((c) => c.name === command.name);
    if (currentIndex === -1) return;

    const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
    if (targetIndex < 0 || targetIndex >= favoriteCommands.length) return;

    const targetCommand = favoriteCommands[targetIndex];

    // Swap sort orders
    const currentOrder = command.sortOrder ?? currentIndex;
    const targetOrder = targetCommand.sortOrder ?? targetIndex;

    try {
      await invoke('update_slash_command_sort_order', {
        name: command.name,
        sortOrder: targetOrder,
      });
      await invoke('update_slash_command_sort_order', {
        name: targetCommand.name,
        sortOrder: currentOrder,
      });
      await fetchCommands();
    } catch (error) {
      console.error('Failed to move favorite:', error);
      onMessage?.('error', 'SYSTEM', `Failed to move favorite: ${error}`);
    }
  }, [favoriteCommands, onMessage]);

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

  // Handler for initiating delete confirmation
  const handleRequestDelete = (name: string) => {
    setDeletingCommand(name);
  };

  // Handler for confirming deletion
  const handleConfirmDelete = async () => {
    if (!deletingCommand) return;

    const nameToDelete = deletingCommand;
    setDeletingCommand(null);

    try {
      await invoke('remove_slash_command', { name: nameToDelete });
      await fetchCommands();
      onMessage?.('system', 'SYSTEM', `Deleted slash command: /${nameToDelete}`);
    } catch (error) {
      console.error('Failed to delete slash command:', error);
      onMessage?.('error', 'SYSTEM', `Failed to delete command: ${error}`);
    }
  };

  // Handler for cancelling deletion
  const handleCancelDelete = () => {
    setDeletingCommand(null);
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

  const renderCommand = (command: SlashCommand, showMoveButtons: boolean = false) => {
    const favoriteIndex = favoriteCommands.findIndex((c) => c.name === command.name);
    const canMoveUp = showMoveButtons && favoriteIndex > 0;
    const canMoveDown = showMoveButtons && favoriteIndex < favoriteCommands.length - 1;

    return (
      <Box
        key={command.name}
        style={{
          borderRadius: '8px',
          border: '1px solid var(--mantine-color-gray-3)',
          backgroundColor: command.isFavorite ? 'var(--mantine-color-yellow-0)' : 'transparent',
          transition: 'all 0.15s ease',
          cursor: 'pointer',
          overflow: 'hidden',
        }}
        onClick={() => handleRunCommand(command)}
      >
        {/* Header with actions */}
        <Group
          gap="xs"
          px="sm"
          py={4}
          justify="flex-end"
          style={{
            backgroundColor: command.isFavorite
              ? 'var(--mantine-color-yellow-1)'
              : 'var(--mantine-color-gray-1)',
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Include in system prompt toggle */}
          <Tooltip label={command.includeInSystemPrompt ? 'Exclude from system prompt' : 'Include in system prompt'} withArrow>
            <ActionIcon
              variant="subtle"
              color={command.includeInSystemPrompt ? 'cyan' : 'gray'}
              size="xs"
              onClick={() => handleToggleIncludeInSystemPrompt(command)}
              style={{ opacity: command.includeInSystemPrompt ? 1 : 0.4 }}
            >
              <IconBrain size={14} />
            </ActionIcon>
          </Tooltip>

          {/* Favorite toggle */}
          <Tooltip label={command.isFavorite ? 'Remove from favorites' : 'Add to favorites'} withArrow>
            <ActionIcon
              variant="subtle"
              color={command.isFavorite ? 'yellow' : 'gray'}
              size="xs"
              onClick={() => handleToggleFavorite(command)}
            >
              {command.isFavorite ? <IconStarFilled size={14} /> : <IconStar size={14} />}
            </ActionIcon>
          </Tooltip>

          {/* Context menu */}
          <Menu position="bottom-end" withinPortal>
            <Menu.Target>
              <ActionIcon variant="subtle" color="gray" size="xs">
                <IconDotsVertical size={14} />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown onClick={(e) => e.stopPropagation()}>
              {command.isFavorite && favoriteCommands.length >= 2 && (
                <>
                  <Menu.Item
                    leftSection={<IconArrowUp size={14} />}
                    onClick={() => handleMoveFavorite(command, 'up')}
                    disabled={!canMoveUp}
                  >
                    Move Up
                  </Menu.Item>
                  <Menu.Item
                    leftSection={<IconArrowDown size={14} />}
                    onClick={() => handleMoveFavorite(command, 'down')}
                    disabled={!canMoveDown}
                  >
                    Move Down
                  </Menu.Item>
                  <Menu.Divider />
                </>
              )}
              <Menu.Item
                leftSection={<IconPencil size={14} />}
                onClick={() => handleOpenModal(command)}
              >
                Edit
              </Menu.Item>
              <Menu.Item
                leftSection={<IconTrash size={14} />}
                color="red"
                onClick={() => handleRequestDelete(command.name)}
              >
                Delete
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>

        {/* Content */}
        <Box p="sm">
          <Group gap="xs" mb={4} wrap="nowrap">
            <Text size="lg">{command.icon}</Text>
            <Text size="sm" fw={600} truncate style={{ flex: 1 }}>
              /{command.name}
            </Text>
            <Badge size="xs" color={COMMAND_TYPE_COLORS[command.type]}>
              {COMMAND_TYPE_LABELS[command.type]}
            </Badge>
          </Group>
          <Text size="xs" c="dimmed" lineClamp={2}>
            {command.description}
          </Text>
        </Box>
      </Box>
    );
  };

  const requiresArgs = runningCommand
    ? runningCommand.content.includes('{args}') || runningCommand.workingDir?.includes('{args}')
    : false;

  const filterButtons: { type: FilterType; label: string; color?: string }[] = [
    { type: 'all', label: 'All' },
    { type: 'favorites', label: 'Fav', color: 'yellow' },
    { type: 'prompt', label: 'Prompt', color: 'blue' },
    { type: 'shell', label: 'Shell', color: 'violet' },
    { type: 'task', label: 'Task', color: 'orange' },
    { type: 'action', label: 'Action', color: 'teal' },
    { type: 'pipeline', label: 'Pipeline', color: 'cyan' },
  ];

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
            {filteredCount} / {commands.length}
          </Text>
        </Group>

        {/* Search input */}
        <TextInput
          placeholder="Search commands..."
          leftSection={<IconSearch size={14} />}
          size="xs"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.currentTarget.value)}
        />

        {/* Type filter buttons */}
        <Group gap={4}>
          {filterButtons.map((btn) => (
            <Badge
              key={btn.type}
              size="sm"
              variant={filterType === btn.type ? 'filled' : 'light'}
              color={btn.color || 'gray'}
              style={{ cursor: 'pointer' }}
              onClick={() => setFilterType(btn.type)}
            >
              {btn.label}
            </Badge>
          ))}
        </Group>
      </Stack>

      {/* Command list */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {/* Favorites section */}
          {favoriteCommands.length > 0 && filterType !== 'favorites' && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" tt="uppercase">
                Favorites
              </Text>
              <Stack gap="xs">
                {favoriteCommands.map((cmd) => renderCommand(cmd, true))}
              </Stack>
            </Box>
          )}

          {/* Regular commands or filtered favorites */}
          {filterType === 'favorites' ? (
            favoriteCommands.length > 0 ? (
              <Box>
                <Stack gap="xs">
                  {favoriteCommands.map((cmd) => renderCommand(cmd, true))}
                </Stack>
              </Box>
            ) : null
          ) : (
            regularCommands.length > 0 && (
              <Box>
                {favoriteCommands.length > 0 && (
                  <Text size="xs" fw={600} c="dimmed" mb="xs" tt="uppercase">
                    Commands
                  </Text>
                )}
                <Stack gap="xs">
                  {regularCommands.map((cmd) => renderCommand(cmd, false))}
                </Stack>
              </Box>
            )
          )}

          {/* Empty state */}
          {filteredCount === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                {commands.length === 0
                  ? 'No custom commands yet'
                  : 'No commands match your filter'}
              </Text>
              {commands.length === 0 && (
                <Text size="xs" c="dimmed" mt="xs">
                  Click the + button to create your first slash command
                </Text>
              )}
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

      {/* Delete confirmation modal */}
      <Modal
        opened={!!deletingCommand}
        onClose={handleCancelDelete}
        title="Delete Command"
        centered
        size="sm"
      >
        <Stack gap="md">
          <Text size="sm">
            Are you sure you want to delete the command "/{deletingCommand}"?
          </Text>
          <Text size="xs" c="dimmed">
            This action cannot be undone.
          </Text>
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={handleCancelDelete}>
              Cancel
            </Button>
            <Button color="red" onClick={handleConfirmDelete}>
              Delete
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  );
}
