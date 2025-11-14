import { describe, it, expect, vi, beforeEach } from 'vitest';
import { GeminiProvider } from '../GeminiProvider';
import type { AIContextInfo } from '../../core/types/ai';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('GeminiProvider', () => {
  let provider: GeminiProvider;

  beforeEach(() => {
    provider = new GeminiProvider();
    vi.clearAllMocks();
  });

  describe('generate', () => {
    it('should call ai_generate command with correct parameters', async () => {
      const mockResult = 'Generated text from AI';
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const context: AIContextInfo = {
        scope: 'Test.Scope',
        type: 'string',
      };

      const result = await provider.generate(context);

      expect(invoke).toHaveBeenCalledWith('ai_generate', {
        prompt: expect.stringContaining('Test.Scope'),
        context: {
          scope: 'Test.Scope',
          type: 'string',
          maxLength: undefined,
          metadata: undefined,
        },
      });
      expect(result).toBe(mockResult);
    });

    it('should build proper prompt with direction', async () => {
      const mockResult = 'Generated with direction';
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const context: AIContextInfo = {
        scope: 'UserProfile.Bio',
        type: 'long_text',
        maxLength: 500,
      };

      await provider.generate(context, 'professional and friendly');

      expect(invoke).toHaveBeenCalledWith('ai_generate', {
        prompt: expect.stringContaining('professional and friendly'),
        context: {
          scope: 'UserProfile.Bio',
          type: 'long_text',
          maxLength: 500,
          metadata: undefined,
        },
      });
    });

    it('should handle different content types', async () => {
      vi.mocked(invoke).mockResolvedValue('OK');

      const types: Array<AIContextInfo['type']> = ['string', 'long_text', 'markdown', 'code'];

      for (const type of types) {
        const context: AIContextInfo = {
          scope: 'Test',
          type,
        };

        await provider.generate(context);

        const call = vi.mocked(invoke).mock.calls[vi.mocked(invoke).mock.calls.length - 1];
        expect(call[1]).toMatchObject({
          context: expect.objectContaining({ type }),
        });
      }
    });

    it('should handle errors from Tauri', async () => {
      const mockError = new Error('Tauri command failed');
      vi.mocked(invoke).mockRejectedValue(mockError);

      const context: AIContextInfo = {
        scope: 'Test',
        type: 'string',
      };

      await expect(provider.generate(context)).rejects.toThrow('AI生成に失敗しました');
    });

    it('should include metadata in prompt', async () => {
      vi.mocked(invoke).mockResolvedValue('OK');

      const context: AIContextInfo = {
        scope: 'Test',
        type: 'string',
        metadata: { user: 'John', role: 'developer' },
      };

      await provider.generate(context);

      const call = vi.mocked(invoke).mock.calls[0];
      expect(call?.[1]).toBeDefined();
      const args = call[1] as { prompt: string };
      expect(args.prompt).toContain('user: John');
      expect(args.prompt).toContain('role: developer');
    });
  });

  describe('refine', () => {
    it('should call ai_refine command with correct parameters', async () => {
      const mockResult = 'Refined text';
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const context: AIContextInfo = {
        scope: 'MessageItem',
        type: 'string',
      };

      const result = await provider.refine('Original text here', context);

      expect(invoke).toHaveBeenCalledWith('ai_refine', {
        prompt: expect.stringContaining('Original text here'),
        currentText: 'Original text here',
        context: {
          scope: 'MessageItem',
          type: 'string',
          maxLength: undefined,
          metadata: undefined,
        },
      });
      expect(result).toBe(mockResult);
    });

    it('should build proper prompt with direction', async () => {
      const mockResult = 'Refined with direction';
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const context: AIContextInfo = {
        scope: 'Test',
        type: 'long_text',
        maxLength: 200,
      };

      await provider.refine('Original', context, 'more concise');

      expect(invoke).toHaveBeenCalledWith('ai_refine', {
        prompt: expect.stringContaining('more concise'),
        currentText: 'Original',
        context: {
          scope: 'Test',
          type: 'long_text',
          maxLength: 200,
          metadata: undefined,
        },
      });
    });

    it('should handle errors from Tauri', async () => {
      const mockError = new Error('Refine failed');
      vi.mocked(invoke).mockRejectedValue(mockError);

      const context: AIContextInfo = {
        scope: 'Test',
        type: 'string',
      };

      await expect(provider.refine('Text', context)).rejects.toThrow('AI修正に失敗しました');
    });

    it('should respect maxLength in prompt', async () => {
      vi.mocked(invoke).mockResolvedValue('OK');

      const context: AIContextInfo = {
        scope: 'Test',
        type: 'string',
        maxLength: 100,
      };

      await provider.refine('Text', context);

      const call = vi.mocked(invoke).mock.calls[0];
      expect(call?.[1]).toBeDefined();
      const args = call[1] as { prompt: string };
      expect(args.prompt).toContain('Maximum 100 characters');
    });
  });

  describe('concurrent requests', () => {
    it('should handle multiple generate calls in parallel', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('First')
        .mockResolvedValueOnce('Second')
        .mockResolvedValueOnce('Third');

      const context: AIContextInfo = { scope: 'Test', type: 'string' };

      const results = await Promise.all([
        provider.generate(context),
        provider.generate(context),
        provider.generate(context),
      ]);

      expect(results).toEqual(['First', 'Second', 'Third']);
      expect(invoke).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed generate and refine calls', async () => {
      vi.mocked(invoke).mockResolvedValueOnce('Generated').mockResolvedValueOnce('Refined');

      const context: AIContextInfo = { scope: 'Test', type: 'string' };

      const [generated, refined] = await Promise.all([
        provider.generate(context),
        provider.refine('Original', context),
      ]);

      expect(generated).toBe('Generated');
      expect(refined).toBe('Refined');
      expect(invoke).toHaveBeenCalledTimes(2);
    });
  });
});
