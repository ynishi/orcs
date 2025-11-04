import { useState, useEffect } from 'react';
import { Stack, ScrollArea, Group, Text, Box, Checkbox, ActionIcon, Tooltip, Select, Badge } from '@mantine/core';
import { IconPlus, IconPencil, IconTrash } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { PersonaConfig } from '../../types/agent';
import { PersonaEditorModal } from './PersonaEditorModal';
import { handleSystemMessage, conversationMessage } from '../../utils/systemMessage';
import { CONVERSATION_MODES, TALK_STYLES } from '../../types/conversation';
import { MessageType } from '../../types/message';

// Available execution strategies
const STRATEGIES = [
  { value: 'broadcast', label: '📢 Broadcast' },
  { value: 'sequential', label: '➡️ Sequential' },
  { value: 'mentioned', label: '👤 Mentioned' },
];

const BACKEND_LABELS: Record<PersonaConfig['backend'], string> = {
  claude_cli: 'Claude CLI',
  claude_api: 'Claude API',
  gemini_cli: 'Gemini CLI',
  gemini_api: 'Gemini API',
  open_ai_api: 'OpenAI API',
  codex_cli: 'Codex CLI',
};

interface PersonasListProps {
  onStrategyChange?: (strategy: string) => void;
  onConversationModeChange?: (mode: string) => void;
  onTalkStyleChange?: (style: string | null) => void;
  onMessage?: (type: MessageType, author: string, text: string) => void;
  personas?: PersonaConfig[];
  activeParticipantIds?: string[];
  onRefresh?: () => Promise<void>;
}

export function PersonasList({
  onStrategyChange,
  onConversationModeChange,
  onTalkStyleChange,
  onMessage,
  personas: propsPersonas,
  activeParticipantIds: propsActiveParticipantIds,
  onRefresh,
}: PersonasListProps) {
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

  const handleStrategyChange = async (value: string | null) => {
    const strategy = value || 'broadcast';
    setSelectedStrategy(strategy);
    onStrategyChange?.(strategy);

    // Update backend
    try {
      await invoke('set_execution_strategy', { strategy });

      // Show system message
      const strategyLabel = STRATEGIES.find(s => s.value === strategy)?.label || strategy;
      const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
      handleSystemMessage(
        conversationMessage(
          `Execution strategy changed to: ${strategyLabel} [${timestamp}]`,
          'info'
        ),
        onMessage
      );
    } catch (error) {
      console.error('Failed to set execution strategy:', error);
      handleSystemMessage(
        conversationMessage(
          `Failed to set execution strategy: ${error}`,
          'error'
        ),
        onMessage
      );
    }
  };

  const handleConversationModeChange = async (value: string | null) => {
    const mode = value || 'normal';
    setSelectedConversationMode(mode);

    // Update backend
    try {
      await invoke('set_conversation_mode', { mode });

      // Notify parent component
      onConversationModeChange?.(mode);

      // Show system message
      const modeLabel = CONVERSATION_MODES.find(m => m.value === mode)?.label || mode;
      const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
      handleSystemMessage(
        conversationMessage(
          `Conversation mode changed to: ${modeLabel} [${timestamp}]`,
          'info'
        ),
        onMessage
      );
    } catch (error) {
      console.error('Failed to set conversation mode:', error);
      handleSystemMessage(
        conversationMessage(
          `Failed to set conversation mode: ${error}`,
          'error'
        ),
        onMessage
      );
    }
  };

  const handleTalkStyleChange = async (value: string | null) => {
    const style = value || null;
    setSelectedTalkStyle(style);

    // Update backend
    try {
      await invoke('set_talk_style', { style });

      // Notify parent component
      onTalkStyleChange?.(style);

      // Show system message
      const styleLabel = style ? (TALK_STYLES.find(s => s.value === style)?.label || style) : 'None';
      const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
      handleSystemMessage(
        conversationMessage(
          `Talk style changed to: ${styleLabel} [${timestamp}]`,
          'info'
        ),
        onMessage
      );
    } catch (error) {
      console.error('Failed to set talk style:', error);
      handleSystemMessage(
        conversationMessage(
          `Failed to set talk style: ${error}`,
          'error'
        ),
        onMessage
      );
    }
  };

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

  const handleToggleParticipant = async (personaId: string, isChecked: boolean) => {
    try {
      const persona = personas.find(p => p.id === personaId);
      if (!persona) return;

      if (isChecked) {
        await invoke('add_participant', { personaId });
        handleSystemMessage(
          conversationMessage(`${persona.name} が会話に参加しました`, 'success'),
          onMessage
        );
      } else {
        await invoke('remove_participant', { personaId });
        handleSystemMessage(
          conversationMessage(`${persona.name} が会話から退出しました`, 'info'),
          onMessage
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
      handleSystemMessage(
        conversationMessage(`Failed to update participant: ${error}`, 'error'),
        onMessage
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
  const saveAndReload = async (updatedConfigs: PersonaConfig[]) => {
    try {
      await invoke('save_persona_configs', { configs: updatedConfigs });

      // Refresh using callback if provided, otherwise fetch locally
      if (onRefresh) {
        await onRefresh();
      } else {
        await fetchPersonas();
      }

      handleSystemMessage(
        conversationMessage('Persona configuration saved successfully.', 'success', '✅'),
        onMessage
      );
    } catch (error) {
      console.error('Failed to save persona configs:', error);
      handleSystemMessage(
        conversationMessage(`Failed to save configuration: ${error}`, 'error', '❌'),
        onMessage
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
    } else {
      // Add new persona
      updatedConfigs = [...personas, personaToSave];
    }

    await saveAndReload(updatedConfigs);
    handleCloseModal();
  };

  // Handler for deleting a persona
  const handleDeletePersona = async (id: string) => {
    if (!window.confirm('Are you sure you want to delete this persona?')) {
      return;
    }

    const updatedConfigs = personas.filter(p => p.id !== id);
    await saveAndReload(updatedConfigs);
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
            <Text size="sm" fw={600} truncate>
              {persona.name}
            </Text>
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
              onClick={() => handleDeletePersona(persona.id)}
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
              { value: '', label: '❌ None' },
              ...TALK_STYLES.map(style => ({
                value: style.value,
                label: `${style.icon} ${style.label}`,
              })),
            ]}
            value={selectedTalkStyle || ''}
            onChange={handleTalkStyleChange}
            placeholder="None"
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
            data={STRATEGIES}
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
    </Stack>
  );
}
