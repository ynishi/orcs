import { Stack, ScrollArea, Group, Text, UnstyledButton, Box } from '@mantine/core';
import { UploadedFile } from '../../types/workspace';

interface FileListProps {
  files: UploadedFile[];
  onFileSelect?: (file: UploadedFile) => void;
}

export function FileList({ files, onFileSelect }: FileListProps) {

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
                  <Text size="xs" c="dimmed">
                    {formatFileSize(file.size)}
                  </Text>
                </Box>
              </Group>
            </UnstyledButton>
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
