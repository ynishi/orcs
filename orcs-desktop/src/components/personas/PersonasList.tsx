import { useState, useEffect } from 'react';
import { Stack, ScrollArea, Group, Text, Box, Checkbox, ActionIcon, Tooltip, Select, Badge, Modal, Button } from '@mantine/core';
import { IconPlus, IconPencil, IconTrash, IconDeviceFloppy } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { PersonaConfig } from '../../types/agent';
import { PersonaEditorModal } from './PersonaEditorModal';
import { handleAndPersistSystemMessage, conversationMessage } from '../../utils/systemMessage';
import { CONVERSATION_MODES, DEFAULT_STYLE_ICON, DEFAULT_STYLE_LABEL, TALK_STYLES, EXECUTION_STRATEGIES } from '../../types/conversation';
import { MessageType } from '../../types/message';
import { usePersonaStore } from '../../stores/personaStore';

const BACKEND_LABELS: Record<PersonaConfig['backend'], string> = {
  claude_cli: 'Claude CLI',
  claude_api: 'Claude API',
  gemini_cli: 'Gemini CLI',
  gemini_api: 'Gemini API',
  open_ai_api: 'OpenAI API',
  codex_cli: 'Codex CLI',
  kaiba_api: 'Kaiba API',
};

interface PersonasListProps {
  onStrategyChange?: (strategy: string) => void;
  onConversationModeChange?: (mode: string) => void;
  onTalkStyleChange?: (style: string | null) => void;
  onToggleParticipant?: (personaId: string, isActive: boolean) => Promise<void>;
  onMessage?: (type: MessageType, author: string, text: string) => void;
  personas?: PersonaConfig[];
  activeParticipantIds?: string[];
  executionStrategy?: string;
  conversationMode?: string;
  talkStyle?: string | null;
  onRefresh?: () => Promise<void>;
  onRefreshSessions?: () => Promise<void>;
}

export function PersonasList({
  onStrategyChange,
  onConversationModeChange,
  onTalkStyleChange,
  onToggleParticipant,
  onMessage,
  personas: propsPersonas,
  activeParticipantIds: propsActiveParticipantIds,
  executionStrategy: propsExecutionStrategy,
  conversationMode: propsConversationMode,
  talkStyle: propsTalkStyle,
  onRefresh,
  onRefreshSessions,
}: PersonasListProps) {
  // Use personaStore for save operations
  const { addPersona, updatePersona, deletePersona, saveAdhocPersona } = usePersonaStore();

  // Use props if provided, otherwise maintain local state for backwards compatibility
  const [personaConfigs, setPersonaConfigs] = useState<PersonaConfig[]>([]);
  const [activeParticipantIds, setActiveParticipantIds] = useState<string[]>([]);

  const personas = propsPersonas ?? personaConfigs;
  const activeIds = propsActiveParticipantIds ?? activeParticipantIds;
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingPersona, setEditingPersona] = useState<Partial<PersonaConfig> | null>(null);
  const [selectedStrategy, setSelectedStrategy] = useState<string>('default');
  const [selectedConversationMode, setSelectedConversationMode] = useState<string>('normal');
  const [selectedTalkStyle, setSelectedTalkStyle] = useState<string | null>(null);
  const [deletingPersonaId, setDeletingPersonaId] = useState<string | null>(null);

  const handleStrategyChange = (value: string | null) => {
    const strategy = value || 'broadcast';

    // Update local state
    setSelectedStrategy(strategy);

    // Notify parent component (parent will handle backend and system message)
    onStrategyChange?.(strategy);
  };

  const handleConversationModeChange = (value: string | null) => {
    const mode = value || 'normal';

    // Update local state
    setSelectedConversationMode(mode);

    // Notify parent component (parent will handle backend and system message)
    onConversationModeChange?.(mode);
  };

  const handleTalkStyleChange = (value: string | null) => {
    const style = value || null;

    // Update local state
    setSelectedTalkStyle(style);

    // Notify parent component (parent will handle backend and system message)
    onTalkStyleChange?.(style);
  };

  // Sync local state with props (for global changes from StatusBar)
  useEffect(() => {
    if (propsTalkStyle !== undefined) {
      setSelectedTalkStyle(propsTalkStyle);
    }
  }, [propsTalkStyle]);

  useEffect(() => {
    if (propsConversationMode !== undefined) {
      setSelectedConversationMode(propsConversationMode);
    }
  }, [propsConversationMode]);

  // Fetch personas from backend (only if not provided via props)
  const fetchPersonas = async () => {
    try {
      const personas = await invoke<PersonaConfig[]>('get_personas');
      const activeIds = await invoke<string[]>('get_active_participants');
      const strategy = await invoke<string>('get_execution_strategy');
      const conversationMode = await invoke<string>('get_conversation_mode');
      const talkStyle = await invoke<string | null>('get_talk_style');
      setPersonaConfigs(personas);
      setActiveParticipantIds(activeIds);
      setSelectedStrategy(strategy);
      setSelectedConversationMode(conversationMode);
      setSelectedTalkStyle(talkStyle);
    } catch (error) {
      console.error('Failed to fetch personas:', error);
    }
  };

  useEffect(() => {
    // Only fetch if personas not provided via props
    if (!propsPersonas) {
      fetchPersonas();
    }
  }, [propsPersonas]);

  // Sync execution strategy from props
  useEffect(() => {
    if (propsExecutionStrategy) {
      setSelectedStrategy(propsExecutionStrategy);
    }
  }, [propsExecutionStrategy]);

  const handleToggleParticipant = async (personaId: string, isChecked: boolean) => {
    // If onToggleParticipant provided (from App.tsx via sessionSettingsStore), use it
    if (onToggleParticipant) {
      try {
        await onToggleParticipant(personaId, isChecked);
      } catch (error) {
        console.error('Failed to toggle participant:', error);
      }
      return;
    }

    // Otherwise fallback to direct backend call (backwards compatibility)
    try {
      const persona = personas.find(p => p.id === personaId);
      if (!persona) return;

      if (isChecked) {
        await invoke('add_participant', { personaId });
        onMessage && await handleAndPersistSystemMessage(
          conversationMessage(`${persona.name} が会話に参加しました`, 'success'),
          onMessage, invoke
        );
      } else {
        await invoke('remove_participant', { personaId });
        onMessage && await handleAndPersistSystemMessage(
          conversationMessage(`${persona.name} が会話から退出しました`, 'info'),
          onMessage, invoke
        );
      }

      // If onRefresh provided, call it to sync with parent
      if (onRefresh) {
        await onRefresh();
      } else {
        // Otherwise update local state for backwards compatibility
        const updatedIds = await invoke<string[]>('get_active_participants');
        setActiveParticipantIds(updatedIds);
      }
    } catch (error) {
      console.error(error);
      onMessage && await handleAndPersistSystemMessage(
        conversationMessage(`Failed to update participant: ${error}`, 'error'),
        onMessage, invoke
      );
    }
  };

  // Handler to open modal for creating or editing
  const handleOpenModal = (persona?: PersonaConfig) => {
    setEditingPersona(persona || { source: 'User' });
    setIsModalOpen(true);
  };

  // Handler to close modal
  const handleCloseModal = () => {
    setIsModalOpen(false);
    setEditingPersona(null);
  };

  // Helper function to save and reload
  const saveAndReload = async (
    operation: () => Promise<void>,
    updatedConfigs: PersonaConfig[]
  ) => {
    try {
      await operation();
      setPersonaConfigs(updatedConfigs);

      // personaStore automatically updates its state, so onRefresh is optional now
      // Keep onRefresh call for any additional UI updates if needed
      if (onRefresh) {
        await onRefresh();
      }

      onMessage && await handleAndPersistSystemMessage(
        conversationMessage('Persona configuration saved successfully.', 'success', '✅'),
        onMessage, invoke
      );
    } catch (error) {
      console.error('Failed to save persona configs:', error);
      onMessage && await handleAndPersistSystemMessage(
        conversationMessage(`Failed to save configuration: ${error}`, 'error', '❌'),
        onMessage, invoke
      );
    }
  };

  // Handler for saving a persona (create or update)
  const handleSavePersona = async (personaToSave: PersonaConfig) => {
    const existingIndex = personas.findIndex(p => p.id === personaToSave.id);
    let updatedConfigs: PersonaConfig[];

    if (existingIndex >= 0) {
      // Update existing persona
      updatedConfigs = [...personas];
      updatedConfigs[existingIndex] = personaToSave;
      await saveAndReload(() => updatePersona(personaToSave), updatedConfigs);
    } else {
      // Add new persona
      updatedConfigs = [...personas, personaToSave];
      await saveAndReload(() => addPersona(personaToSave), updatedConfigs);
    }

    handleCloseModal();
  };

  // Handler for initiating delete confirmation
  const handleRequestDelete = (id: string) => {
    setDeletingPersonaId(id);
  };

  // Handler for confirming deletion
  const handleConfirmDelete = async () => {
    if (!deletingPersonaId) return;

    const idToDelete = deletingPersonaId;
    setDeletingPersonaId(null);

    const updatedConfigs = personas.filter(p => p.id !== idToDelete);
    await saveAndReload(() => deletePersona(idToDelete), updatedConfigs);
  };

  // Handler for cancelling deletion
  const handleCancelDelete = () => {
    setDeletingPersonaId(null);
  };

  const renderPersona = (persona: PersonaConfig) => {
    const isActive = activeIds.includes(persona.id);

    return (
      <Group
        key={persona.id}
        gap="sm"
        wrap="nowrap"
        p="xs"
        style={{
          borderRadius: '8px',
          backgroundColor: isActive ? 'rgba(64, 192, 87, 0.1)' : 'transparent',
          transition: 'background-color 0.15s ease',
        }}
      >
        {/* チェックボックス */}
        <Checkbox
          checked={isActive}
          onChange={(event) => handleToggleParticipant(persona.id, event.currentTarget.checked)}
          size="sm"
          color="green"
        />

        {/* ペルソナ情報 */}
        <Box style={{ flex: 1, minWidth: 0 }}>
          <Group gap="xs" mb={4}>
            <Text size="sm" fw={600} truncate style={{ minWidth: '80px' }}>
              {persona.icon}{persona.name}
            </Text>
            {persona.source === 'Adhoc' && (
              <Badge size="xs" color="orange" variant="filled">
                🔶 Adhoc
              </Badge>
            )}
            <Badge size="xs" color={persona.backend === 'gemini_cli' ? 'violet' : 'gray'}>
              {BACKEND_LABELS[persona.backend] || 'Claude CLI'}
            </Badge>
            {persona.model_name && (
              <Badge size="xs" variant="outline" color="blue">
                {persona.model_name}
              </Badge>
            )}
          </Group>
          <Text size="xs" c="dimmed" truncate>
            {persona.role}
          </Text>
          <Text size="xs" c="dimmed" truncate>
            {persona.background}
          </Text>
        </Box>

        {/* アクションボタン */}
        <Group gap={4}>
          {persona.source === 'Adhoc' && (
            <Tooltip label="Save as permanent persona" withArrow>
              <ActionIcon
                variant="filled"
                color="green"
                size="sm"
                onClick={async () => {
                  try {
                    // Use personaStore to save adhoc persona
                    await saveAdhocPersona(persona.id);

                    // Persist success message to session
                    await invoke('append_system_messages', {
                      messages: [{
                        content: `💾 ${persona.name} saved as permanent persona`,
                        messageType: 'info',
                        severity: 'info',
                      }]
                    });

                    // personaStore automatically reloads personas after save
                    if (onRefreshSessions) await onRefreshSessions();
                  } catch (error) {
                    console.error('Failed to save adhoc persona:', error);

                    // Persist error message to session
                    await invoke('append_system_messages', {
                      messages: [{
                        content: `❌ Failed to save persona: ${error}`,
                        messageType: 'error',
                        severity: 'error',
                      }]
                    });
                  }
                }}
              >
                <IconDeviceFloppy size={14} />
              </ActionIcon>
            </Tooltip>
          )}
          <Tooltip label="Edit" withArrow>
            <ActionIcon
              variant="subtle"
              color="blue"
              size="sm"
              onClick={() => handleOpenModal(persona)}
            >
              <IconPencil size={14} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Delete" withArrow>
            <ActionIcon
              variant="subtle"
              color="red"
              size="sm"
              onClick={() => handleRequestDelete(persona.id)}
            >
              <IconTrash size={14} />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>
    );
  };

  return (
    <Stack gap="md" h="100%">
      {/* ヘッダー */}
      <Stack gap="xs" px="md" pt="md">
        <Group justify="space-between">
          <Group gap="xs">
            <Text size="lg" fw={700}>
              Personas
            </Text>
            <Tooltip label="Add persona" withArrow>
              <ActionIcon variant="subtle" color="blue" onClick={() => handleOpenModal()}>
                <IconPlus size={16} />
              </ActionIcon>
            </Tooltip>
          </Group>
          <Text size="sm" c="dimmed">
            {activeIds.length} participating
          </Text>
        </Group>

        {/* Talk Style Selection */}
        <Box>
          <Text size="xs" c="dimmed" mb={4}>
            Talk Style
          </Text>
          <Select
            size="xs"
            data={[
              { value: '', label: `${DEFAULT_STYLE_ICON} ${DEFAULT_STYLE_LABEL}` },
              ...TALK_STYLES.map(style => ({
                value: style.value,
                label: `${style.icon} ${style.label}`,
              })),
            ]}
            value={selectedTalkStyle || ''}
            onChange={handleTalkStyleChange}
            placeholder={DEFAULT_STYLE_LABEL}
            clearable
          />
        </Box>

        {/* Strategy Selection */}
        <Box>
          <Text size="xs" c="dimmed" mb={4}>
            Execution Strategy
          </Text>
          <Select
            size="xs"
            data={EXECUTION_STRATEGIES.map(strategy => ({
              value: strategy.value,
              label: `${strategy.icon} ${strategy.label}`,
            }))}
            value={selectedStrategy}
            onChange={handleStrategyChange}
            placeholder="Select strategy"
            allowDeselect={false}
          />
        </Box>

        {/* Conversation Mode Selection */}
        <Box>
          <Text size="xs" c="dimmed" mb={4}>
            Conversation Mode
          </Text>
          <Select
            size="xs"
            data={CONVERSATION_MODES.map(mode => ({
              value: mode.value,
              label: `${mode.icon} ${mode.label}`,
            }))}
            value={selectedConversationMode}
            onChange={handleConversationModeChange}
            placeholder="Select mode"
            allowDeselect={false}
          />
        </Box>
      </Stack>

      {/* ペルソナリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {/* ペルソナリスト */}
          {personas.length > 0 && (
            <Box>
              <Stack gap={4}>
                {personas.map(renderPersona)}
              </Stack>
            </Box>
          )}

          {/* 空の状態 */}
          {personas.length === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                No personas available
              </Text>
              <Text size="xs" c="dimmed" mt="xs">
                Click the + button to create your first persona
              </Text>
            </Box>
          )}
        </Stack>
      </ScrollArea>

      {/* フッター */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          {personaConfigs.length} total personas
        </Text>
      </Box>

      {/* モーダル */}
      <PersonaEditorModal
        opened={isModalOpen}
        onClose={handleCloseModal}
        persona={editingPersona}
        onSave={handleSavePersona}
      />

      {/* Delete confirmation modal */}
      <Modal
        opened={!!deletingPersonaId}
        onClose={handleCancelDelete}
        title="Delete Persona"
        centered
        size="sm"
      >
        <Stack gap="md">
          <Text size="sm">
            Are you sure you want to delete this persona?
          </Text>
          <Text size="xs" c="dimmed">
            This action cannot be undone.
          </Text>
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={handleCancelDelete}>
              Cancel
            </Button>
            <Button color="red" onClick={handleConfirmDelete}>
              Delete
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  );
}
