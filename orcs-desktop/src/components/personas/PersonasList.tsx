import { useState, useEffect } from 'react';
import { Stack, ScrollArea, Group, Text, Box, Checkbox, ActionIcon, Tooltip } from '@mantine/core';
import { IconSettings, IconPlus, IconPencil, IconTrash } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { PersonaConfig } from '../../types/agent';
import { PersonaEditorModal } from './PersonaEditorModal';

export function PersonasList() {
  const [personaConfigs, setPersonaConfigs] = useState<PersonaConfig[]>([]);
  const [activeParticipantIds, setActiveParticipantIds] = useState<string[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingPersona, setEditingPersona] = useState<Partial<PersonaConfig> | null>(null);

  // Fetch personas from backend
  const fetchPersonas = async () => {
    try {
      const personas = await invoke<PersonaConfig[]>('get_personas');
      const activeIds = await invoke<string[]>('get_active_participants');
      setPersonaConfigs(personas);
      setActiveParticipantIds(activeIds);
    } catch (error) {
      console.error('Failed to fetch personas:', error);
    }
  };

  useEffect(() => {
    fetchPersonas();
  }, []);

  const handleToggleParticipant = async (personaId: string, isChecked: boolean) => {
    try {
      if (isChecked) {
        await invoke('add_participant', { personaId });
      } else {
        await invoke('remove_participant', { personaId });
      }
      const updatedIds = await invoke<string[]>('get_active_participants');
      setActiveParticipantIds(updatedIds);
    } catch (error) {
      console.error(error);
    }
  };

  const handleOpenConfigFile = async () => {
    try {
      const path = await invoke<string>('get_config_path');
      await openPath(path);
    } catch (error) {
      console.error('Failed to open config file:', error);
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
      await fetchPersonas();
      window.alert('Configuration saved. Please restart the application for all changes to take effect.');
    } catch (error) {
      console.error('Failed to save persona configs:', error);
      window.alert('Failed to save configuration. Please check the console for details.');
    }
  };

  // Handler for saving a persona (create or update)
  const handleSavePersona = async (personaToSave: PersonaConfig) => {
    const existingIndex = personaConfigs.findIndex(p => p.id === personaToSave.id);
    let updatedConfigs: PersonaConfig[];

    if (existingIndex >= 0) {
      // Update existing persona
      updatedConfigs = [...personaConfigs];
      updatedConfigs[existingIndex] = personaToSave;
    } else {
      // Add new persona
      updatedConfigs = [...personaConfigs, personaToSave];
    }

    await saveAndReload(updatedConfigs);
    handleCloseModal();
  };

  // Handler for deleting a persona
  const handleDeletePersona = async (id: string) => {
    if (!window.confirm('Are you sure you want to delete this persona?')) {
      return;
    }

    const updatedConfigs = personaConfigs.filter(p => p.id !== id);
    await saveAndReload(updatedConfigs);
  };

  const renderPersona = (persona: PersonaConfig) => {
    const isActive = activeParticipantIds.includes(persona.id);

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
      <Group justify="space-between" px="md" pt="md">
        <Group gap="xs">
          <Text size="lg" fw={700}>
            Personas
          </Text>
          <Tooltip label="Add persona" withArrow>
            <ActionIcon variant="subtle" color="blue" onClick={() => handleOpenModal()}>
              <IconPlus size={16} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Open config file" withArrow>
            <ActionIcon variant="subtle" color="gray" onClick={handleOpenConfigFile}>
              <IconSettings size={16} />
            </ActionIcon>
          </Tooltip>
        </Group>
        <Text size="sm" c="dimmed">
          {activeParticipantIds.length} participating
        </Text>
      </Group>

      {/* ペルソナリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {/* ペルソナリスト */}
          {personaConfigs.length > 0 && (
            <Box>
              <Stack gap={4}>
                {personaConfigs.map(renderPersona)}
              </Stack>
            </Box>
          )}

          {/* 空の状態 */}
          {personaConfigs.length === 0 && (
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
