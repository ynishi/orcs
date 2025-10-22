import { Paper, Stack, Group, Text, Box, Badge, UnstyledButton } from '@mantine/core';
import { Agent, getAgentIcon, getAgentColor } from '../../types/agent';
import { useEffect, useRef } from 'react';

interface AgentSuggestionsProps {
  agents: Agent[];
  filter: string;
  selectedIndex: number;
  onSelect: (agent: Agent) => void;
  position?: { top: number; left: number };
}

export function AgentSuggestions({
  agents,
  filter,
  selectedIndex,
  onSelect,
  position,
}: AgentSuggestionsProps) {
  const selectedRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to selected item
  useEffect(() => {
    if (selectedRef.current) {
      selectedRef.current.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
      });
    }
  }, [selectedIndex]);

  if (agents.length === 0) return null;

  return (
    <Paper
      shadow="md"
      p="xs"
      style={{
        position: 'absolute',
        bottom: position?.top || 60,
        left: position?.left || 0,
        width: '400px',
        maxHeight: '300px',
        overflowY: 'auto',
        zIndex: 1000,
      }}
    >
      <Text size="xs" c="dimmed" mb="xs" px="xs">
        Mention an agent
      </Text>
      <Stack gap={4}>
        {agents.map((agent, index) => (
          <UnstyledButton
            key={agent.id}
            ref={index === selectedIndex ? selectedRef : null}
            onClick={() => onSelect(agent)}
            style={{
              padding: '8px',
              borderRadius: '6px',
              backgroundColor: index === selectedIndex ? '#e7f5ff' : 'transparent',
              transition: 'background-color 0.15s ease',
            }}
          >
            <Group gap="sm" wrap="nowrap">
              {/* ステータスインジケーター */}
              <Text size="md">{getAgentIcon(agent.status)}</Text>

              {/* エージェント情報 */}
              <Box style={{ flex: 1, minWidth: 0 }}>
                <Group gap="xs" mb={2}>
                  <Text size="sm" fw={600}>
                    @{agent.name}
                  </Text>
                  <Badge size="xs" color={getAgentColor(agent.status)} variant="light">
                    {agent.status}
                  </Badge>
                  {agent.isActive && (
                    <Badge size="xs" color="green" variant="filled">
                      participating
                    </Badge>
                  )}
                </Group>
                <Text size="xs" c="dimmed" truncate>
                  {agent.description}
                </Text>
              </Box>
            </Group>
          </UnstyledButton>
        ))}
      </Stack>
    </Paper>
  );
}
