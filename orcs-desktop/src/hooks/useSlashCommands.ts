/**
 * useSlashCommands - SlashCommandの処理を管理するカスタムフック
 * App.tsxから分離して、保守性を向上
 */
import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import { parseCommand, isValidCommand, getCommandHelp } from '../utils/commandParser';
import {
  handleAndPersistSystemMessage,
  conversationMessage,
  commandMessage,
  shellOutputMessage,
} from '../utils/systemMessage';
import type { MessageType } from '../types/message';
import type { StatusInfo } from '../types/status';
import type { Workspace } from '../types/workspace';
import type { UploadedFile } from '../types/workspace';
import type { SlashCommand, ExpandedSlashCommand } from '../types/slash_command';
import type { SearchResult, SearchScope as SearchScopeType } from '../types/search';

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
      try {
        await handleAndPersistSystemMessage(
          commandMessage(commandLabel),
          addMessage,
          invoke
        );
      } catch (error) {
        console.error('Failed to persist command log:', error);
      }

      const isBuiltinCommand = isValidCommand(parsed.command);

      if (isBuiltinCommand) {
        switch (parsed.command) {
          case 'help':
            await handleAndPersistSystemMessage(
              conversationMessage(getCommandHelp()),
              addMessage,
              invoke
            );
            await saveCurrentSession();
            break;

          case 'status':
            await handleAndPersistSystemMessage(
              conversationMessage(
                `Connection: ${status.connection}\nTasks: ${status.activeTasks}\nAgent: ${status.currentAgent}\nApp Status: ${status.mode}`
              ),
              addMessage,
              invoke
            );
            await saveCurrentSession();
            break;

          case 'agents':
            try {
              const personas = await invoke<import('../types/agent').PersonaConfig[]>('get_personas');
              const agentList = personas.length > 0
                ? personas.map((p) => {
                    const icon = p.icon || '🤖';
                    const source = p.source === 'Adhoc' ? '(Adhoc)' : '';
                    return `${icon} ${p.name} - ${p.role} ${source}`;
                  }).join('\n')
                : 'No agents available';

              await handleAndPersistSystemMessage(
                conversationMessage(`Available agents:\n${agentList}`),
                addMessage,
                invoke
              );
            } catch (error) {
              console.error('Failed to get agents:', error);
              await handleAndPersistSystemMessage(
                conversationMessage(`Failed to get agents: ${error}`, 'error'),
                addMessage,
                invoke
              );
            }
            await saveCurrentSession();
            break;

          case 'task':
            if (parsed.args && parsed.args.length > 0) {
              const taskDescription = parsed.args.join(' ');
              try {
                await handleAndPersistSystemMessage(
                  conversationMessage(`Executing task: ${taskDescription}`, 'info', '🚀'),
                  addMessage,
                  invoke
                );

                const taskResult = await invoke<string>('execute_message_as_task', {
                  messageContent: taskDescription,
                });

                await handleAndPersistSystemMessage(
                  conversationMessage(`Task completed:\n${taskResult}`, 'info', '✅'),
                  addMessage,
                  invoke
                );
              } catch (error) {
                console.error('Failed to execute task:', error);
                await handleAndPersistSystemMessage(
                  conversationMessage(`Task execution failed: ${error}`, 'error', '❌'),
                  addMessage,
                  invoke
                );
              }
            } else {
              await handleAndPersistSystemMessage(
                conversationMessage('Usage: /task <description>\nExample: /task Create a README file', 'error'),
                addMessage,
                invoke
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

                const successMessage = `Expert persona created: ${persona.name} ${persona.icon || '🔶'}\nRole: ${persona.role}\nBackground: ${persona.background}`;

                // Display in UI and persist to backend
                await handleAndPersistSystemMessage(
                  conversationMessage(successMessage, 'info', '🔶'),
                  addMessage,
                  invoke
                );

                await refreshPersonas();
                await refreshSessions();
              } catch (error) {
                console.error('Failed to create expert:', error);
                notifications.hide('expert-creation');

                const errorMessage = `Failed to create expert: ${error}`;

                // Display in UI and persist to backend
                await handleAndPersistSystemMessage(
                  conversationMessage(errorMessage, 'error', '❌'),
                  addMessage,
                  invoke
                );
              }
            } else {
              await handleAndPersistSystemMessage(
                conversationMessage('Usage: /expert <expertise>\nExample: /expert 映画制作プロセス', 'error'),
                addMessage,
                invoke
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
              await handleAndPersistSystemMessage(
                conversationMessage(
                  'Usage: /blueprint <task description>\nExample: /blueprint Create technical article about Rust',
                  'error'
                ),
                addMessage,
                invoke
              );
            }
            await saveCurrentSession();
            break;

          case 'create-slash-command':
            if (parsed.args && parsed.args.length > 0) {
              const jsonString = parsed.args.join(' ');
              try {
                // Parse and validate JSON
                const commandRequest = JSON.parse(jsonString);

                notifications.show({
                  title: 'Creating Slash Command',
                  message: `Creating command: ${commandRequest.name || 'Unknown'}...`,
                  color: 'blue',
                  autoClose: false,
                  id: 'command-creation',
                });

                const command = await invoke<import('../types/slash_command').SlashCommand>('create_slash_command', {
                  request: commandRequest,
                });

                notifications.hide('command-creation');

                const successMessage = `Slash command created: /${command.name} ${command.icon}\nType: ${command.type}\nDescription: ${command.description}`;

                await handleAndPersistSystemMessage(
                  conversationMessage(successMessage, 'info', '✅'),
                  addMessage,
                  invoke
                );

                // No need to refresh sessions, but might want to refresh command list in UI
              } catch (error) {
                console.error('Failed to create slash command:', error);
                notifications.hide('command-creation');

                const errorMessage = error instanceof SyntaxError
                  ? `Invalid JSON format: ${error.message}`
                  : `Failed to create slash command: ${error}`;

                await handleAndPersistSystemMessage(
                  conversationMessage(errorMessage, 'error', '❌'),
                  addMessage,
                  invoke
                );
              }
            } else {
              await handleAndPersistSystemMessage(
                conversationMessage(
                  'Usage: /create-slash-command <json>\nExample: /create-slash-command {"name": "greet", "type": "prompt", "description": "Greet someone", "content": "Say hello to {{args}}", "icon": "👋"}',
                  'error'
                ),
                addMessage,
                invoke
              );
            }
            await saveCurrentSession();
            break;

          case 'create-persona':
            if (parsed.args && parsed.args.length > 0) {
              const jsonString = parsed.args.join(' ');
              try {
                // Parse and validate JSON
                const personaRequest = JSON.parse(jsonString);

                notifications.show({
                  title: 'Creating Persona',
                  message: `Creating persona: ${personaRequest.name || 'Unknown'}...`,
                  color: 'blue',
                  autoClose: false,
                  id: 'persona-creation',
                });

                const persona = await invoke<import('../types/agent').PersonaConfig>('create_persona', {
                  request: personaRequest,
                });

                notifications.hide('persona-creation');

                const successMessage = `Persona created: ${persona.name} ${persona.icon || '👤'}\nRole: ${persona.role}\nBackground: ${persona.background}`;

                await handleAndPersistSystemMessage(
                  conversationMessage(successMessage, 'info', '✅'),
                  addMessage,
                  invoke
                );

                await refreshPersonas();
                await refreshSessions();
              } catch (error) {
                console.error('Failed to create persona:', error);
                notifications.hide('persona-creation');

                const errorMessage = error instanceof SyntaxError
                  ? `Invalid JSON format: ${error.message}`
                  : `Failed to create persona: ${error}`;

                await handleAndPersistSystemMessage(
                  conversationMessage(errorMessage, 'error', '❌'),
                  addMessage,
                  invoke
                );
              }
            } else {
              await handleAndPersistSystemMessage(
                conversationMessage(
                  'Usage: /create-persona <json>\nExample: /create-persona {"name": "Expert", "role": "Specialist", "background": "Experienced professional...", "communication_style": "Clear and helpful...", "backend": "claude_api"}',
                  'error'
                ),
                addMessage,
                invoke
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
                  await handleAndPersistSystemMessage(
                    conversationMessage(`Switched to workspace: ${targetWorkspace.name}`, 'success', '✅'),
                    addMessage,
                    invoke
                  );
                } catch (err) {
                  await handleAndPersistSystemMessage(
                    conversationMessage(`Failed to switch workspace: ${err}`, 'error'),
                    addMessage,
                    invoke
                  );
                }
              } else if (!targetWorkspace) {
                await handleAndPersistSystemMessage(
                  conversationMessage(
                    `Workspace not found: ${workspaceName}\n\nAvailable workspaces:\n${allWorkspaces.map((ws) => `- ${ws.name}`).join('\n')}`,
                    'error'
                  ),
                  addMessage,
                  invoke
                );
              } else {
                await handleAndPersistSystemMessage(
                  conversationMessage('No active session', 'error'),
                  addMessage,
                  invoke
                );
              }
            } else {
              const workspaceList = allWorkspaces
                .map(
                  (ws) =>
                    `${ws.id === workspace?.id ? '📍' : '  '} ${ws.name}${ws.isFavorite ? ' ⭐' : ''}`
                )
                .join('\n');
              await handleAndPersistSystemMessage(
                conversationMessage(`Available workspaces:\n${workspaceList}\n\nUsage: /workspace <name>`),
                addMessage,
                invoke
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
            await handleAndPersistSystemMessage(
              conversationMessage(`Files in workspace "${workspace?.name}":\n${fileList}`),
              addMessage,
              invoke
            );
            await saveCurrentSession();
            break;

          case 'search': {
            if (!parsed.args || parsed.args.length === 0) {
              await handleAndPersistSystemMessage(
                conversationMessage(
                  'Usage: /search <query> [scope:workspace|local|global]\nExample: /search session.rs\nExample: /search scope:local agent.rs',
                  'error'
                ),
                addMessage,
                invoke
              );
              await saveCurrentSession();
              break;
            }

            let scope: SearchScopeType = 'workspace';
            const queryParts: string[] = [];
            let invalidScope = false;

            parsed.args.forEach((part) => {
              const normalized = part.toLowerCase();
              if (normalized.startsWith('scope:') || normalized.startsWith('scope=')) {
                const [, rawValue = ''] = normalized.split(/[:=]/, 2);
                if (rawValue === 'workspace' || rawValue === 'local' || rawValue === 'global') {
                  scope = rawValue as SearchScopeType;
                } else {
                  invalidScope = true;
                }
              } else {
                queryParts.push(part);
              }
            });

            if (invalidScope) {
              await handleAndPersistSystemMessage(
                conversationMessage(
                  'Invalid scope. Available scopes: workspace, local, global.',
                  'error'
                ),
                addMessage,
                invoke
              );
              await saveCurrentSession();
              break;
            }

            if (queryParts.length === 0) {
              await handleAndPersistSystemMessage(
                conversationMessage('Please provide a search query after /search.', 'error'),
                addMessage,
                invoke
              );
              await saveCurrentSession();
              break;
            }

            const query = queryParts.join(' ');
            try {
              const result = await invoke<SearchResult>('execute_search', {
                request: {
                  query,
                  scope,
                  filters: null,
                },
              });

              const maxDisplay = 20;
              const displayedItems = result.items.slice(0, maxDisplay);
              const summaryText = result.summary ? `${result.summary.trim()}\n\n` : '';
              const resultLines =
                displayedItems.length > 0
                  ? displayedItems
                      .map((item) => {
                        const location = item.line_number
                          ? `${item.path}:${item.line_number}`
                          : item.path;
                        const snippet = item.content.trim();
                        return `• ${location}\n  ${snippet}`;
                      })
                      .join('\n')
                  : '_No matches found._';
              const overflow =
                result.total_matches > maxDisplay
                  ? `\n…and ${result.total_matches - maxDisplay} more matches.`
                  : '';

              await handleAndPersistSystemMessage(
                conversationMessage(
                  `Search results for "${result.query}" (scope: ${result.scope})\n\n${summaryText}${resultLines}${overflow}`,
                  'info',
                  '🔍'
                ),
                addMessage,
                invoke
              );
            } catch (error) {
              await handleAndPersistSystemMessage(
                conversationMessage(`Search failed: ${error}`, 'error', '❌'),
                addMessage,
                invoke
              );
            }

            await saveCurrentSession();
            break;
          }

          case 'mode':
            if (parsed.args && parsed.args.length > 0) {
              const mode = parsed.args[0].toLowerCase();
              const validModes = ['normal', 'concise', 'brief', 'discussion'];

              if (!validModes.includes(mode)) {
                await handleAndPersistSystemMessage(
                  conversationMessage(
                    `Invalid mode: ${mode}\n\nAvailable modes:\n- normal (通常)\n- concise (簡潔・300文字)\n- brief (極簡潔・150文字)\n- discussion (議論)`,
                    'error'
                  ),
                  addMessage,
                  invoke
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
                await handleAndPersistSystemMessage(
                  conversationMessage(`Conversation mode changed to: ${modeLabels[mode]}`, 'success', '✅'),
                  addMessage,
                  invoke
                );
              } catch (error) {
                await handleAndPersistSystemMessage(
                  conversationMessage(`Failed to set conversation mode: ${error}`, 'error'),
                  addMessage,
                  invoke
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
                await handleAndPersistSystemMessage(
                  conversationMessage(
                    `Current mode: ${modeLabels[currentMode] || currentMode}\n\nUsage: /mode <normal|concise|brief|discussion>`
                  ),
                  addMessage,
                  invoke
                );
              } catch (error) {
                await handleAndPersistSystemMessage(
                  conversationMessage('Usage: /mode <normal|concise|brief|discussion>', 'error'),
                  addMessage,
                  invoke
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
                await handleAndPersistSystemMessage(
                  conversationMessage(
                    `Invalid style: ${style}\n\nAvailable styles:\n- brainstorm (ブレインストーミング)\n- casual (カジュアル)\n- decision_making (意思決定)\n- debate (議論)\n- problem_solving (問題解決)\n- review (レビュー)\n- planning (計画)\n- none (解除)`,
                    'error'
                  ),
                  addMessage,
                  invoke
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
                await handleAndPersistSystemMessage(
                  conversationMessage(`Talk style changed to: ${styleLabels[style]}`, 'success', '✅'),
                  addMessage,
                  invoke
                );
              } catch (error) {
                await handleAndPersistSystemMessage(
                  conversationMessage(`Failed to set talk style: ${error}`, 'error'),
                  addMessage,
                  invoke
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
                await handleAndPersistSystemMessage(
                  conversationMessage(
                    `Current talk style: ${currentLabel}\n\nUsage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>`
                  ),
                  addMessage,
                  invoke
                );
              } catch (error) {
                await handleAndPersistSystemMessage(
                  conversationMessage(
                    'Usage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>',
                    'error'
                  ),
                  addMessage,
                  invoke
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
            await handleAndPersistSystemMessage(
              conversationMessage(`Unknown command: /${parsed.command}\nType /help for available commands`, 'error'),
              addMessage,
              invoke
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
                await handleAndPersistSystemMessage(
                  conversationMessage(`Command /${parsed.command} produced empty content.`, 'error'),
                  addMessage,
                  invoke
                );
                nextInput = null;
                suppressUserMessage = true;
              }
            } else if (customCommand.type === 'task') {
              // Execute task command
              try {
                await handleAndPersistSystemMessage(
                  conversationMessage(`Executing task: ${parsed.command}...`, 'info', '🚀'),
                  addMessage,
                  invoke
                );

                const taskResult = await invoke<string>('execute_task_command', {
                  commandName: parsed.command,
                  args: argsStr || null,
                });

                await handleAndPersistSystemMessage(
                  conversationMessage(`Task completed:\n${taskResult}`, 'info', '✅'),
                  addMessage,
                  invoke
                );

                nextInput = null;
                suppressUserMessage = true;
              } catch (error) {
                console.error(`Failed to execute task command /${parsed.command}:`, error);
                await handleAndPersistSystemMessage(
                  conversationMessage(`Task execution failed: ${error}`, 'error', '❌'),
                  addMessage,
                  invoke
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

                // Display in UI and persist to backend
                // (survives session switches, no agent reaction)
                await handleAndPersistSystemMessage(
                  shellOutputMessage(shellMessage),
                  addMessage,
                  invoke
                );

                // Don't send to agent (ContextInfo is stored but doesn't trigger reactions)
                nextInput = null;
                suppressUserMessage = true;
              } catch (error) {
                console.error(`Failed to run slash command /${parsed.command}:`, error);
                await handleAndPersistSystemMessage(
                  conversationMessage(`Failed to run slash command: ${error}`, 'error'),
                  addMessage,
                  invoke
                );
                nextInput = null;
                suppressUserMessage = true;
              }
            }

            await saveCurrentSession();
          } catch (error) {
            console.error('Failed to expand slash command:', error);
            await handleAndPersistSystemMessage(
              conversationMessage(`Failed to expand command: ${error}`, 'error'),
              addMessage,
              invoke
            );
            await saveCurrentSession();
          }
        } catch (error) {
          console.error('Failed to fetch slash command:', error);
          await handleAndPersistSystemMessage(
            conversationMessage(`Failed to fetch command: ${error}`, 'error'),
            addMessage,
            invoke
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
          await handleAndPersistSystemMessage(
            conversationMessage(
              `Failed to submit generated command content: ${error}`,
              'error'
            ),
            addMessage,
            invoke
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
