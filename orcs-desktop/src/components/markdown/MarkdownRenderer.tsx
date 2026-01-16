import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import { Paper, Text, Box, CopyButton, ActionIcon, Tooltip, Badge, Anchor } from '@mantine/core';
import { SaveableCodeBlock } from './SaveableCodeBlock';
import { openPath } from '@tauri-apps/plugin-opener';
import 'highlight.js/styles/github-dark.css';
import React from 'react';

interface MarkdownRendererProps {
  content: string;
  onSaveFile?: (path: string, content: string) => Promise<void>;
  workspaceRootPath?: string;
}

interface CodeBlockMetadata {
  language: string;
  saveable: boolean;
  path?: string;
}

/**
 * Render text with mentions as badges
 */
function renderTextWithMentions(text: string): (string | React.ReactElement)[] {
  const mentionRegex = /@(\S+)/g;
  const parts: (string | React.ReactElement)[] = [];
  let lastIndex = 0;
  let match;
  let key = 0;

  while ((match = mentionRegex.exec(text)) !== null) {
    // Text before mention
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }

    // Mention part as Badge (convert _ to space for display)
    const mentionName = match[1];
    const displayName = mentionName.replace(/_/g, ' ');
    parts.push(
      <Badge
        key={key++}
        component="span"
        size="sm"
        variant="light"
        color="blue"
        style={{ margin: '0 2px' }}
      >
        @{displayName}
      </Badge>
    );

    lastIndex = match.index + match[0].length;
  }

  // Remaining text
  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts.length > 0 ? parts : [text];
}

/**
 * Parse code block metadata from info string
 * Supported formats:
 * - ```toml:path=/path/to/file.toml:saveable (full format)
 * - ```toml:file.toml (filename only, saveable assumed)
 * - ```toml:path=/path/to/file.toml (path without saveable flag)
 * - ```python:script.py (any extension triggers saveable mode)
 */
function parseCodeBlockMetadata(info: string): CodeBlockMetadata {
  const parts = info.split(':');
  const language = parts[0] || '';

  // Check for explicit saveable flag
  let saveable = parts.includes('saveable');

  // Check for explicit path=...
  const pathPart = parts.find(p => p.startsWith('path='));
  let path = pathPart ? pathPart.replace('path=', '') : undefined;

  // If no explicit path, check for filename pattern (e.g., :filename.ext)
  // This supports formats like ```python:hello.py
  if (!path && parts.length > 1) {
    const possibleFilename = parts[1];
    // Check if it looks like a filename (contains a dot and extension)
    if (possibleFilename && possibleFilename.includes('.')) {
      // It's just a filename, not a full path
      path = possibleFilename;
      saveable = true; // Auto-enable saveable mode for filename format
    }
  }

  // If path is set but not absolute, it's just a suggested filename
  // User will need to edit it to add the full path

  return { language, saveable, path };
}

export function MarkdownRenderer({ content, onSaveFile, workspaceRootPath }: MarkdownRendererProps) {
  return (
    <Box>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight]}
        components={{
          // Custom code block renderer
          code({ className, children, ...props }) {
            // Extract text content from children (handle both string and ReactNode array)
            const codeString = (Array.isArray(children)
              ? children.join('')
              : String(children)
            ).replace(/\n$/, '');

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
                  workspaceRootPath={workspaceRootPath}
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

          // Custom paragraph renderer with mention support
          p({ children }) {
            // Process children to handle mentions
            const processedChildren = React.Children.map(children, (child) => {
              if (typeof child === 'string') {
                return renderTextWithMentions(child);
              }
              return child;
            });

            return (
              <Text size="sm" mb="sm">
                {processedChildren}
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

          // Custom link renderer - handle file paths
          a({ href, children }) {
            // Check if it's a file path (starts with / or file://)
            const isFilePath = href && (
              href.startsWith('/') ||
              href.startsWith('file://') ||
              href.startsWith('orcs-file://')
            );

            if (isFilePath) {
              const filePath = href
                .replace('file://', '')
                .replace('orcs-file://', '');

              const handleClick = async (e: React.MouseEvent) => {
                e.preventDefault();
                try {
                  await openPath(filePath);
                } catch (err) {
                  console.error('Failed to open file:', err);
                }
              };

              return (
                <Anchor
                  href={href}
                  onClick={handleClick}
                  size="sm"
                  style={{ cursor: 'pointer' }}
                >
                  {children}
                </Anchor>
              );
            }

            // Regular external link
            return (
              <Anchor href={href} target="_blank" rel="noopener noreferrer" size="sm">
                {children}
              </Anchor>
            );
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </Box>
  );
}
