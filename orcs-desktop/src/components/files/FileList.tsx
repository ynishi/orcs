import { Stack, ScrollArea, Group, Text, Box, ActionIcon, TextInput, Badge, Menu, UnstyledButton, Tooltip, Switch } from '@mantine/core';
import { IconMessage, IconMessages, IconExternalLink, IconTrash, IconPencil, IconMessageCircle, IconDotsVertical, IconMessagePlus, IconCopy, IconArchive, IconStar, IconArrowUp, IconArrowDown, IconFile, IconFileText, IconBrandJavascript, IconBrandTypescript, IconSettings, IconClipboard, IconFolderShare, IconChecklist, IconPaperclip, IconFolder } from '@tabler/icons-react';
import { notifications } from '@mantine/notifications';
import { invoke } from '@tauri-apps/api/core';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { useState, useMemo } from 'react';
import { UploadedFile } from '../../types/workspace';

interface FileListProps {
  files: UploadedFile[];
  onAttachToChat?: (file: UploadedFile) => void;
  onOpenFile?: (file: UploadedFile) => void;
  onRenameFile?: (file: UploadedFile, newName: string) => void;
  onDeleteFile?: (file: UploadedFile) => void;
  onGoToSession?: (file: UploadedFile) => void;
  onNewSessionWithFile?: (file: UploadedFile) => void;
  onToggleArchive?: (file: UploadedFile) => void;
  onToggleFavorite?: (file: UploadedFile) => void;
  onToggleDefaultAttachment?: (file: UploadedFile) => void;
  onMoveSortOrder?: (fileId: string, direction: 'up' | 'down') => void;
  onCopyToWorkspace?: (file: UploadedFile) => void;
}

export function FileList({ files, onAttachToChat, onOpenFile, onRenameFile, onDeleteFile, onGoToSession, onNewSessionWithFile, onToggleArchive, onToggleFavorite, onToggleDefaultAttachment, onMoveSortOrder, onCopyToWorkspace }: FileListProps) {
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null);
  const [editingFileId, setEditingFileId] = useState<string | null>(null);
  const [editingFileName, setEditingFileName] = useState<string>('');
  const [filePreviewCache, setFilePreviewCache] = useState<Record<string, string>>({});
  const [showArchived, setShowArchived] = useState<boolean>(false); // デフォルトOFF（非表示）

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
      console.log('[FileList] Copying file to clipboard:', file.path);

      // Create a promise for the clipboard content
      const contentPromise = (async () => {
        // Read file content from workspace
        const fileData = await invoke<number[]>('read_workspace_file', {
          filePath: file.path,
        });

        console.log('[FileList] File data received, length:', fileData.length);

        // Convert to string (assuming text file)
        const uint8Array = new Uint8Array(fileData);
        const decoder = new TextDecoder('utf-8');
        const content = decoder.decode(uint8Array);

        console.log('[FileList] Decoded content length:', content.length);
        return content;
      })();

      // Write to clipboard using promise-based approach to maintain user interaction context
      await navigator.clipboard.write([
        new ClipboardItem({
          'text/plain': contentPromise.then(text => new Blob([text], { type: 'text/plain' }))
        })
      ]);

      console.log('[FileList] Successfully copied to clipboard');

      notifications.show({
        title: 'Copied!',
        message: `File content copied to clipboard`,
        color: 'green',
      });
    } catch (err) {
      console.error('[FileList] Failed to copy to clipboard:', err);
      const errorMessage = err instanceof Error ? err.message : String(err);
      notifications.show({
        title: 'Error',
        message: `Failed to copy file content: ${errorMessage}`,
        color: 'red',
      });
    }
  };

  const handleFileHover = async (file: UploadedFile) => {
    // Only preview text files
    if (!file.mimeType.startsWith('text/')) {
      return;
    }

    // Skip if already cached
    if (filePreviewCache[file.id]) {
      return;
    }

    try {
      // Read file content from workspace
      const fileData = await invoke<number[]>('read_workspace_file', {
        filePath: file.path,
      });

      // Convert to string
      const uint8Array = new Uint8Array(fileData);
      const decoder = new TextDecoder('utf-8');
      const content = decoder.decode(uint8Array);

      // Extract preview content
      let previewContent = content;

      // For session exports, skip metadata section (everything before ---)
      if (file.name.startsWith('session_') && file.name.endsWith('.md')) {
        const separatorIndex = content.indexOf('\n---\n');
        if (separatorIndex !== -1) {
          // Skip past separator and get the actual conversation content
          previewContent = content.slice(separatorIndex + 5).trim();
          // Remove the ## Author header line if present
          if (previewContent.startsWith('## ')) {
            const firstNewline = previewContent.indexOf('\n');
            if (firstNewline !== -1) {
              previewContent = previewContent.slice(firstNewline + 1).trim();
            }
          }
        }
      }

      // Cache the preview (first 80 characters for better context)
      const preview = previewContent.slice(0, 80).trim() + (previewContent.length > 80 ? '...' : '');
      setFilePreviewCache(prev => ({
        ...prev,
        [file.id]: preview || file.name,
      }));
    } catch (err) {
      console.error('[FileList] Failed to load file preview:', err);
    }
  };

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const getFileIcon = (file: UploadedFile) => {
    // 出所に基づくアイコン判定（優先）
    // Session保存: session_*.md
    if (file.name.startsWith('session_') && file.name.endsWith('.md')) {
      return <IconMessages size={16} color="var(--mantine-color-blue-6)" />;
    }
    // Task保存: task_*.md
    if (file.name.startsWith('task_') && file.name.endsWith('.md')) {
      return <IconChecklist size={16} color="var(--mantine-color-violet-6)" />;
    }
    // Chat/Message保存: sessionIdがある場合（上記以外）
    if (file.sessionId) {
      return <IconMessage size={16} color="var(--mantine-color-teal-6)" />;
    }

    // 拡張子に基づくアイコン判定（フォールバック）
    const ext = file.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'rs': return <IconFile size={16} color="orange" />;
      case 'ts':
      case 'tsx': return <IconBrandTypescript size={16} color="blue" />;
      case 'js':
      case 'jsx': return <IconBrandJavascript size={16} color="yellow" />;
      case 'md': return <IconFileText size={16} color="gray" />;
      case 'json': return <IconSettings size={16} color="gray" />;
      case 'toml': return <IconClipboard size={16} color="gray" />;
      default: return <IconFile size={16} color="gray" />;
    }
  };

  // ソート済みファイル（メモ化）
  const sortedFiles = useMemo(() => {
    return [...files].sort((a, b) => {
      // 1. Archivedは常に最後
      if (a.isArchived !== b.isArchived) {
        return a.isArchived ? 1 : -1;
      }

      // 2. Favoriteは常に上
      if (a.isFavorite !== b.isFavorite) {
        return a.isFavorite ? -1 : 1;
      }

      // 3. Favorite内では、sort_orderがあればそれを優先
      if (a.isFavorite && b.isFavorite) {
        if (a.sortOrder != null && b.sortOrder != null) {
          return a.sortOrder - b.sortOrder;
        }
        if (a.sortOrder != null) return -1;
        if (b.sortOrder != null) return 1;
      }

      // 4. DefaultAttachmentはFavoriteの次
      if (a.isDefaultAttachment !== b.isDefaultAttachment) {
        return a.isDefaultAttachment ? -1 : 1;
      }

      // 5. それ以外はuploadedAtで降順
      return b.uploadedAt - a.uploadedAt;
    });
  }, [files]);

  // 表示するファイル（メモ化）
  const visibleFiles = useMemo(() => {
    return showArchived
      ? sortedFiles
      : sortedFiles.filter(f => !f.isArchived);
  }, [sortedFiles, showArchived]);

  // カテゴリ別ファイル（メモ化）- SessionListパターン
  const { favoriteFiles, defaultAttachmentFiles, recentFiles, archivedFiles } = useMemo(() => {
    return {
      favoriteFiles: visibleFiles.filter(f => f.isFavorite && !f.isArchived),
      defaultAttachmentFiles: visibleFiles.filter(f => f.isDefaultAttachment && !f.isFavorite && !f.isArchived),
      recentFiles: visibleFiles.filter(f => !f.isFavorite && !f.isDefaultAttachment && !f.isArchived),
      archivedFiles: visibleFiles.filter(f => f.isArchived),
    };
  }, [visibleFiles]);

  // Favoriteファイルの数を数える（UP/DOWNボタンの表示判定用）
  const favoriteFilesCount = favoriteFiles.length;

  // ファイルレンダリング関数（SessionListパターン）
  const renderFile = (file: UploadedFile) => {
    // 背景色の決定：選択中 > Archived > デフォルト
    const getBackgroundColor = () => {
      if (file.id === selectedFileId) return '#e7f5ff';
      if (file.isArchived) return '#fafafa';
      return 'white';
    };

    const getHeaderBackgroundColor = () => {
      if (file.id === selectedFileId) return '#d0ebff';
      if (file.isArchived) return '#f0f0f0';
      return '#f8f9fa';
    };

    return (
      <Box
        key={file.id}
        style={{
          borderRadius: '8px',
          border: '1px solid var(--mantine-color-gray-3)',
          backgroundColor: getBackgroundColor(),
          transition: 'all 0.15s ease',
          cursor: 'pointer',
          overflow: 'hidden',
          opacity: file.isArchived ? 0.85 : 1,
        }}
      >
      {editingFileId === file.id ? (
        // 編集モード
        <Box p="md">
          <Group gap="sm" wrap="nowrap">
            <Box style={{ display: 'flex', alignItems: 'center' }}>{getFileIcon(file)}</Box>
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
              backgroundColor: getHeaderBackgroundColor(),
              borderBottom: '1px solid var(--mantine-color-gray-3)',
            }}
          >
            {/* Left group: ファイルアイコン + Favorite */}
            <Group gap="xs">
              <Box style={{ display: 'flex', alignItems: 'center' }}>{getFileIcon(file)}</Box>

              {/* Favoriteボタン */}
              {onToggleFavorite && (
                <Tooltip label={file.isFavorite ? "Remove from favorites" : "Add to favorites"} withArrow>
                  <ActionIcon
                    size="sm"
                    color={file.isFavorite ? "yellow" : "gray"}
                    variant="subtle"
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleFavorite(file);
                    }}
                  >
                    {file.isFavorite ? <IconStar size={16} fill="currentColor" /> : <IconStar size={16} />}
                  </ActionIcon>
                </Tooltip>
              )}

              {/* Default Attachmentボタン */}
              {onToggleDefaultAttachment && (
                <Tooltip label={file.isDefaultAttachment ? "Remove from auto-attach" : "Auto-attach to new sessions"} withArrow>
                  <ActionIcon
                    size="sm"
                    color={file.isDefaultAttachment ? "blue" : "gray"}
                    variant="subtle"
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleDefaultAttachment(file);
                    }}
                  >
                    {file.isDefaultAttachment ? <IconPaperclip size={16} style={{ transform: 'rotate(-45deg)' }} /> : <IconPaperclip size={16} />}
                  </ActionIcon>
                </Tooltip>
              )}
            </Group>

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

                {/* Reveal in Finder */}
                <Menu.Item
                  leftSection={<IconFolder size={14} />}
                  onClick={() => {
                    void revealItemInDir(file.path);
                  }}
                >
                  Reveal in Finder
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

                {/* Copy to another workspace */}
                {onCopyToWorkspace && (
                  <Menu.Item
                    leftSection={<IconFolderShare size={14} />}
                    onClick={() => onCopyToWorkspace(file)}
                  >
                    Copy to workspace...
                  </Menu.Item>
                )}

                {/* Move Up/Down (favoriteファイルが2つ以上ある場合のみ) */}
                {file.isFavorite && onMoveSortOrder && favoriteFilesCount >= 2 && (
                  <>
                    <Menu.Item
                      leftSection={<IconArrowUp size={14} />}
                      onClick={() => onMoveSortOrder(file.id, 'up')}
                    >
                      Move Up
                    </Menu.Item>
                    <Menu.Item
                      leftSection={<IconArrowDown size={14} />}
                      onClick={() => onMoveSortOrder(file.id, 'down')}
                    >
                      Move Down
                    </Menu.Item>
                    <Menu.Divider />
                  </>
                )}

                {/* Archive/Unarchive */}
                <Menu.Item
                  leftSection={<IconArchive size={14} />}
                  onClick={() => onToggleArchive?.(file)}
                >
                  {file.isArchived ? 'Unarchive' : 'Archive'}
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
          <Tooltip
            label={filePreviewCache[file.id] || 'Hover to preview...'}
            disabled={!file.mimeType.startsWith('text/')}
            withArrow
            position="right"
            multiline
            w={220}
          >
            <UnstyledButton
              onClick={() => setSelectedFileId(file.id)}
              onDoubleClick={(e) => handleStartEdit(file, e)}
              onMouseEnter={() => {
                void handleFileHover(file);
              }}
              style={{ width: '100%', textAlign: 'left' }}
            >
              <Box p="md">
                <Box style={{ flex: 1, minWidth: 0 }}>
                  {/* Primary: ファイル名 */}
                  <Text size="sm" fw={600} truncate>
                    {file.name}
                  </Text>

                  {/* Secondary: サイズ + タイプ + Badges */}
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
                    {file.isArchived && (
                      <>
                        <Text size="xs" c="dimmed">•</Text>
                        <Badge size="xs" variant="light" color="gray" style={{ textTransform: 'none' }}>
                          Archived
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
          </Tooltip>
        </>
      )}
    </Box>
    );
  };

  return (
    <Stack gap="md" h="100%">
      {/* ヘッダー */}
      <Group px="sm" justify="space-between">
        <Text size="sm" fw={500}>Files ({visibleFiles.length})</Text>
        <Switch
          label="Show Archived"
          size="xs"
          checked={showArchived}
          onChange={(e) => setShowArchived(e.currentTarget.checked)}
        />
      </Group>

      {/* ファイルリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm" type="auto">
        <Stack gap="md">
          {/* Favoritesセクション */}
          {favoriteFiles.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                FAVORITES
              </Text>
              <Stack gap={4}>
                {favoriteFiles.map(renderFile)}
              </Stack>
            </Box>
          )}

          {/* Default Attachmentsセクション */}
          {defaultAttachmentFiles.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                AUTO-ATTACH
              </Text>
              <Stack gap={4}>
                {defaultAttachmentFiles.map(renderFile)}
              </Stack>
            </Box>
          )}

          {/* Recentセクション */}
          {recentFiles.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                RECENT
              </Text>
              <Stack gap={4}>
                {recentFiles.map(renderFile)}
              </Stack>
            </Box>
          )}

          {/* Archivedセクション */}
          {showArchived && archivedFiles.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                ARCHIVED
              </Text>
              <Stack gap={4}>
                {archivedFiles.map(renderFile)}
              </Stack>
            </Box>
          )}

          {/* 空の状態 */}
          {files.length === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                No files yet
              </Text>
            </Box>
          )}
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
