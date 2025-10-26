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
} from '@mantine/core';
import {
  IconFolder,
  IconFolderOpen,
  IconStar,
  IconStarFilled,
  IconCheck,
} from '@tabler/icons-react';
import { useWorkspace } from '../../hooks/useWorkspace';
import type { Workspace } from '../../types/workspace';

interface WorkspaceSwitcherProps {
  /** Current session ID for workspace switching */
  sessionId: string | null;
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
export function WorkspaceSwitcher({ sessionId }: WorkspaceSwitcherProps) {
  const { workspace, allWorkspaces, switchWorkspace, toggleFavorite } = useWorkspace();
  const [isOpen, setIsOpen] = useState(false);

  const handleSwitch = async (targetWorkspaceId: string) => {
    if (!sessionId || targetWorkspaceId === workspace?.id) {
      return;
    }

    try {
      await switchWorkspace(sessionId, targetWorkspaceId);
      setIsOpen(false);
    } catch (error) {
      console.error('Failed to switch workspace:', error);
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
          <ActionIcon
            size="xs"
            variant="subtle"
            color={ws.isFavorite ? 'yellow' : 'gray'}
            onClick={(e) => handleToggleFavorite(ws.id, e)}
          >
            {ws.isFavorite ? <IconStarFilled size={14} /> : <IconStar size={14} />}
          </ActionIcon>
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
    <Menu
      opened={isOpen}
      onChange={setIsOpen}
      width={400}
      position="bottom-end"
      shadow="md"
      withArrow
    >
      <Menu.Target>
        <Tooltip label="Switch workspace" position="right">
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

        <ScrollArea.Autosize mah={400} type="auto">
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
  );
}
