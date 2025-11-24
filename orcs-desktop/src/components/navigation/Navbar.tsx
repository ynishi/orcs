import { useState } from 'react';
import { ScrollArea, Box, Stack, Text } from '@mantine/core';
import { Session } from '../../types/session';
import { Task, TaskProgress } from '../../types/task';
import { MessageType } from '../../types/message';
import { Workspace } from '../../types/workspace';
import { SessionList } from '../sessions/SessionList';
import { WorkspacePanel } from '../workspace/WorkspacePanel';
import { TaskList } from '../tasks/TaskList';
import { PersonasList } from '../personas/PersonasList';
import { SlashCommandList } from '../slash_commands/SlashCommandList';
import { DialoguePresetList } from '../dialogue_presets/DialoguePresetList';
import { SlashCommand } from '../../types/slash_command';
import { NavbarIcon } from './NavbarIcon';

interface NavbarProps {
  // Sessions
  sessions: Session[];
  currentSessionId: string | null;
  currentWorkspaceId?: string;
  workspaces?: Workspace[];
  onSessionSelect: (session: Session) => void;
  onSessionDelete: (sessionId: string) => void;
  onSessionRename: (sessionId: string, newTitle: string) => void;
  onToggleFavorite?: (sessionId: string) => void;
  onToggleArchive?: (sessionId: string) => void;
  onMoveSortOrder?: (sessionId: string, direction: 'up' | 'down') => void;
  onNewSession: () => void;

  // Tasks
  tasks: Task[];
  taskProgress?: Map<string, TaskProgress>;
  onTaskToggle: (taskId: string) => void;
  onTaskDelete: (taskId: string) => void;
  onRefreshTasks: () => void;
  onSaveTaskToWorkspace?: (task: Task) => Promise<void>;

  // Workspace
  onAttachFile?: (file: File) => void;
  includeWorkspaceInPrompt?: boolean;
  onToggleIncludeWorkspaceInPrompt?: (value: boolean) => void;
  onGoToSession?: (sessionId: string, messageTimestamp?: string) => void;
  onNewSessionWithFile?: (file: File) => void;

  // Common
  onMessage: (type: MessageType, author: string, text: string) => void;
  onSlashCommandsUpdated?: (commands: SlashCommand[]) => void;
  onRunSlashCommand?: (command: SlashCommand, args: string) => void | Promise<void>;
  onConversationModeChange?: (mode: string) => void;
  onTalkStyleChange?: (style: string | null) => void;
  onStrategyChange?: (strategy: string) => void;

  // Personas
  personas?: import('../../types/agent').PersonaConfig[];
  activeParticipantIds?: string[];
  executionStrategy?: string;
  conversationMode?: string;
  talkStyle?: string | null;
  onRefreshPersonas?: () => Promise<void>;
  onRefreshSessions?: () => Promise<void>;
  onToggleParticipant?: (personaId: string, isActive: boolean) => Promise<void>;
}

export function Navbar({
  sessions,
  currentSessionId,
  currentWorkspaceId,
  workspaces,
  onSessionSelect,
  onSessionDelete,
  onSessionRename,
  onToggleFavorite,
  onToggleArchive,
  onMoveSortOrder,
  onNewSession,
  tasks,
  taskProgress,
  onTaskToggle,
  onTaskDelete,
  onRefreshTasks,
  onSaveTaskToWorkspace,
  onAttachFile,
  includeWorkspaceInPrompt,
  onToggleIncludeWorkspaceInPrompt,
  onGoToSession,
  onNewSessionWithFile,
  onMessage,
  onSlashCommandsUpdated,
  onRunSlashCommand,
  onConversationModeChange,
  onTalkStyleChange,
  onStrategyChange,
  personas,
  activeParticipantIds,
  executionStrategy,
  conversationMode,
  talkStyle,
  onRefreshPersonas,
  onRefreshSessions,
  onToggleParticipant,
}: NavbarProps) {
  const [activeTab, setActiveTab] = useState<'sessions' | 'workspace' | 'tasks' | 'personas' | 'commands' | 'presets'>('sessions');

  const activeTasks = tasks.filter(t => t.status === 'Running' || t.status === 'Pending').length;

  return (
    <Box style={{ display: 'flex', height: '100vh' }}>
      {/* Icon Bar */}
      <Stack gap={0} style={{
        width: 48,
        borderRight: '1px solid var(--mantine-color-gray-3)',
        flexShrink: 0,
      }}>
        <NavbarIcon
          icon="💬"
          label="Sessions"
          active={activeTab === 'sessions'}
          onClick={() => setActiveTab('sessions')}
          badge={sessions.length}
        />
        <NavbarIcon
          icon="📁"
          label="Workspace"
          active={activeTab === 'workspace'}
          onClick={() => setActiveTab('workspace')}
        />
        <NavbarIcon
          icon="✅"
          label="Tasks"
          active={activeTab === 'tasks'}
          onClick={() => setActiveTab('tasks')}
          badge={activeTasks}
        />
        <NavbarIcon
          icon="⭐️"
          label="Personas"
          active={activeTab === 'personas'}
          onClick={() => setActiveTab('personas')}
        />
        <NavbarIcon
          icon="⚡"
          label="Slash Commands"
          active={activeTab === 'commands'}
          onClick={() => setActiveTab('commands')}
        />
        <NavbarIcon
          icon="🎨"
          label="Dialogue Presets"
          active={activeTab === 'presets'}
          onClick={() => setActiveTab('presets')}
        />
      </Stack>

      {/* Content Panel */}
      <ScrollArea h="100vh" style={{ flex: 1 }} type="auto">
        <Box p="md">
          {/* Section Header */}
          <Text size="lg" fw={600} mb="md" style={{
            paddingBottom: '8px',
            borderBottom: '2px solid var(--mantine-color-gray-3)'
          }}>
            {activeTab === 'sessions' && 'Sessions'}
            {activeTab === 'workspace' && 'Workspace'}
            {activeTab === 'tasks' && 'Tasks'}
            {activeTab === 'personas' && 'Personas'}
            {activeTab === 'commands' && 'Slash Commands'}
            {activeTab === 'presets' && 'Dialogue Presets'}
          </Text>

          {/* Content */}
          {activeTab === 'sessions' && (
            <SessionList
              sessions={sessions}
              currentSessionId={currentSessionId || undefined}
              currentWorkspaceId={currentWorkspaceId}
              workspaces={workspaces}
              onSessionSelect={onSessionSelect}
              onSessionDelete={onSessionDelete}
              onSessionRename={onSessionRename}
              onToggleFavorite={onToggleFavorite}
              onToggleArchive={onToggleArchive}
              onMoveSortOrder={onMoveSortOrder}
              onNewSession={onNewSession}
            />
          )}

          {activeTab === 'workspace' && (
            <WorkspacePanel
              onAttachFile={onAttachFile}
              includeInPrompt={includeWorkspaceInPrompt}
              onToggleIncludeInPrompt={onToggleIncludeWorkspaceInPrompt}
              onGoToSession={onGoToSession}
              onNewSessionWithFile={onNewSessionWithFile}
            />
          )}

          {activeTab === 'tasks' && (
            <TaskList
              tasks={tasks}
              taskProgress={taskProgress}
              sessions={sessions}
              workspaces={workspaces}
              currentWorkspaceId={currentWorkspaceId}
              onTaskToggle={onTaskToggle}
              onTaskDelete={onTaskDelete}
              onRefresh={onRefreshTasks}
              onSaveToWorkspace={onSaveTaskToWorkspace}
            />
          )}

          {activeTab === 'personas' && (
            <PersonasList
              onMessage={onMessage}
              onConversationModeChange={onConversationModeChange}
              onTalkStyleChange={onTalkStyleChange}
              onStrategyChange={onStrategyChange}
              personas={personas}
              activeParticipantIds={activeParticipantIds}
              executionStrategy={executionStrategy}
              conversationMode={conversationMode}
              talkStyle={talkStyle}
              onRefresh={onRefreshPersonas}
              onRefreshSessions={onRefreshSessions}
              onToggleParticipant={onToggleParticipant}
            />
          )}

          {activeTab === 'commands' && (
            <SlashCommandList
              onMessage={onMessage}
              onCommandsUpdated={onSlashCommandsUpdated}
              onRunCommand={onRunSlashCommand}
            />
          )}

          {activeTab === 'presets' && (
            <DialoguePresetList />
          )}
        </Box>
      </ScrollArea>
    </Box>
  );
}
