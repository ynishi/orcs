import { useState } from 'react';
import {
  Menu,
  ActionIcon,
  Text,
  Group,
  Badge,
  Stack,
  Tooltip,
  ScrollArea,
  Popover,
  CloseButton,
} from '@mantine/core';
import {
  IconFolder,
  IconFolderOpen,
  IconStar,
  IconStarFilled,
  IconCheck,
  IconPlus,
  IconTrash,
} from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { notifications } from '@mantine/notifications';
import { useWorkspace } from '../../hooks/useWorkspace';
import type { Workspace } from '../../types/workspace';

interface WorkspaceSwitcherProps {
  /** Current session ID for workspace switching */
  sessionId: string | null;
  /** Show initial tip for workspace selection */
  showTip?: boolean;
  /** Callback when tip is closed */
  onCloseTip?: () => void;
}

/**
 * Workspace switcher component that displays all workspaces and allows switching between them.
 *
 * Features:
 * - Shows current workspace
 * - Lists all registered workspaces sorted by last accessed
 * - Allows switching to a different workspace
 * - Toggle favorite status
 * - Visual indication of current workspace
 */
export function WorkspaceSwitcher({ sessionId, showTip = false, onCloseTip }: WorkspaceSwitcherProps) {
  const { workspace, allWorkspaces, switchWorkspace, toggleFavorite, refreshWorkspaces, refresh } = useWorkspace();
  const [isOpen, setIsOpen] = useState(false);

  const handleSwitch = async (targetWorkspaceId: string) => {
    console.log('[Workspace] handleSwitch called:', {
      sessionId,
      targetWorkspaceId,
      currentWorkspaceId: workspace?.id,
    });

    if (!sessionId) {
      console.error('[Workspace] Cannot switch: No session ID');
      notifications.show({
        title: 'Cannot Switch Workspace',
        message: 'Please create or select a session first',
        color: 'orange',
      });
      return;
    }

    if (targetWorkspaceId === workspace?.id) {
      console.log('[Workspace] Already on this workspace');
      return;
    }

    try {
      console.log('[Workspace] Switching to workspace:', targetWorkspaceId);
      await switchWorkspace(sessionId, targetWorkspaceId);
      setIsOpen(false);
    } catch (error) {
      console.error('[Workspace] Failed to switch workspace:', error);
      notifications.show({
        title: 'Failed to Switch Workspace',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleCreateWorkspace = async () => {
    try {
      // Open directory picker dialog
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select workspace directory',
      });

      if (!selected) {
        return; // User cancelled
      }

      console.log('[Workspace] Creating workspace for:', selected);
      console.log('[Workspace] Current workspace count before:', allWorkspaces.length);

      // Create workspace for selected directory
      const newWorkspace = await invoke('create_workspace', { rootPath: selected });
      console.log('[Workspace] Created workspace:', newWorkspace);

      // Refresh all workspaces list
      console.log('[Workspace] Refreshing workspace list...');
      await refreshWorkspaces();

      // Also refresh current workspace if needed
      await refresh();

      // Wait a bit for React state to update and log the result
      setTimeout(() => {
        console.log('[Workspace] After refresh - workspace count:', allWorkspaces.length);
        console.log('[Workspace] All workspaces:', allWorkspaces.map(w => ({ id: w.id, name: w.name })));
      }, 100);

      // Show success notification
      notifications.show({
        title: 'Workspace Created',
        message: `New workspace registered for ${selected}`,
        color: 'green',
        icon: '✅',
      });

      // Keep menu open so user can see the new workspace in the list
      // setIsOpen(false);
    } catch (error) {
      console.error('[Workspace] Failed to create workspace:', error);
      notifications.show({
        title: 'Failed to create workspace',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleToggleFavorite = async (workspaceId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await toggleFavorite(workspaceId);
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  };

  const handleDeleteWorkspace = async (workspaceId: string, workspaceName: string, e: React.MouseEvent) => {
    e.stopPropagation();

    if (!confirm(`Are you sure you want to delete workspace "${workspaceName}"?\n\nThis will remove the workspace metadata and uploaded files. Your project files will not be affected.`)) {
      return;
    }

    try {
      await invoke('delete_workspace', { workspaceId });

      notifications.show({
        title: 'Workspace Deleted',
        message: `${workspaceName} has been removed`,
        color: 'green',
      });

      // Refresh workspace list
      await refreshWorkspaces();

      // If deleted current workspace, refresh to switch to another
      if (workspaceId === workspace?.id) {
        await refresh();
      }
    } catch (error) {
      console.error('Failed to delete workspace:', error);
      notifications.show({
        title: 'Failed to Delete Workspace',
        message: String(error),
        color: 'red',
      });
    }
  };

  const formatLastAccessed = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffInMs = now.getTime() - date.getTime();
    const diffInMins = Math.floor(diffInMs / (1000 * 60));
    const diffInHours = Math.floor(diffInMs / (1000 * 60 * 60));
    const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

    if (diffInMins < 1) return 'Just now';
    if (diffInMins < 60) return `${diffInMins}m ago`;
    if (diffInHours < 24) return `${diffInHours}h ago`;
    if (diffInDays < 7) return `${diffInDays}d ago`;
    return date.toLocaleDateString();
  };

  const renderWorkspaceItem = (ws: Workspace) => {
    const isCurrent = ws.id === workspace?.id;

    return (
      <Menu.Item
        key={ws.id}
        onClick={() => handleSwitch(ws.id)}
        leftSection={isCurrent ? <IconCheck size={16} /> : <IconFolder size={16} />}
        rightSection={
          <Group gap={4}>
            <ActionIcon
              size="xs"
              variant="subtle"
              color={ws.isFavorite ? 'yellow' : 'gray'}
              onClick={(e) => handleToggleFavorite(ws.id, e)}
            >
              {ws.isFavorite ? <IconStarFilled size={14} /> : <IconStar size={14} />}
            </ActionIcon>
            <ActionIcon
              size="xs"
              variant="subtle"
              color="red"
              onClick={(e) => handleDeleteWorkspace(ws.id, ws.name, e)}
            >
              <IconTrash size={14} />
            </ActionIcon>
          </Group>
        }
        style={{
          backgroundColor: isCurrent ? 'var(--mantine-color-blue-light)' : undefined,
        }}
      >
        <Group justify="space-between" gap="xs" wrap="nowrap">
          <Stack gap={2} style={{ flex: 1, minWidth: 0 }}>
            <Text size="sm" fw={isCurrent ? 600 : 400} truncate>
              {ws.name}
            </Text>
            <Text size="xs" c="dimmed" truncate>
              {ws.rootPath}
            </Text>
          </Stack>
          <Text size="xs" c="dimmed" style={{ whiteSpace: 'nowrap' }}>
            {formatLastAccessed(ws.lastAccessed)}
          </Text>
        </Group>
      </Menu.Item>
    );
  };

  // Separate favorites and recent
  const favorites = allWorkspaces.filter(ws => ws.isFavorite);
  const recent = allWorkspaces.filter(ws => !ws.isFavorite);

  return (
    <Popover 
      opened={showTip} 
      position="bottom" 
      withArrow 
      shadow="md"
      width={300}
    >
      <Popover.Target>
        <Menu
          opened={isOpen}
          onChange={setIsOpen}
          width={550}
          position="bottom-end"
          shadow="md"
          withArrow
        >
          <Menu.Target>
            <Tooltip label="Switch workspace" position="right" disabled={showTip}>
              <ActionIcon
                variant={isOpen ? 'filled' : 'subtle'}
                color={isOpen ? 'blue' : 'gray'}
                size="lg"
              >
                {isOpen ? <IconFolderOpen size={20} /> : <IconFolder size={20} />}
              </ActionIcon>
            </Tooltip>
          </Menu.Target>

      <Menu.Dropdown>
        <Menu.Label>
          <Group justify="space-between">
            <Text size="xs">Workspaces</Text>
            {workspace && (
              <Badge size="xs" variant="light">
                {allWorkspaces.length} total
              </Badge>
            )}
          </Group>
        </Menu.Label>

        <Menu.Item
          leftSection={<IconPlus size={16} />}
          onClick={handleCreateWorkspace}
          style={{
            backgroundColor: 'var(--mantine-color-green-light)',
            fontWeight: 500,
          }}
        >
          Create New Workspace
        </Menu.Item>

        <Menu.Divider />

        <ScrollArea.Autosize mah={500} type="auto">
          {favorites.length > 0 && (
            <>
              <Menu.Label>
                <Group gap={4}>
                  <IconStarFilled size={12} />
                  <Text size="xs">Favorites</Text>
                </Group>
              </Menu.Label>
              {favorites.map(renderWorkspaceItem)}
              <Menu.Divider />
            </>
          )}

          {recent.length > 0 && (
            <>
              <Menu.Label>Recent</Menu.Label>
              {recent.map(renderWorkspaceItem)}
            </>
          )}

          {allWorkspaces.length === 0 && (
            <Menu.Item disabled>
              <Text size="sm" c="dimmed">
                No workspaces found
              </Text>
            </Menu.Item>
          )}
        </ScrollArea.Autosize>
      </Menu.Dropdown>
        </Menu>
      </Popover.Target>
      
      <Popover.Dropdown>
        <Group justify="space-between" align="flex-start">
          <Stack gap="xs" style={{ flex: 1 }}>
            <Text size="sm" fw={600}>📁 ワークスペースを開く</Text>
            <Text size="xs" c="dimmed">
              このアイコンをクリックして作業ディレクトリを選択してください
            </Text>
          </Stack>
          <CloseButton size="sm" onClick={onCloseTip} />
        </Group>
      </Popover.Dropdown>
    </Popover>
  );
}
