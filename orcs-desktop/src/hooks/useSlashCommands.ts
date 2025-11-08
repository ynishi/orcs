/**
 * useSlashCommands - SlashCommandの処理を管理するカスタムフック
 * App.tsxから分離して、保守性を向上
 */
import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import { parseCommand, isValidCommand, getCommandHelp } from '../utils/commandParser';
import { handleSystemMessage, conversationMessage, commandMessage, MessageSeverity } from '../utils/systemMessage';
import type { MessageType } from '../types/message';
import type { StatusInfo } from '../types/status';
import type { Workspace } from '../types/workspace';
import type { UploadedFile } from '../types/workspace';
import type { SlashCommand } from '../types/slash_command';

interface UseSlashCommandsProps {
  addMessage: (type: MessageType, author: string, text: string) => void;
  saveCurrentSession: () => Promise<void>;
  status: StatusInfo;
  currentSessionId: string | null;
  workspace: Workspace | null;
  allWorkspaces: Workspace[];
  workspaceFiles: UploadedFile[];
  switchWorkspace: (sessionId: string, workspaceId: string) => Promise<void>;
  setConversationMode: (mode: string) => void;
  setTalkStyle: (style: string | null) => void;
  setInput: (value: string) => void;
  refreshPersonas: () => Promise<void>;
  refreshSessions: () => Promise<void>;
}

export function useSlashCommands({
  addMessage,
  saveCurrentSession,
  status,
  currentSessionId,
  workspace,
  allWorkspaces,
  workspaceFiles,
  switchWorkspace,
  setConversationMode,
  setTalkStyle,
  setInput,
  refreshPersonas,
  refreshSessions,
}: UseSlashCommandsProps) {
  /**
   * SlashCommandを処理する
   * @returns {boolean} promptCommandExecuted - プロンプトコマンドが実行されたか
   */
  const handleSlashCommand = useCallback(
    async (rawInput: string): Promise<boolean> => {
      const parsed = parseCommand(rawInput);
      let promptCommandExecuted = false;

      if (!parsed.isCommand || !parsed.command) {
        return promptCommandExecuted;
      }

      handleSystemMessage(commandMessage(rawInput), addMessage);

      const isBuiltinCommand = isValidCommand(parsed.command);

      if (isBuiltinCommand) {
        switch (parsed.command) {
          case 'help':
            handleSystemMessage(conversationMessage(getCommandHelp()), addMessage);
            await saveCurrentSession();
            break;

          case 'status':
            handleSystemMessage(
              conversationMessage(
                `Connection: ${status.connection}\nTasks: ${status.activeTasks}\nAgent: ${status.currentAgent}\nApp Status: ${status.mode}`
              ),
              addMessage
            );
            await saveCurrentSession();
            break;

          case 'task':
            handleSystemMessage(
              conversationMessage('Use the 🚀 button on messages to execute them as tasks', 'info'),
              addMessage
            );
            await saveCurrentSession();
            break;

          case 'expert':
            if (parsed.args && parsed.args.length > 0) {
              const expertise = parsed.args.join(' ');
              try {
                notifications.show({
                  title: 'Creating Expert',
                  message: `Generating expert for: ${expertise}...`,
                  color: 'blue',
                  autoClose: false,
                  id: 'expert-creation',
                });

                const persona = await invoke<import('../types/agent').PersonaConfig>('create_adhoc_persona', {
                  expertise,
                });

                notifications.hide('expert-creation');

                await invoke('append_system_messages', {
                  messages: [
                    {
                      content: `🔶 Expert persona created: ${persona.name} ${persona.icon || '🔶'}\nRole: ${persona.role}\nBackground: ${persona.background}`,
                      messageType: 'info',
                      severity: 'info',
                    },
                  ],
                });

                await refreshPersonas();
                await refreshSessions();
              } catch (error) {
                console.error('Failed to create expert:', error);
                notifications.hide('expert-creation');

                await invoke('append_system_messages', {
                  messages: [
                    {
                      content: `❌ Failed to create expert: ${error}`,
                      messageType: 'error',
                      severity: 'error',
                    },
                  ],
                });
              }
            } else {
              handleSystemMessage(
                conversationMessage('Usage: /expert <expertise>\nExample: /expert 映画制作プロセス', 'error'),
                addMessage
              );
            }
            await saveCurrentSession();
            break;

          case 'blueprint':
            if (parsed.args && parsed.args.length > 0) {
              const taskDescription = parsed.args.join(' ');
              const blueprintPrompt = `# Task: Create BlueprintWorkflow for ORCS Task Execution

Convert the following into a BlueprintWorkflow format:

${taskDescription}

## Output Format

Provide a BlueprintWorkflow with:

1. **Goal**: Clear, measurable goal statement
2. **Workflow Steps**: Numbered steps with clear deliverables
3. **Output Type**: Classify each step (📋 Clarification, 💡 Proposal, 📝 Documentation, 🔧 Implementation, ✅ Validation)
4. **Dependencies**: Note which steps can run in parallel
5. **Estimated Time**: Total execution time estimate

Example format:
\`\`\`
Goal: [Goal statement]

Workflow:
1. **[Step Name]** (📋 Type): [Description]
2. **[Step Name]** (💡 Type): [Description]
...

Dependencies: 1→2→3 (or note parallel opportunities)
Estimated time: X minutes
\`\`\`

Generate the BlueprintWorkflow now.`;

              setInput(blueprintPrompt);
              setTimeout(() => {
                const textarea = document.querySelector('textarea');
                if (textarea) {
                  const event = new KeyboardEvent('keydown', { key: 'Enter', ctrlKey: true });
                  textarea.dispatchEvent(event);
                }
              }, 100);
            } else {
              handleSystemMessage(
                conversationMessage(
                  'Usage: /blueprint <task description>\nExample: /blueprint Create technical article about Rust',
                  'error'
                ),
                addMessage
              );
            }
            await saveCurrentSession();
            break;

          case 'workspace':
            if (parsed.args && parsed.args.length > 0) {
              const workspaceName = parsed.args.join(' ');
              const targetWorkspace = allWorkspaces.find(
                (ws) => ws.name.toLowerCase() === workspaceName.toLowerCase()
              );
              if (targetWorkspace && currentSessionId) {
                try {
                  await switchWorkspace(currentSessionId, targetWorkspace.id);
                  handleSystemMessage(
                    conversationMessage(`✅ Switched to workspace: ${targetWorkspace.name}`),
                    addMessage
                  );
                } catch (err) {
                  handleSystemMessage(
                    conversationMessage(`Failed to switch workspace: ${err}`, 'error'),
                    addMessage
                  );
                }
              } else if (!targetWorkspace) {
                handleSystemMessage(
                  conversationMessage(
                    `Workspace not found: ${workspaceName}\n\nAvailable workspaces:\n${allWorkspaces.map((ws) => `- ${ws.name}`).join('\n')}`,
                    'error'
                  ),
                  addMessage
                );
              } else {
                handleSystemMessage(conversationMessage('No active session', 'error'), addMessage);
              }
            } else {
              const workspaceList = allWorkspaces
                .map(
                  (ws) =>
                    `${ws.id === workspace?.id ? '📍' : '  '} ${ws.name}${ws.isFavorite ? ' ⭐' : ''}`
                )
                .join('\n');
              handleSystemMessage(
                conversationMessage(`Available workspaces:\n${workspaceList}\n\nUsage: /workspace <name>`),
                addMessage
              );
            }
            await saveCurrentSession();
            break;

          case 'files':
            const fileList =
              workspaceFiles.length > 0
                ? workspaceFiles
                    .map(
                      (f) =>
                        `📄 ${f.name} (${(f.size / 1024).toFixed(2)} KB)${f.author ? ` - by ${f.author}` : ''}`
                    )
                    .join('\n')
                : 'No files in current workspace';
            handleSystemMessage(
              conversationMessage(`Files in workspace "${workspace?.name}":\n${fileList}`),
              addMessage
            );
            await saveCurrentSession();
            break;

          case 'mode':
            if (parsed.args && parsed.args.length > 0) {
              const mode = parsed.args[0].toLowerCase();
              const validModes = ['normal', 'concise', 'brief', 'discussion'];

              if (!validModes.includes(mode)) {
                handleSystemMessage(
                  conversationMessage(
                    `Invalid mode: ${mode}\n\nAvailable modes:\n- normal (通常)\n- concise (簡潔・300文字)\n- brief (極簡潔・150文字)\n- discussion (議論)`,
                    'error'
                  ),
                  addMessage
                );
                return promptCommandExecuted;
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
                handleSystemMessage(
                  conversationMessage(`✅ Conversation mode changed to: ${modeLabels[mode]}`),
                  addMessage
                );
              } catch (error) {
                handleSystemMessage(
                  conversationMessage(`Failed to set conversation mode: ${error}`, 'error'),
                  addMessage
                );
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
                handleSystemMessage(
                  conversationMessage(
                    `Current mode: ${modeLabels[currentMode] || currentMode}\n\nUsage: /mode <normal|concise|brief|discussion>`
                  ),
                  addMessage
                );
              } catch (error) {
                handleSystemMessage(
                  conversationMessage('Usage: /mode <normal|concise|brief|discussion>', 'error'),
                  addMessage
                );
              }
            }
            await saveCurrentSession();
            break;

          case 'talk':
            if (parsed.args && parsed.args.length > 0) {
              const style = parsed.args[0].toLowerCase();
              const validStyles = [
                'brainstorm',
                'casual',
                'decision_making',
                'debate',
                'problem_solving',
                'review',
                'planning',
                'none',
              ];

              if (!validStyles.includes(style)) {
                handleSystemMessage(
                  conversationMessage(
                    `Invalid style: ${style}\n\nAvailable styles:\n- brainstorm (ブレインストーミング)\n- casual (カジュアル)\n- decision_making (意思決定)\n- debate (議論)\n- problem_solving (問題解決)\n- review (レビュー)\n- planning (計画)\n- none (解除)`,
                    'error'
                  ),
                  addMessage
                );
                await saveCurrentSession();
                return promptCommandExecuted;
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
                handleSystemMessage(
                  conversationMessage(`✅ Talk style changed to: ${styleLabels[style]}`),
                  addMessage
                );
              } catch (error) {
                handleSystemMessage(
                  conversationMessage(`Failed to set talk style: ${error}`, 'error'),
                  addMessage
                );
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
                const currentLabel = currentStyle ? styleLabels[currentStyle] || currentStyle : 'Not set';
                handleSystemMessage(
                  conversationMessage(
                    `Current talk style: ${currentLabel}\n\nUsage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>`
                  ),
                  addMessage
                );
              } catch (error) {
                handleSystemMessage(
                  conversationMessage(
                    'Usage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>',
                    'error'
                  ),
                  addMessage
                );
              }
            }
            await saveCurrentSession();
            break;

          default:
            break;
        }
      } else {
        // カスタムコマンドの処理
        const persistedSystemMessages: {
          content: string;
          messageType: MessageType;
          severity?: MessageSeverity;
        }[] = [];

        const persistMessages = async () => {
          if (persistedSystemMessages.length === 0) {
            return;
          }
          const messagesToPersist = [...persistedSystemMessages];
          persistedSystemMessages.length = 0;
          try {
            await invoke('append_system_messages', { messages: messagesToPersist });
          } catch (persistError) {
            console.error('Failed to persist slash command messages:', persistError);
            persistedSystemMessages.unshift(...messagesToPersist);
          }
        };

        const queuePersistedMessage = (
          content: string,
          messageType: MessageType,
          severity?: MessageSeverity
        ) => {
          persistedSystemMessages.push({ content, messageType, severity });
        };

        try {
          const customCommand = await invoke<SlashCommand | null>('get_slash_command', {
            name: parsed.command,
          });

          if (!customCommand) {
            handleSystemMessage(
              conversationMessage(`Unknown command: /${parsed.command}\nType /help for available commands`, 'error'),
              addMessage
            );
            await saveCurrentSession();
            return promptCommandExecuted;
          }

          // カスタムコマンド実行
          const argsStr = parsed.args ? parsed.args.join(' ') : '';

          try {
            const result = await invoke<import('../types/slash_command').ExpandedSlashCommand>('run_slash_command', {
              name: parsed.command,
              args: argsStr,
            });

            if (result.has_prompt_template) {
              promptCommandExecuted = true;
            }

            if (result.immediate_messages && result.immediate_messages.length > 0) {
              for (const msg of result.immediate_messages) {
                if (msg.persist_to_session) {
                  queuePersistedMessage(msg.content, msg.message_type as import('../types/message').MessageType, msg.severity as any);
                }
                addMessage(msg.message_type as import('../types/message').MessageType, 'System', msg.content);
              }
            }

            if (result.prompt_to_send) {
              addMessage('user', 'User', result.prompt_to_send);
            }

            await persistMessages();
            await saveCurrentSession();
          } catch (error) {
            console.error(`Failed to run slash command /${parsed.command}:`, error);
            queuePersistedMessage(`Failed to run slash command: ${error}`, 'error', 'error');
            await persistMessages();
            await saveCurrentSession();
          }
        } catch (error) {
          console.error('Failed to fetch slash command:', error);
          handleSystemMessage(
            conversationMessage(`Failed to fetch command: ${error}`, 'error'),
            addMessage
          );
          await saveCurrentSession();
        }
      }

      return promptCommandExecuted;
    },
    [
      addMessage,
      saveCurrentSession,
      status,
      currentSessionId,
      workspace,
      allWorkspaces,
      workspaceFiles,
      switchWorkspace,
      setConversationMode,
      setTalkStyle,
      setInput,
      refreshPersonas,
      refreshSessions,
    ]
  );

  return { handleSlashCommand };
}


