import React, { useMemo } from 'react';
import {
  Modal,
  Stack,
  Text,
  Button,
  Group,
  Box,
  ScrollArea,
  UnstyledButton,
} from '@mantine/core';
import { IconFolder, IconStar, IconCheck } from '@tabler/icons-react';
import type { UploadedFile, Workspace } from '../../types/workspace';

interface CopyToWorkspaceModalProps {
  opened: boolean;
  onClose: () => void;
  file: UploadedFile | null;
  currentWorkspaceId: string;
  allWorkspaces: Workspace[];
  onCopy: (file: UploadedFile, targetWorkspaceId: string) => Promise<void>;
}

export const CopyToWorkspaceModal: React.FC<CopyToWorkspaceModalProps> = ({
  opened,
  onClose,
  file,
  currentWorkspaceId,
  allWorkspaces,
  onCopy,
}) => {
  const [selectedWorkspaceId, setSelectedWorkspaceId] = React.useState<string | null>(null);
  const [isCopying, setIsCopying] = React.useState(false);

  // Filter out current workspace and sort (favorites first, then by lastAccessed)
  const availableWorkspaces = useMemo(() => {
    return allWorkspaces
      .filter((ws) => ws.id !== currentWorkspaceId)
      .sort((a, b) => {
        if (a.isFavorite !== b.isFavorite) {
          return a.isFavorite ? -1 : 1;
        }
        return b.lastAccessed - a.lastAccessed;
      });
  }, [allWorkspaces, currentWorkspaceId]);

  const favoriteWorkspaces = availableWorkspaces.filter((ws) => ws.isFavorite);
  const recentWorkspaces = availableWorkspaces.filter((ws) => !ws.isFavorite);

  const handleCopy = async () => {
    if (!file || !selectedWorkspaceId) return;

    setIsCopying(true);
    try {
      await onCopy(file, selectedWorkspaceId);
      onClose();
    } finally {
      setIsCopying(false);
    }
  };

  const handleClose = () => {
    setSelectedWorkspaceId(null);
    onClose();
  };

  const WorkspaceItem = ({ workspace }: { workspace: Workspace }) => {
    const isSelected = selectedWorkspaceId === workspace.id;

    return (
      <UnstyledButton
        onClick={() => setSelectedWorkspaceId(workspace.id)}
        style={{
          display: 'block',
          width: '100%',
          padding: '8px 12px',
          borderRadius: '4px',
          backgroundColor: isSelected ? 'var(--mantine-color-blue-light)' : 'transparent',
          border: isSelected ? '1px solid var(--mantine-color-blue-6)' : '1px solid transparent',
        }}
      >
        <Group gap="sm" wrap="nowrap">
          {isSelected ? (
            <IconCheck size={16} color="var(--mantine-color-blue-6)" />
          ) : (
            <IconFolder size={16} color="var(--mantine-color-dimmed)" />
          )}
          <Stack gap={0} style={{ flex: 1, minWidth: 0 }}>
            <Group gap={4}>
              <Text size="sm" fw={isSelected ? 600 : 400} truncate>
                {workspace.name}
              </Text>
              {workspace.isFavorite && (
                <IconStar size={12} fill="var(--mantine-color-yellow-5)" color="var(--mantine-color-yellow-5)" />
              )}
            </Group>
            <Text size="xs" c="dimmed" truncate>
              {workspace.rootPath}
            </Text>
          </Stack>
        </Group>
      </UnstyledButton>
    );
  };

  return (
    <Modal
      opened={opened}
      onClose={handleClose}
      title="Copy to Workspace"
      centered
      size="md"
    >
      <Stack gap="md">
        {file && (
          <Box
            style={{
              padding: '8px 12px',
              backgroundColor: 'var(--mantine-color-gray-light)',
              borderRadius: '4px',
            }}
          >
            <Text size="sm" fw={500}>
              {file.name}
            </Text>
            <Text size="xs" c="dimmed">
              {(file.size / 1024).toFixed(1)} KB
            </Text>
          </Box>
        )}

        {availableWorkspaces.length === 0 ? (
          <Text size="sm" c="dimmed" ta="center" py="xl">
            No other workspaces available.
            <br />
            Create a new workspace first.
          </Text>
        ) : (
          <ScrollArea.Autosize mah={300}>
            <Stack gap="xs">
              {favoriteWorkspaces.length > 0 && (
                <>
                  <Text size="xs" c="dimmed" fw={500} tt="uppercase">
                    Favorites
                  </Text>
                  {favoriteWorkspaces.map((ws) => (
                    <WorkspaceItem key={ws.id} workspace={ws} />
                  ))}
                </>
              )}

              {recentWorkspaces.length > 0 && (
                <>
                  {favoriteWorkspaces.length > 0 && <Box my="xs" />}
                  <Text size="xs" c="dimmed" fw={500} tt="uppercase">
                    Recent
                  </Text>
                  {recentWorkspaces.map((ws) => (
                    <WorkspaceItem key={ws.id} workspace={ws} />
                  ))}
                </>
              )}
            </Stack>
          </ScrollArea.Autosize>
        )}

        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={handleClose} disabled={isCopying}>
            Cancel
          </Button>
          <Button
            onClick={handleCopy}
            disabled={!selectedWorkspaceId || isCopying}
            loading={isCopying}
          >
            Copy
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
};
