import { Paper, Stack, Group, Text, Box, ScrollArea, Kbd } from '@mantine/core';
import { CommandDefinition } from '../../types/command';
import { memo, useEffect, useRef } from 'react';

interface CommandSuggestionsProps {
  commands: CommandDefinition[];
  selectedIndex: number;
  onSelect: (command: CommandDefinition) => void;
  onHover: (index: number) => void;
}

export const CommandSuggestions = memo(function CommandSuggestions({
  commands,
  selectedIndex,
  onSelect,
  onHover,
}: CommandSuggestionsProps) {
  const selectedItemRef = useRef<HTMLDivElement>(null);

  // 選択項目が変更されたら自動スクロール
  useEffect(() => {
    if (selectedItemRef.current) {
      selectedItemRef.current.scrollIntoView({
        block: 'nearest',
        behavior: 'smooth',
      });
    }
  }, [selectedIndex]);

  if (commands.length === 0) {
    return null;
  }

  return (
    <Paper
      shadow="lg"
      p="xs"
      radius="md"
      withBorder
      style={{
        position: 'absolute',
        bottom: '100%',
        left: 0,
        right: 0,
        marginBottom: '8px',
        maxHeight: '280px',
        zIndex: 1000,
        backgroundColor: 'white',
        overflow: 'hidden',
      }}
    >
      <ScrollArea style={{ maxHeight: '260px' }} type="auto">
        <Stack gap="xs">
          {/* ヘッダー */}
          <Group justify="space-between" px="xs">
            <Text size="xs" fw={600} c="dimmed">
              Available Commands
            </Text>
            <Group gap={4}>
              <Kbd size="xs">↑↓</Kbd>
              <Text size="xs" c="dimmed">
                navigate
              </Text>
              <Kbd size="xs">Tab</Kbd>
              <Text size="xs" c="dimmed">
                select
              </Text>
            </Group>
          </Group>

          {/* コマンドリスト */}
          {commands.map((command, index) => (
            <Box
              key={command.name}
              ref={selectedIndex === index ? selectedItemRef : null}
              p="sm"
              style={{
                backgroundColor: selectedIndex === index ? '#e7f5ff' : 'transparent',
                borderRadius: '8px',
                cursor: 'pointer',
                transition: 'background-color 0.15s ease',
              }}
              onMouseEnter={() => onHover(index)}
              onClick={() => onSelect(command)}
            >
              <Group gap="md" wrap="nowrap">
                {/* アイコン */}
                <Text size="xl" style={{ minWidth: '32px', textAlign: 'center' }}>
                  {command.icon}
                </Text>

                {/* コマンド情報 */}
                <Box style={{ flex: 1 }}>
                  <Group gap="xs" mb={4}>
                    <Text size="sm" fw={600} c="blue">
                      /{command.name}
                    </Text>
                    <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace' }}>
                      {command.usage}
                    </Text>
                  </Group>
                  <Text size="xs" c="dimmed">
                    {command.description}
                  </Text>
                  {command.argsDescription && (
                    <Text size="xs" c="dimmed" mt={4} style={{ opacity: 0.8 }}>
                      Args: {command.argsDescription}
                    </Text>
                  )}
                  {command.examples && command.examples.length > 0 && (
                    <Text size="xs" c="dimmed" mt={4} style={{ fontStyle: 'italic' }}>
                      e.g. {command.examples[0]}
                    </Text>
                  )}
                </Box>

                {/* 選択インジケーター */}
                {selectedIndex === index && (
                  <Text size="sm" c="blue" fw={600}>
                    →
                  </Text>
                )}
              </Group>
            </Box>
          ))}
        </Stack>
      </ScrollArea>
    </Paper>
  );
});
