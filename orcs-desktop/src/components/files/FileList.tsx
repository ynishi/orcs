import { Stack, ScrollArea, Group, Text, Box, ActionIcon, Tooltip, TextInput } from '@mantine/core';
import { IconMessage, IconExternalLink, IconTrash, IconPencil, IconMessageCircle } from '@tabler/icons-react';
import { useState } from 'react';
import { UploadedFile } from '../../types/workspace';

interface FileListProps {
  files: UploadedFile[];
  onAttachToChat?: (file: UploadedFile) => void;
  onOpenFile?: (file: UploadedFile) => void;
  onRenameFile?: (file: UploadedFile, newName: string) => void;
  onDeleteFile?: (file: UploadedFile) => void;
  onGoToSession?: (file: UploadedFile) => void;
}

export function FileList({ files, onAttachToChat, onOpenFile, onRenameFile, onDeleteFile, onGoToSession }: FileListProps) {
  const [hoveredFile, setHoveredFile] = useState<string | null>(null);
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

  return (
    <Stack gap="md" h="100%">
      {/* ファイルリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="xs">
          {files.map((file) => (
            <Box
              key={file.id}
              onMouseEnter={() => setHoveredFile(file.id)}
              onMouseLeave={() => setHoveredFile(null)}
              style={{
                padding: '8px 12px',
                borderRadius: '8px',
                transition: 'background-color 0.15s ease',
                backgroundColor: hoveredFile === file.id ? '#f1f3f5' : 'transparent',
              }}
            >
              <Group gap="sm" wrap="nowrap" justify="space-between">
                {editingFileId === file.id ? (
                  // 編集モード
                  <>
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
                      <Text size="xs" c="dimmed" mt={4}>
                        {formatFileSize(file.size)}
                      </Text>
                    </Box>
                  </>
                ) : (
                  // 表示モード
                  <>
                    <Group gap="sm" wrap="nowrap" style={{ flex: 1, minWidth: 0 }}>
                      <Text size="lg">{getFileIcon(file)}</Text>
                      <Box style={{ flex: 1, minWidth: 0 }}>
                        <Text size="sm" fw={500} truncate>
                          {file.name}
                        </Text>
                        <Text size="xs" c="dimmed">
                          {formatFileSize(file.size)}
                        </Text>
                      </Box>
                    </Group>

                    {/* Action buttons - show on hover */}
                    {hoveredFile === file.id && (
                      <Group gap={4}>
                        {file.sessionId && (
                          <Tooltip label="Go to conversation">
                            <ActionIcon
                              variant="subtle"
                              color="violet"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                onGoToSession?.(file);
                              }}
                            >
                              <IconMessageCircle size={16} />
                            </ActionIcon>
                          </Tooltip>
                        )}

                        <Tooltip label="Attach to chat">
                          <ActionIcon
                            variant="subtle"
                            color="blue"
                            size="sm"
                            onClick={(e) => {
                              e.stopPropagation();
                              onAttachToChat?.(file);
                            }}
                          >
                            <IconMessage size={16} />
                          </ActionIcon>
                        </Tooltip>

                        <Tooltip label="Open file">
                          <ActionIcon
                            variant="subtle"
                            color="gray"
                            size="sm"
                            onClick={(e) => {
                              e.stopPropagation();
                              onOpenFile?.(file);
                            }}
                          >
                            <IconExternalLink size={16} />
                          </ActionIcon>
                        </Tooltip>

                        <Tooltip label="Rename">
                          <ActionIcon
                            variant="subtle"
                            color="gray"
                            size="sm"
                            onClick={(e) => handleStartEdit(file, e)}
                          >
                            <IconPencil size={16} />
                          </ActionIcon>
                        </Tooltip>

                        <Tooltip label="Delete">
                          <ActionIcon
                            variant="subtle"
                            color="red"
                            size="sm"
                            onClick={(e) => {
                              e.stopPropagation();
                              onDeleteFile?.(file);
                            }}
                          >
                            <IconTrash size={16} />
                          </ActionIcon>
                        </Tooltip>
                      </Group>
                    )}
                  </>
                )}
              </Group>
            </Box>
          ))}
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
