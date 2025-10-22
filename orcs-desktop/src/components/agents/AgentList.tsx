import { Stack, ScrollArea, Group, Text, Box, Badge, Checkbox } from '@mantine/core';
import { Agent, getAgentIcon, getAgentColor } from '../../types/agent';

interface AgentListProps {
  agents: Agent[];
  onAgentToggle?: (agentId: string) => void;
}

export function AgentList({ agents, onAgentToggle }: AgentListProps) {
  const activeAgents = agents.filter(a => a.status !== 'offline');
  const offlineAgents = agents.filter(a => a.status === 'offline');

  const renderAgent = (agent: Agent) => (
    <Group
      key={agent.id}
      gap="sm"
      wrap="nowrap"
      p="xs"
      style={{
        borderRadius: '8px',
        backgroundColor: agent.isActive ? 'rgba(64, 192, 87, 0.1)' : 'transparent',
        transition: 'background-color 0.15s ease',
      }}
    >
      {/* チェックボックス */}
      <Checkbox
        checked={agent.isActive}
        onChange={() => onAgentToggle?.(agent.id)}
        size="sm"
        color="green"
      />

      {/* ステータスインジケーター */}
      <Text size="lg">{getAgentIcon(agent.status)}</Text>

      {/* エージェント情報 */}
      <Box style={{ flex: 1, minWidth: 0 }}>
        <Group gap="xs" mb={4}>
          <Text size="sm" fw={600} truncate>
            {agent.name}
          </Text>
          <Badge size="xs" color={getAgentColor(agent.status)} variant="light">
            {agent.status}
          </Badge>
        </Group>
        <Text size="xs" c="dimmed" truncate>
          {agent.description}
        </Text>
        {agent.lastActive && (
          <Text size="xs" c="dimmed" mt={2}>
            Last: {agent.lastActive.toLocaleTimeString()}
          </Text>
        )}
      </Box>
    </Group>
  );

  const participatingAgents = agents.filter(a => a.isActive);

  return (
    <Stack gap="md" h="100%">
      {/* ヘッダー */}
      <Group justify="space-between" px="md" pt="md">
        <Text size="lg" fw={700}>
          Agents
        </Text>
        <Text size="sm" c="dimmed">
          {participatingAgents.length} participating
        </Text>
      </Group>

      {/* エージェントリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {/* アクティブエージェント */}
          {activeAgents.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                ACTIVE
              </Text>
              <Stack gap={4}>
                {activeAgents.map(renderAgent)}
              </Stack>
            </Box>
          )}

          {/* オフラインエージェント */}
          {offlineAgents.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                OFFLINE
              </Text>
              <Stack gap={4}>
                {offlineAgents.map(renderAgent)}
              </Stack>
            </Box>
          )}

          {/* 空の状態 */}
          {agents.length === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                No agents available
              </Text>
              <Text size="xs" c="dimmed" mt="xs">
                Agents will appear here
              </Text>
            </Box>
          )}
        </Stack>
      </ScrollArea>

      {/* フッター */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          {agents.length} total agents
        </Text>
      </Box>
    </Stack>
  );
}
