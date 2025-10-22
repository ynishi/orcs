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
    name: 'clear',
    icon: '🗑️',
    description: 'Clear all chat messages from the screen',
    usage: '/clear',
    examples: ['/clear'],
  },
  {
    name: 'agents',
    icon: '🤖',
    description: 'List all available agents and their current status',
    usage: '/agents',
    examples: ['/agents'],
  },
  {
    name: 'files',
    icon: '📁',
    description: 'List files in the current directory',
    usage: '/files',
    examples: ['/files'],
  },
  {
    name: 'ls',
    icon: '📂',
    description: 'List contents of a directory (like ls command)',
    usage: '/ls [path]',
    examples: ['/ls', '/ls src', '/ls ../'],
  },
  {
    name: 'cd',
    icon: '📍',
    description: 'Change current working directory',
    usage: '/cd <path>',
    examples: ['/cd src', '/cd ..', '/cd ~/projects'],
  },
  {
    name: 'pwd',
    icon: '🗂️',
    description: 'Print current working directory',
    usage: '/pwd',
    examples: ['/pwd'],
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
