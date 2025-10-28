import { SlashCommand } from './slash_command';

/**
 * コマンド定義
 */
export interface CommandDefinition {
  name: string;
  icon: string;
  description: string;
  usage: string;
  examples?: string[];
}

/**
 * 利用可能なコマンド定義
 */
export const COMMAND_DEFINITIONS: CommandDefinition[] = [
  {
    name: 'help',
    icon: '❓',
    description: 'Show available commands and their usage',
    usage: '/help [command]',
    examples: ['/help', '/help task'],
  },
  {
    name: 'task',
    icon: '✅',
    description: 'Create a new task with the specified description',
    usage: '/task <description>',
    examples: ['/task Implement login feature', '/task Fix bug in parser'],
  },
  {
    name: 'mode',
    icon: '🔄',
    description: 'Switch between different operation modes',
    usage: '/mode <mode_name>',
    examples: ['/mode analysis', '/mode debug', '/mode chat'],
  },
  {
    name: 'status',
    icon: '📊',
    description: 'Display current system status and active tasks',
    usage: '/status',
    examples: ['/status'],
  },
  {
    name: 'agents',
    icon: '🤖',
    description: 'List all available agents and their current status',
    usage: '/agents',
    examples: ['/agents'],
  },
  {
    name: 'workspace',
    icon: '🗂️',
    description: 'Switch to a different workspace or list all workspaces',
    usage: '/workspace [name]',
    examples: ['/workspace', '/workspace my-project', '/workspace orcs'],
  },
  {
    name: 'files',
    icon: '📁',
    description: 'List files in the current workspace',
    usage: '/files',
    examples: ['/files'],
  },
];

/**
 * コマンド名から定義を取得
 */
export function getCommandDefinition(name: string): CommandDefinition | undefined {
  return COMMAND_DEFINITIONS.find(cmd => cmd.name === name);
}

/**
 * 入力文字列に基づいてコマンドをフィルタリング
 */
export function filterCommands(input: string): CommandDefinition[] {
  // `/` を除去
  const query = input.startsWith('/') ? input.slice(1).toLowerCase() : input.toLowerCase();

  if (!query) {
    return COMMAND_DEFINITIONS;
  }

  return COMMAND_DEFINITIONS.filter(cmd =>
    cmd.name.toLowerCase().startsWith(query) ||
    cmd.description.toLowerCase().includes(query)
  );
}

/**
 * SlashCommand を CommandDefinition に変換
 */
export function slashCommandToDefinition(cmd: SlashCommand): CommandDefinition {
  const usage = cmd.type === 'prompt'
    ? `/${cmd.name}`
    : `/${cmd.name} (shell)`;

  return {
    name: cmd.name,
    icon: cmd.icon,
    description: cmd.description,
    usage,
    examples: [],
  };
}

/**
 * カスタムコマンドを含めてフィルタリング
 */
export function filterCommandsWithCustom(
  input: string,
  customCommands: SlashCommand[]
): CommandDefinition[] {
  // `/` を除去
  const query = input.startsWith('/') ? input.slice(1).toLowerCase() : input.toLowerCase();

  // ビルトインコマンドとカスタムコマンドをマージ
  const allCommands = [
    ...COMMAND_DEFINITIONS,
    ...customCommands.map(slashCommandToDefinition),
  ];

  if (!query) {
    return allCommands;
  }

  return allCommands.filter(cmd =>
    cmd.name.toLowerCase().startsWith(query) ||
    cmd.description.toLowerCase().includes(query)
  );
}
