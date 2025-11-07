/**
 * ChatPanel - 1つのタブ（セッション）のチャット画面を管理
 */
import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { notifications } from '@mantine/notifications';
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
import type { Message } from '../../types/message';
import type { StatusInfo } from '../../types/status';
import type { Task } from '../../types/task';
import type { Agent } from '../../types/agent';
import type { GitInfo } from '../../types/git';
import type { Workspace, UploadedFile } from '../../types/workspace';
import type { CommandDefinition } from '../../types/command';
import type { PersonaConfig } from '../../types/agent';

interface ChatPanelProps {
  sessionId: string;
  messages: Message[];
  onMessagesChange: (messages: Message[]) => void;
  status: StatusInfo;
  onStatusChange: (status: StatusInfo) => void;
  userNickname: string;
  gitInfo: GitInfo;
  isAiThinking: boolean;
  thinkingPersona: string;
  autoMode: boolean;
  conversationMode: string;
  talkStyle: string | null;
  executionStrategy: string;
  personas: PersonaConfig[];
  activeParticipantIds: string[];
  workspace: Workspace | null;
  onSaveCurrentSession: () => Promise<void>;
  onExecuteAsTask: (content: string) => void;
  onAutoModeChange: (autoMode: boolean) => void;
}

export function ChatPanel({
  sessionId,
  messages,
  onMessagesChange,
  status,
  onStatusChange,
  userNickname,
  gitInfo,
  isAiThinking,
  thinkingPersona,
  autoMode,
  conversationMode,
  talkStyle,
  executionStrategy,
  personas,
  activeParticipantIds,
  workspace,
  onSaveCurrentSession,
  onExecuteAsTask,
  onAutoModeChange,
}: ChatPanelProps) {
  const [input, setInput] = useState('');
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [filteredCommands, setFilteredCommands] = useState<CommandDefinition[]>([]);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(0);
  const [showAgentSuggestions, setShowAgentSuggestions] = useState(false);
  const [filteredAgents, setFilteredAgents] = useState<Agent[]>([]);
  const [selectedAgentIndex, setSelectedAgentIndex] = useState(0);
  const [attachedFiles, setAttachedFiles] = useState<File[]>([]);
  const [isDragging, setIsDragging] = useState(false);

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
  }, [messages]);

  const addMessage = useCallback(
    (type: Message['type'], author: string, text: string) => {
      const newMessage: Message = {
        id: Date.now().toString(),
        type,
        author,
        text,
        timestamp: new Date(),
      };
      onMessagesChange([...messages, newMessage]);
    },
    [messages, onMessagesChange]
  );

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() && attachedFiles.length === 0) return;

    const userMessage = input.trim();
    setInput('');

    // Add user message to UI
    if (userMessage) {
      addMessage('user', userNickname, userMessage);
    }

    // TODO: Implement file upload and message sending
    // This needs to be integrated with the backend

    setAttachedFiles([]);
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files) {
      setAttachedFiles((prev) => [...prev, ...Array.from(files)]);
    }
  };

  const removeAttachedFile = (index: number) => {
    setAttachedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    setAttachedFiles((prev) => [...prev, ...files]);
  };

  const handleSaveMessageToWorkspace = async (
    content: string,
    filename: string,
    mimeType: string
  ) => {
    if (!workspace) {
      notifications.show({
        title: 'Error',
        message: 'No workspace selected',
        color: 'red',
      });
      return;
    }

    try {
      await invoke('save_code_to_workspace', {
        workspaceId: workspace.id,
        content,
        filename,
        mimeType,
      });

      notifications.show({
        title: 'Saved',
        message: `File saved to workspace: ${filename}`,
        color: 'green',
      });

      await onSaveCurrentSession();
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to save file: ${err}`,
        color: 'red',
      });
    }
  };

  const getThreadAsText = (): string => {
    return messages
      .map((m) => `[${m.author}] ${m.text}`)
      .join('\n\n');
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // TODO: Implement command suggestions and keyboard shortcuts
  };

  const selectCommand = (command: CommandDefinition) => {
    // TODO: Implement command selection
  };

  const selectAgent = (agent: Agent) => {
    // TODO: Implement agent selection
  };

  return (
    <Stack gap={0} style={{ height: '100%', maxHeight: '100vh', overflow: 'hidden' }}>
      <Box
        style={{ flex: 1, position: 'relative', minHeight: 0 }}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <ScrollArea h="100%" viewportRef={viewport}>
          <Stack gap="sm" p="md">
            {messages.map((message) => (
              <MessageItem
                key={message.id}
                message={message}
                onSaveToWorkspace={handleSaveMessageToWorkspace}
                onExecuteAsTask={onExecuteAsTask}
                workspaceRootPath={workspace?.rootPath}
              />
            ))}
            {isAiThinking && <ThinkingIndicator personaName={thinkingPersona} />}
          </Stack>
        </ScrollArea>

        {isDragging && (
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
      <form onSubmit={handleSubmit}>
        <Stack gap="xs">
          {attachedFiles.length > 0 && (
            <Group gap="xs">
              {attachedFiles.map((file, index) => (
                <Badge
                  key={index}
                  size="lg"
                  variant="light"
                  rightSection={
                    <CloseButton size="xs" onClick={() => removeAttachedFile(index)} />
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
                onSelect={selectCommand}
                onHover={setSelectedSuggestionIndex}
              />
            )}

            {showAgentSuggestions && (
              <AgentSuggestions
                agents={filteredAgents}
                selectedIndex={selectedAgentIndex}
                onSelect={selectAgent}
              />
            )}

            <Textarea
              ref={textareaRef}
              value={input}
              onChange={(e) => setInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
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
                <input type="file" multiple hidden onChange={handleFileSelect} />
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

