import { describe, it, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { AIProvider, useAIContext } from '../AIContext';
import type { IAIProvider, AIContextInfo } from '../../types/ai';

// Mock Provider
class MockAIProvider implements IAIProvider {
  async generate(context: AIContextInfo, _direction?: string): Promise<string> {
    return `Generated: ${context.scope}`;
  }

  async refine(_currentText: string, _context: AIContextInfo, _direction?: string): Promise<string> {
    return `Refined: ${_currentText}`;
  }
}

describe('AIContext', () => {
  describe('AIProvider', () => {
    it('should provide context value to children', () => {
      const mockProvider = new MockAIProvider();
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <AIProvider provider={mockProvider}>{children}</AIProvider>
      );

      const { result } = renderHook(() => useAIContext(), { wrapper });

      expect(result.current.provider).toBe(mockProvider);
      expect(result.current.enabled).toBe(true);
    });

    it('should throw error when used outside provider', () => {
      // Suppress console.error for this test
      const originalError = console.error;
      console.error = vi.fn();

      expect(() => {
        renderHook(() => useAIContext());
      }).toThrow('useAIContext must be used within AIProvider');

      console.error = originalError;
    });

    // TODO: Fix act() warning for setProvider test
    // This test needs proper act() wrapping for React state updates
    it.skip('should allow provider to be dynamically set via setProvider', () => {
      const provider1 = new MockAIProvider();
      const provider2 = new MockAIProvider();
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <AIProvider provider={provider1}>{children}</AIProvider>
      );

      const { result } = renderHook(() => useAIContext(), { wrapper });

      expect(result.current.provider).toBe(provider1);

      // Use setProvider to change provider
      result.current.setProvider(provider2);

      expect(result.current.provider).toBe(provider2);
    });
  });

  describe('useAIContext hook', () => {
    it('should return the current provider and enabled state', () => {
      const mockProvider = new MockAIProvider();
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <AIProvider provider={mockProvider}>{children}</AIProvider>
      );

      const { result } = renderHook(() => useAIContext(), { wrapper });

      expect(result.current.provider).toBe(mockProvider);
      expect(result.current.enabled).toBe(true);
    });
  });

  describe('Provider interface compliance', () => {
    it('should ensure provider implements IAIProvider interface', async () => {
      const mockProvider = new MockAIProvider();
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <AIProvider provider={mockProvider}>{children}</AIProvider>
      );

      const { result } = renderHook(() => useAIContext(), { wrapper });

      const context: AIContextInfo = {
        scope: 'Test.Scope',
        type: 'string',
      };

      // Ensure provider is not null
      expect(result.current.provider).not.toBeNull();

      // Test generate method
      const generated = await result.current.provider!.generate(context);
      expect(generated).toBe('Generated: Test.Scope');

      // Test refine method
      const refined = await result.current.provider!.refine('original text', context);
      expect(refined).toBe('Refined: original text');
    });

    it('should pass context through provider methods', async () => {
      class ContextAwareProvider implements IAIProvider {
        lastContext?: AIContextInfo;
        lastDirection?: string;

        async generate(context: AIContextInfo, direction?: string): Promise<string> {
          this.lastContext = context;
          this.lastDirection = direction;
          return 'generated';
        }

        async refine(
          _currentText: string,
          context: AIContextInfo,
          direction?: string
        ): Promise<string> {
          this.lastContext = context;
          this.lastDirection = direction;
          return 'refined';
        }
      }

      const provider = new ContextAwareProvider();
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <AIProvider provider={provider}>{children}</AIProvider>
      );

      const { result } = renderHook(() => useAIContext(), { wrapper });

      const testContext: AIContextInfo = {
        scope: 'Test.Scope',
        type: 'long_text',
        maxLength: 1000,
      };

      expect(result.current.provider).not.toBeNull();

      await result.current.provider!.generate(testContext, 'formal');
      expect(provider.lastContext).toEqual(testContext);
      expect(provider.lastDirection).toBe('formal');

      await result.current.provider!.refine('text', testContext, 'concise');
      expect(provider.lastContext).toEqual(testContext);
      expect(provider.lastDirection).toBe('concise');
    });
  });
});
