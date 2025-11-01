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
  argsDescription?: string;
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
    description: 'Switch conversation mode to control agent verbosity',
    usage: '/mode [normal|concise|brief|discussion]',
    examples: ['/mode', '/mode concise', '/mode brief', '/mode discussion'],
    argsDescription: 'normal (通常) | concise (簡潔・300文字) | brief (極簡潔・150文字) | discussion (議論)',
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
  // コマンドが {args} を使用しているか、または argsDescription が設定されている場合
  const usesArgs = cmd.content.includes('{args}') ||
                   (cmd.workingDir?.includes('{args}')) ||
                   !!cmd.argsDescription;

  let usage: string;
  if (cmd.type === 'prompt') {
    usage = usesArgs ? `/${cmd.name} <args>` : `/${cmd.name}`;
  } else {
    usage = usesArgs ? `/${cmd.name} <args>` : `/${cmd.name}`;
  }

  return {
    name: cmd.name,
    icon: cmd.icon,
    description: cmd.description,
    usage,
    examples: [],
    argsDescription: cmd.argsDescription,
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

/**
 * ヘルプテキストを動的生成
 */
export function generateCommandHelp(command?: string): string {
  if (!command) {
    // 全コマンドのリストを生成
    const commandList = COMMAND_DEFINITIONS
      .map(cmd => `${cmd.icon} ${cmd.usage.padEnd(25)} - ${cmd.description}`)
      .join('\n');
    return `Available commands:\n${commandList}`;
  }

  // 特定コマンドの詳細ヘルプ
  const cmdDef = getCommandDefinition(command);
  if (!cmdDef) {
    return `Unknown command: /${command}`;
  }

  let helpText = `${cmdDef.icon} ${cmdDef.usage}\n\n${cmdDef.description}`;

  if (cmdDef.argsDescription) {
    helpText += `\n\nArguments:\n  ${cmdDef.argsDescription}`;
  }

  if (cmdDef.examples && cmdDef.examples.length > 0) {
    helpText += `\n\nExamples:\n${cmdDef.examples.map(ex => `  ${ex}`).join('\n')}`;
  }

  return helpText;
}

/**
 * ビルトインコマンド名のリストを取得
 */
export function getBuiltinCommandNames(): readonly string[] {
  return COMMAND_DEFINITIONS.map(cmd => cmd.name);
}
