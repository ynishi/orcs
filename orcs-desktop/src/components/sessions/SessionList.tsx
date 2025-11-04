import { Stack, ScrollArea, Group, Text, Box, UnstyledButton, ActionIcon, Tooltip, TextInput, Switch } from '@mantine/core';
import { Session, getMessageCount, getLastActive } from '../../types/session';
import { useState } from 'react';

interface SessionListProps {
  sessions: Session[];
  currentSessionId?: string;
  currentWorkspaceId?: string;
  onSessionSelect?: (session: Session) => void;
  onSessionDelete?: (sessionId: string) => void;
  onSessionRename?: (sessionId: string, newTitle: string) => void;
  onNewSession?: () => void;
}

export function SessionList({
  sessions,
  currentSessionId,
  currentWorkspaceId,
  onSessionSelect,
  onSessionDelete,
  onSessionRename,
  onNewSession,
}: SessionListProps) {
  const [editingSessionId, setEditingSessionId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState<string>('');
  const [filterByWorkspace, setFilterByWorkspace] = useState<boolean>(true); // デフォルトON

  // フィルタリングされたセッション
  const filteredSessions = filterByWorkspace && currentWorkspaceId
    ? sessions.filter(s => {
        // workspace_idがnullまたはundefinedのSessionは除外
        if (!s.workspace_id) {
          console.log('[SessionList] Filtering out session with no workspace_id:', s.id, s.title);
          return false;
        }
        const matches = s.workspace_id === currentWorkspaceId;
        console.log('[SessionList] Filter check:', s.id.substring(0, 8), 'workspace_id:', s.workspace_id?.substring(0, 8), 'current:', currentWorkspaceId?.substring(0, 8), 'matches:', matches);
        return matches;
      })
    : sessions;

  console.log('[SessionList] Filter active:', filterByWorkspace, 'currentWorkspaceId:', currentWorkspaceId?.substring(0, 8), 'total sessions:', sessions.length, 'filtered:', filteredSessions.length);

  const sortedSessions = [...filteredSessions].sort(
    (a, b) => getLastActive(b).getTime() - getLastActive(a).getTime()
  );

  const handleStartEdit = (session: Session, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingSessionId(session.id);
    setEditingTitle(session.title);
  };

  const handleSaveEdit = (sessionId: string) => {
    if (editingTitle.trim()) {
      onSessionRename?.(sessionId, editingTitle.trim());
    }
    setEditingSessionId(null);
  };

  const handleCancelEdit = () => {
    setEditingSessionId(null);
    setEditingTitle('');
  };

  return (
    <Stack gap="xs" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* ヘッダー */}
      <Stack gap="xs" px="sm">
        <Group justify="space-between">
          <Text size="sm" fw={600}>
            Sessions
          </Text>
          <Tooltip label="New session" withArrow>
            <ActionIcon
              color="blue"
              variant="light"
              onClick={onNewSession}
              size="xs"
            >
              ➕
            </ActionIcon>
          </Tooltip>
        </Group>

        {/* ワークスペースフィルタートグル */}
        {currentWorkspaceId && (
          <Switch
            size="xs"
            label="Filter by Workspace"
            checked={filterByWorkspace}
            onChange={(e) => setFilterByWorkspace(e.currentTarget.checked)}
          />
        )}
      </Stack>

      {/* セッションリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm" type="auto">
        <Stack gap={4}>
          {sortedSessions.map((session) => (
            <Group
              key={session.id}
              gap="sm"
              wrap="nowrap"
              p="xs"
              style={{
                borderRadius: '8px',
                backgroundColor: session.id === currentSessionId ? '#e7f5ff' : 'transparent',
                transition: 'background-color 0.15s ease',
                cursor: 'pointer',
                position: 'relative',
              }}
              onMouseEnter={(e) => {
                const actionBtns = e.currentTarget.querySelectorAll('.action-btn');
                actionBtns.forEach((btn) => {
                  (btn as HTMLElement).style.opacity = '1';
                });
              }}
              onMouseLeave={(e) => {
                const actionBtns = e.currentTarget.querySelectorAll('.action-btn');
                actionBtns.forEach((btn) => {
                  (btn as HTMLElement).style.opacity = '0';
                });
              }}
            >
              {editingSessionId === session.id ? (
                // 編集モード
                <Box style={{ flex: 1, minWidth: 0 }}>
                  <TextInput
                    size="xs"
                    value={editingTitle}
                    onChange={(e) => setEditingTitle(e.currentTarget.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        handleSaveEdit(session.id);
                      } else if (e.key === 'Escape') {
                        handleCancelEdit();
                      }
                    }}
                    onBlur={() => handleSaveEdit(session.id)}
                    autoFocus
                    onClick={(e) => e.stopPropagation()}
                  />
                </Box>
              ) : (
                // 表示モード
                <>
                  <UnstyledButton
                    onClick={() => onSessionSelect?.(session)}
                    onDoubleClick={(e) => handleStartEdit(session, e)}
                    style={{ flex: 1, minWidth: 0 }}
                  >
                    <Box>
                      <Text size="sm" fw={600} truncate>
                        {session.title}
                      </Text>
                      <Group gap="xs" mt={2}>
                        <Text size="xs" c="dimmed">
                          {getMessageCount(session)} msgs
                        </Text>
                        <Text size="xs" c="dimmed">
                          •
                        </Text>
                        <Text size="xs" c="dimmed">
                          {formatDate(getLastActive(session))}
                        </Text>
                      </Group>
                    </Box>
                  </UnstyledButton>

                  {/* 編集ボタン */}
                  <ActionIcon
                    className="action-btn"
                    size="sm"
                    color="blue"
                    variant="subtle"
                    onClick={(e) => handleStartEdit(session, e)}
                    style={{
                      opacity: 0,
                      transition: 'opacity 0.15s ease',
                      flexShrink: 0,
                    }}
                  >
                    ✏️
                  </ActionIcon>

                  {/* 削除ボタン */}
                  <ActionIcon
                    className="action-btn"
                    size="sm"
                    color="red"
                    variant="subtle"
                    onClick={(e) => {
                      e.stopPropagation();
                      onSessionDelete?.(session.id);
                    }}
                    style={{
                      opacity: 0,
                      transition: 'opacity 0.15s ease',
                      flexShrink: 0,
                    }}
                  >
                    🗑️
                  </ActionIcon>
                </>
              )}
            </Group>
          ))}

          {/* 空の状態 */}
          {sessions.length === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                No sessions yet
              </Text>
              <Text size="xs" c="dimmed" mt="xs">
                Click + to create a new session
              </Text>
            </Box>
          )}
        </Stack>
      </ScrollArea>

      {/* フッター */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          {filterByWorkspace && currentWorkspaceId
            ? `${sortedSessions.length} / ${sessions.length} sessions (filtered by workspace)`
            : `${sessions.length} total sessions`}
        </Text>
      </Box>
    </Stack>
  );
}

// 日付フォーマット
function formatDate(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}
