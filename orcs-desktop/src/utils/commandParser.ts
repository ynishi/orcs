/**
 * コマンドの解析結果
 */
export interface ParsedCommand {
  isCommand: boolean;
  command?: string;
  args?: string[];
  rawInput: string;
}

/**
 * 利用可能なコマンドリスト
 */
export const AVAILABLE_COMMANDS = [
  'help',
  'task',
  'mode',
  'status',
  'clear',
  'agents',
  'files',
  'ls',
  'cd',
  'pwd',
] as const;

export type AvailableCommand = typeof AVAILABLE_COMMANDS[number];

/**
 * 入力文字列がコマンドかどうかを判定し、解析する
 */
export function parseCommand(input: string): ParsedCommand {
  const trimmedInput = input.trim();

  // コマンドでない場合
  if (!trimmedInput.startsWith('/')) {
    return {
      isCommand: false,
      rawInput: input,
    };
  }

  // `/` を除去
  const commandString = trimmedInput.slice(1);

  // スペースで分割
  const parts = commandString.split(/\s+/);
  const command = parts[0].toLowerCase();
  const args = parts.slice(1);

  return {
    isCommand: true,
    command,
    args,
    rawInput: input,
  };
}

/**
 * コマンドが有効かどうかを確認
 */
export function isValidCommand(command: string): boolean {
  return AVAILABLE_COMMANDS.includes(command as AvailableCommand);
}

/**
 * コマンドのヘルプテキストを取得
 */
export function getCommandHelp(command?: string): string {
  if (!command) {
    return `Available commands:
/help          - Show this help message
/task [text]   - Create a new task
/mode [name]   - Switch mode
/status        - Show current status
/clear         - Clear chat history
/agents        - List available agents`;
  }

  const helpTexts: Record<string, string> = {
    help: '/help - Show available commands',
    task: '/task [text] - Create a new task with the given text',
    mode: '/mode [name] - Switch to the specified mode (e.g., chat, analysis, debug)',
    status: '/status - Display current system status',
    clear: '/clear - Clear all chat messages',
    agents: '/agents - List all available agents and their status',
  };

  return helpTexts[command] || `Unknown command: /${command}`;
}
