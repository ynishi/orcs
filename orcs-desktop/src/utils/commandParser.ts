import { getBuiltinCommandNames, generateCommandHelp } from '../types/command';

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
 * コマンドが有効なビルトインコマンドかどうかを確認
 */
export function isValidCommand(command: string): boolean {
  return getBuiltinCommandNames().includes(command);
}

/**
 * Agentレスポンスなど任意のテキストから SlashCommand を抽出する
 * - `<Slash><Name>...` フォーマットのみに対応（行頭 `/command` パターンは誤爆防止のため削除）
 */
export function extractSlashCommands(text: string): string[] {
  const commands: string[] = [];

  // XML-style blocks only
  const slashBlocks = text.matchAll(/<Slash>([\s\S]*?)<\/Slash>/gi);
  for (const block of slashBlocks) {
    const inner = block[1];
    const nameMatch = inner.match(/<Name>([\s\S]*?)<\/Name>/i);
    if (!nameMatch) continue;
    const rawName = nameMatch[1].trim().replace(/^\/+/, '');
    if (!rawName) continue;

    const argsMatch = inner.match(/<Args>([\s\S]*?)<\/Args>/i);
    const args = argsMatch ? argsMatch[1].trim() : '';
    const fullCommand = args ? `/${rawName} ${args}` : `/${rawName}`;
    commands.push(fullCommand.trim());
  }

  return commands;
}

/**
 * コマンドのヘルプテキストを取得
 * @deprecated Use generateCommandHelp from command.ts instead
 */
export function getCommandHelp(command?: string): string {
  return generateCommandHelp(command);
}
