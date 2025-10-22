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
} from "@mantine/core";
import { useDisclosure } from '@mantine/hooks';
import "./App.css";
import { Message, MessageType } from "./types/message";
import { StatusInfo, getDefaultStatus } from "./types/status";
import { Task } from "./types/task";
import { Agent } from "./types/agent";
import { Session } from "./types/session";
import { MessageItem } from "./components/chat/MessageItem";
import { StatusBar } from "./components/chat/StatusBar";
import { CommandSuggestions } from "./components/chat/CommandSuggestions";
import { AgentSuggestions } from "./components/chat/AgentSuggestions";
import { FileList } from "./components/files/FileList";
import { TaskList } from "./components/tasks/TaskList";
import { AgentList } from "./components/agents/AgentList";
import { SessionList } from "./components/sessions/SessionList";
import { parseCommand, isValidCommand, getCommandHelp } from "./utils/commandParser";
import { filterCommands, CommandDefinition } from "./types/command";
import { extractMentions, getCurrentMention } from "./utils/mentionParser";

type InteractionResult =
  | { type: 'NewMessage'; data: string }
  | { type: 'ModeChanged'; data: { [key: string]: any } } // More specific if needed
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
  const [agents, setAgents] = useState<Agent[]>([
    {
      id: '1',
      name: 'CodeAnalyzer',
      status: 'idle',
      description: 'Analyzes code structure and patterns',
      lastActive: new Date(),
      isActive: false,
    },
    {
      id: '2',
      name: 'TaskManager',
      status: 'idle',
      description: 'Manages and prioritizes tasks',
      lastActive: new Date(),
      isActive: false,
    },
    {
      id: '3',
      name: 'FileSearcher',
      status: 'idle',
      description: 'Searches files and content',
      isActive: false,
    },
  ]);
  const [sessions, setSessions] = useState<Session[]>([
    {
      id: '1',
      title: 'Initial project setup and architecture',
      createdAt: new Date(Date.now() - 86400000 * 2),
      lastActive: new Date(Date.now() - 3600000),
      messageCount: 15,
    },
    {
      id: '2',
      title: 'Implementing @Agent mention functionality',
      createdAt: new Date(Date.now() - 86400000),
      lastActive: new Date(),
      messageCount: messages.length,
    },
  ]);
  const [currentSessionId, setCurrentSessionId] = useState<string>('2');
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
      const filtered = agents.filter(agent =>
        agent.name.toLowerCase().includes(mentionFilter.toLowerCase())
      );
      setFilteredAgents(filtered);
      setShowAgentSuggestions(filtered.length > 0);
      setSelectedAgentIndex(0);
    } else {
      setShowAgentSuggestions(false);
    }
  }, [input, agents]);

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
    console.log('[DEBUG] Drag over detected');
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    console.log('[DEBUG] Drag leave detected');
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    console.log('[DEBUG] Drop detected');
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    console.log('[DEBUG] Dropped files:', files);

    if (files.length > 0) {
      setAttachedFiles((prev) => [...prev, ...files]);
      addMessage('system', 'System', `📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`);
    } else {
      console.log('[DEBUG] No files detected in drop');
      addMessage('error', 'System', 'No files detected. Try using the Attach button instead.');
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
          // ステータスカウントを更新
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
    // 完了していないタスクを削除した場合のみカウントを減らす
    if (taskToDelete && taskToDelete.status !== 'completed') {
      setStatus((prev) => ({ ...prev, activeTasks: Math.max(0, prev.activeTasks - 1) }));
    }
  };

  const handleAgentToggle = (agentId: string) => {
    setAgents((prev) =>
      prev.map((agent) =>
        agent.id === agentId ? { ...agent, isActive: !agent.isActive } : agent
      )
    );
  };

  const handleSessionSelect = (session: Session) => {
    setCurrentSessionId(session.id);
    // TODO: Load session messages from backend
    addMessage('system', 'System', `Switched to session: ${session.title}`);
  };

  const handleSessionDelete = (sessionId: string) => {
    setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    if (currentSessionId === sessionId) {
      // 削除されたセッションが現在のセッションの場合、新規セッションを作成
      handleNewSession();
    }
  };

  const handleSessionRename = (sessionId: string, newTitle: string) => {
    setSessions((prev) =>
      prev.map((session) =>
        session.id === sessionId ? { ...session, title: newTitle } : session
      )
    );
  };

  const handleNewSession = () => {
    const newSession: Session = {
      id: Date.now().toString(),
      title: 'New conversation',
      createdAt: new Date(),
      lastActive: new Date(),
      messageCount: 0,
    };
    setSessions((prev) => [newSession, ...prev]);
    setCurrentSessionId(newSession.id);
    setMessages([]);
    addMessage('system', 'System', 'Started new session');
  };

  const handleSearchRequest = () => {
    if (!input.trim()) return;

    const searchQuery = input.trim();

    // サーチクエリをユーザーメッセージとして表示
    addMessage('user', 'You', `🔍 Search: ${searchQuery}`);
    setInput(''); // Clear input field

    // Thinking表示
    const thinkingMessage: Message = {
      id: `thinking-${Date.now()}`,
      type: 'thinking',
      author: 'SearchAgent',
      text: '🔍 Searching...',
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, thinkingMessage]);

    // モックで検索エージェントからのレスポンスを生成（実際はバックエンドから取得）
    setTimeout(() => {
      // Thinkingメッセージを削除
      setMessages((prev) => prev.filter((m) => m.id !== thinkingMessage.id));

      const mockSearchResults = `🔍 **Search Results for:** "${searchQuery}"

**Found 3 relevant items:**

1. **src/components/chat/MessageItem.tsx** (Line 89)
   - Implements message rendering with mention highlighting
   - Related to: ${searchQuery}

2. **src/App.tsx** (Line 344)
   - Agent toggle functionality
   - Match: Function implementation

3. **docs/architecture.md**
   - Architecture overview and design decisions
   - Context: ${searchQuery}

**Suggestions:**
- Try refining your search with more specific terms
- Use @Agent mentions to ask specific agents
- Use /help to see available commands

*Search powered by AI Agent - Backend integration pending*`;

      addMessage('ai', 'SearchAgent', mockSearchResults);
    }, 1000);
  };

  const handleSummaryRequest = () => {
    // サマリーリクエストメッセージを送信
    addMessage('user', 'You', 'Please create a summary of this conversation.');

    // Thinking表示
    const thinkingMessage: Message = {
      id: `thinking-${Date.now()}`,
      type: 'thinking',
      author: 'Assistant',
      text: '💭 Thinking...',
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, thinkingMessage]);

    // モックでAIエージェントからのレスポンスを生成（実際はバックエンドから取得）
    setTimeout(() => {
      // Thinkingメッセージを削除
      setMessages((prev) => prev.filter((m) => m.id !== thinkingMessage.id));

      const mockSummary = `📝 **Conversation Summary**

**Main Topics:**
- Discussed @Agent mention functionality implementation
- Added agent suggestions dropdown with keyboard navigation
- Implemented mention parsing and event emission
- Added visual highlighting for mentions in messages
- Moved thread copy button to send area
- Added summary request feature

**Key Actions:**
- ${messages.length} messages exchanged
- ${tasks.filter(t => t.status === 'completed').length} tasks completed
- ${agents.filter(a => a.isActive).length} agents participating

**Current State:**
All UI components implemented and ready for Tauri backend integration.`;

      addMessage('ai', 'Assistant', mockSummary);
    }, 1500);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!input.trim() && attachedFiles.length === 0) return;

    const currentInput = input;
    const currentFiles = [...attachedFiles];
    setInput(""); // Clear input field
    setAttachedFiles([]); // Clear attached files
    setShowSuggestions(false);
    setShowAgentSuggestions(false);

    // メンションをパース
    const mentions = extractMentions(currentInput);
    if (mentions.length > 0) {
      const mentionedAgentNames = mentions.map(m => m.agentName);
      const mentionedAgents = agents.filter(a => mentionedAgentNames.includes(a.name));

      // イベント発行（現状はコンソールログ）
      console.log('[MENTION EVENT] Agents mentioned:', mentionedAgents.map(a => a.name));
      console.log('[MENTION EVENT] Message:', currentInput);
      console.log('[MENTION EVENT] Attached files:', currentFiles.map(f => f.name));

      // TODO: Tauri バックエンドへの通知
      // await invoke('notify_agents', { agents: mentionedAgentNames, message: currentInput });
    }

    // コマンドをパース
    const parsed = parseCommand(currentInput);

    if (parsed.isCommand && parsed.command) {
      // コマンドメッセージを表示
      addMessage('command', 'You', currentInput);

      // コマンド処理
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
        case 'mode':
          if (parsed.args && parsed.args.length > 0) {
            const newMode = parsed.args[0];
            setStatus(prev => ({ ...prev, mode: newMode }));
            addMessage('system', 'System', `Mode changed to: ${newMode}`);
          } else {
            addMessage('error', 'System', 'Usage: /mode [mode_name]');
          }
          break;
        case 'agents':
          const agentsList = agents.map(a => `- ${a.name} (${a.status}): ${a.description}`).join('\n');
          addMessage('system', 'System', `Available agents:\n\n${agentsList}`);
          break;
        case 'pwd':
          addMessage('system', 'System', `Current directory: ${currentDir}`);
          break;
        case 'cd':
          if (parsed.args && parsed.args.length > 0) {
            const newDir = parsed.args.join(' ');
            setCurrentDir(newDir);
            addMessage('system', 'System', `Changed directory to: ${newDir}`);
            // TODO: Tauriバックエンドで実際のディレクトリ変更
          } else {
            addMessage('error', 'System', 'Usage: /cd <path>');
          }
          break;
        case 'files':
        case 'ls':
          const targetDir = parsed.args && parsed.args.length > 0 ? parsed.args.join(' ') : currentDir;
          // TODO: Tauriバックエンドでファイル一覧取得
          const mockFiles = [
            '📂 src/',
            '📂 tests/',
            '📄 Cargo.toml',
            '📄 Cargo.lock',
            '📄 README.md',
            '📄 .gitignore',
          ];
          addMessage('system', 'System', `📁 Files in "${targetDir}":\n\n${mockFiles.join('\n')}\n\n⚠️ Mock data - TODO: Fetch from Tauri backend`);
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

      // Process AI response based on result type
      if (result.type === 'NewMessage') {
        const responseText = result.data;
        let author = 'AI'; // Default author
        let text = responseText;

        const parts = responseText.split(': ');
        if (parts.length > 1) {
          author = parts[0];
          text = parts.slice(1).join(': ');
        }

        addMessage('ai', author, text);
      }
      // Add other result type handlers here later
    } catch (error) {
      console.error("Error calling backend:", error);
      addMessage('error', 'System', `Error: ${error}`);
    }
  };

  return (
    <AppShell
      navbar={{
        width: 280,
        breakpoint: 'sm',
        collapsed: { mobile: !navbarOpened, desktop: !navbarOpened },
      }}
      padding="md"
    >
      {/* 左ペイン（ファイルリスト & タスクリスト） */}
      <AppShell.Navbar>
        <ScrollArea h="100vh" type="auto">
          <Box p="md">
            <Accordion
              defaultValue={['files', 'tasks', 'agents']}
              multiple
              variant="separated"
              styles={{
                item: {
                  border: 'none',
                  backgroundColor: 'transparent',
                },
                control: {
                  padding: '8px 12px',
                  '&:hover': {
                    backgroundColor: '#f1f3f5',
                  },
                },
              }}
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
                      currentSessionId={currentSessionId}
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
                  <Box style={{ maxHeight: '400px' }}>
                    <FileList onFileSelect={(file) => {
                      addMessage('system', 'System', `Selected file: ${file.path}`);
                    }} />
                  </Box>
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
                  <Box style={{ maxHeight: '400px' }}>
                    <TaskList
                      tasks={tasks}
                      onTaskToggle={handleTaskToggle}
                      onTaskDelete={handleTaskDelete}
                    />
                  </Box>
                </Accordion.Panel>
              </Accordion.Item>

              {/* エージェント */}
              <Accordion.Item value="agents">
                <Accordion.Control>
                  <Group gap="xs">
                    <Text>🤖</Text>
                    <Text fw={600}>Agents</Text>
                    <Badge size="sm" color="green" variant="light">
                      {agents.filter(a => a.isActive).length}
                    </Badge>
                  </Group>
                </Accordion.Control>
                <Accordion.Panel>
                  <Box style={{ maxHeight: '400px' }}>
                    <AgentList
                      agents={agents}
                      onAgentToggle={handleAgentToggle}
                    />
                  </Box>
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
            {/* ヘッダー */}
            <Group gap="sm">
              <Burger
                opened={navbarOpened}
                onClick={toggleNavbar}
                size="sm"
              />
              <Text size="xl" fw={700}>
                ORCS Chat Interface
              </Text>
            </Group>

            {/* メッセージエリア（ドロップゾーン） */}
            <Box
              style={{ flex: 1, position: 'relative', minHeight: 0 }}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              <ScrollArea
                h="100%"
                viewportRef={viewport}
              >
                <Stack gap="sm" p="md">
                  {messages.map((message) => (
                    <MessageItem key={message.id} message={message} />
                  ))}
                </Stack>
              </ScrollArea>

              {/* ドラッグオーバー表示 */}
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
                {/* 添付ファイル表示 */}
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
                            style={{ marginLeft: 4 }}
                          />
                        }
                        style={{ paddingRight: 4 }}
                      >
                        📎 {file.name} ({(file.size / 1024).toFixed(1)} KB)
                      </Badge>
                    ))}
                  </Group>
                )}

                <Box style={{ position: 'relative' }}>
                  {/* コマンドサジェスト */}
                  {showSuggestions && (
                    <CommandSuggestions
                      commands={filteredCommands}
                      selectedIndex={selectedSuggestionIndex}
                      onSelect={selectCommand}
                      onHover={setSelectedSuggestionIndex}
                    />
                  )}

                  {/* エージェントサジェスト */}
                  {showAgentSuggestions && (
                    <AgentSuggestions
                      agents={filteredAgents}
                      filter=""
                      selectedIndex={selectedAgentIndex}
                      onSelect={selectAgent}
                    />
                  )}

                  {/* 入力エリア */}
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
                  {/* ファイル添付ボタン */}
                  <Tooltip label="Attach files">
                    <Button
                      variant="light"
                      size="sm"
                      component="label"
                      leftSection="📎"
                      style={{ flex: '0 0 auto' }}
                    >
                      Attach
                      <input
                        type="file"
                        multiple
                        hidden
                        onChange={handleFileSelect}
                      />
                    </Button>
                  </Tooltip>

                  {/* 送信ボタン */}
                  <Button type="submit" style={{ flex: 1 }}>
                    Send
                  </Button>

                  {/* サーチボタン */}
                  <Tooltip label="Search with current input" withArrow>
                    <ActionIcon
                      color="cyan"
                      variant="light"
                      onClick={handleSearchRequest}
                      size="lg"
                      disabled={!input.trim()}
                    >
                      🔍
                    </ActionIcon>
                  </Tooltip>

                  {/* サマリーボタン */}
                  <Tooltip label="Request conversation summary" withArrow>
                    <ActionIcon
                      color="violet"
                      variant="light"
                      onClick={handleSummaryRequest}
                      size="lg"
                    >
                      📊
                    </ActionIcon>
                  </Tooltip>

                  {/* AUTOモード切り替えボタン */}
                  <Tooltip label={autoMode ? 'Stop AUTO mode' : 'Start AUTO mode'} withArrow>
                    <ActionIcon
                      color={autoMode ? 'red' : 'green'}
                      variant={autoMode ? 'filled' : 'light'}
                      onClick={() => {
                        setAutoMode(!autoMode);
                        addMessage('system', 'System', `AUTO mode ${!autoMode ? 'enabled' : 'disabled'}. ${!autoMode ? 'Agents will proceed automatically without asking for confirmation.' : 'Agents will ask for confirmation before proceeding.'}`);
                      }}
                      size="lg"
                    >
                      {autoMode ? '⏹️' : '▶️'}
                    </ActionIcon>
                  </Tooltip>

                  {/* スレッドコピーボタン */}
                  <CopyButton value={getThreadAsText()}>
                    {({ copied, copy }) => (
                      <Tooltip label={copied ? 'Copied!' : 'Copy entire thread'} withArrow>
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

            {/* ステータスバー */}
            <StatusBar
              status={status}
              currentDir={currentDir}
              participatingAgentsCount={agents.filter(a => a.isActive).length}
              autoMode={autoMode}
            />
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}

export default App;
