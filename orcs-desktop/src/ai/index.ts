/**
 * AI Integration Module
 *
 * ORCS AI統合機能のエントリーポイント
 *
 * @example
 * ```tsx
 * import { AIProvider, useAIRegister, GeminiProvider } from '@/ai';
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

// Core exports (ヘッドレスロジック)
export * from './core';

// Providers
export * from './providers';

// Components
export * from './components';
