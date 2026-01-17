import React, { useState } from 'react';
import { Paper, Text, Group, Badge, Avatar, Box, ActionIcon, CopyButton, Tooltip, Anchor, Image, Stack, Collapse, Code } from '@mantine/core';
import { IconDeviceFloppy, IconRocket, IconCommand, IconUser, IconCheck, IconClipboard, IconChevronDown, IconChevronUp, IconBug, IconRefresh, IconSquareCheck, IconSquare } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { Message, getMessageStyle } from '../../types/message';
import { MarkdownRenderer } from '../markdown/MarkdownRenderer';
import { useDebugStore } from '../../stores/debugStore';

interface MessageItemProps {
  message: Message;
  onSaveToWorkspace?: (message: Message) => Promise<void>;
  onExecuteAsTask?: (message: Message) => Promise<void>;
  onCreateSlashCommand?: (message: Message) => void;
  onCreatePersona?: (message: Message) => void;
  onRedo?: (message: Message) => void;
  onCloseMessage?: (message: Message, isClosed: boolean) => Promise<void>;
  workspaceRootPath?: string;
}

// 長いテキストを折りたたむしきい値（文字数）
const COLLAPSE_THRESHOLD = 200;

// メンションとSlashコマンドをハイライト表示するヘルパー
function renderTextWithMentions(text: string) {
  const parts: (string | React.ReactElement)[] = [];
  let key = 0;

  // First, handle Slash commands: <Slash> <Name>command</Name> <Args>args</Args> </Slash>
  const slashRegex = /<Slash>\s*<Name>(.*?)<\/Name>\s*<Args>(.*?)<\/Args>\s*<\/Slash>/g;
  const slashMatches: Array<{ index: number; length: number; name: string; args: string }> = [];
  let slashMatch;

  while ((slashMatch = slashRegex.exec(text)) !== null) {
    slashMatches.push({
      index: slashMatch.index,
      length: slashMatch[0].length,
      name: slashMatch[1],
      args: slashMatch[2],
    });
  }

  // Process text segment by segment
  let currentIndex = 0;

  for (const slash of slashMatches) {
    // Text before slash command
    const beforeText = text.slice(currentIndex, slash.index);
    if (beforeText) {
      // Process mentions in the before text
      processMentions(beforeText, parts, key);
    }

    // Add slash command badge
    parts.push(
      <Badge
        key={`slash-${key++}`}
        component="span"
        size="sm"
        variant="filled"
        color="blue"
        style={{ margin: '0 2px' }}
      >
        /{slash.name} {slash.args}
      </Badge>
    );

    currentIndex = slash.index + slash.length;
  }

  // Remaining text after all slash commands
  const remainingText = text.slice(currentIndex);
  if (remainingText) {
    processMentions(remainingText, parts, key);
  }

  return parts.length > 0 ? parts : text;
}

// Helper function to process mentions in a text segment
function processMentions(text: string, parts: (string | React.ReactElement)[], startKey: number) {
  // スペース以外の文字にマッチ（日本語、ハイフン、記号などもサポート）
  const mentionRegex = /@(\S+)/g;
  let lastIndex = 0;
  let match;
  let key = startKey;

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
        key={`mention-${key++}`}
        component="span"
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

export function MessageItem({ message, onSaveToWorkspace, onExecuteAsTask, onCreateSlashCommand, onCreatePersona, onRedo, onCloseMessage, workspaceRootPath }: MessageItemProps) {
  const style = getMessageStyle(message.type);
  const [isHovered, setIsHovered] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [debugExpanded, setDebugExpanded] = useState(false);
  const [closedExpanded, setClosedExpanded] = useState(false);

  const { debugSettings } = useDebugStore();

  // Check if message is "closed" (starts with ~~)
  const isClosed = message.text.startsWith('~~');
  // Get the actual content without ~~ prefix
  const actualText = isClosed ? message.text.slice(2) : message.text;

  // Apply base color with opacity for subtle background tinting
  // Closed messages get a more muted background
  const backgroundColor = isClosed
    ? '#f0f0f0'
    : message.baseColor && message.type === 'ai'
      ? `${message.baseColor}20` // Add alpha channel for ~12% opacity
      : style.backgroundColor;

  // For closed user messages: show only first line
  const firstLine = actualText.split('\n')[0];

  // For system messages only: truncate to COLLAPSE_THRESHOLD (200 chars)
  const isSystemLongText = actualText.length > COLLAPSE_THRESHOLD;
  const systemTruncatedText = isSystemLongText && !isExpanded
    ? actualText.slice(0, COLLAPSE_THRESHOLD) + '...'
    : actualText;

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
              {/* Use MarkdownRenderer for action_result to support tables, etc. */}
              {message.type === 'action_result' ? (
                <MarkdownRenderer
                  content={actualText}
                  onSaveFile={handleSaveFile}
                  workspaceRootPath={workspaceRootPath}
                />
              ) : (
                <Text
                  size="sm"
                  c={style.textColor}
                  style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}
                >
                  {/* Only system messages use 200-char truncation */}
                  {renderTextWithMentions(message.type === 'system' ? systemTruncatedText : actualText)}
                </Text>
              )}

              {/* 折りたたみトグル - only for system messages */}
              {message.type === 'system' && isSystemLongText && (
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
                      {attachment.mimeType?.startsWith('image/') && attachment.data ? (
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

              {/* Debug情報表示（デバッグモード時のみ） */}
              {debugSettings?.enableLlmDebug && message.metadata?.llmDebugInfo && (
                <Box mt="md">
                  <Anchor
                    size="xs"
                    c="orange"
                    onClick={() => setDebugExpanded(!debugExpanded)}
                    style={{
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '4px',
                    }}
                  >
                    <IconBug size={14} />
                    <Text size="xs" fw={600}>LLM Debug Info</Text>
                    {debugExpanded ? <IconChevronUp size={14} /> : <IconChevronDown size={14} />}
                  </Anchor>

                  <Collapse in={debugExpanded}>
                    <Stack gap="xs" mt="xs">
                      {message.metadata.llmDebugInfo.model && (
                        <Box>
                          <Text size="xs" fw={600} c="dimmed">Model:</Text>
                          <Badge size="sm" variant="light" color="orange">
                            {message.metadata.llmDebugInfo.model}
                          </Badge>
                        </Box>
                      )}

                      <Box>
                        <Text size="xs" fw={600} c="dimmed" mb={4}>Prompt:</Text>
                        <Code
                          block
                          style={{
                            maxHeight: '200px',
                            overflow: 'auto',
                            fontSize: '11px',
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-word',
                          }}
                        >
                          {message.metadata.llmDebugInfo.prompt}
                        </Code>
                      </Box>

                      <Box>
                        <Text size="xs" fw={600} c="dimmed" mb={4}>Raw Response:</Text>
                        <Code
                          block
                          style={{
                            maxHeight: '200px',
                            overflow: 'auto',
                            fontSize: '11px',
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-word',
                          }}
                        >
                          {message.metadata.llmDebugInfo.rawResponse}
                        </Code>
                      </Box>
                    </Stack>
                  </Collapse>
                </Box>
              )}
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
                        {copied ? <IconCheck size={16} /> : <IconClipboard size={16} />}
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

                {onRedo && message.type === 'command' && (
                  <Tooltip label="Redo" withArrow>
                    <ActionIcon
                      color="orange"
                      variant="subtle"
                      onClick={() => onRedo(message)}
                      size="sm"
                    >
                      <IconRefresh size={16} />
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
                        {copied ? <IconCheck size={16} /> : <IconClipboard size={16} />}
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

                {/* Close/Open button for user messages */}
                {onCloseMessage && message.type === 'user' && (
                  <Tooltip label={isClosed ? 'Open' : 'Close'} withArrow>
                    <ActionIcon
                      color={isClosed ? 'green' : 'gray'}
                      variant="subtle"
                      onClick={() => onCloseMessage(message, !isClosed)}
                      size="sm"
                    >
                      {isClosed ? <IconSquareCheck size={16} /> : <IconSquare size={16} />}
                    </ActionIcon>
                  </Tooltip>
                )}
              </Group>
            )}
          </Group>

          {/* Content area - clickable to expand/collapse for closed messages */}
          <Box
            onClick={isClosed ? () => setClosedExpanded(!closedExpanded) : undefined}
            style={isClosed ? { cursor: 'pointer', opacity: 0.7 } : undefined}
          >
            {isClosed && (
              <Badge size="xs" variant="light" color="gray" mb="xs">
                ✓ Closed
              </Badge>
            )}
            <MarkdownRenderer
              content={isClosed && !closedExpanded ? firstLine + '...' : actualText}
              onSaveFile={handleSaveFile}
              workspaceRootPath={workspaceRootPath}
            />
          </Box>

          {/* 添付ファイルプレビュー */}
          {message.attachments && message.attachments.length > 0 && (
            <Stack gap="xs" mt="md">
              {message.attachments.map((attachment, index) => (
                <Box key={index}>
                  {attachment.mimeType?.startsWith('image/') && attachment.data ? (
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
