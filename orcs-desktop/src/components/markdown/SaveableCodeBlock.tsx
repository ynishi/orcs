import { useState } from 'react';
import { Paper, Text, Group, Button, TextInput, ActionIcon, Box, CopyButton, Tooltip } from '@mantine/core';
import { IconDeviceFloppy, IconEdit, IconCheck, IconX } from '@tabler/icons-react';
import { notifications } from '@mantine/notifications';

interface SaveableCodeBlockProps {
  language: string;
  code: string;
  suggestedPath?: string;
  onSave: (path: string, content: string) => Promise<void>;
}

export function SaveableCodeBlock({ language, code, suggestedPath, onSave }: SaveableCodeBlockProps) {
  const [isEditingPath, setIsEditingPath] = useState(false);
  const [targetPath, setTargetPath] = useState(suggestedPath || '');
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    if (!targetPath.trim()) {
      notifications.show({
        title: 'Error',
        message: 'Please specify a file path',
        color: 'red',
      });
      return;
    }

    setIsSaving(true);
    try {
      await onSave(targetPath, code);
      notifications.show({
        title: 'Success',
        message: `File saved to ${targetPath}`,
        color: 'green',
      });
    } catch (error) {
      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to save file',
        color: 'red',
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleEditPath = () => {
    setIsEditingPath(true);
  };

  const handleConfirmPath = () => {
    setIsEditingPath(false);
  };

  const handleCancelEdit = () => {
    setTargetPath(suggestedPath || '');
    setIsEditingPath(false);
  };

  return (
    <Paper
      p="md"
      radius="md"
      mb="sm"
      style={{
        border: '2px solid var(--mantine-color-green-6)',
        backgroundColor: 'var(--mantine-color-dark-8)',
      }}
    >
      {/* Header with path and actions */}
      <Group justify="space-between" mb="sm">
        <Group gap="xs" style={{ flex: 1 }}>
          {isEditingPath ? (
            <>
              <TextInput
                value={targetPath}
                onChange={(e) => setTargetPath(e.currentTarget.value)}
                placeholder="/path/to/file"
                size="xs"
                style={{ flex: 1 }}
              />
              <ActionIcon color="green" size="sm" onClick={handleConfirmPath}>
                <IconCheck size={16} />
              </ActionIcon>
              <ActionIcon color="red" size="sm" onClick={handleCancelEdit}>
                <IconX size={16} />
              </ActionIcon>
            </>
          ) : (
            <>
              <Text size="xs" c="dimmed" ff="monospace">
                {targetPath || 'No path specified'}
              </Text>
              <ActionIcon size="xs" variant="subtle" onClick={handleEditPath}>
                <IconEdit size={14} />
              </ActionIcon>
            </>
          )}
        </Group>

        <Group gap="xs">
          <CopyButton value={code}>
            {({ copied, copy }) => (
              <Tooltip label={copied ? 'Copied!' : 'Copy code'} withArrow>
                <ActionIcon
                  color={copied ? 'teal' : 'gray'}
                  variant="subtle"
                  onClick={copy}
                  size="sm"
                >
                  {copied ? '✓' : '📋'}
                </ActionIcon>
              </Tooltip>
            )}
          </CopyButton>

          <Button
            size="xs"
            leftSection={<IconDeviceFloppy size={14} />}
            onClick={handleSave}
            loading={isSaving}
            color="green"
          >
            Save to file
          </Button>
        </Group>
      </Group>

      {/* Code block */}
      <Box
        style={{
          backgroundColor: 'var(--mantine-color-dark-9)',
          borderRadius: 'var(--mantine-radius-sm)',
          padding: 'var(--mantine-spacing-sm)',
          overflow: 'auto',
        }}
      >
        <pre style={{ margin: 0 }}>
          <code className={`language-${language}`} style={{ fontSize: '13px' }}>
            {code}
          </code>
        </pre>
      </Box>

      {/* Language badge */}
      {language && (
        <Text size="xs" c="dimmed" mt="xs">
          Language: {language}
        </Text>
      )}
    </Paper>
  );
}
