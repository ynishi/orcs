/**
 * useSlashCommands - SlashCommandの処理を管理するカスタムフック
 * App.tsxから分離して、保守性を向上
 */
import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import { parseCommand, isValidCommand, getCommandHelp } from '../utils/commandParser';
import {
  handleSystemMessage,
  conversationMessage,
  commandMessage,
  shellOutputMessage,
} from '../utils/systemMessage';
import type { MessageType } from '../types/message';
import type { StatusInfo } from '../types/status';
import type { Workspace } from '../types/workspace';
import type { UploadedFile } from '../types/workspace';
import type { SlashCommand, ExpandedSlashCommand } from '../types/slash_command';

export interface SlashCommandResult {
  nextInput: string | null;
  suppressUserMessage?: boolean;
}

export type SlashCommandSource = 'user' | 'agent';

export interface SlashCommandOptions {
  source?: SlashCommandSource;
  actorName?: string;
  autoSubmit?: boolean;
}

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
   * @returns {Promise<SlashCommandResult>} - backendに送るテキストと表示制御
   */
  const handleSlashCommand = useCallback(
    async (
      rawInput: string,
      options: SlashCommandOptions = {}
    ): Promise<SlashCommandResult> => {
      const { source = 'user', actorName, autoSubmit = false } = options;
      const parsed = parseCommand(rawInput);

      if (!parsed.isCommand || !parsed.command) {
        return { nextInput: rawInput, suppressUserMessage: false };
      }

      let nextInput: string | null = null;
      let suppressUserMessage = false;
      const provenanceActor =
        source === 'agent' ? `${actorName ?? 'Agent'}'s ` : '';
      const commandProvenance = `\n\n(Generated via ${provenanceActor}/${parsed.command} command.)`;
      const commandLabel =
        source === 'agent'
          ? `${actorName ?? 'Agent'} issued ${rawInput}`
          : rawInput;

      // NOTE: Slash command log entries are critical for UI parity.
      // We add the live bubble via handleSystemMessage, but ALSO persist them
      // immediately so a reload shows the same COMMAND card (session restore pulls
      // from system_messages).  This code path has regressed multiple times,
      // so keep it colocated with the live rendering.
      handleSystemMessage(commandMessage(commandLabel), addMessage);
      try {
        await invoke('append_system_messages', {
          messages: [
            {
              content: commandLabel,
              messageType: 'command',
              severity: 'info',
            },
          ],
        });
      } catch (error) {
        console.error('Failed to persist command log:', error);
      }

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
            if (parsed.args && parsed.args.length > 0) {
              const taskDescription = parsed.args.join(' ');
              try {
                handleSystemMessage(
                  conversationMessage(`🚀 Executing task: ${taskDescription}`, 'info'),
                  addMessage
                );

                const taskResult = await invoke<string>('execute_message_as_task', {
                  messageContent: taskDescription,
                });

                handleSystemMessage(
                  conversationMessage(`✅ Task completed:\n${taskResult}`, 'info'),
                  addMessage
                );
              } catch (error) {
                console.error('Failed to execute task:', error);
                handleSystemMessage(
                  conversationMessage(`❌ Task execution failed: ${error}`, 'error'),
                  addMessage
                );
              }
            } else {
              handleSystemMessage(
                conversationMessage('Usage: /task <description>\nExample: /task Create a README file', 'error'),
                addMessage
              );
            }
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

                const successMessage = `🔶 Expert persona created: ${persona.name} ${persona.icon || '🔶'}\nRole: ${persona.role}\nBackground: ${persona.background}`;

                // Display immediately in UI
                handleSystemMessage(conversationMessage(successMessage, 'info'), addMessage);

                // Persist to backend
                await invoke('append_system_messages', {
                  messages: [
                    {
                      content: successMessage,
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

                const errorMessage = `❌ Failed to create expert: ${error}`;

                // Display immediately in UI
                handleSystemMessage(conversationMessage(errorMessage, 'error'), addMessage);

                // Persist to backend
                await invoke('append_system_messages', {
                  messages: [
                    {
                      content: errorMessage,
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

              nextInput = blueprintPrompt;
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
                return { nextInput: null, suppressUserMessage: true };
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
                return { nextInput: null, suppressUserMessage: true };
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
            return { nextInput, suppressUserMessage };
          }

          // カスタムコマンド実行
          const argsStr = parsed.args ? parsed.args.join(' ') : '';

          try {
            const expanded = await invoke<ExpandedSlashCommand>('expand_command_template', {
              commandName: parsed.command,
              args: argsStr || null,
            });

            if (customCommand.type === 'prompt') {
              const expandedPrompt = expanded.content.trim();
              nextInput = expandedPrompt ? `${expandedPrompt}${commandProvenance}` : '';
              if (!nextInput.trim()) {
                handleSystemMessage(
                  conversationMessage(`Command /${parsed.command} produced empty content.`, 'error'),
                  addMessage
                );
                nextInput = null;
                suppressUserMessage = true;
              }
            } else if (customCommand.type === 'task') {
              // Execute task command
              try {
                handleSystemMessage(
                  conversationMessage(`🚀 Executing task: ${parsed.command}...`, 'info'),
                  addMessage
                );

                const taskResult = await invoke<string>('execute_task_command', {
                  commandName: parsed.command,
                  args: argsStr || null,
                });

                handleSystemMessage(
                  conversationMessage(`✅ Task completed:\n${taskResult}`, 'info'),
                  addMessage
                );

                nextInput = null;
                suppressUserMessage = true;
              } catch (error) {
                console.error(`Failed to execute task command /${parsed.command}:`, error);
                handleSystemMessage(
                  conversationMessage(`❌ Task execution failed: ${error}`, 'error'),
                  addMessage
                );
                nextInput = null;
                suppressUserMessage = true;
              }
            } else {
              // Shell command execution
              try {
                const executionOutput = await invoke<string>('execute_shell_command', {
                  command: expanded.content,
                  working_dir: expanded.workingDir ?? null,
                });
                const trimmedOutput = executionOutput.trim();
                const shellHeader = `/${parsed.command} ($ ${expanded.content.trim()})`;
                const shellBody = trimmedOutput
                  ? `\`\`\`\n${trimmedOutput}\n\`\`\``
                  : '_No output_';
                const shellMessage = `${shellHeader}\n${shellBody}`;

                // 1. Display immediately in UI
                handleSystemMessage(shellOutputMessage(shellMessage), addMessage);

                // 2. Persist to Backend as ContextInfo (survives session switches, no agent reaction)
                // Persist shell output as well so reload shows the dark card.
                // Without this, the UI only had the immediate toast.
                await invoke('append_system_messages', {
                  messages: [
                    {
                      content: shellMessage,
                      messageType: 'shell_output',
                      severity: 'info',
                    },
                  ],
                });

                // Don't send to agent (ContextInfo is stored but doesn't trigger reactions)
                nextInput = null;
                suppressUserMessage = true;
              } catch (error) {
                console.error(`Failed to run slash command /${parsed.command}:`, error);
                handleSystemMessage(
                  conversationMessage(`Failed to run slash command: ${error}`, 'error'),
                  addMessage
                );
                nextInput = null;
                suppressUserMessage = true;
              }
            }

            await saveCurrentSession();
          } catch (error) {
            console.error('Failed to expand slash command:', error);
            handleSystemMessage(
              conversationMessage(`Failed to expand command: ${error}`, 'error'),
              addMessage
            );
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

      if (
        autoSubmit &&
        nextInput !== null &&
        typeof nextInput === 'string' &&
        nextInput.trim().length > 0
      ) {
        try {
          await invoke('publish_session_event', {
            event: {
              type: 'user_input',
              content: nextInput,
            },
          });
          await saveCurrentSession();
        } catch (error) {
          console.error('Failed to submit generated command content:', error);
          handleSystemMessage(
            conversationMessage(
              `Failed to submit generated command content: ${error}`,
              'error'
            ),
            addMessage
          );
        }
        return { nextInput: null, suppressUserMessage: true };
      }

      return { nextInput, suppressUserMessage };
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
