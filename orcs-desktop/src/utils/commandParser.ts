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
 * コマンドのヘルプテキストを取得
 * @deprecated Use generateCommandHelp from command.ts instead
 */
export function getCommandHelp(command?: string): string {
  return generateCommandHelp(command);
}
