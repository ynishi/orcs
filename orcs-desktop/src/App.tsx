import { useState, useRef, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { notifications } from '@mantine/notifications';
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
  Loader,
} from "@mantine/core";
import { useDisclosure } from '@mantine/hooks';
import "./App.css";
import { Message, MessageType } from "./types/message";
import { StatusInfo, getDefaultStatus } from "./types/status";
import { Task } from "./types/task";
import { Agent } from "./types/agent";
import { Session, getMessageCount } from "./types/session";
import { GitInfo } from "./types/git";
import { MessageItem } from "./components/chat/MessageItem";
import { StatusBar } from "./components/chat/StatusBar";
import { CommandSuggestions } from "./components/chat/CommandSuggestions";
import { AgentSuggestions } from "./components/chat/AgentSuggestions";
import { ThinkingIndicator } from "./components/chat/ThinkingIndicator";
import { Navbar } from "./components/navigation/Navbar";
import { WorkspaceSwitcher } from "./components/workspace/WorkspaceSwitcher";
import { SettingsMenu } from "./components/settings/SettingsMenu";
import { parseCommand, isValidCommand, getCommandHelp } from "./utils/commandParser";
import { filterCommandsWithCustom, CommandDefinition } from "./types/command";
import { extractMentions, getCurrentMention } from "./utils/mentionParser";
import { useSessions } from "./hooks/useSessions";
import { useWorkspace } from "./hooks/useWorkspace";
import { convertSessionToMessages, isIdleMode } from "./types/session";
import { SlashCommand, ExpandedSlashCommand } from "./types/slash_command";

type InteractionResult =
  | { type: 'NewDialogueMessages'; data: { author: string; content: string }[] }
  | { type: 'NewMessage'; data: string }
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
  const [tasks, setTasks] = useState<Task[]>([]);
  const [userNickname, setUserNickname] = useState<string>('You');
  const [gitInfo, setGitInfo] = useState<GitInfo>({
    is_repo: false,
    branch: null,
    repo_name: null,
  });
  const [isAiThinking, setIsAiThinking] = useState<boolean>(false);
  const [thinkingPersona, setThinkingPersona] = useState<string>('AI');
  const [customCommands, setCustomCommands] = useState<SlashCommand[]>([]);
  const [conversationMode, setConversationMode] = useState<string>('normal');
  const [talkStyle, setTalkStyle] = useState<string | null>(null);

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
    refreshSessions,
  } = useSessions();

  // ワークスペース管理
  const { workspace, allWorkspaces, files: workspaceFiles, refresh: refreshWorkspace, refreshWorkspaces, switchWorkspace } = useWorkspace();
  const [includeWorkspaceInPrompt, setIncludeWorkspaceInPrompt] = useState<boolean>(false);

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

  // Listen for real-time dialogue turn events from backend
  // Use ref to ensure only one listener is registered
  const listenerRegistered = useRef(false);

  useEffect(() => {
    // Skip if listener already registered (prevents duplicate in React Strict Mode)
    if (listenerRegistered.current) {
      console.log('[EFFECT] Listener already registered, skipping');
      return;
    }

    let unlisten: (() => void) | undefined;
    listenerRegistered.current = true;

    console.log('[EFFECT] Setting up dialogue-turn listener');

    const setupListener = async () => {
      unlisten = await listen<{ author: string; content: string }>('dialogue-turn', (event) => {
        console.log('[STREAM] Event received:', event.payload.author);
        console.log('[STREAM] Adding message:', event.payload.author);

        // If author is empty, it's an error message
        if (event.payload.author === '') {
          addMessage('error', '', event.payload.content);

          // Show error toast
          notifications.show({
            title: 'Agent Error',
            message: event.payload.content,
            color: 'red',
            icon: '❌',
            autoClose: 10000,
          });
        } else {
          addMessage('ai', event.payload.author, event.payload.content);
        }
      });
      console.log('[EFFECT] Listener setup complete');
    };

    setupListener();

    return () => {
      console.log('[EFFECT] Cleanup: removing listener');
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Load user nickname from backend on startup
  useEffect(() => {
    const loadNickname = async () => {
      try {
        const nickname = await invoke<string>('get_user_nickname');
        setUserNickname(nickname);
      } catch (error) {
        console.error('Failed to load user nickname:', error);
      }
    };
    loadNickname();
  }, []);

  // Load Git repository information on startup
  useEffect(() => {
    const loadGitInfo = async () => {
      try {
        const info = await invoke<GitInfo>('get_git_info');
        setGitInfo(info);
      } catch (error) {
        console.error('Failed to load Git info:', error);
      }
    };
    loadGitInfo();
  }, []);

  // Load conversation mode and talk style on session change
  useEffect(() => {
    const loadConversationSettings = async () => {
      if (!currentSessionId) return;

      try {
        const mode = await invoke<string>('get_conversation_mode');
        setConversationMode(mode);
      } catch (error) {
        console.error('Failed to load conversation mode:', error);
      }

      try {
        const style = await invoke<string | null>('get_talk_style');
        setTalkStyle(style);
      } catch (error) {
        console.error('Failed to load talk style:', error);
      }
    };
    loadConversationSettings();
  }, [currentSessionId]);

  // Load active session messages on startup or when currentSessionId changes
  useEffect(() => {
    const loadActiveSessionMessages = async () => {
      if (!currentSessionId || sessionsLoading) {
        return;
      }

      try {
        console.log('[App] Loading messages for active session:', currentSessionId);
        const activeSession = sessions.find(s => s.id === currentSessionId);
        if (activeSession) {
          const restoredMessages = convertSessionToMessages(activeSession, userNickname);
          setMessages(restoredMessages);
          console.log('[App] Loaded', restoredMessages.length, 'messages from session', currentSessionId);
        }
      } catch (error) {
        console.error('[App] Failed to load active session messages:', error);
      }
    };

    loadActiveSessionMessages();
  }, [currentSessionId, sessions, sessionsLoading, userNickname]);

  const refreshCustomCommands = useCallback(async () => {
    try {
      const commands = await invoke<SlashCommand[]>('list_slash_commands');
      setCustomCommands(commands);
      console.log('[App] Loaded custom commands:', commands.length);
    } catch (error) {
      console.error('Failed to load custom slash commands:', error);
    }
  }, []);

  // Load custom slash commands on startup
  useEffect(() => {
    refreshCustomCommands();
  }, [refreshCustomCommands]);

  // Listen for workspace-switched events to refresh workspace data and Git info
  useEffect(() => {
    const unlisten = listen<string>('workspace-switched', async () => {
      console.log('[App] workspace-switched event received, refreshing workspace and Git info');
      console.log('[App] Calling refreshWorkspace...');
      await refreshWorkspace();
      console.log('[App] Calling refreshWorkspaces...');
      await refreshWorkspaces();

      // Refresh session list (workspace-specific sessions)
      console.log('[App] Refreshing sessions...');
      await refreshSessions();

      // Load active session (which should have been switched by the backend)
      try {
        console.log('[App] Loading active session...');
        const activeSession = await invoke<Session | null>('get_active_session');
        if (activeSession) {
          console.log('[App] Active session loaded:', activeSession.id);
          // Switch to this session (this will update currentSessionId and messages)
          await switchSession(activeSession.id);
        } else {
          console.log('[App] No active session, clearing messages');
          setMessages([]);
          setTasks([]);
        }
      } catch (error) {
        console.error('[App] Failed to load active session:', error);
      }

      // Reload Git info for the new workspace
      try {
        console.log('[App] Reloading Git info...');
        const info = await invoke<GitInfo>('get_git_info');
        setGitInfo(info);
        console.log('[App] Git info reloaded:', info);
      } catch (error) {
        console.error('[App] Failed to reload Git info:', error);
      }

      console.log('[App] Workspace refresh complete');
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [refreshWorkspace, refreshWorkspaces, refreshSessions]);

  // 入力内容が変更されたときにコマンド/エージェントサジェストを更新
  useEffect(() => {
    const cursorPosition = textareaRef.current?.selectionStart || input.length;
    const spaceIndex = input.indexOf(' ');
    const isCommandPhase = input.startsWith('/') && (spaceIndex === -1 || cursorPosition <= spaceIndex);

    // コマンドサジェスト（コマンド名入力中のみ表示）
    if (isCommandPhase) {
      const commands = filterCommandsWithCustom(input, customCommands);
      setFilteredCommands(commands);
      setShowSuggestions(commands.length > 0);
      setSelectedSuggestionIndex(0);
      setShowAgentSuggestions(false);
    } else {
      setShowSuggestions(false);
    }

    // エージェントサジェスト（@メンション）
    const mentionFilter = getCurrentMention(input, cursorPosition);

    if (mentionFilter !== null) {
      const filtered: Agent[] = [];
      setFilteredAgents(filtered);
      setShowAgentSuggestions(filtered.length > 0);
      setSelectedAgentIndex(0);
    } else {
      setShowAgentSuggestions(false);
    }
  }, [input, customCommands]);

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

  const processInput = useCallback(
    async (rawInput: string, attachedFiles: File[] = []) => {
      if (!rawInput.trim() && attachedFiles.length === 0) {
        return;
      }

      const currentFiles = [...attachedFiles];

      const mentions = extractMentions(rawInput);
      if (mentions.length > 0) {
        console.log('[MENTION EVENT] Agents mentioned:', mentions.map(m => m.agentName));
      }

      const parsed = parseCommand(rawInput);
      let backendInput = rawInput;
      let promptCommandExecuted = false;

      if (parsed.isCommand && parsed.command) {
        addMessage('command', userNickname, rawInput);

        const isBuiltinCommand = isValidCommand(parsed.command);

        if (isBuiltinCommand) {
          switch (parsed.command) {
            case 'help':
              addMessage('system', 'System', getCommandHelp());
              return;
            case 'status':
              addMessage('system', 'System', `Connection: ${status.connection}\nTasks: ${status.activeTasks}\nAgent: ${status.currentAgent}\nApp Status: ${status.mode}`);
              return;
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
              return;
            case 'workspace':
              if (parsed.args && parsed.args.length > 0) {
                const workspaceName = parsed.args.join(' ');
                const targetWorkspace = allWorkspaces.find(ws =>
                  ws.name.toLowerCase() === workspaceName.toLowerCase()
                );
                if (targetWorkspace && currentSessionId) {
                  switchWorkspace(currentSessionId, targetWorkspace.id)
                    .then(() => {
                      addMessage('system', 'System', `✅ Switched to workspace: ${targetWorkspace.name}`);
                    })
                    .catch(err => {
                      addMessage('error', 'System', `Failed to switch workspace: ${err}`);
                    });
                } else if (!targetWorkspace) {
                  addMessage('error', 'System', `Workspace not found: ${workspaceName}\n\nAvailable workspaces:\n${allWorkspaces.map(ws => `- ${ws.name}`).join('\n')}`);
                } else {
                  addMessage('error', 'System', 'No active session');
                }
              } else {
                const workspaceList = allWorkspaces.map(ws =>
                  `${ws.id === workspace?.id ? '📍' : '  '} ${ws.name}${ws.isFavorite ? ' ⭐' : ''}`
                ).join('\n');
                addMessage('system', 'System', `Available workspaces:\n${workspaceList}\n\nUsage: /workspace <name>`);
              }
              return;
            case 'files':
              const fileList = workspaceFiles.length > 0
                ? workspaceFiles.map(f => `📄 ${f.name} (${(f.size / 1024).toFixed(2)} KB)${f.author ? ` - by ${f.author}` : ''}`).join('\n')
                : 'No files in current workspace';
              addMessage('system', 'System', `Files in workspace "${workspace?.name}":\n${fileList}`);
              return;
            case 'mode':
              if (parsed.args && parsed.args.length > 0) {
                const mode = parsed.args[0].toLowerCase();
                const validModes = ['normal', 'concise', 'brief', 'discussion'];

                if (!validModes.includes(mode)) {
                  addMessage('error', 'System', `Invalid mode: ${mode}\n\nAvailable modes:\n- normal (通常)\n- concise (簡潔・300文字)\n- brief (極簡潔・150文字)\n- discussion (議論)`);
                  return;
                }

                try {
                  await invoke('set_conversation_mode', { mode });
                  setConversationMode(mode);
                  const modeLabels: Record<string, string> = {
                    normal: '通常 (Normal)',
                    concise: '簡潔 (300文字)',
                    brief: '極簡潔 (150文字)',
                    discussion: '議論 (Discussion)',
                  };
                  addMessage('system', 'System', `✅ Conversation mode changed to: ${modeLabels[mode]}`);
                } catch (error) {
                  addMessage('error', 'System', `Failed to set conversation mode: ${error}`);
                }
              } else {
                try {
                  const currentMode = await invoke<string>('get_conversation_mode');
                  const modeLabels: Record<string, string> = {
                    normal: '通常 (Normal)',
                    concise: '簡潔 (300文字)',
                    brief: '極簡潔 (150文字)',
                    discussion: '議論 (Discussion)',
                  };
                  addMessage('system', 'System', `Current mode: ${modeLabels[currentMode] || currentMode}\n\nUsage: /mode <normal|concise|brief|discussion>`);
                } catch (error) {
                  addMessage('error', 'System', 'Usage: /mode <normal|concise|brief|discussion>');
                }
              }
              return;
            case 'talk':
              if (parsed.args && parsed.args.length > 0) {
                const style = parsed.args[0].toLowerCase();
                const validStyles = ['brainstorm', 'casual', 'decision_making', 'debate', 'problem_solving', 'review', 'planning', 'none'];

                if (!validStyles.includes(style)) {
                  addMessage('error', 'System', `Invalid style: ${style}\n\nAvailable styles:\n- brainstorm (ブレインストーミング)\n- casual (カジュアル)\n- decision_making (意思決定)\n- debate (議論)\n- problem_solving (問題解決)\n- review (レビュー)\n- planning (計画)\n- none (解除)`);
                  return;
                }

                try {
                  const styleValue = style === 'none' ? null : style;
                  await invoke('set_talk_style', { style: styleValue });
                  setTalkStyle(styleValue);
                  const styleLabels: Record<string, string> = {
                    brainstorm: 'ブレインストーミング (Brainstorm)',
                    casual: 'カジュアル (Casual)',
                    decision_making: '意思決定 (Decision Making)',
                    debate: '議論 (Debate)',
                    problem_solving: '問題解決 (Problem Solving)',
                    review: 'レビュー (Review)',
                    planning: '計画 (Planning)',
                    none: '解除 (None)',
                  };
                  addMessage('system', 'System', `✅ Talk style changed to: ${styleLabels[style]}`);
                } catch (error) {
                  addMessage('error', 'System', `Failed to set talk style: ${error}`);
                }
              } else {
                try {
                  const currentStyle = await invoke<string | null>('get_talk_style');
                  const styleLabels: Record<string, string> = {
                    brainstorm: 'ブレインストーミング (Brainstorm)',
                    casual: 'カジュアル (Casual)',
                    decision_making: '意思決定 (Decision Making)',
                    debate: '議論 (Debate)',
                    problem_solving: '問題解決 (Problem Solving)',
                    review: 'レビュー (Review)',
                    planning: '計画 (Planning)',
                  };
                  const currentLabel = currentStyle ? (styleLabels[currentStyle] || currentStyle) : 'Not set';
                  addMessage('system', 'System', `Current talk style: ${currentLabel}\n\nUsage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>`);
                } catch (error) {
                  addMessage('error', 'System', 'Usage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>');
                }
              }
              return;
            default:
              break;
          }
        } else {
          try {
            const customCommand = await invoke<SlashCommand | null>('get_slash_command', { name: parsed.command });

            if (!customCommand) {
              addMessage('error', 'System', `Unknown command: /${parsed.command}\n\nType /help for available commands.`);
              return;
            }

            const argsText = parsed.args?.join(' ') ?? '';
            const expanded = await invoke<ExpandedSlashCommand>('expand_command_template', {
              commandName: customCommand.name,
              args: argsText,
            });

            if (customCommand.type === 'prompt') {
              addMessage('system', 'System', `✨ Executing custom command: /${customCommand.name}`);
              backendInput = expanded.content;
              promptCommandExecuted = true;
            } else {
              addMessage('system', 'System', `⚡ Executing shell command: /${customCommand.name}`);
              if (expanded.workingDir) {
                addMessage('shell_output', 'System', `(cwd: ${expanded.workingDir})`);
              }
              addMessage('shell_output', 'System', `$ ${expanded.content}`);

              try {
                const output = await invoke<string>('execute_shell_command', {
                  command: expanded.content,
                  workingDir: expanded.workingDir ?? null,
                });
                addMessage('shell_output', 'System', output || '(no output)');
              } catch (shellError) {
                addMessage('error', 'System', `Shell command failed: ${shellError}`);
              }
              return;
            }
          } catch (error) {
            console.error('Failed to execute custom command:', error);
            addMessage('error', 'System', `Failed to execute command: ${error}`);
            return;
          }
        }
      }

      if (promptCommandExecuted && !backendInput.trim()) {
        addMessage('error', 'System', `Command ${rawInput} produced empty content.`);
        return;
      }

      let messageText = backendInput;

      if (currentFiles.length > 0) {
        const fileInfo = currentFiles.map(f => `📎 ${f.name} (${(f.size / 1024).toFixed(1)} KB)`).join('\n');
        messageText = backendInput ? `${backendInput}\n\n${fileInfo}` : fileInfo;
      }

      if (includeWorkspaceInPrompt && workspaceFiles.length > 0) {
        const uploadedDir = workspace?.workspaceDir
          ? `${workspace.workspaceDir}/resources/uploaded/`
          : '~/.orcs/workspaces/{workspace-id}/resources/uploaded/';

        const workspaceInfo = [
          '',
          '---',
          'Available workspace files:',
          ...workspaceFiles.map(f => `  - ${f.name} (${(f.size / 1024).toFixed(1)} KB)`),
          '',
          `Workspace location: ${uploadedDir}`,
        ].join('\n');
        messageText = messageText + workspaceInfo;
      }

      addMessage('user', userNickname, messageText);

      setIsAiThinking(true);
      setThinkingPersona('AI Assistant');

      try {
        // Upload files to workspace and get paths
        const filePaths: string[] = [];
        if (currentFiles.length > 0 && workspace) {
          for (const file of currentFiles) {
            try {
              const arrayBuffer = await file.arrayBuffer();
              const fileData = Array.from(new Uint8Array(arrayBuffer));
              const uploadedFile = await invoke<{ path: string }>("upload_file_from_bytes", {
                workspaceId: workspace.id,
                filename: file.name,
                fileData: fileData,
                sessionId: currentSessionId || null,
                messageTimestamp: null,
                author: null,
              });
              filePaths.push(uploadedFile.path);
              console.log('[FILE] Uploaded file:', file.name, 'to', uploadedFile.path);
            } catch (uploadError) {
              console.error('[FILE] Failed to upload file:', file.name, uploadError);
              addMessage('error', 'System', `Failed to upload file ${file.name}: ${uploadError}`);
            }
          }
        }

        const result = await invoke<InteractionResult>("handle_input", {
          input: backendInput,
          filePaths: filePaths.length > 0 ? filePaths : null,
        });

        if (result.type === 'NewDialogueMessages') {
          console.log('[BATCH] Received', result.data.length, 'messages (already streamed)');
          // Note: Errors are also returned as NewDialogueMessages (empty array) after streaming
        } else if (result.type === 'NewMessage') {
          // This should not happen anymore, but keep for backward compatibility
          console.error('[ERROR] Backend returned error:', result.data);
          addMessage('error', '', result.data);

          notifications.show({
            title: 'Agent Error',
            message: result.data,
            color: 'red',
            icon: '❌',
            autoClose: 10000,
          });
        }

        await saveCurrentSession();
      } catch (error) {
        console.error("Error calling backend:", error);
        addMessage('error', 'System', `Error: ${error}`);
      } finally {
        setIsAiThinking(false);
      }
    },
    [
      addMessage,
      allWorkspaces,
      currentSessionId,
      getCommandHelp,
      includeWorkspaceInPrompt,
      invoke,
      saveCurrentSession,
      setIsAiThinking,
      setStatus,
      setTasks,
      setThinkingPersona,
      status.activeTasks,
      status.connection,
      status.currentAgent,
      status.mode,
      switchWorkspace,
      userNickname,
      workspace,
      workspaceFiles,
    ]
  );

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

  // Workspace からファイルをアタッチするハンドラー
  const handleAttachFileFromWorkspace = (file: File) => {
    setAttachedFiles((prev) => [...prev, file]);

    // Show toast notification instead of adding to chat history
    notifications.show({
      title: 'File Attached',
      message: `${file.name} from workspace`,
      color: 'blue',
      icon: '📎',
    });
  };

  // ワークスペースファイルからセッションに移動するハンドラー
  const handleGoToSessionFromFile = (sessionId: string) => {
    const session = sessions.find(s => s.id === sessionId);
    if (session) {
      handleSessionSelect(session);
    } else {
      addMessage('error', 'System', `Session not found: ${sessionId}`);
    }
  };

  // メッセージをワークスペースに保存するハンドラー
  const handleSaveMessageToWorkspace = async (message: Message) => {
    try {
      // ファイル名を生成（タイムスタンプ + 作者名）
      const timestamp = message.timestamp.toISOString().replace(/[:.]/g, '-');
      const filename = `${timestamp}_${message.author}_${message.type}.txt`;

      // メッセージテキストをバイト配列に変換
      const encoder = new TextEncoder();
      const data = encoder.encode(message.text);
      const fileData = Array.from(data);

      // ワークスペースIDを取得
      const workspace = await invoke<{ id: string }>('get_current_workspace');

      // ワークスペースに保存（セッションID、メッセージタイムスタンプ、作者を含める）
      await invoke('upload_file_from_bytes', {
        workspaceId: workspace.id,
        filename: filename,
        fileData: fileData,
        sessionId: currentSessionId || null,
        messageTimestamp: message.timestamp.toISOString(),
        author: message.author,
      });

      // ワークスペースのファイルリストを更新
      await refreshWorkspace();

      // Toast notification instead of system message
      notifications.show({
        title: 'File saved',
        message: `${filename}`,
        color: 'green',
        icon: '💾',
      });
    } catch (err) {
      console.error('Failed to save message to workspace:', err);
      notifications.show({
        title: 'Failed to save message',
        message: String(err),
        color: 'red',
      });
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
      const restoredMessages = convertSessionToMessages(fullSession, userNickname);
      setMessages(restoredMessages);

      // AppModeをステータスに反映
      if (isIdleMode(fullSession.app_mode)) {
        setStatus(prev => ({ ...prev, mode: 'Idle' }));
      } else {
        // AwaitingConfirmation mode
        setStatus(prev => ({ ...prev, mode: 'Awaiting' }));
        console.log('Session has AwaitingConfirmation mode:', fullSession.app_mode);
      }

      // Show toast notification instead of adding to chat history
      notifications.show({
        title: 'Session Switched',
        message: `${session.title} (${restoredMessages.length} messages restored)`,
        color: 'blue',
        icon: '✅',
      });

      // Scroll to bottom after session switch
      setTimeout(() => {
        if (viewport.current) {
          viewport.current.scrollTo({
            top: viewport.current.scrollHeight,
            behavior: "smooth",
          });
        }
      }, 100);
    } catch (err) {
      addMessage('error', 'System', `Failed to switch session: ${err}`);
    }
  };

  const handleSessionDelete = async (sessionId: string) => {
    try {
      await deleteSession(sessionId);
      // Show toast notification
      notifications.show({
        title: 'Session Deleted',
        message: 'The session has been removed',
        color: 'red',
        icon: '🗑️',
      });
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
      // Show toast notification
      notifications.show({
        title: 'New Session Created',
        message: 'Started a fresh conversation',
        color: 'green',
        icon: '✨',
      });
    } catch (err) {
      addMessage('error', 'System', `Failed to create session: ${err}`);
    }
  };

  const handleConversationModeChange = (mode: string) => {
    setConversationMode(mode);
  };

  const handleTalkStyleChange = (style: string | null) => {
    setTalkStyle(style);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!input.trim() && attachedFiles.length === 0) {
      return;
    }

    const currentInput = input;
    const currentFiles = [...attachedFiles];
    setInput("");
    setAttachedFiles([]);
    setShowSuggestions(false);
    setShowAgentSuggestions(false);
    await processInput(currentInput, currentFiles);
  };

  const handleRunSlashCommand = useCallback(
    async (command: SlashCommand, args: string) => {
      setShowSuggestions(false);
      setShowAgentSuggestions(false);
      setInput('');
      const trimmedArgs = args.trim();
      const commandInput = trimmedArgs ? `/${command.name} ${trimmedArgs}` : `/${command.name}`;
      await processInput(commandInput);
    },
    [processInput]
  );

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
        <Navbar
          sessions={sessions}
          currentSessionId={currentSessionId}
          currentWorkspaceId={workspace?.id}
          onSessionSelect={handleSessionSelect}
          onSessionDelete={handleSessionDelete}
          onSessionRename={handleSessionRename}
          onNewSession={handleNewSession}
          tasks={tasks}
          onTaskToggle={handleTaskToggle}
          onTaskDelete={handleTaskDelete}
          onAttachFile={handleAttachFileFromWorkspace}
          includeWorkspaceInPrompt={includeWorkspaceInPrompt}
          onToggleIncludeWorkspaceInPrompt={setIncludeWorkspaceInPrompt}
          onGoToSession={handleGoToSessionFromFile}
          onRefreshWorkspace={refreshWorkspace}
          onMessage={addMessage}
          onSlashCommandsUpdated={refreshCustomCommands}
          onRunSlashCommand={handleRunSlashCommand}
          onConversationModeChange={handleConversationModeChange}
          onTalkStyleChange={handleTalkStyleChange}
        />
      </AppShell.Navbar>

      {/* メインコンテンツ */}
      <AppShell.Main>
        <Container size="md" h="100vh" p="md" style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <Stack style={{ flex: 1, minHeight: 0 }} gap="md">
            <Group gap="sm" justify="space-between">
              <Group gap="sm">
                <Burger opened={navbarOpened} onClick={toggleNavbar} size="sm" />
                <Text size="xl" fw={700}>ORCS</Text>
              </Group>
              <Group gap="md">
                {/* Workspace Switcher */}
                <Group gap="xs">
                  <WorkspaceSwitcher sessionId={currentSessionId} />
                  {workspace && (
                    <>
                      <Text size="sm" c="dimmed">Workspace:</Text>
                      <Badge size="sm" variant="dot" color="green">
                        {workspace.name}
                      </Badge>
                    </>
                  )}
                </Group>

                {/* Session Info */}
                {currentSessionId && (
                  <Group gap="xs">
                    <Text size="sm" c="dimmed">Session:</Text>
                    <Badge size="lg" variant="light">
                      {sessions.find(s => s.id === currentSessionId)?.title || 'Untitled'}
                    </Badge>
                    <Badge size="sm" color="gray" variant="outline">
                      {getMessageCount(sessions.find(s => s.id === currentSessionId)!) || 0} msgs
                    </Badge>
                  </Group>
                )}

                {/* Settings Menu */}
                <SettingsMenu />
              </Group>
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
                    <MessageItem
                      key={message.id}
                      message={message}
                      onSaveToWorkspace={handleSaveMessageToWorkspace}
                    />
                  ))}
                  {isAiThinking && (
                    <ThinkingIndicator personaName={thinkingPersona} />
                  )}
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
              gitInfo={gitInfo}
              participatingAgentsCount={0}
              autoMode={autoMode}
              conversationMode={conversationMode}
              talkStyle={talkStyle}
            />
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}

export default App;
