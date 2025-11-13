import { Stack, ScrollArea, Group, Text, Box, ActionIcon, TextInput, Badge, Menu, UnstyledButton } from '@mantine/core';
import { IconMessage, IconExternalLink, IconTrash, IconPencil, IconMessageCircle, IconDotsVertical, IconMessagePlus, IconCopy } from '@tabler/icons-react';
import { notifications } from '@mantine/notifications';
import { useState } from 'react';
import { UploadedFile } from '../../types/workspace';

interface FileListProps {
  files: UploadedFile[];
  onAttachToChat?: (file: UploadedFile) => void;
  onOpenFile?: (file: UploadedFile) => void;
  onRenameFile?: (file: UploadedFile, newName: string) => void;
  onDeleteFile?: (file: UploadedFile) => void;
  onGoToSession?: (file: UploadedFile) => void;
  onNewSessionWithFile?: (file: UploadedFile) => void;
}

export function FileList({ files, onAttachToChat, onOpenFile, onRenameFile, onDeleteFile, onGoToSession, onNewSessionWithFile }: FileListProps) {
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null);
  const [editingFileId, setEditingFileId] = useState<string | null>(null);
  const [editingFileName, setEditingFileName] = useState<string>('');

  const handleStartEdit = (file: UploadedFile, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingFileId(file.id);
    setEditingFileName(file.name);
  };

  const handleSaveEdit = (file: UploadedFile) => {
    if (editingFileName.trim() && editingFileName !== file.name) {
      onRenameFile?.(file, editingFileName.trim());
    }
    setEditingFileId(null);
  };

  const handleCancelEdit = () => {
    setEditingFileId(null);
    setEditingFileName('');
  };

  const handleCopyToClipboard = async (file: UploadedFile) => {
    try {
      // Read file content from workspace
      const { invoke } = await import('@tauri-apps/api/core');
      const fileData = await invoke<number[]>('read_workspace_file', {
        filePath: file.path,
      });

      // Convert to string (assuming text file)
      const uint8Array = new Uint8Array(fileData);
      const decoder = new TextDecoder('utf-8');
      const content = decoder.decode(uint8Array);

      // Copy to clipboard
      await navigator.clipboard.writeText(content);

      notifications.show({
        title: 'Copied!',
        message: `File content copied to clipboard`,
        color: 'green',
      });
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
      notifications.show({
        title: 'Error',
        message: 'Failed to copy file content',
        color: 'red',
      });
    }
  };

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const getFileIcon = (file: UploadedFile) => {
    const ext = file.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'rs': return '🦀';
      case 'ts':
      case 'tsx': return '📘';
      case 'js':
      case 'jsx': return '📜';
      case 'md': return '📝';
      case 'json': return '⚙️';
      case 'toml': return '📋';
      default: return '📄';
    }
  };

  // ファイルレンダリング関数（SessionListパターン）
  const renderFile = (file: UploadedFile) => (
    <Box
      key={file.id}
      style={{
        borderRadius: '8px',
        border: '1px solid var(--mantine-color-gray-3)',
        backgroundColor: file.id === selectedFileId ? '#e7f5ff' : 'white',
        transition: 'all 0.15s ease',
        cursor: 'pointer',
        overflow: 'hidden',
      }}
    >
      {editingFileId === file.id ? (
        // 編集モード
        <Box p="md">
          <Group gap="sm" wrap="nowrap">
            <Text size="lg">{getFileIcon(file)}</Text>
            <Box style={{ flex: 1, minWidth: 0 }}>
              <TextInput
                size="xs"
                value={editingFileName}
                onChange={(e) => setEditingFileName(e.currentTarget.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    handleSaveEdit(file);
                  } else if (e.key === 'Escape') {
                    handleCancelEdit();
                  }
                }}
                onBlur={() => handleSaveEdit(file)}
                autoFocus
                onClick={(e) => e.stopPropagation()}
              />
            </Box>
          </Group>
        </Box>
      ) : (
        <>
          {/* TOPメニュー行（SessionListパターン） */}
          <Group
            gap="xs"
            px="md"
            py="xs"
            justify="space-between"
            style={{
              backgroundColor: file.id === selectedFileId ? '#d0ebff' : '#f8f9fa',
              borderBottom: '1px solid var(--mantine-color-gray-3)',
            }}
          >
            {/* ファイルアイコン（左寄せ） */}
            <Text size="lg">{getFileIcon(file)}</Text>

            {/* コンテキストメニュー */}
            <Menu position="bottom-end" withinPortal>
              <Menu.Target>
                <ActionIcon
                  size="sm"
                  color="gray"
                  variant="subtle"
                  onClick={(e) => e.stopPropagation()}
                  style={{ flexShrink: 0 }}
                >
                  <IconDotsVertical size={16} />
                </ActionIcon>
              </Menu.Target>

              <Menu.Dropdown onClick={(e) => e.stopPropagation()}>
                {/* Go to conversation（sessionIdがある場合のみ） */}
                {file.sessionId && (
                  <>
                    <Menu.Item
                      leftSection={<IconMessageCircle size={14} />}
                      color="violet"
                      onClick={() => onGoToSession?.(file)}
                    >
                      Go to conversation
                    </Menu.Item>
                    <Menu.Divider />
                  </>
                )}

                {/* New chat with this file */}
                <Menu.Item
                  leftSection={<IconMessagePlus size={14} />}
                  color="blue"
                  onClick={() => onNewSessionWithFile?.(file)}
                >
                  New chat with this file
                </Menu.Item>

                {/* Attach to chat */}
                <Menu.Item
                  leftSection={<IconMessage size={14} />}
                  onClick={() => onAttachToChat?.(file)}
                >
                  Attach to chat
                </Menu.Item>

                {/* Copy to clipboard */}
                <Menu.Item
                  leftSection={<IconCopy size={14} />}
                  onClick={() => {
                    void handleCopyToClipboard(file);
                  }}
                >
                  Copy to clipboard
                </Menu.Item>

                {/* Open file */}
                <Menu.Item
                  leftSection={<IconExternalLink size={14} />}
                  onClick={() => onOpenFile?.(file)}
                >
                  Open file
                </Menu.Item>

                {/* Rename */}
                <Menu.Item
                  leftSection={<IconPencil size={14} />}
                  onClick={() => {
                    setEditingFileId(file.id);
                    setEditingFileName(file.name);
                  }}
                >
                  Rename
                </Menu.Item>

                <Menu.Divider />

                {/* Delete */}
                <Menu.Item
                  leftSection={<IconTrash size={14} />}
                  color="red"
                  onClick={() => onDeleteFile?.(file)}
                >
                  Delete
                </Menu.Item>
              </Menu.Dropdown>
            </Menu>
          </Group>

          {/* コンテンツエリア */}
          <UnstyledButton
            onClick={() => setSelectedFileId(file.id)}
            onDoubleClick={(e) => handleStartEdit(file, e)}
            style={{ width: '100%', textAlign: 'left' }}
          >
            <Box p="md">
              <Box style={{ flex: 1, minWidth: 0 }}>
                {/* Primary: ファイル名 */}
                <Text size="sm" fw={600} truncate>
                  {file.name}
                </Text>

                {/* Secondary: サイズ + タイプ + From chat Badge */}
                <Group gap="xs" mt={4}>
                  <Text size="xs" c="dimmed">
                    {formatFileSize(file.size)}
                  </Text>
                  <Text size="xs" c="dimmed">•</Text>
                  <Text size="xs" c="dimmed">
                    {getFileTypeCategory(file.mimeType)}
                  </Text>
                  {file.sessionId && (
                    <>
                      <Text size="xs" c="dimmed">•</Text>
                      <Badge size="xs" variant="light" color="violet" style={{ textTransform: 'none' }}>
                        From chat
                      </Badge>
                    </>
                  )}
                </Group>

                {/* Tertiary: 相対時間 */}
                <Text size="xs" c="dimmed" mt={2}>
                  {formatRelativeTime(file.uploadedAt)}
                </Text>
              </Box>
            </Box>
          </UnstyledButton>
        </>
      )}
    </Box>
  );

  return (
    <Stack gap="md" h="100%">
      {/* ファイルリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap={4}>
          {files.map(renderFile)}
        </Stack>
      </ScrollArea>

      {/* フッター */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          {files.length} items
        </Text>
      </Box>
    </Stack>
  );
}

// ファイルタイプカテゴリ取得（MIMEタイプから人間が読める形式に変換）
function getFileTypeCategory(mimeType: string): string {
  if (mimeType.startsWith('text/')) return 'Text';
  if (mimeType.startsWith('image/')) return 'Image';
  if (mimeType.startsWith('video/')) return 'Video';
  if (mimeType.startsWith('audio/')) return 'Audio';
  if (mimeType.includes('pdf')) return 'PDF';
  if (mimeType.includes('json')) return 'JSON';
  if (mimeType.includes('zip') || mimeType.includes('tar') || mimeType.includes('gz')) return 'Archive';
  if (mimeType.includes('javascript') || mimeType.includes('typescript')) return 'Code';
  return 'File';
}

// 相対時間フォーマット（SessionListのformatDateと同じロジック）
function formatRelativeTime(timestamp: number): string {
  const now = Date.now() / 1000; // Convert to seconds
  const diff = now - timestamp;
  const minutes = Math.floor(diff / 60);
  const hours = Math.floor(diff / 3600);
  const days = Math.floor(diff / 86400);

  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return new Date(timestamp * 1000).toLocaleDateString();
}
