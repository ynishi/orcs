import { Paper, Stack, Group, Avatar, Text, Badge, Progress } from '@mantine/core';

interface ThinkingIndicatorProps {
  personaName: string;
  personaInitial?: string;
  color?: string;
}

export function ThinkingIndicator({
  personaName,
  personaInitial,
  color = 'blue'
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
        </Group>
        <Progress value={100} animated size="xs" color={color} />
      </Stack>
    </Paper>
  );
}
