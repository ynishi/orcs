import { Paper, Stack, Group, Avatar, Text, Badge, Progress, ActionIcon, Tooltip } from '@mantine/core';
import { IconX } from '@tabler/icons-react';

interface ThinkingIndicatorProps {
  personaName: string;
  personaInitial?: string;
  color?: string;
  onCancel?: () => void;
}

export function ThinkingIndicator({
  personaName,
  personaInitial,
  color = 'blue',
  onCancel
}: ThinkingIndicatorProps) {
  const initial = personaInitial || personaName.charAt(0).toUpperCase();

  return (
    <Paper
      p="md"
      withBorder
      radius="md"
      style={{
        borderColor: '#228be6',
        backgroundColor: '#f0f7ff'
      }}
    >
      <Stack gap="xs">
        <Group gap="xs">
          <Avatar size="sm" color={color}>{initial}</Avatar>
          <Text size="sm" fw={500}>{personaName}</Text>
          <Badge size="sm" color={color} variant="dot">Thinking</Badge>
          {onCancel && (
            <Tooltip label="Cancel" withArrow>
              <ActionIcon
                color="red"
                variant="subtle"
                onClick={onCancel}
                size="sm"
              >
                <IconX size={16} />
              </ActionIcon>
            </Tooltip>
          )}
        </Group>
        <Progress value={100} animated size="xs" color={color} />
      </Stack>
    </Paper>
  );
}
