import React, { useState } from 'react';
import { Paper, Text, Group, Badge, Avatar, Box, ActionIcon, CopyButton, Tooltip } from '@mantine/core';
import { Message, getMessageStyle } from '../../types/message';

interface MessageItemProps {
  message: Message;
}

// メンションをハイライト表示するヘルパー
function renderTextWithMentions(text: string) {
  const mentionRegex = /@(\w+)/g;
  const parts: (string | React.ReactElement)[] = [];
  let lastIndex = 0;
  let match;
  let key = 0;

  while ((match = mentionRegex.exec(text)) !== null) {
    // メンション前のテキスト
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }

    // メンション部分
    parts.push(
      <Badge
        key={key++}
        size="sm"
        variant="light"
        color="green"
        style={{ margin: '0 2px' }}
      >
        @{match[1]}
      </Badge>
    );

    lastIndex = match.index + match[0].length;
  }

  // 残りのテキスト
  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts.length > 0 ? parts : text;
}

export function MessageItem({ message }: MessageItemProps) {
  const style = getMessageStyle(message.type);
  const [isHovered, setIsHovered] = useState(false);

  // バッジタイプのメッセージ（System, Error, Command, Task）
  if (style.showBadge) {
    return (
      <Box
        style={{
          display: 'flex',
          justifyContent: style.align === 'center' ? 'center' : 'flex-start',
          marginBottom: '8px',
        }}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <Paper
          p="xs"
          radius="md"
          style={{
            backgroundColor: style.backgroundColor,
            borderLeft: style.borderColor ? `4px solid ${style.borderColor}` : undefined,
            maxWidth: '80%',
            display: 'inline-block',
            position: 'relative',
          }}
        >
          <Group gap="xs" wrap="nowrap">
            <Badge
              color={style.iconColor || style.textColor}
              variant="filled"
              size="sm"
              style={{ textTransform: 'uppercase' }}
            >
              {message.type}
            </Badge>
            <Box style={{ flex: 1 }}>
              <Text
                size="sm"
                c={style.textColor}
                style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}
              >
                {renderTextWithMentions(message.text)}
              </Text>
              <Text size="xs" c="dimmed" mt={4}>
                {message.timestamp.toLocaleTimeString()}
              </Text>
            </Box>

            {/* コピーボタン */}
            {isHovered && (
              <CopyButton value={message.text}>
                {({ copied, copy }) => (
                  <Tooltip label={copied ? 'Copied!' : 'Copy'} withArrow>
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
            )}
          </Group>
        </Paper>
      </Box>
    );
  }

  // 通常のメッセージ（User, AI）
  return (
    <Box
      style={{
        display: 'flex',
        justifyContent: style.align === 'right' ? 'flex-end' : 'flex-start',
        marginBottom: '12px',
      }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <Group gap="sm" wrap="nowrap" style={{ maxWidth: '80%' }}>
        {style.showAvatar && style.align === 'left' && (
          <Avatar
            color={message.type === 'ai' ? 'blue' : 'green'}
            radius="xl"
            size="sm"
          >
            {message.author.charAt(0).toUpperCase()}
          </Avatar>
        )}

        <Paper
          p="md"
          radius="lg"
          style={{
            backgroundColor: style.backgroundColor,
            flex: 1,
            position: 'relative',
          }}
        >
          <Group justify="space-between" mb={4}>
            <Text fw={600} size="sm" c="dimmed">
              {message.author}
            </Text>

            {/* コピーボタン */}
            {isHovered && (
              <CopyButton value={message.text}>
                {({ copied, copy }) => (
                  <Tooltip label={copied ? 'Copied!' : 'Copy'} withArrow>
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
            )}
          </Group>

          <Text
            size="sm"
            c={style.textColor}
            style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}
          >
            {renderTextWithMentions(message.text)}
          </Text>
          <Text size="xs" c="dimmed" mt={8}>
            {message.timestamp.toLocaleTimeString()}
          </Text>
        </Paper>

        {style.showAvatar && style.align === 'right' && (
          <Avatar
            color={message.type === 'user' ? 'cyan' : 'green'}
            radius="xl"
            size="sm"
          >
            {message.author.charAt(0).toUpperCase()}
          </Avatar>
        )}
      </Group>
    </Box>
  );
}
