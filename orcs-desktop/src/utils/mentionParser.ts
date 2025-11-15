/**
 * メンション情報
 */
export interface Mention {
  mentionText: string; // 入力から抽出したテキスト（例: "Ayaka_Nakamura"）
  searchName: string; // Persona検索用（_ をスペースに変換、例: "Ayaka Nakamura"）
  startIndex: number;
  endIndex: number;
}

/**
 * メンションテキストを検索名に変換（_ をスペースに変換）
 */
export function mentionTextToSearchName(mentionText: string): string {
  return mentionText.replace(/_/g, ' ');
}

/**
 * テキスト内のメンションを正規化（_ をスペースに変換）
 * 例: "@Ayaka_Nakamura hello" → "@Ayaka Nakamura hello"
 */
export function normalizeMentionsInText(text: string): string {
  const mentions = extractMentions(text);
  let result = text;

  // 後ろから置換していく（インデックスがずれないように）
  for (let i = mentions.length - 1; i >= 0; i--) {
    const mention = mentions[i];
    const before = text.slice(0, mention.startIndex);
    const after = text.slice(mention.endIndex);
    result = before + '@' + mention.searchName + after;
  }

  return result;
}

/**
 * テキストから@メンションを抽出
 * 注意: スペース付き名前は "@Name_With_Space" のように _ を使用してください
 */
export function extractMentions(text: string): Mention[] {
  // スペース以外の文字にマッチ（日本語、ハイフン、記号などもサポート）
  const mentionRegex = /@(\S+)/g;
  const mentions: Mention[] = [];
  let match;

  while ((match = mentionRegex.exec(text)) !== null) {
    const extractedText = match[1];
    mentions.push({
      mentionText: extractedText,
      searchName: mentionTextToSearchName(extractedText),
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
