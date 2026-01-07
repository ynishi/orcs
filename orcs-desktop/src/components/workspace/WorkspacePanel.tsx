import { useState } from 'react';
import { Stack, Text, ActionIcon, Group, Tooltip, Switch, ScrollArea, Box, Center } from '@mantine/core';
import { IconPlus, IconFolder, IconTerminal, IconClipboard, IconSearch } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { notifications } from '@mantine/notifications';
import { useWorkspace } from '../../hooks/useWorkspace';
import { FileList } from '../files/FileList';
import { CopyToWorkspaceModal } from '../files/CopyToWorkspaceModal';
import { UploadedFile } from '../../types/workspace';

interface WorkspacePanelProps {
  onAttachFile?: (file: File) => void;
  includeInPrompt?: boolean;
  onToggleIncludeInPrompt?: (value: boolean) => void;
  onGoToSession?: (sessionId: string, messageTimestamp?: string) => void;
  onNewSessionWithFile?: (file: File) => void;
}

export function WorkspacePanel({ onAttachFile, includeInPrompt, onToggleIncludeInPrompt, onGoToSession, onNewSessionWithFile }: WorkspacePanelProps) {
  const { workspace, files, toggleFileArchive, allWorkspaces } = useWorkspace();
  const [copyModalOpen, setCopyModalOpen] = useState(false);
  const [fileToCopy, setFileToCopy] = useState<UploadedFile | null>(null);

  // Phase 4: No need to manually refresh - event-driven via workspace:update
  // onRefresh prop kept for backward compatibility but not used

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

      // Phase 4: No need to refresh - event-driven via workspace:update

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

      // Phase 4: No need to refresh - event-driven via workspace:update
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

      // Phase 4: No need to refresh - event-driven via workspace:update
    } catch (err) {
      console.error('Failed to delete file:', err);
    }
  };

  // Handle toggling favorite status
  const handleToggleFavorite = async (file: UploadedFile) => {
    if (!workspace) return;

    try {
      await invoke('toggle_workspace_file_favorite', {
        workspaceId: workspace.id,
        fileId: file.id,
      });

      // Phase 4: No need to refresh - event-driven via workspace:update
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
    }
  };

  // Handle moving sort order
  const handleMoveSortOrder = async (fileId: string, direction: 'up' | 'down') => {
    if (!workspace) return;

    try {
      await invoke('move_workspace_file_sort_order', {
        workspaceId: workspace.id,
        fileId: fileId,
        direction: direction,
      });

      // Phase 4: No need to refresh - event-driven via workspace:update
    } catch (err) {
      console.error('Failed to move sort order:', err);
    }
  };

  // Handle navigating to session
  const handleGoToSession = (file: UploadedFile) => {
    if (file.sessionId) {
      onGoToSession?.(file.sessionId, file.messageTimestamp ?? undefined);
    }
  };

  // Handle creating new session with file
  const handleNewSessionWithFile = async (file: UploadedFile) => {
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

      // Call the callback to create new session with file
      onNewSessionWithFile?.(browserFile);
    } catch (err) {
      console.error('Failed to create new session with file:', err);
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

  // Handle investigating workspace
  const handleInvestigateWorkspace = async () => {
    if (!workspace) return;

    try {
      const result = await invoke<any>('investigate_workspace', {
        workspaceId: workspace.id,
        investigationType: 'comprehensive',
      });

      // Format the investigation result
      const formattedResult = formatInvestigationResult(result);

      // Create a temporary file with the result
      const timestamp = new Date().toISOString().replace(/:/g, '-').split('.')[0];
      const fileName = `workspace-investigation-${timestamp}.md`;

      // Upload the investigation result as a workspace file
      await invoke('upload_file_from_bytes', {
        workspaceId: workspace.id,
        filename: fileName,
        fileData: Array.from(new TextEncoder().encode(formattedResult)),
        sessionId: null,
        messageTimestamp: null,
      });

      notifications.show({
        title: 'Investigation Complete',
        message: `Results saved to ${fileName}`,
        color: 'green',
      });
    } catch (error) {
      console.error('Failed to investigate workspace:', error);
      notifications.show({
        title: 'Investigation Failed',
        message: error instanceof Error ? error.message : 'Unknown error',
        color: 'red',
      });
    }
  };

  // Helper function to format investigation result
  const formatInvestigationResult = (result: any): string => {
    // New format: Agent returns report directly as Markdown
    if (result.report) {
      return result.report;
    }

    // Fallback for unexpected format
    return `# Investigation Result\n\n${JSON.stringify(result, null, 2)}`;
  };

  // Handle opening copy to workspace modal
  const handleOpenCopyModal = (file: UploadedFile) => {
    setFileToCopy(file);
    setCopyModalOpen(true);
  };

  // Handle closing copy to workspace modal
  const handleCloseCopyModal = () => {
    setCopyModalOpen(false);
    setFileToCopy(null);
  };

  // Handle copying file to another workspace
  const handleCopyToWorkspace = async (file: UploadedFile, targetWorkspaceId: string) => {
    if (!workspace) return;

    try {
      const copiedFile = await invoke<UploadedFile>('copy_file_to_workspace', {
        sourceWorkspaceId: workspace.id,
        fileId: file.id,
        targetWorkspaceId: targetWorkspaceId,
      });

      const targetWorkspace = allWorkspaces.find((ws) => ws.id === targetWorkspaceId);
      const targetName = targetWorkspace?.name || 'target workspace';

      notifications.show({
        title: 'File copied',
        message: `"${copiedFile.name}" copied to ${targetName}`,
        color: 'green',
      });
    } catch (err) {
      console.error('Failed to copy file:', err);
      notifications.show({
        title: 'Error',
        message: `Failed to copy file: ${err instanceof Error ? err.message : String(err)}`,
        color: 'red',
      });
      throw err; // Re-throw to let the modal handle loading state
    }
  };

  // Handle pasting from clipboard
  const handlePasteFromClipboard = async () => {
    if (!workspace) return;

    try {
      // Read clipboard content using Tauri clipboard plugin
      const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
      const clipboardText = await readText();

      if (!clipboardText || clipboardText.trim().length === 0) {
        notifications.show({
          title: 'Empty clipboard',
          message: 'No text content in clipboard',
          color: 'yellow',
        });
        return;
      }

      // Generate filename with timestamp
      const now = new Date();
      const timestamp = now.toISOString().replace(/[:.]/g, '-').split('T')[0] + '_' +
                        now.toTimeString().split(' ')[0].replace(/:/g, '-');
      const filename = `clipboard_${timestamp}.txt`;

      // Convert text to bytes
      const encoder = new TextEncoder();
      const bytes = encoder.encode(clipboardText);
      const fileData = Array.from(bytes);

      // Upload to workspace
      await invoke('upload_file_from_bytes', {
        workspaceId: workspace.id,
        filename: filename,
        fileData: fileData,
        sessionId: null,
        messageTimestamp: null,
      });

      notifications.show({
        title: 'Clipboard pasted',
        message: `Created "${filename}" from clipboard`,
        color: 'green',
      });
    } catch (err) {
      console.error('Failed to paste from clipboard:', err);
      notifications.show({
        title: 'Error',
        message: `Failed to paste from clipboard: ${err instanceof Error ? err.message : String(err)}`,
        color: 'red',
      });
    }
  };

  // Phase 4: No loading/error states - workspace data comes from event-driven store

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
            <Tooltip label="Paste from clipboard as file" withArrow>
              <ActionIcon
                onClick={() => {
                  void handlePasteFromClipboard();
                }}
                variant="subtle"
                color="teal"
                aria-label="Paste from clipboard"
              >
                <IconClipboard size={18} />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="Investigate workspace" withArrow>
              <ActionIcon
                onClick={() => {
                  void handleInvestigateWorkspace();
                }}
                variant="subtle"
                color="indigo"
                aria-label="Investigate workspace"
              >
                <IconSearch size={18} />
              </ActionIcon>
            </Tooltip>
            <ActionIcon
              component="label"
              variant="subtle"
              color="blue"
              aria-label="Upload file"
            >
              <IconPlus size={16} />
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
          <Tooltip label="Paste from clipboard as file" withArrow>
            <ActionIcon
              onClick={() => {
                void handlePasteFromClipboard();
              }}
              variant="subtle"
              color="teal"
              aria-label="Paste from clipboard"
            >
              <IconClipboard size={18} />
            </ActionIcon>
          </Tooltip>
          <ActionIcon
            component="label"
            variant="subtle"
            color="blue"
            aria-label="Upload file"
          >
            <IconPlus size={16} />
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
          onNewSessionWithFile={handleNewSessionWithFile}
          onToggleArchive={toggleFileArchive}
          onToggleFavorite={handleToggleFavorite}
          onMoveSortOrder={handleMoveSortOrder}
          onCopyToWorkspace={handleOpenCopyModal}
        />
      </ScrollArea>

      {/* Copy to Workspace Modal */}
      <CopyToWorkspaceModal
        opened={copyModalOpen}
        onClose={handleCloseCopyModal}
        file={fileToCopy}
        currentWorkspaceId={workspace?.id || ''}
        allWorkspaces={allWorkspaces}
        onCopy={handleCopyToWorkspace}
      />
    </Stack>
  );
}
