import { Stack, ScrollArea, Group, Text, Box, UnstyledButton, ActionIcon, Tooltip, TextInput, Switch, Badge, Menu } from '@mantine/core';
import { IconDotsVertical, IconArrowUp, IconArrowDown, IconPencil, IconArchive, IconTrash } from '@tabler/icons-react';
import { Session, getMessageCount, getLastActive } from '../../types/session';
import { Workspace } from '../../types/workspace';
import { useState } from 'react';

interface SessionListProps {
  sessions: Session[];
  currentSessionId?: string;
  currentWorkspaceId?: string;
  workspaces?: Workspace[];
  onSessionSelect?: (session: Session) => void;
  onSessionDelete?: (sessionId: string) => void;
  onSessionRename?: (sessionId: string, newTitle: string) => void;
  onNewSession?: () => void;
  onToggleFavorite?: (sessionId: string) => void;
  onToggleArchive?: (sessionId: string) => void;
  onMoveSortOrder?: (sessionId: string, direction: 'up' | 'down') => void;
}

export function SessionList({
  sessions,
  currentSessionId,
  currentWorkspaceId,
  workspaces = [],
  onSessionSelect,
  onSessionDelete,
  onSessionRename,
  onNewSession,
  onToggleFavorite,
  onToggleArchive,
  onMoveSortOrder,
}: SessionListProps) {
  const [editingSessionId, setEditingSessionId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState<string>('');
  const [filterByWorkspace, setFilterByWorkspace] = useState<boolean>(true); // デフォルトON
  const [showArchived, setShowArchived] = useState<boolean>(false); // デフォルトOFF（非表示）

  // workspace_idからWorkspace名を取得するヘルパー関数
  const getWorkspaceName = (workspaceId?: string): string | null => {
    if (!workspaceId) return null;
    const workspace = workspaces.find(w => w.id === workspaceId);
    return workspace?.name || null;
  };

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

  const sortedSessions = [...filteredSessions].sort((a, b) => {
    // 1. Archivedは常に最後
    if (a.is_archived !== b.is_archived) {
      return a.is_archived ? 1 : -1;
    }

    // 2. Favoriteは常に上
    if (a.is_favorite !== b.is_favorite) {
      return a.is_favorite ? -1 : 1;
    }

    // 3. Favorite内では、sort_orderがあればそれを優先
    if (a.is_favorite && b.is_favorite) {
      if (a.sort_order !== undefined && b.sort_order !== undefined) {
        return a.sort_order - b.sort_order;
      }
      if (a.sort_order !== undefined) return -1;
      if (b.sort_order !== undefined) return 1;
    }

    // 4. それ以外はupdated_atで降順
    return getLastActive(b).getTime() - getLastActive(a).getTime();
  });

  // Show Archivedがfalseの場合は、Archivedセッションを除外
  const visibleSessions = showArchived
    ? sortedSessions
    : sortedSessions.filter(s => !s.is_archived);

  // Favorites/Recentに分離
  const favoriteSessions = visibleSessions.filter(s => s.is_favorite && !s.is_archived);
  const recentSessions = visibleSessions.filter(s => !s.is_favorite && !s.is_archived);
  const archivedSessions = visibleSessions.filter(s => s.is_archived);

  // Favoriteセッションの数を数える（UP/DOWNボタンの表示判定用）
  const favoriteSessionsCount = favoriteSessions.length;

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

  // セッションレンダリング関数
  const renderSession = (session: Session) => (
    <Box
      key={session.id}
      style={{
        borderRadius: '8px',
        border: '1px solid var(--mantine-color-gray-3)',
        backgroundColor: session.id === currentSessionId ? '#e7f5ff' : 'white',
        transition: 'all 0.15s ease',
        cursor: 'pointer',
        overflow: 'hidden',
      }}
    >
      {editingSessionId === session.id ? (
        // 編集モード
        <Box p="md">
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
        <>
          {/* TOPメニュー行 */}
          <Group
            gap="xs"
            px="md"
            py="xs"
            justify="flex-end"
            style={{
              backgroundColor: session.id === currentSessionId ? '#d0ebff' : '#f8f9fa',
              borderBottom: '1px solid var(--mantine-color-gray-3)',
            }}
          >
            {/* Favoriteボタン */}
            <Tooltip label={session.is_favorite ? "Remove from favorites" : "Add to favorites"} withArrow>
              <ActionIcon
                size="sm"
                color={session.is_favorite ? "yellow" : "gray"}
                variant="subtle"
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleFavorite?.(session.id);
                }}
                style={{ flexShrink: 0 }}
              >
                {session.is_favorite ? "⭐" : "☆"}
              </ActionIcon>
            </Tooltip>

            {/* メニュー */}
            <Menu position="bottom-end" withinPortal>
              <Menu.Target>
                <ActionIcon
                  size="sm"
                  color="gray"
                  variant="subtle"
                  onClick={(e) => e.stopPropagation()}
                  style={{ flexShrink: 0 }}
                >
                  <IconDotsVertical size={16} />
                </ActionIcon>
              </Menu.Target>

              <Menu.Dropdown onClick={(e) => e.stopPropagation()}>
                {/* UP/DOWN（Favoriteが2個以上ある場合のみ表示） */}
                {session.is_favorite && onMoveSortOrder && favoriteSessionsCount >= 2 && (
                  <>
                    <Menu.Item
                      leftSection={<IconArrowUp size={14} />}
                      onClick={() => onMoveSortOrder(session.id, 'up')}
                    >
                      Move Up
                    </Menu.Item>
                    <Menu.Item
                      leftSection={<IconArrowDown size={14} />}
                      onClick={() => onMoveSortOrder(session.id, 'down')}
                    >
                      Move Down
                    </Menu.Item>
                    <Menu.Divider />
                  </>
                )}

                {/* Rename */}
                <Menu.Item
                  leftSection={<IconPencil size={14} />}
                  onClick={() => {
                    setEditingSessionId(session.id);
                    setEditingTitle(session.title);
                  }}
                >
                  Rename
                </Menu.Item>

                {/* Archive/Unarchive */}
                <Menu.Item
                  leftSection={<IconArchive size={14} />}
                  onClick={() => onToggleArchive?.(session.id)}
                >
                  {session.is_archived ? 'Unarchive' : 'Archive'}
                </Menu.Item>

                <Menu.Divider />

                {/* Delete */}
                <Menu.Item
                  leftSection={<IconTrash size={14} />}
                  color="red"
                  onClick={() => onSessionDelete?.(session.id)}
                >
                  Delete
                </Menu.Item>
              </Menu.Dropdown>
            </Menu>
          </Group>

          {/* コンテンツエリア */}
          <UnstyledButton
            onClick={() => onSessionSelect?.(session)}
            onDoubleClick={(e) => handleStartEdit(session, e)}
            style={{ width: '100%', textAlign: 'left' }}
          >
            <Box p="md">
              <Text size="sm" fw={600} lineClamp={2} style={{ wordBreak: 'break-word' }}>
                {session.title}
              </Text>
              <Group gap="xs" mt={4}>
                {getWorkspaceName(session.workspace_id) && (
                  <>
                    <Badge size="xs" variant="light" color="blue" style={{ textTransform: 'none' }}>
                      {getWorkspaceName(session.workspace_id)}
                    </Badge>
                    <Text size="xs" c="dimmed">
                      •
                    </Text>
                  </>
                )}
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
        </>
      )}
    </Box>
  );

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

        {/* Show Archivedトグル */}
        <Switch
          size="xs"
          label="Show Archived"
          checked={showArchived}
          onChange={(e) => setShowArchived(e.currentTarget.checked)}
        />
      </Stack>

      {/* セッションリスト */}
      <ScrollArea style={{ flex: 1 }} px="md" type="auto">
        <Stack gap="md">
          {/* Favoritesセクション */}
          {favoriteSessions.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                FAVORITES
              </Text>
              <Stack gap={4}>
                {favoriteSessions.map(renderSession)}
              </Stack>
            </Box>
          )}

          {/* Recentセクション */}
          {recentSessions.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                RECENT
              </Text>
              <Stack gap={4}>
                {recentSessions.map(renderSession)}
              </Stack>
            </Box>
          )}

          {/* Archivedセクション */}
          {showArchived && archivedSessions.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                ARCHIVED
              </Text>
              <Stack gap={4}>
                {archivedSessions.map(renderSession)}
              </Stack>
            </Box>
          )}

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
            ? `${visibleSessions.length} / ${sessions.length} sessions (filtered)`
            : showArchived
            ? `${visibleSessions.length} total sessions`
            : `${visibleSessions.length} / ${sessions.length} sessions (archived hidden)`}
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
