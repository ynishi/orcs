import { Stack, Text, Loader, Center, Alert, ActionIcon, Group } from '@mantine/core';
import { IconPlus } from '@tabler/icons-react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useWorkspace } from '../../hooks/useWorkspace';
import { FileList } from '../files/FileList';

export function WorkspacePanel() {
  const { workspace, files, isLoading, error, refresh } = useWorkspace();

  // Handle file upload
  const handleUpload = async () => {
    try {
      // Open file picker
      const filePath = await open({ multiple: false });

      if (filePath && workspace) {
        // Upload the file to the workspace
        await invoke('upload_file_to_workspace', {
          workspaceId: workspace.id,
          filePath: filePath,
        });

        // Refresh the file list
        await refresh();
      }
    } catch (err) {
      console.error('Failed to upload file:', err);
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <Center p="xl">
        <Stack align="center" gap="md">
          <Loader size="sm" />
          <Text size="sm" c="dimmed">
            Loading workspace...
          </Text>
        </Stack>
      </Center>
    );
  }

  // Error state
  if (error) {
    return (
      <Alert color="red" title="Error loading workspace" m="md">
        <Text size="sm">{error}</Text>
      </Alert>
    );
  }

  // No files state
  if (files.length === 0) {
    return (
      <Stack p="md" gap="md">
        <Group justify="space-between">
          <Text size="sm" fw={500}>
            Workspace Files
          </Text>
          <ActionIcon
            onClick={handleUpload}
            variant="subtle"
            color="blue"
            aria-label="Upload file"
          >
            <IconPlus size={18} />
          </ActionIcon>
        </Group>
        <Center p="xl">
          <Text size="sm" c="dimmed">
            No files in workspace
          </Text>
        </Center>
      </Stack>
    );
  }

  // Render the file list with real data
  return (
    <Stack p="md" gap="md">
      <Group justify="space-between">
        <Text size="sm" fw={500}>
          Workspace Files
        </Text>
        <ActionIcon
          onClick={handleUpload}
          variant="subtle"
          color="blue"
          aria-label="Upload file"
        >
          <IconPlus size={18} />
        </ActionIcon>
      </Group>
      <FileList files={files} />
    </Stack>
  );
}
