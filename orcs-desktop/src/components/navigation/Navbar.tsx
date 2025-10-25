import { ScrollArea, Box, Accordion, Group, Text, Badge } from '@mantine/core';
import { Session } from '../../types/session';
import { Task } from '../../types/task';
import { MessageType } from '../../types/message';
import { SessionList } from '../sessions/SessionList';
import { WorkspacePanel } from '../workspace/WorkspacePanel';
import { TaskList } from '../tasks/TaskList';
import { PersonasList } from '../personas/PersonasList';

interface NavbarProps {
  // Sessions
  sessions: Session[];
  currentSessionId: string | null;
  onSessionSelect: (session: Session) => void;
  onSessionDelete: (sessionId: string) => void;
  onSessionRename: (sessionId: string, newTitle: string) => void;
  onNewSession: () => void;

  // Tasks
  tasks: Task[];
  onTaskToggle: (taskId: string) => void;
  onTaskDelete: (taskId: string) => void;

  // Workspace
  onAttachFile?: (file: File) => void;
  includeWorkspaceInPrompt?: boolean;
  onToggleIncludeWorkspaceInPrompt?: (value: boolean) => void;
  onGoToSession?: (sessionId: string) => void;

  // Common
  onMessage: (type: MessageType, author: string, text: string) => void;
}

export function Navbar({
  sessions,
  currentSessionId,
  onSessionSelect,
  onSessionDelete,
  onSessionRename,
  onNewSession,
  tasks,
  onTaskToggle,
  onTaskDelete,
  onAttachFile,
  includeWorkspaceInPrompt,
  onToggleIncludeWorkspaceInPrompt,
  onGoToSession,
  onMessage,
}: NavbarProps) {
  return (
    <ScrollArea h="100vh" type="auto">
      <Box p="md">
        <Accordion
          defaultValue={['sessions', 'files', 'tasks', 'personas']}
          multiple
          variant="separated"
        >
          {/* セッション */}
          <Accordion.Item value="sessions">
            <Accordion.Control>
              <Group gap="xs">
                <Text>💬</Text>
                <Text fw={600}>Sessions</Text>
                <Badge size="sm" color="blue" variant="light">
                  {sessions.length}
                </Badge>
              </Group>
            </Accordion.Control>
            <Accordion.Panel>
              <SessionList
                sessions={sessions}
                currentSessionId={currentSessionId || undefined}
                onSessionSelect={onSessionSelect}
                onSessionDelete={onSessionDelete}
                onSessionRename={onSessionRename}
                onNewSession={onNewSession}
              />
            </Accordion.Panel>
          </Accordion.Item>

          {/* ワークスペース */}
          <Accordion.Item value="files">
            <Accordion.Control>
              <Group gap="xs">
                <Text>📁</Text>
                <Text fw={600}>Workspace</Text>
              </Group>
            </Accordion.Control>
            <Accordion.Panel>
              <WorkspacePanel
                onAttachFile={onAttachFile}
                includeInPrompt={includeWorkspaceInPrompt}
                onToggleIncludeInPrompt={onToggleIncludeWorkspaceInPrompt}
                onGoToSession={onGoToSession}
              />
            </Accordion.Panel>
          </Accordion.Item>

          {/* タスク */}
          <Accordion.Item value="tasks">
            <Accordion.Control>
              <Group gap="xs">
                <Text>✅</Text>
                <Text fw={600}>Tasks</Text>
                <Badge size="sm" color="blue" variant="light">
                  {tasks.filter(t => t.status !== 'completed').length}
                </Badge>
              </Group>
            </Accordion.Control>
            <Accordion.Panel>
              <TaskList
                tasks={tasks}
                onTaskToggle={onTaskToggle}
                onTaskDelete={onTaskDelete}
              />
            </Accordion.Panel>
          </Accordion.Item>

          {/* ペルソナ */}
          <Accordion.Item value="personas">
            <Accordion.Control>
              <Group gap="xs">
                <Text>⭐️</Text>
                <Text fw={600}>Personas</Text>
              </Group>
            </Accordion.Control>
            <Accordion.Panel>
              <PersonasList onMessage={onMessage} />
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
      </Box>
    </ScrollArea>
  );
}
