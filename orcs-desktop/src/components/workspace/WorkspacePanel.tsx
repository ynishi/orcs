import { Stack, Text, Loader, Center, Alert, ActionIcon, Group, Tooltip, Switch, ScrollArea, Box } from '@mantine/core';
import { IconPlus, IconFolder, IconTerminal } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { useCallback } from 'react';
import { useWorkspace } from '../../hooks/useWorkspace';
import { FileList } from '../files/FileList';
import { UploadedFile } from '../../types/workspace';

interface WorkspacePanelProps {
  onAttachFile?: (file: File) => void;
  includeInPrompt?: boolean;
  onToggleIncludeInPrompt?: (value: boolean) => void;
  onGoToSession?: (sessionId: string) => void;
  onRefresh?: () => Promise<void>;
}

export function WorkspacePanel({ onAttachFile, includeInPrompt, onToggleIncludeInPrompt, onGoToSession, onRefresh }: WorkspacePanelProps) {
  const { workspace, files, isLoading, error, refresh } = useWorkspace();

  // Keep local list in sync and notify parent consumers when provided
  const refreshWorkspaceState = useCallback(async () => {
    await refresh();
    if (onRefresh) {
      await onRefresh();
    }
  }, [refresh, onRefresh]);

  // Handle file upload from file input
  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(e.target.files || []);

    if (selectedFiles.length === 0 || !workspace) {
      return;
    }

    try {
      // Upload each file
      for (const file of selectedFiles) {
        // Read file as ArrayBuffer
        const arrayBuffer = await file.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);

        // Convert to regular array for Tauri IPC
        const fileData = Array.from(uint8Array);

        // Upload the file to the workspace (no session info for manual uploads)
        await invoke('upload_file_from_bytes', {
          workspaceId: workspace.id,
          filename: file.name,
          fileData: fileData,
          sessionId: null,
          messageTimestamp: null,
        });
      }

      // Refresh the file list
      await refreshWorkspaceState();

      // Clear the input so the same file can be selected again
      e.target.value = '';
    } catch (err) {
      console.error('Failed to upload file:', err);
    }
  };

  // Handle attaching file to chat
  const handleAttachToChat = async (file: UploadedFile) => {
    try {
      // Read file content from workspace
      const fileData = await invoke<number[]>('read_workspace_file', {
        filePath: file.path,
      });

      // Convert to Uint8Array then to Blob
      const uint8Array = new Uint8Array(fileData);
      const blob = new Blob([uint8Array], { type: file.mimeType });

      // Create a File object
      const browserFile = new File([blob], file.name, {
        type: file.mimeType,
      });

      // Call the callback to attach to chat
      onAttachFile?.(browserFile);
    } catch (err) {
      console.error('Failed to attach file to chat:', err);
    }
  };

  // Handle opening file
  const handleOpenFile = async (file: UploadedFile) => {
    try {
      await openPath(file.path);
    } catch (err) {
      console.error('Failed to open file:', err);
    }
  };

  // Handle renaming file
  const handleRenameFile = async (file: UploadedFile, newName: string) => {
    if (!workspace) return;

    try {
      await invoke('rename_file_in_workspace', {
        workspaceId: workspace.id,
        fileId: file.id,
        newName: newName,
      });

      // Refresh the file list
      await refreshWorkspaceState();
    } catch (err) {
      console.error('Failed to rename file:', err);
    }
  };

  // Handle deleting file
  const handleDeleteFile = async (file: UploadedFile) => {
    if (!workspace) return;

    try {
      await invoke('delete_file_from_workspace', {
        workspaceId: workspace.id,
        fileId: file.id,
      });

      // Refresh the file list
      await refreshWorkspaceState();
    } catch (err) {
      console.error('Failed to delete file:', err);
    }
  };

  // Handle navigating to session
  const handleGoToSession = (file: UploadedFile) => {
    if (file.sessionId) {
      onGoToSession?.(file.sessionId);
    }
  };

  // Handle opening workspace directory in Finder/Explorer
  const handleOpenWorkspaceDir = async () => {
    if (!workspace) return;

    try {
      await openPath(workspace.rootPath);
    } catch (err) {
      console.error('Failed to open workspace directory:', err);
    }
  };

  // Handle opening terminal in workspace directory
  const handleOpenTerminal = async () => {
    if (!workspace) return;

    try {
      await invoke('open_terminal', {
        directory: workspace.rootPath,
      });
    } catch (err) {
      console.error('Failed to open terminal:', err);
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
      <Stack gap="xs" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        {/* ヘッダー */}
        <Stack gap={4} px="sm">
          <Group gap={4} justify="flex-start">
            <Tooltip label="Open workspace folder" withArrow>
              <ActionIcon
                onClick={() => {
                  void handleOpenWorkspaceDir();
                }}
                variant="subtle"
                color="gray"
                aria-label="Open folder"
              >
                <IconFolder size={18} />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="Open terminal in workspace" withArrow>
              <ActionIcon
                onClick={() => {
                  void handleOpenTerminal();
                }}
                variant="subtle"
                color="gray"
                aria-label="Open terminal"
              >
                <IconTerminal size={18} />
              </ActionIcon>
            </Tooltip>
            <ActionIcon
              component="label"
              variant="subtle"
              color="blue"
              aria-label="Upload file"
            >
              <IconPlus size={18} />
              <input type="file" multiple hidden onChange={handleFileSelect} />
            </ActionIcon>
          </Group>
          <Text size="sm" fw={500}>
            Workspace Files
          </Text>
        </Stack>

        {/* Include in prompt toggle */}
        <Box px="sm">
          <Tooltip label="Include workspace file list in AI prompts" withArrow>
            <Switch
              size="xs"
              label="Include in prompt"
              checked={includeInPrompt || false}
              onChange={(e) => onToggleIncludeInPrompt?.(e.currentTarget.checked)}
            />
          </Tooltip>
        </Box>

        {/* スクロールエリア */}
        <ScrollArea style={{ flex: 1 }} px="sm">
          <Center p="xl">
            <Text size="sm" c="dimmed">
              No files in workspace
            </Text>
          </Center>
        </ScrollArea>
      </Stack>
    );
  }

  // Render the file list with real data
  return (
    <Stack gap="xs" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* ヘッダー */}
      <Stack gap={4} px="sm">
        <Group gap={4} justify="flex-start">
          <Tooltip label="Open workspace folder" withArrow>
            <ActionIcon
              onClick={() => {
                void handleOpenWorkspaceDir();
              }}
              variant="subtle"
              color="gray"
              aria-label="Open folder"
            >
              <IconFolder size={18} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Open terminal in workspace" withArrow>
            <ActionIcon
              onClick={() => {
                void handleOpenTerminal();
              }}
              variant="subtle"
              color="gray"
              aria-label="Open terminal"
            >
              <IconTerminal size={18} />
            </ActionIcon>
          </Tooltip>
          <ActionIcon
            component="label"
            variant="subtle"
            color="blue"
            aria-label="Upload file"
          >
            <IconPlus size={18} />
            <input type="file" multiple hidden onChange={handleFileSelect} />
          </ActionIcon>
        </Group>
        <Text size="sm" fw={500}>
          Workspace Files
        </Text>
      </Stack>

      {/* Include in prompt toggle */}
      <Box px="sm">
        <Tooltip label="Include workspace file list in AI prompts" withArrow>
          <Switch
            size="xs"
            label="Include in prompt"
            checked={includeInPrompt || false}
            onChange={(e) => onToggleIncludeInPrompt?.(e.currentTarget.checked)}
          />
        </Tooltip>
      </Box>

      {/* スクロールエリア */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <FileList
          files={files}
          onAttachToChat={handleAttachToChat}
          onOpenFile={handleOpenFile}
          onRenameFile={handleRenameFile}
          onDeleteFile={handleDeleteFile}
          onGoToSession={handleGoToSession}
        />
      </ScrollArea>
    </Stack>
  );
}
