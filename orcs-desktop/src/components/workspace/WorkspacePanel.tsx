import { Stack, Text, Loader, Center, Alert } from '@mantine/core';
import { useWorkspace } from '../../hooks/useWorkspace';
import { FileList } from '../files/FileList';

export function WorkspacePanel() {
  const { files, isLoading, error } = useWorkspace();

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
      <Center p="xl">
        <Text size="sm" c="dimmed">
          No files in workspace
        </Text>
      </Center>
    );
  }

  // Render the file list with real data
  return <FileList files={files} />;
}
