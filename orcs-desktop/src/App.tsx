import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Textarea,
  Button,
  ScrollArea,
  Stack,
  Text,
  Container,
  Box,
  Group,
  CopyButton,
  ActionIcon,
  Tooltip,
  AppShell,
  Burger,
  Badge,
  CloseButton,
  Paper,
  Accordion,
  Loader,
} from "@mantine/core";
import { useDisclosure } from '@mantine/hooks';
import "./App.css";
import { Message, MessageType } from "./types/message";
import { StatusInfo, getDefaultStatus } from "./types/status";
import { Task } from "./types/task";
import { Agent } from "./types/agent";
import { Session, getMessageCount } from "./types/session";
import { MessageItem } from "./components/chat/MessageItem";
import { StatusBar } from "./components/chat/StatusBar";
import { CommandSuggestions } from "./components/chat/CommandSuggestions";
import { AgentSuggestions } from "./components/chat/AgentSuggestions";
import { FileList } from "./components/files/FileList";
import { TaskList } from "./components/tasks/TaskList";
import { PersonasList } from "./components/personas/PersonasList";
import { SessionList } from "./components/sessions/SessionList";
import { parseCommand, isValidCommand, getCommandHelp } from "./utils/commandParser";
import { filterCommands, CommandDefinition } from "./types/command";
import { extractMentions, getCurrentMention } from "./utils/mentionParser";
import { useSessions } from "./hooks/useSessions";
import { convertSessionToMessages, isIdleMode } from "./types/session";

type InteractionResult =
  | { type: 'NewDialogueMessages'; data: { author: string; content: string }[] }
  | { type: 'ModeChanged'; data: { [key: string]: any } }
  | { type: 'TasksToDispatch'; data: { tasks: string[] } }
  | { type: 'NoOp' };

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [status, setStatus] = useState<StatusInfo>(getDefaultStatus());
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [filteredCommands, setFilteredCommands] = useState<CommandDefinition[]>([]);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(0);
  const [showAgentSuggestions, setShowAgentSuggestions] = useState(false);
  const [filteredAgents, setFilteredAgents] = useState<Agent[]>([]);
  const [selectedAgentIndex, setSelectedAgentIndex] = useState(0);
  const [navbarOpened, { toggle: toggleNavbar }] = useDisclosure(true);
  const [attachedFiles, setAttachedFiles] = useState<File[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [currentDir, setCurrentDir] = useState<string>('.');
  const [tasks, setTasks] = useState<Task[]>([]);

  // セッション管理をカスタムフックに切り替え
  const {
    sessions,
    currentSessionId,
    loading: sessionsLoading,
    createSession,
    switchSession,
    deleteSession,
    renameSession,
    saveCurrentSession,
  } = useSessions();

  const [autoMode, setAutoMode] = useState<boolean>(false);
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    if (viewport.current) {
      viewport.current.scrollTo({
        top: viewport.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [messages]);

  // 入力内容が変更されたときにコマンド/エージェントサジェストを更新
  useEffect(() => {
    const trimmedInput = input.trim();

    // コマンドサジェスト
    if (trimmedInput.startsWith('/')) {
      const commands = filterCommands(trimmedInput);
      setFilteredCommands(commands);
      setShowSuggestions(commands.length > 0);
      setSelectedSuggestionIndex(0);
      setShowAgentSuggestions(false);
    } else {
      setShowSuggestions(false);
    }

    // エージェントサジェスト（@メンション）
    const cursorPosition = textareaRef.current?.selectionStart || input.length;
    const mentionFilter = getCurrentMention(input, cursorPosition);

    if (mentionFilter !== null) {
      const filtered: Agent[] = [];
      setFilteredAgents(filtered);
      setShowAgentSuggestions(filtered.length > 0);
      setSelectedAgentIndex(0);
    } else {
      setShowAgentSuggestions(false);
    }
  }, [input]);

  // メッセージを追加するヘルパー関数
  const addMessage = (type: MessageType, author: string, text: string) => {
    const newMessage: Message = {
      id: `${Date.now()}-${Math.random()}`,
      type,
      author,
      text,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, newMessage]);
  };

  // スレッド全体をテキストとして取得
  const getThreadAsText = () => {
    return messages
      .map((msg) => {
        const time = msg.timestamp.toLocaleString();
        return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
      })
      .join('\n---\n\n');
  };

  // キーボードイベントハンドラー
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Cmd+Enter または Ctrl+Enter で送信
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSubmit(e as any);
      return;
    }

    // エージェントサジェスト表示中のキーボード操作
    if (showAgentSuggestions) {
      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          setSelectedAgentIndex((prev) =>
            prev > 0 ? prev - 1 : filteredAgents.length - 1
          );
          break;
        case 'ArrowDown':
          e.preventDefault();
          setSelectedAgentIndex((prev) =>
            prev < filteredAgents.length - 1 ? prev + 1 : 0
          );
          break;
        case 'Tab':
          e.preventDefault();
          selectAgent(filteredAgents[selectedAgentIndex]);
          break;
        case 'Enter':
          if (!e.shiftKey && !e.metaKey && !e.ctrlKey) {
            e.preventDefault();
            selectAgent(filteredAgents[selectedAgentIndex]);
          }
          break;
        case 'Escape':
          e.preventDefault();
          setShowAgentSuggestions(false);
          break;
      }
      return;
    }

    // コマンドサジェスト表示中のキーボード操作
    if (showSuggestions) {
      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          setSelectedSuggestionIndex((prev) =>
            prev > 0 ? prev - 1 : filteredCommands.length - 1
          );
          break;
        case 'ArrowDown':
          e.preventDefault();
          setSelectedSuggestionIndex((prev) =>
            prev < filteredCommands.length - 1 ? prev + 1 : 0
          );
          break;
        case 'Tab':
          e.preventDefault();
          selectCommand(filteredCommands[selectedSuggestionIndex]);
          break;
        case 'Enter':
          if (!e.shiftKey && !e.metaKey && !e.ctrlKey) {
            e.preventDefault();
            selectCommand(filteredCommands[selectedSuggestionIndex]);
          }
          break;
        case 'Escape':
          e.preventDefault();
          setShowSuggestions(false);
          break;
      }
      return;
    }
  };

  // コマンドを選択
  const selectCommand = (command: CommandDefinition) => {
    setInput(`/${command.name} `);
    setShowSuggestions(false);
    textareaRef.current?.focus();
  };

  // エージェントを選択
  const selectAgent = (agent: Agent) => {
    const cursorPosition = textareaRef.current?.selectionStart || input.length;
    const beforeCursor = input.slice(0, cursorPosition);
    const afterCursor = input.slice(cursorPosition);
    const lastAtIndex = beforeCursor.lastIndexOf('@');

    if (lastAtIndex !== -1) {
      const newInput = beforeCursor.slice(0, lastAtIndex) + `@${agent.name} ` + afterCursor;
      setInput(newInput);
    }

    setShowAgentSuggestions(false);
    textareaRef.current?.focus();
  };

  // ドラッグ&ドロップハンドラー
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);

    if (files.length > 0) {
      setAttachedFiles((prev) => [...prev, ...files]);
      addMessage('system', 'System', `📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`);
    }
  };

  const removeAttachedFile = (index: number) => {
    setAttachedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  // ファイル選択ボタンのハンドラー
  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (files.length > 0) {
      setAttachedFiles((prev) => [...prev, ...files]);
      addMessage('system', 'System', `📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`);
    }
  };

  // タスク操作ハンドラー
  const handleTaskToggle = (taskId: string) => {
    setTasks((prev) =>
      prev.map((task) => {
        if (task.id === taskId) {
          const newStatus = task.status === 'completed' ? 'pending' : 'completed';
          if (newStatus === 'completed') {
            setStatus((s) => ({ ...s, activeTasks: Math.max(0, s.activeTasks - 1) }));
          } else {
            setStatus((s) => ({ ...s, activeTasks: s.activeTasks + 1 }));
          }
          return { ...task, status: newStatus };
        }
        return task;
      })
    );
  };

  const handleTaskDelete = (taskId: string) => {
    const taskToDelete = tasks.find((t) => t.id === taskId);
    setTasks((prev) => prev.filter((task) => task.id !== taskId));
    if (taskToDelete && taskToDelete.status !== 'completed') {
      setStatus((prev) => ({ ...prev, activeTasks: Math.max(0, prev.activeTasks - 1) }));
    }
  };

  // セッション操作ハンドラー（Tauri統合版）
  const handleSessionSelect = async (session: Session) => {
    try {
      // セッションを切り替え（バックエンドで履歴付きSessionDataを取得）
      const fullSession = await switchSession(session.id);

      // メッセージ履歴を復元
      const restoredMessages = convertSessionToMessages(fullSession);
      setMessages(restoredMessages);

      // AppModeを復元
      if (isIdleMode(fullSession.app_mode)) {
        // Idle mode - 特に何もしない
      } else {
        // AwaitingConfirmation mode - 将来的に対応
        console.log('Session has AwaitingConfirmation mode:', fullSession.app_mode);
      }

      addMessage('system', 'System', `✅ Switched to session: ${session.name} (${restoredMessages.length} messages restored)`);
    } catch (err) {
      addMessage('error', 'System', `Failed to switch session: ${err}`);
    }
  };

  const handleSessionDelete = async (sessionId: string) => {
    try {
      await deleteSession(sessionId);
      addMessage('system', 'System', 'Session deleted');
    } catch (err) {
      addMessage('error', 'System', `Failed to delete session: ${err}`);
    }
  };

  const handleSessionRename = async (sessionId: string, newTitle: string) => {
    try {
      await renameSession(sessionId, newTitle);
    } catch (err) {
      addMessage('error', 'System', `Failed to rename session: ${err}`);
    }
  };

  const handleNewSession = async () => {
    try {
      await createSession();
      setMessages([]);
      addMessage('system', 'System', 'Started new session');
    } catch (err) {
      addMessage('error', 'System', `Failed to create session: ${err}`);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!input.trim() && attachedFiles.length === 0) return;

    const currentInput = input;
    const currentFiles = [...attachedFiles];
    setInput("");
    setAttachedFiles([]);
    setShowSuggestions(false);
    setShowAgentSuggestions(false);

    // メンションをパース
    const mentions = extractMentions(currentInput);
    if (mentions.length > 0) {
      console.log('[MENTION EVENT] Agents mentioned:', mentions.map(m => m.agentName));
    }

    // コマンドをパース
    const parsed = parseCommand(currentInput);

    if (parsed.isCommand && parsed.command) {
      addMessage('command', 'You', currentInput);

      if (!isValidCommand(parsed.command)) {
        addMessage('error', 'System', `Unknown command: /${parsed.command}\n\nType /help for available commands.`);
        return;
      }

      // コマンド実行
      switch (parsed.command) {
        case 'help':
          addMessage('system', 'System', getCommandHelp());
          break;
        case 'status':
          addMessage('system', 'System', `Status: ${status.connection}\nTasks: ${status.activeTasks}\nAgent: ${status.currentAgent}\nMode: ${status.mode}`);
          break;
        case 'clear':
          setMessages([]);
          addMessage('system', 'System', 'Chat history cleared.');
          break;
        case 'task':
          if (parsed.args && parsed.args.length > 0) {
            const taskText = parsed.args.join(' ');
            const newTask: Task = {
              id: `${Date.now()}-${Math.random()}`,
              description: taskText,
              status: 'pending',
              createdAt: new Date(),
            };
            setTasks((prev) => [...prev, newTask]);
            setStatus(prev => ({ ...prev, activeTasks: prev.activeTasks + 1 }));
            addMessage('task', 'System', `✅ Task created: ${taskText}`);
          } else {
            addMessage('error', 'System', 'Usage: /task [description]');
          }
          break;
        default:
          addMessage('error', 'System', `Command not implemented: /${parsed.command}`);
      }
      return;
    }

    // 通常のメッセージ処理
    let messageText = currentInput;
    if (currentFiles.length > 0) {
      const fileInfo = currentFiles.map(f => `📎 ${f.name} (${(f.size / 1024).toFixed(1)} KB)`).join('\n');
      messageText = currentInput ? `${currentInput}\n\n${fileInfo}` : fileInfo;
    }
    addMessage('user', 'You', messageText);

    try {
      // Call Tauri backend
      const result = await invoke<InteractionResult>("handle_input", {
        input: currentInput,
      });

      // Process AI response
      if (result.type === 'NewDialogueMessages') {
        for (const message of result.data) {
          addMessage('ai', message.author, message.content);
        }
      }

      // Auto-save after interaction
      await saveCurrentSession();
    } catch (error) {
      console.error("Error calling backend:", error);
      addMessage('error', 'System', `Error: ${error}`);
    }
  };

  // セッションローディング中の表示
  if (sessionsLoading) {
    return (
      <Container size="md" h="100vh" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Stack align="center" gap="md">
          <Loader size="lg" />
          <Text>Loading sessions...</Text>
        </Stack>
      </Container>
    );
  }

  return (
    <AppShell
      navbar={{
        width: 280,
        breakpoint: 'sm',
        collapsed: { mobile: !navbarOpened, desktop: !navbarOpened },
      }}
      padding="md"
    >
      {/* 左ペイン */}
      <AppShell.Navbar>
        <ScrollArea h="100vh" type="auto">
          <Box p="md">
            <Accordion
              defaultValue={['sessions', 'files', 'tasks', 'personas']}
              multiple
              variant="separated"
            >
              {/* セッション */}
              <Accordion.Item value="sessions">
                <Accordion.Control>
                  <Group gap="xs">
                    <Text>💬</Text>
                    <Text fw={600}>Sessions</Text>
                    <Badge size="sm" color="blue" variant="light">
                      {sessions.length}
                    </Badge>
                  </Group>
                </Accordion.Control>
                <Accordion.Panel>
                  <Box style={{ maxHeight: '400px' }}>
                    <SessionList
                      sessions={sessions}
                      currentSessionId={currentSessionId || undefined}
                      onSessionSelect={handleSessionSelect}
                      onSessionDelete={handleSessionDelete}
                      onSessionRename={handleSessionRename}
                      onNewSession={handleNewSession}
                    />
                  </Box>
                </Accordion.Panel>
              </Accordion.Item>

              {/* ファイル */}
              <Accordion.Item value="files">
                <Accordion.Control>
                  <Group gap="xs">
                    <Text>📁</Text>
                    <Text fw={600}>Files</Text>
                  </Group>
                </Accordion.Control>
                <Accordion.Panel>
                  <FileList onFileSelect={(file) => {
                    addMessage('system', 'System', `Selected file: ${file.path}`);
                  }} />
                </Accordion.Panel>
              </Accordion.Item>

              {/* タスク */}
              <Accordion.Item value="tasks">
                <Accordion.Control>
                  <Group gap="xs">
                    <Text>✅</Text>
                    <Text fw={600}>Tasks</Text>
                    <Badge size="sm" color="blue" variant="light">
                      {tasks.filter(t => t.status !== 'completed').length}
                    </Badge>
                  </Group>
                </Accordion.Control>
                <Accordion.Panel>
                  <TaskList
                    tasks={tasks}
                    onTaskToggle={handleTaskToggle}
                    onTaskDelete={handleTaskDelete}
                  />
                </Accordion.Panel>
              </Accordion.Item>

              {/* ペルソナ */}
              <Accordion.Item value="personas">
                <Accordion.Control>
                  <Group gap="xs">
                    <Text>⭐️</Text>
                    <Text fw={600}>Personas</Text>
                  </Group>
                </Accordion.Control>
                <Accordion.Panel>
                  <PersonasList />
                </Accordion.Panel>
              </Accordion.Item>
            </Accordion>
          </Box>
        </ScrollArea>
      </AppShell.Navbar>

      {/* メインコンテンツ */}
      <AppShell.Main>
        <Container size="md" h="100vh" p="md" style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <Stack style={{ flex: 1, minHeight: 0 }} gap="md">
            <Group gap="sm">
              <Burger opened={navbarOpened} onClick={toggleNavbar} size="sm" />
              <Text size="xl" fw={700}>ORCS Chat Interface</Text>
            </Group>

            {/* メッセージエリア */}
            <Box
              style={{ flex: 1, position: 'relative', minHeight: 0 }}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              <ScrollArea h="100%" viewportRef={viewport}>
                <Stack gap="sm" p="md">
                  {messages.map((message) => (
                    <MessageItem key={message.id} message={message} />
                  ))}
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
                          <CloseButton
                            size="xs"
                            onClick={() => removeAttachedFile(index)}
                          />
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
                      filter=""
                      selectedIndex={selectedAgentIndex}
                      onSelect={selectAgent}
                    />
                  )}

                  <Textarea
                    ref={textareaRef}
                    value={input}
                    onChange={(e) => setInput(e.currentTarget.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Type your message or /help for commands... (⌘+Enter to send)"
                    size="md"
                    minRows={1}
                    maxRows={4}
                    autosize
                  />
                </Box>

                <Group gap="xs">
                  <Tooltip label="Attach files">
                    <Button
                      variant="light"
                      size="sm"
                      component="label"
                      leftSection="📎"
                    >
                      Attach
                      <input type="file" multiple hidden onChange={handleFileSelect} />
                    </Button>
                  </Tooltip>

                  <Button type="submit" style={{ flex: 1 }}>Send</Button>

                  <Tooltip label={autoMode ? 'Stop AUTO mode' : 'Start AUTO mode'}>
                    <ActionIcon
                      color={autoMode ? 'red' : 'green'}
                      variant={autoMode ? 'filled' : 'light'}
                      onClick={() => setAutoMode(!autoMode)}
                      size="lg"
                    >
                      {autoMode ? '⏹️' : '▶️'}
                    </ActionIcon>
                  </Tooltip>

                  <CopyButton value={getThreadAsText()}>
                    {({ copied, copy }) => (
                      <Tooltip label={copied ? 'Copied!' : 'Copy thread'}>
                        <ActionIcon color={copied ? 'teal' : 'blue'} variant="light" onClick={copy} size="lg">
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
              currentDir={currentDir}
              participatingAgentsCount={0}
              autoMode={autoMode}
            />
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}

export default App;
