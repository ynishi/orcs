import { useState, useEffect } from 'react';
import { Stack, Button, Group, Text, ScrollArea, ActionIcon, Modal, TextInput, Select, MultiSelect, Tooltip, Badge, Box } from '@mantine/core';
import { IconPlus, IconTrash } from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import type { DialoguePreset } from '../../types/conversation';
import { EXECUTION_STRATEGIES, CONVERSATION_MODES, TALK_STYLES, DEFAULT_STYLE_ICON, DEFAULT_STYLE_LABEL } from '../../types/conversation';
import { usePersonaStore } from '../../stores/personaStore';

interface DialoguePresetListProps {
  onStrategyChange?: (strategy: string) => void;
  onConversationModeChange?: (mode: string) => void;
  onTalkStyleChange?: (style: string | null) => void;
  executionStrategy?: string;
  conversationMode?: string;
  talkStyle?: string | null;
}

export function DialoguePresetList({
  onStrategyChange,
  onConversationModeChange,
  onTalkStyleChange,
  executionStrategy,
  conversationMode,
  talkStyle,
}: DialoguePresetListProps) {
  const [presets, setPresets] = useState<DialoguePreset[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalOpened, setModalOpened] = useState(false);
  const [newPreset, setNewPreset] = useState({
    name: '',
    icon: '',
    description: '',
    executionStrategy: 'broadcast',
    conversationMode: 'normal',
    talkStyle: 'Brainstorm',
    defaultPersonaIds: [] as string[],
  });
  const personas = usePersonaStore((s) => s.personas);

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
        executionStrategy: newPreset.executionStrategy as any,
        conversationMode: newPreset.conversationMode as any,
        talkStyle: newPreset.talkStyle as any,
        createdAt: new Date().toISOString(),
        source: 'user',
        defaultPersonaIds: newPreset.defaultPersonaIds.length > 0 ? newPreset.defaultPersonaIds : undefined,
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
        executionStrategy: 'broadcast',
        conversationMode: 'normal',
        talkStyle: 'Brainstorm',
        defaultPersonaIds: [],
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

      {/* Current Session Settings */}
      <Box p="sm" style={{ backgroundColor: 'var(--mantine-color-gray-0)', borderRadius: 8 }}>
        <Text size="xs" fw={600} c="dimmed" mb="xs">
          Current Session
        </Text>
        <Stack gap="xs">
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
              value={talkStyle || ''}
              onChange={(value) => onTalkStyleChange?.(value || null)}
              placeholder={DEFAULT_STYLE_LABEL}
              clearable
              maxDropdownHeight={280}
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
              value={executionStrategy || 'broadcast'}
              onChange={(value) => onStrategyChange?.(value || 'broadcast')}
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
              value={conversationMode || 'normal'}
              onChange={(value) => onConversationModeChange?.(value || 'normal')}
              placeholder="Select mode"
              allowDeselect={false}
            />
          </Box>
        </Stack>
      </Box>

      <ScrollArea style={{ flex: 1 }}>
        <Stack gap="xs">
          {/* User Presets */}
          {presets.filter(p => p.source === 'user').length > 0 && (
            <>
              <Text size="sm" c="dimmed" fw={600}>User Presets</Text>
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

          {/* System Presets */}
          <Text size="sm" c="dimmed" fw={600} mt={presets.filter(p => p.source === 'user').length > 0 ? "md" : undefined}>System Presets</Text>
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
                  <Badge size="xs" variant="light" color="blue">System</Badge>
                </Group>
                {preset.description && (
                  <Text size="xs" c="dimmed">{preset.description}</Text>
                )}
              </Stack>
            </Group>
          ))}

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
            value={newPreset.talkStyle}
            onChange={(value) => setNewPreset({ ...newPreset, talkStyle: value || 'Brainstorm' })}
            required
          />

          <Select
            label="Execution Strategy"
            data={EXECUTION_STRATEGIES.map(s => ({ value: s.value, label: s.label }))}
            value={newPreset.executionStrategy}
            onChange={(value) => setNewPreset({ ...newPreset, executionStrategy: value || 'broadcast' })}
            required
          />

          <Select
            label="Conversation Mode"
            data={CONVERSATION_MODES.map(m => ({ value: m.value, label: m.label }))}
            value={newPreset.conversationMode}
            onChange={(value) => setNewPreset({ ...newPreset, conversationMode: value || 'normal' })}
            required
          />

          <MultiSelect
            label="Default Personas"
            description="Auto-added when preset is applied"
            data={personas.map(p => ({ value: p.id, label: `${p.icon || ''} ${p.name} (${p.role})`.trim() }))}
            value={newPreset.defaultPersonaIds}
            onChange={(value) => setNewPreset({ ...newPreset, defaultPersonaIds: value })}
            placeholder="Select personas..."
            clearable
            searchable
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
