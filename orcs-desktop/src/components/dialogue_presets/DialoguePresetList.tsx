import { useState, useEffect } from 'react';
import { Stack, Button, Group, Text, ScrollArea, ActionIcon, Modal, TextInput, Select, Tooltip } from '@mantine/core';
import { IconPlus, IconTrash, IconStar } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import type { DialoguePreset } from '../../types/conversation';
import { EXECUTION_STRATEGIES, CONVERSATION_MODES, TALK_STYLES } from '../../types/conversation';

export function DialoguePresetList() {
  const [presets, setPresets] = useState<DialoguePreset[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalOpened, setModalOpened] = useState(false);
  const [newPreset, setNewPreset] = useState({
    name: '',
    icon: '',
    description: '',
    execution_strategy: 'broadcast',
    conversation_mode: 'normal',
    talk_style: 'brainstorm',
  });

  const loadPresets = async () => {
    try {
      setLoading(true);
      const loadedPresets = await invoke<DialoguePreset[]>('get_dialogue_presets');
      setPresets(loadedPresets);
    } catch (error) {
      console.error('Failed to load dialogue presets:', error);
      notifications.show({
        title: 'Error',
        message: `Failed to load presets: ${error}`,
        color: 'red',
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPresets();
  }, []);

  const handleCreatePreset = async () => {
    if (!newPreset.name.trim()) {
      notifications.show({
        title: 'Validation Error',
        message: 'Please enter a preset name',
        color: 'red',
      });
      return;
    }

    try {
      const preset: DialoguePreset = {
        id: crypto.randomUUID(),
        name: newPreset.name,
        icon: newPreset.icon || undefined,
        description: newPreset.description || undefined,
        execution_strategy: newPreset.execution_strategy as any,
        conversation_mode: newPreset.conversation_mode as any,
        talk_style: newPreset.talk_style as any,
        created_at: new Date().toISOString(),
        source: 'user',
      };

      await invoke('save_dialogue_preset', { preset });

      notifications.show({
        title: 'Success',
        message: `Preset "${preset.name}" created`,
        color: 'green',
      });

      setModalOpened(false);
      setNewPreset({
        name: '',
        icon: '',
        description: '',
        execution_strategy: 'broadcast',
        conversation_mode: 'normal',
        talk_style: 'brainstorm',
      });

      await loadPresets();
    } catch (error) {
      console.error('Failed to create preset:', error);
      notifications.show({
        title: 'Error',
        message: `Failed to create preset: ${error}`,
        color: 'red',
      });
    }
  };

  const handleDeletePreset = async (presetId: string, presetName: string) => {
    if (!confirm(`Delete preset "${presetName}"?`)) {
      return;
    }

    try {
      await invoke('delete_dialogue_preset', { presetId });

      notifications.show({
        title: 'Success',
        message: `Preset "${presetName}" deleted`,
        color: 'green',
      });

      await loadPresets();
    } catch (error) {
      console.error('Failed to delete preset:', error);
      notifications.show({
        title: 'Error',
        message: `Failed to delete preset: ${error}`,
        color: 'red',
      });
    }
  };

  return (
    <Stack gap="md" style={{ height: '100%' }}>
      <Group justify="space-between">
        <Text size="lg" fw={700}>Dialogue Presets</Text>
        <Tooltip label="Add preset" withArrow>
          <ActionIcon variant="subtle" color="blue" onClick={() => setModalOpened(true)}>
            <IconPlus size={16} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <ScrollArea style={{ flex: 1 }}>
        <Stack gap="xs">
          {/* System Presets */}
          <Text size="sm" c="dimmed" fw={600}>System Presets</Text>
          {presets.filter(p => p.source === 'system').map((preset) => (
            <Group key={preset.id} gap="xs" p="xs" style={{
              borderRadius: 4,
              border: '1px solid #e0e0e0',
              backgroundColor: '#f8f9fa'
            }}>
              {preset.icon && <Text size="lg">{preset.icon}</Text>}
              <Stack gap={0} style={{ flex: 1 }}>
                <Group gap="xs">
                  <Text size="sm" fw={600}>{preset.name}</Text>
                  <IconStar size={14} color="gold" />
                </Group>
                {preset.description && (
                  <Text size="xs" c="dimmed">{preset.description}</Text>
                )}
              </Stack>
            </Group>
          ))}

          {/* User Presets */}
          {presets.filter(p => p.source === 'user').length > 0 && (
            <>
              <Text size="sm" c="dimmed" fw={600} mt="md">User Presets</Text>
              {presets.filter(p => p.source === 'user').map((preset) => (
                <Group key={preset.id} gap="xs" p="xs" style={{
                  borderRadius: 4,
                  border: '1px solid #e0e0e0',
                }}>
                  {preset.icon && <Text size="lg">{preset.icon}</Text>}
                  <Stack gap={0} style={{ flex: 1 }}>
                    <Text size="sm" fw={600}>{preset.name}</Text>
                    {preset.description && (
                      <Text size="xs" c="dimmed">{preset.description}</Text>
                    )}
                  </Stack>
                  <ActionIcon
                    color="red"
                    variant="subtle"
                    onClick={() => handleDeletePreset(preset.id, preset.name)}
                  >
                    <IconTrash size={16} />
                  </ActionIcon>
                </Group>
              ))}
            </>
          )}

          {presets.length === 0 && !loading && (
            <Text size="sm" c="dimmed" ta="center" mt="md">
              No presets available
            </Text>
          )}
        </Stack>
      </ScrollArea>

      <Modal
        opened={modalOpened}
        onClose={() => setModalOpened(false)}
        title="Create New Preset"
        size="md"
      >
        <Stack gap="md">
          <TextInput
            label="Preset Name"
            placeholder="e.g., Quick Brainstorm"
            value={newPreset.name}
            onChange={(e) => setNewPreset({ ...newPreset, name: e.target.value })}
            required
          />

          <TextInput
            label="Icon (Emoji)"
            placeholder="e.g., 💡"
            value={newPreset.icon}
            onChange={(e) => setNewPreset({ ...newPreset, icon: e.target.value })}
          />

          <TextInput
            label="Description"
            placeholder="e.g., Broadcast + Concise + Brainstorm"
            value={newPreset.description}
            onChange={(e) => setNewPreset({ ...newPreset, description: e.target.value })}
          />

          <Select
            label="Talk Style"
            data={TALK_STYLES.map(s => ({ value: s.value, label: s.label }))}
            value={newPreset.talk_style}
            onChange={(value) => setNewPreset({ ...newPreset, talk_style: value || 'brainstorm' })}
            required
          />

          <Select
            label="Execution Strategy"
            data={EXECUTION_STRATEGIES.map(s => ({ value: s.value, label: s.label }))}
            value={newPreset.execution_strategy}
            onChange={(value) => setNewPreset({ ...newPreset, execution_strategy: value || 'broadcast' })}
            required
          />

          <Select
            label="Conversation Mode"
            data={CONVERSATION_MODES.map(m => ({ value: m.value, label: m.label }))}
            value={newPreset.conversation_mode}
            onChange={(value) => setNewPreset({ ...newPreset, conversation_mode: value || 'normal' })}
            required
          />

          <Group justify="flex-end" mt="md">
            <Button variant="subtle" onClick={() => setModalOpened(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreatePreset}>
              Create
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  );
}
