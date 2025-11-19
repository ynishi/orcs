import React, { useState } from 'react';
import { Paper, Text, Group, Badge, Avatar, Box, ActionIcon, CopyButton, Tooltip, Anchor, Image, Stack } from '@mantine/core';
import { IconDeviceFloppy, IconRocket, IconCommand, IconUser } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { Message, getMessageStyle } from '../../types/message';
import { MarkdownRenderer } from '../markdown/MarkdownRenderer';

interface MessageItemProps {
  message: Message;
  onSaveToWorkspace?: (message: Message) => Promise<void>;
  onExecuteAsTask?: (message: Message) => Promise<void>;
  onCreateSlashCommand?: (message: Message) => void;
  onCreatePersona?: (message: Message) => void;
  workspaceRootPath?: string;
}

// 長いテキストを折りたたむしきい値（文字数）
const COLLAPSE_THRESHOLD = 200;

// メンションをハイライト表示するヘルパー
function renderTextWithMentions(text: string) {
  // スペース以外の文字にマッチ（日本語、ハイフン、記号などもサポート）
  const mentionRegex = /@(\S+)/g;
  const parts: (string | React.ReactElement)[] = [];
  let lastIndex = 0;
  let match;
  let key = 0;

  while ((match = mentionRegex.exec(text)) !== null) {
    // メンション前のテキスト
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }

    // メンション部分（_ をスペースに変換して表示）
    const mentionName = match[1];
    const displayName = mentionName.replace(/_/g, ' ');
    parts.push(
      <Badge
        key={key++}
        size="sm"
        variant="light"
        color="green"
        style={{ margin: '0 2px' }}
      >
        @{displayName}
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

// メッセージタイプを表示用ラベルに変換
function formatMessageTypeLabel(type: string): string {
  return type.replace(/_/g, ' ').toUpperCase();
}

// バックエンド名をフォーマット
function formatBackendName(backend: string): string {
  const backendNames: Record<string, string> = {
    claude_cli: 'Claude CLI',
    claude_api: 'Claude API',
    gemini_cli: 'Gemini CLI',
    gemini_api: 'Gemini API',
    open_ai_api: 'OpenAI API',
    codex_cli: 'Codex CLI',
  };
  return backendNames[backend] || backend;
}

// モデル名をフォーマット（短縮版）
function formatModelName(modelName: string | null | undefined): string | null {
  if (!modelName) return null;

  // Claude models
  if (modelName.includes('claude-sonnet-4-5')) return 'Sonnet 4.5';
  if (modelName.includes('claude-sonnet-4')) return 'Sonnet 4';
  if (modelName.includes('claude-opus')) return 'Opus';
  if (modelName.includes('claude-haiku')) return 'Haiku';

  // Gemini models
  if (modelName.includes('gemini-2.5-flash')) return 'Gemini 2.5 Flash';
  if (modelName.includes('gemini-2.0-flash')) return 'Gemini 2.0 Flash';
  if (modelName.includes('gemini-pro')) return 'Gemini Pro';

  // OpenAI models
  if (modelName.includes('gpt-4o')) return 'GPT-4o';
  if (modelName.includes('gpt-4')) return 'GPT-4';
  if (modelName.includes('gpt-3.5')) return 'GPT-3.5';

  // Return first 20 chars if no match
  return modelName.slice(0, 20);
}

export function MessageItem({ message, onSaveToWorkspace, onExecuteAsTask, onCreateSlashCommand, onCreatePersona, workspaceRootPath }: MessageItemProps) {
  const style = getMessageStyle(message.type);
  const [isHovered, setIsHovered] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);

  // Apply base color with opacity for subtle background tinting
  const backgroundColor = message.baseColor && message.type === 'ai'
    ? `${message.baseColor}20` // Add alpha channel for ~12% opacity
    : style.backgroundColor;

  // テキストが長い場合の判定
  const isLongText = message.text.length > COLLAPSE_THRESHOLD;
  const displayText = isLongText && !isExpanded
    ? message.text.slice(0, COLLAPSE_THRESHOLD) + '...'
    : message.text;

  // Handle file save from markdown code blocks
  const handleSaveFile = async (path: string, content: string) => {
    await invoke('save_code_snippet', {
      filePath: path,
      content: content,
    });
  };

  // バッジタイプのメッセージ（System, Error, Command, Task）
  if (style.showBadge) {
    return (
      <Box
        id={`message-${message.id}`}
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
            backgroundColor,
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
            >
              {formatMessageTypeLabel(message.type)}
            </Badge>
            <Box style={{ flex: 1 }}>
              <Text
                size="sm"
                c={style.textColor}
                style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}
              >
                {renderTextWithMentions(displayText)}
              </Text>

              {/* 折りたたみトグル */}
              {isLongText && (
                <Anchor
                  size="xs"
                  c="dimmed"
                  onClick={() => setIsExpanded(!isExpanded)}
                  style={{
                    cursor: 'pointer',
                    display: 'inline-block',
                    marginTop: '4px',
                    textDecoration: 'underline'
                  }}
                >
                  {isExpanded ? 'Show less' : 'Show more'}
                </Anchor>
              )}

              {/* 添付ファイルプレビュー */}
              {message.attachments && message.attachments.length > 0 && (
                <Stack gap="xs" mt="md">
                  {message.attachments.map((attachment, index) => (
                    <Box key={index}>
                      {attachment.mimeType.startsWith('image/') && attachment.data ? (
                        <Box>
                          <Image
                            src={`data:${attachment.mimeType};base64,${attachment.data}`}
                            alt={attachment.name}
                            radius="md"
                            style={{ maxWidth: '400px', maxHeight: '400px' }}
                          />
                          <Text size="xs" c="dimmed" mt={4}>
                            📎 {attachment.name} ({(attachment.size / 1024).toFixed(1)} KB)
                          </Text>
                        </Box>
                      ) : (
                        <Badge size="lg" variant="light" leftSection="📎">
                          {attachment.name} ({(attachment.size / 1024).toFixed(1)} KB)
                        </Badge>
                      )}
                    </Box>
                  ))}
                </Stack>
              )}

              <Text size="xs" c="dimmed" mt={4}>
                {message.timestamp.toLocaleTimeString()}
              </Text>
            </Box>

            {/* アクションボタン */}
            {isHovered && (
              <Group gap={4}>
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

                {onSaveToWorkspace && (
                  <Tooltip label="Save to Workspace" withArrow>
                    <ActionIcon
                      color="blue"
                      variant="subtle"
                      onClick={() => onSaveToWorkspace(message)}
                      size="sm"
                    >
                      <IconDeviceFloppy size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}

                {onCreateSlashCommand && (
                  <Tooltip label="Create Slash Command" withArrow>
                    <ActionIcon
                      color="grape"
                      variant="subtle"
                      onClick={() => onCreateSlashCommand(message)}
                      size="sm"
                    >
                      <IconCommand size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}

                {onCreatePersona && (
                  <Tooltip label="Create Persona" withArrow>
                    <ActionIcon
                      color="pink"
                      variant="subtle"
                      onClick={() => onCreatePersona(message)}
                      size="sm"
                    >
                      <IconUser size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}
              </Group>
            )}
          </Group>
        </Paper>
      </Box>
    );
  }

  // 通常のメッセージ（User, AI）
  return (
    <Box
      id={`message-${message.id}`}
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
            backgroundColor,
            flex: 1,
            position: 'relative',
          }}
        >
          <Group justify="space-between" mb={4}>
            <Group gap={4}>
              {message.icon && (
                <Text size="sm">{message.icon}</Text>
              )}
              <Text fw={600} size="sm" c="dimmed">
                {message.author}
              </Text>
              {message.backend && (
                <Badge size="xs" color="gray" variant="outline">
                  {formatBackendName(message.backend)}
                </Badge>
              )}
              {message.modelName && formatModelName(message.modelName) && (
                <Badge size="xs" color="blue" variant="outline">
                  {formatModelName(message.modelName)}
                </Badge>
              )}
            </Group>

            {/* アクションボタン */}
            {isHovered && (
              <Group gap={4}>
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

                {onSaveToWorkspace && (
                  <Tooltip label="Save to Workspace" withArrow>
                    <ActionIcon
                      color="blue"
                      variant="subtle"
                      onClick={() => onSaveToWorkspace(message)}
                      size="sm"
                    >
                      <IconDeviceFloppy size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}

                {onCreateSlashCommand && (
                  <Tooltip label="Create Slash Command" withArrow>
                    <ActionIcon
                      color="grape"
                      variant="subtle"
                      onClick={() => onCreateSlashCommand(message)}
                      size="sm"
                    >
                      <IconCommand size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}

                {onCreatePersona && (
                  <Tooltip label="Create Persona" withArrow>
                    <ActionIcon
                      color="pink"
                      variant="subtle"
                      onClick={() => onCreatePersona(message)}
                      size="sm"
                    >
                      <IconUser size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}

                {onExecuteAsTask && (message.type === 'ai' || message.type === 'user') && (
                  <Tooltip label="Execute as Task" withArrow>
                    <ActionIcon
                      color="violet"
                      variant="subtle"
                      onClick={() => onExecuteAsTask(message)}
                      size="sm"
                    >
                      <IconRocket size={16} />
                    </ActionIcon>
                  </Tooltip>
                )}
              </Group>
            )}
          </Group>

          <Box>
            <MarkdownRenderer
              content={message.text}
              onSaveFile={handleSaveFile}
              workspaceRootPath={workspaceRootPath}
            />
          </Box>

          {/* 添付ファイルプレビュー */}
          {message.attachments && message.attachments.length > 0 && (
            <Stack gap="xs" mt="md">
              {message.attachments.map((attachment, index) => (
                <Box key={index}>
                  {attachment.mimeType.startsWith('image/') && attachment.data ? (
                    <Box>
                      <Image
                        src={`data:${attachment.mimeType};base64,${attachment.data}`}
                        alt={attachment.name}
                        radius="md"
                        style={{ maxWidth: '400px', maxHeight: '400px' }}
                      />
                      <Text size="xs" c="dimmed" mt={4}>
                        📎 {attachment.name} ({(attachment.size / 1024).toFixed(1)} KB)
                      </Text>
                    </Box>
                  ) : (
                    <Badge size="lg" variant="light" leftSection="📎">
                      {attachment.name} ({(attachment.size / 1024).toFixed(1)} KB)
                    </Badge>
                  )}
                </Box>
              ))}
            </Stack>
          )}

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
