import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import { Paper, Text, Box, CopyButton, ActionIcon, Tooltip } from '@mantine/core';
import { SaveableCodeBlock } from './SaveableCodeBlock';
import 'highlight.js/styles/github-dark.css';

interface MarkdownRendererProps {
  content: string;
  onSaveFile?: (path: string, content: string) => Promise<void>;
}

interface CodeBlockMetadata {
  language: string;
  saveable: boolean;
  path?: string;
}

/**
 * Parse code block metadata from info string
 * Example: ```toml:path=/path/to/file.toml:saveable
 */
function parseCodeBlockMetadata(info: string): CodeBlockMetadata {
  const parts = info.split(':');
  const language = parts[0] || '';
  const saveable = parts.includes('saveable');
  const pathPart = parts.find(p => p.startsWith('path='));
  const path = pathPart ? pathPart.replace('path=', '') : undefined;

  return { language, saveable, path };
}

export function MarkdownRenderer({ content, onSaveFile }: MarkdownRendererProps) {
  return (
    <Box>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight]}
        components={{
          // Custom code block renderer
          code({ className, children, ...props }) {
            const codeString = String(children).replace(/\n$/, '');

            // Check if this is inline code (no className usually means inline)
            const isInline = !className || className.indexOf('language-') === -1;

            // Inline code
            if (isInline) {
              return (
                <Text component="code" c="blue" ff="monospace" {...props}>
                  {children}
                </Text>
              );
            }

            // Parse metadata from className (e.g., "language-toml:path=/path:saveable")
            const fullInfo = className?.replace('language-', '') || '';
            const metadata = parseCodeBlockMetadata(fullInfo);

            // If saveable and onSaveFile handler provided, use SaveableCodeBlock
            if (metadata.saveable && onSaveFile) {
              return (
                <SaveableCodeBlock
                  language={metadata.language}
                  code={codeString}
                  suggestedPath={metadata.path}
                  onSave={onSaveFile}
                />
              );
            }

            // Regular code block with syntax highlighting
            return (
              <Paper p="md" radius="md" bg="dark.8" mb="sm" style={{ position: 'relative' }}>
                {/* Copy button */}
                <Box style={{ position: 'absolute', top: 8, right: 8 }}>
                  <CopyButton value={codeString}>
                    {({ copied, copy }) => (
                      <Tooltip label={copied ? 'Copied!' : 'Copy code'} withArrow>
                        <ActionIcon
                          color={copied ? 'teal' : 'gray'}
                          variant="subtle"
                          onClick={copy}
                          size="sm"
                        >
                          {copied ? '✓' : '📋'}
                        </ActionIcon>
                      </Tooltip>
                    )}
                  </CopyButton>
                </Box>

                <pre style={{ margin: 0, overflow: 'auto' }}>
                  <code className={className} {...props}>
                    {children}
                  </code>
                </pre>
              </Paper>
            );
          },

          // Custom paragraph renderer
          p({ children }) {
            return (
              <Text size="sm" mb="sm">
                {children}
              </Text>
            );
          },

          // Custom heading renderers
          h1({ children }) {
            return (
              <Text size="xl" fw={700} mb="md" mt="lg">
                {children}
              </Text>
            );
          },
          h2({ children }) {
            return (
              <Text size="lg" fw={600} mb="sm" mt="md">
                {children}
              </Text>
            );
          },
          h3({ children }) {
            return (
              <Text size="md" fw={600} mb="xs" mt="sm">
                {children}
              </Text>
            );
          },

          // Custom list renderers
          ul({ children }) {
            return (
              <Box component="ul" mb="sm" pl="md">
                {children}
              </Box>
            );
          },
          ol({ children }) {
            return (
              <Box component="ol" mb="sm" pl="md">
                {children}
              </Box>
            );
          },
          li({ children }) {
            return (
              <Text component="li" size="sm" mb={4}>
                {children}
              </Text>
            );
          },

          // Custom blockquote renderer
          blockquote({ children }) {
            return (
              <Paper
                p="sm"
                radius="md"
                mb="sm"
                style={{
                  borderLeft: '4px solid var(--mantine-color-blue-6)',
                  backgroundColor: 'var(--mantine-color-blue-0)',
                }}
              >
                {children}
              </Paper>
            );
          },

          // Custom table renderers
          table({ children }) {
            return (
              <Box mb="sm" style={{ overflowX: 'auto' }}>
                <table
                  style={{
                    width: '100%',
                    borderCollapse: 'collapse',
                    fontSize: 'var(--mantine-font-size-sm)',
                  }}
                >
                  {children}
                </table>
              </Box>
            );
          },
          th({ children }) {
            return (
              <th
                style={{
                  padding: '8px',
                  borderBottom: '2px solid var(--mantine-color-gray-3)',
                  textAlign: 'left',
                  fontWeight: 600,
                }}
              >
                {children}
              </th>
            );
          },
          td({ children }) {
            return (
              <td
                style={{
                  padding: '8px',
                  borderBottom: '1px solid var(--mantine-color-gray-2)',
                }}
              >
                {children}
              </td>
            );
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </Box>
  );
}
