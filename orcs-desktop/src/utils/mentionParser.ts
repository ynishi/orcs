/**
 * メンション情報
 */
export interface Mention {
  agentName: string;
  startIndex: number;
  endIndex: number;
}

/**
 * テキストから@メンションを抽出
 */
export function extractMentions(text: string): Mention[] {
  // スペース以外の文字にマッチ（日本語、ハイフン、記号などもサポート）
  const mentionRegex = /@(\S+)/g;
  const mentions: Mention[] = [];
  let match;

  while ((match = mentionRegex.exec(text)) !== null) {
    mentions.push({
      agentName: match[1],
      startIndex: match.index,
      endIndex: match.index + match[0].length,
    });
  }

  return mentions;
}

/**
 * 現在のカーソル位置が@メンション入力中かチェック
 */
export function getCurrentMention(text: string, cursorPosition: number): string | null {
  // カーソル位置から前方を検索して、最も近い@を探す
  const beforeCursor = text.slice(0, cursorPosition);
  const lastAtIndex = beforeCursor.lastIndexOf('@');

  if (lastAtIndex === -1) return null;

  // @の後にスペースや改行があったら無効
  const afterAt = beforeCursor.slice(lastAtIndex + 1);
  if (afterAt.includes(' ') || afterAt.includes('\n')) return null;

  return afterAt;
}
