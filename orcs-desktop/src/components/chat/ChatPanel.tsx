/**
 * ChatPanel - 1つのタブ（セッション）のチャット画面を管理
 * TabContextから状態を取得し、軽量なプレゼンテーション層として機能
 */
import { useRef, useEffect } from 'react';
import {
  Textarea,
  Button,
  ScrollArea,
  Stack,
  Box,
  Group,
  CopyButton,
  ActionIcon,
  Tooltip,
  Badge,
  CloseButton,
  Paper,
  Text,
} from '@mantine/core';
import { MessageItem } from './MessageItem';
import { StatusBar } from './StatusBar';
import { CommandSuggestions } from './CommandSuggestions';
import { AgentSuggestions } from './AgentSuggestions';
import { ThinkingIndicator } from './ThinkingIndicator';
import type { SessionTab } from '../../context/TabContext';
import type { StatusInfo } from '../../types/status';
import type { GitInfo } from '../../types/git';
import type { Workspace } from '../../types/workspace';
import type { CommandDefinition } from '../../types/command';
import type { Agent } from '../../types/agent';
import type { PersonaConfig } from '../../types/agent';

interface ChatPanelProps {
  tab: SessionTab;
  status: StatusInfo;
  userNickname: string;
  gitInfo: GitInfo;
  autoMode: boolean;
  conversationMode: string;
  talkStyle: string | null;
  executionStrategy: string;
  personas: PersonaConfig[];
  activeParticipantIds: string[];
  workspace: Workspace | null;
  
  // サジェスト関連
  showSuggestions: boolean;
  filteredCommands: CommandDefinition[];
  selectedSuggestionIndex: number;
  showAgentSuggestions: boolean;
  filteredAgents: Agent[];
  selectedAgentIndex: number;
  
  // イベントハンドラー
  onSubmit: (e: React.FormEvent) => void;
  onInputChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  onFileSelect: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onRemoveFile: (index: number) => void;
  onDragOver: (e: React.DragEvent) => void;
  onDragLeave: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent) => void;
  onSaveMessageToWorkspace: (message: import('../../types/message').Message) => Promise<void>;
  onExecuteAsTask: (message: import('../../types/message').Message) => Promise<void>;
  onAutoModeChange: (autoMode: boolean) => void;
  onSelectCommand: (command: CommandDefinition) => void;
  onSelectAgent: (agent: Agent) => void;
  onHoverSuggestion: (index: number) => void;
}

export function ChatPanel({
  tab,
  status,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  // @ts-ignore - userNickname may be used in future features
  userNickname,
  gitInfo,
  autoMode,
  conversationMode,
  talkStyle,
  executionStrategy,
  personas,
  activeParticipantIds,
  workspace,
  showSuggestions,
  filteredCommands,
  selectedSuggestionIndex,
  showAgentSuggestions,
  filteredAgents,
  selectedAgentIndex,
  onSubmit,
  onInputChange,
  onKeyDown,
  onFileSelect,
  onRemoveFile,
  onDragOver,
  onDragLeave,
  onDrop,
  onSaveMessageToWorkspace,
  onExecuteAsTask,
  onAutoModeChange,
  onSelectCommand,
  onSelectAgent,
  onHoverSuggestion,
}: ChatPanelProps) {
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    if (viewport.current) {
      viewport.current.scrollTo({
        top: viewport.current.scrollHeight,
        behavior: 'smooth',
      });
    }
  }, [tab.messages]);

  const getThreadAsText = (): string => {
    return tab.messages
      .map((msg) => {
        const time = msg.timestamp.toLocaleString();
        return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
      })
      .join('\n---\n\n');
  };

  return (
    <Stack gap={0} style={{ height: '100%', display: 'flex', flexDirection: 'column', minHeight: 0 }}>
      {/* メッセージエリア */}
      <Box
        style={{ flex: 1, position: 'relative', minHeight: 0 }}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
      >
        <ScrollArea h="100%" viewportRef={viewport}>
          <Stack gap="sm" p="md">
            {tab.messages.map((message) => (
              <MessageItem
                key={message.id}
                message={message}
                onSaveToWorkspace={onSaveMessageToWorkspace}
                onExecuteAsTask={onExecuteAsTask}
                workspaceRootPath={workspace?.rootPath}
              />
            ))}
            {tab.isAiThinking && <ThinkingIndicator personaName={tab.thinkingPersona} />}
          </Stack>
        </ScrollArea>

        {tab.isDragging && (
          <Paper
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: 'rgba(30, 144, 255, 0.1)',
              border: '3px dashed #1e90ff',
              borderRadius: '12px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              zIndex: 1000,
              pointerEvents: 'none',
            }}
          >
            <Text size="xl" fw={700} c="blue">
              📁 Drop files here
            </Text>
          </Paper>
        )}
      </Box>

      {/* 入力フォーム */}
      <form onSubmit={onSubmit}>
        <Stack gap="xs">
          {tab.attachedFiles.length > 0 && (
            <Group gap="xs">
              {tab.attachedFiles.map((file, index) => (
                <Badge
                  key={index}
                  size="lg"
                  variant="light"
                  rightSection={
                    <CloseButton size="xs" onClick={() => onRemoveFile(index)} />
                  }
                >
                  📎 {file.name}
                </Badge>
              ))}
            </Group>
          )}

          <Box style={{ position: 'relative' }}>
            {showSuggestions && (
              <CommandSuggestions
                commands={filteredCommands}
                selectedIndex={selectedSuggestionIndex}
                onSelect={onSelectCommand}
                onHover={onHoverSuggestion}
              />
            )}

            {showAgentSuggestions && (
              <AgentSuggestions
                agents={filteredAgents}
                selectedIndex={selectedAgentIndex}
                onSelect={onSelectAgent}
              />
            )}

            <Textarea
              ref={textareaRef}
              value={tab.input}
              onChange={(e) => onInputChange(e.currentTarget.value)}
              onKeyDown={onKeyDown}
              placeholder={
                executionStrategy === 'mentioned'
                  ? 'Type @PersonaName to mention, or /help for commands... (⌘+Enter to send)'
                  : 'Type your message or /help for commands... (⌘+Enter to send)'
              }
              size="md"
              minRows={1}
              maxRows={4}
              autosize
            />
          </Box>

          <Group gap="xs">
            <Tooltip label="Attach files">
              <Button variant="light" size="sm" component="label" leftSection="📎">
                Attach
                <input type="file" multiple hidden onChange={onFileSelect} />
              </Button>
            </Tooltip>

            <Button type="submit" style={{ flex: 1 }}>
              Send
            </Button>

            <Tooltip label={autoMode ? 'Stop AUTO mode' : 'Start AUTO mode'}>
              <ActionIcon
                color={autoMode ? 'red' : 'green'}
                variant={autoMode ? 'filled' : 'light'}
                onClick={() => onAutoModeChange(!autoMode)}
                size="lg"
              >
                {autoMode ? '⏹️' : '▶️'}
              </ActionIcon>
            </Tooltip>

            <CopyButton value={getThreadAsText()}>
              {({ copied, copy }) => (
                <Tooltip label={copied ? 'Copied!' : 'Copy thread'}>
                  <ActionIcon
                    color={copied ? 'teal' : 'blue'}
                    variant="light"
                    onClick={copy}
                    size="lg"
                  >
                    {copied ? '✓' : '📄'}
                  </ActionIcon>
                </Tooltip>
              )}
            </CopyButton>
          </Group>
        </Stack>
      </form>

      <StatusBar
        status={status}
        gitInfo={gitInfo}
        participatingAgentsCount={activeParticipantIds.length}
        totalPersonas={personas.length}
        autoMode={autoMode}
        conversationMode={conversationMode}
        talkStyle={talkStyle}
        executionStrategy={executionStrategy}
      />
    </Stack>
  );
}
