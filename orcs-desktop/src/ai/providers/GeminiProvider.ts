/**
 * Gemini Provider (Tauri Implementation)
 *
 * TauriのinvokeコマンドでRustバックエンド経由でGemini APIを呼び出す実装
 *
 * NOTE: このProviderはTauriに依存しています。
 * Web版で使う場合は、WebGeminiProvider等の別実装を作成してください。
 */

import { invoke } from '@tauri-apps/api/core';
import type { IAIProvider, AIContextInfo } from '../core/types/ai';

/**
 * Gemini Provider for Tauri
 *
 * @example
 * ```tsx
 * import { AIProvider } from '@/ai/core/context/AIContext';
 * import { GeminiProvider } from '@/ai/providers/GeminiProvider';
 *
 * function App() {
 *   return (
 *     <AIProvider provider={new GeminiProvider()}>
 *       <YourApp />
 *     </AIProvider>
 *   );
 * }
 * ```
 */
export class GeminiProvider implements IAIProvider {
  /**
   * 新しいコンテンツを生成
   */
  async generate(context: AIContextInfo, direction?: string): Promise<string> {
    try {
      const prompt = this.buildGeneratePrompt(context, direction);

      console.log('[GeminiProvider] Calling ai_generate...', {
        scope: context.scope,
        direction,
        promptLength: prompt.length,
      });

      // Tauriバックエンドのai_generateコマンドを呼び出し
      const result = await invoke<string>('ai_generate', {
        prompt,
        context: {
          scope: context.scope,
          type: context.type,
          maxLength: context.maxLength,
          metadata: context.metadata,
        },
      });

      console.log('[GeminiProvider] ai_generate successful', {
        resultLength: result.length,
      });

      return result;
    } catch (error) {
      console.error('[GeminiProvider] Generate failed:', error);

      // エラーメッセージを整形
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`AI生成に失敗しました: ${message}`);
    }
  }

  /**
   * 既存のコンテンツを修正
   */
  async refine(currentText: string, context: AIContextInfo, direction?: string): Promise<string> {
    try {
      const prompt = this.buildRefinePrompt(currentText, context, direction);

      console.log('[GeminiProvider] Calling ai_refine...', {
        scope: context.scope,
        direction,
        currentTextLength: currentText.length,
        promptLength: prompt.length,
      });

      // Tauriバックエンドのai_refineコマンドを呼び出し
      const result = await invoke<string>('ai_refine', {
        prompt,
        currentText,
        context: {
          scope: context.scope,
          type: context.type,
          maxLength: context.maxLength,
          metadata: context.metadata,
        },
      });

      console.log('[GeminiProvider] ai_refine successful', {
        resultLength: result.length,
      });

      return result;
    } catch (error) {
      console.error('[GeminiProvider] Refine failed:', error);

      // エラーメッセージを整形
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`AI修正に失敗しました: ${message}`);
    }
  }

  /**
   * 生成用プロンプトを構築
   */
  private buildGeneratePrompt(context: AIContextInfo, direction?: string): string {
    let prompt = `Generate content for "${context.scope}"`;

    // タイプに応じた指示
    switch (context.type) {
      case 'string':
        prompt += '. Keep it concise (1-2 sentences).';
        break;
      case 'long_text':
        prompt += '. Provide detailed content (multiple paragraphs).';
        break;
      case 'markdown':
        prompt += '. Format using Markdown syntax.';
        break;
      case 'code':
        prompt += '. Generate code snippet with appropriate syntax.';
        break;
    }

    // 方向性が指定されている場合
    if (direction) {
      prompt += ` Make it "${direction}".`;
    }

    // 最大文字数制限
    if (context.maxLength) {
      prompt += ` Maximum ${context.maxLength} characters.`;
    }

    // メタデータからの追加コンテキスト
    if (context.metadata) {
      const metadataStr = Object.entries(context.metadata)
        .map(([key, value]) => `${key}: ${value}`)
        .join(', ');
      prompt += ` Additional context: ${metadataStr}.`;
    }

    return prompt;
  }

  /**
   * 修正用プロンプトを構築
   */
  private buildRefinePrompt(
    currentText: string,
    context: AIContextInfo,
    direction?: string
  ): string {
    let prompt = `Refine the following text for "${context.scope}":\n\n${currentText}`;

    // 方向性が指定されている場合
    if (direction) {
      prompt += `\n\nMake it "${direction}".`;
    }

    // タイプに応じた指示
    switch (context.type) {
      case 'string':
        prompt += '\n\nKeep it concise.';
        break;
      case 'long_text':
        prompt += '\n\nProvide detailed improvements.';
        break;
      case 'markdown':
        prompt += '\n\nMaintain Markdown formatting.';
        break;
      case 'code':
        prompt += '\n\nImprove code quality and readability.';
        break;
    }

    // 最大文字数制限
    if (context.maxLength) {
      prompt += ` Maximum ${context.maxLength} characters.`;
    }

    return prompt;
  }
}
