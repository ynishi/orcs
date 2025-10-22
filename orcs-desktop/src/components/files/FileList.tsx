import { Stack, TextInput, ScrollArea, Group, Text, ActionIcon, UnstyledButton, Box } from '@mantine/core';
import { useState } from 'react';

interface FileItem {
  path: string;
  name: string;
  type: 'file' | 'directory';
  size?: number;
}

interface FileListProps {
  onFileSelect?: (file: FileItem) => void;
}

export function FileList({ onFileSelect }: FileListProps) {
  const [searchQuery, setSearchQuery] = useState('');

  // TODO: Tauriバックエンドから取得
  const [files] = useState<FileItem[]>([
    { path: 'src/main.rs', name: 'main.rs', type: 'file', size: 1024 },
    { path: 'src/lib.rs', name: 'lib.rs', type: 'file', size: 2048 },
    { path: 'Cargo.toml', name: 'Cargo.toml', type: 'file', size: 512 },
    { path: 'README.md', name: 'README.md', type: 'file', size: 4096 },
    { path: 'src/', name: 'src', type: 'directory' },
    { path: 'tests/', name: 'tests', type: 'directory' },
  ]);

  const filteredFiles = files.filter(file =>
    file.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const getFileIcon = (file: FileItem) => {
    if (file.type === 'directory') return '📁';
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
      {/* ヘッダー */}
      <Group justify="space-between" px="md" pt="md">
        <Text size="lg" fw={700}>
          Files
        </Text>
        <ActionIcon variant="subtle" size="sm">
          🔄
        </ActionIcon>
      </Group>

      {/* 検索バー */}
      <Box px="md">
        <TextInput
          placeholder="Search files..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.currentTarget.value)}
          size="sm"
        />
      </Box>

      {/* ファイルリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="xs">
          {filteredFiles.map((file) => (
            <UnstyledButton
              key={file.path}
              onClick={() => onFileSelect?.(file)}
              style={{
                padding: '8px 12px',
                borderRadius: '8px',
                transition: 'background-color 0.15s ease',
              }}
              styles={{
                root: {
                  '&:hover': {
                    backgroundColor: '#f1f3f5',
                  },
                },
              }}
            >
              <Group gap="sm" wrap="nowrap">
                <Text size="lg">{getFileIcon(file)}</Text>
                <Box style={{ flex: 1, minWidth: 0 }}>
                  <Text size="sm" fw={500} truncate>
                    {file.name}
                  </Text>
                  {file.type === 'file' && file.size && (
                    <Text size="xs" c="dimmed">
                      {formatFileSize(file.size)}
                    </Text>
                  )}
                </Box>
              </Group>
            </UnstyledButton>
          ))}
        </Stack>
      </ScrollArea>

      {/* フッター */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          {filteredFiles.length} items
        </Text>
      </Box>
    </Stack>
  );
}
