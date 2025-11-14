import React, { useState, useEffect } from 'react';
import { Modal, TextInput, Textarea, Switch, Button, Stack, Group, Select, ColorInput } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import { PersonaConfig } from '../../types/agent';
import { CreatePersonaRequest } from '../../types/persona_request';

const CLAUDE_MODEL_OPTIONS = [
  { value: '', label: 'Default (Sonnet 4.5)' },
  { value: 'claude-opus-4-1-20250805', label: 'Claude Opus 4.1 (2025-08-05)' },
  { value: 'claude-opus-4-20250514', label: 'Claude Opus 4.0 (2025-05-14)' },
  { value: 'claude-sonnet-4-5-20250929', label: 'Claude Sonnet 4.5 (2025-09-29)' },
  { value: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4.0 (2025-05-14)' },
  { value: 'claude-3-5-haiku-20241022', label: 'Claude 3.5 Haiku (2024-10-22)' },
];

const GEMINI_MODEL_OPTIONS = [
  { value: 'gemini-2.5-pro', label: 'Gemini 2.5 Pro' },
  { value: 'gemini-2.5-flash', label: 'Gemini 2.5 Flash' },
  { value: 'gemini-2.5-flash-lite', label: 'Gemini 2.5 Flash Lite' },
];

interface PersonaEditorModalProps {
  opened: boolean;
  onClose: () => void;
  persona: Partial<PersonaConfig> | null;
  onSave: (persona: PersonaConfig) => void;
}

export const PersonaEditorModal: React.FC<PersonaEditorModalProps> = ({
  opened,
  onClose,
  persona,
  onSave,
}) => {
  const [backendOptions, setBackendOptions] = useState<Array<{ value: string; label: string }>>([]);
  const [formData, setFormData] = useState<Partial<PersonaConfig>>({
    id: '',
    name: '',
    role: '',
    background: '',
    communication_style: '',
    default_participant: false,
    backend: 'claude_cli',
    model_name: undefined,
    icon: undefined,
    base_color: undefined,
  });

  // Fetch backend options on mount
  useEffect(() => {
    const fetchBackendOptions = async () => {
      try {
        const options = await invoke<Array<[string, string]>>('get_persona_backend_options');
        setBackendOptions(options.map(([value, label]) => ({ value, label })));
      } catch (error) {
        console.error('Failed to fetch backend options:', error);
        // Fallback to empty array
        setBackendOptions([]);
      }
    };
    fetchBackendOptions();
  }, []);

  // Update form data when persona prop changes
  useEffect(() => {
    if (persona) {
      setFormData({
        id: persona.id || '',
        name: persona.name || '',
        role: persona.role || '',
        background: persona.background || '',
        communication_style: persona.communication_style || '',
        default_participant: persona.default_participant || false,
        backend: persona.backend || 'claude_cli',
        model_name: persona.model_name,
        icon: persona.icon,
        base_color: persona.base_color,
      });
    } else {
      setFormData({
        id: '',
        name: '',
        role: '',
        background: '',
        communication_style: '',
        default_participant: false,
        backend: 'claude_cli',
        model_name: undefined,
        icon: undefined,
        base_color: undefined,
      });
    }
  }, [persona]);

  const handleSave = async () => {
    // Validate required fields
    if (!formData.name) {
      alert('Name is a required field');
      return;
    }

    // If editing, bypass create_persona and let parent persist the updated config
    if (isEditing && persona?.id) {
      const updatedPersona: PersonaConfig = {
        id: persona.id,
        source: persona.source || 'User',
        name: formData.name || '',
        role: formData.role || '',
        background: formData.background || '',
        communication_style: formData.communication_style || '',
        default_participant: formData.default_participant || false,
        backend: (formData.backend || 'claude_cli') as PersonaConfig['backend'],
        model_name: formData.model_name || undefined,
        icon: formData.icon || undefined,
        base_color: formData.base_color || undefined,
      };

      onSave(updatedPersona);
      return;
    }

    try {
      // Build CreatePersonaRequest (ID is always auto-generated)
      const request: CreatePersonaRequest = {
        name: formData.name,
        role: formData.role || '',
        background: formData.background || '',
        communication_style: formData.communication_style || '',
        default_participant: formData.default_participant || false,
        backend: (formData.backend || 'claude_cli') as CreatePersonaRequest['backend'],
        model_name: formData.model_name || undefined,
        icon: formData.icon || undefined,
        base_color: formData.base_color || undefined,
      };

      // Call unified create_persona command
      const createdPersona = await invoke<PersonaConfig>('create_persona', { request });

      // Pass to parent
      onSave(createdPersona);
    } catch (error) {
      console.error('Failed to create persona:', error);
      alert(`Failed to create persona: ${error}`);
    }
  };

  const isEditing = !!persona?.id;

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={isEditing ? 'Edit Persona' : 'Create New Persona'}
      size="lg"
    >
      <Stack gap="md">
        {isEditing && (
          <TextInput
            label="ID"
            placeholder="unique-persona-id"
            value={formData.id}
            disabled
            description="UUID (auto-generated, read-only)"
          />
        )}

        <TextInput
          label="Name"
          placeholder="Persona Name"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.currentTarget.value })}
          required
        />

        <TextInput
          label="Icon (Emoji)"
          placeholder="e.g., 🎨, 🔧, 📊"
          value={formData.icon || ''}
          onChange={(e) => setFormData({ ...formData, icon: e.currentTarget.value || undefined })}
          description="Optional emoji/icon to represent this persona"
          maxLength={10}
        />

        <ColorInput
          label="Base Color"
          placeholder="Pick a color"
          value={formData.base_color || ''}
          onChange={(value) => setFormData({ ...formData, base_color: value || undefined })}
          description="Color for message background tinting (e.g., #FFB6C1 for light pink)"
          format="hex"
          swatches={['#FFB6C1', '#ADD8E6', '#E6F3E6', '#FFF4E6', '#F3E5F5', '#E3F2FD', '#FFF9C4']}
        />

        <TextInput
          label="Role"
          placeholder="e.g., World-Class UX Engineer"
          value={formData.role}
          onChange={(e) => setFormData({ ...formData, role: e.currentTarget.value })}
        />

        <Select
          label="Backend"
          placeholder="Select LLM backend"
          data={backendOptions}
          value={formData.backend || 'claude_cli'}
          onChange={(value) =>
            setFormData({ ...formData, backend: (value as PersonaConfig['backend']) || 'claude_cli' })
          }
          allowDeselect={false}
        />

        {formData.backend === 'claude_cli' && (
          <Select
            label="Model"
            placeholder="Select Claude model"
            description="Choose which Claude model to use for this persona"
            data={CLAUDE_MODEL_OPTIONS}
            value={formData.model_name || ''}
            onChange={(value) => setFormData({ ...formData, model_name: value || undefined })}
            clearable
          />
        )}

        {formData.backend === 'gemini_api' && (
          <Select
            label="Model"
            placeholder="Select Gemini model"
            description="Choose which Gemini model to use for this persona"
            data={GEMINI_MODEL_OPTIONS}
            value={formData.model_name || 'gemini-2.5-flash'}
            onChange={(value) => setFormData({ ...formData, model_name: value || undefined })}
            clearable
          />
        )}

        <Textarea
          label="Background"
          placeholder="Describe the persona's background and expertise..."
          value={formData.background}
          onChange={(e) => setFormData({ ...formData, background: e.currentTarget.value })}
          minRows={3}
          autosize
        />

        <Textarea
          label="Communication Style"
          placeholder="Describe how this persona communicates..."
          value={formData.communication_style}
          onChange={(e) => setFormData({ ...formData, communication_style: e.currentTarget.value })}
          minRows={3}
          autosize
        />

        <Switch
          label="Default Participant"
          description="Include this persona in discussions by default"
          checked={formData.default_participant}
          onChange={(e) => setFormData({ ...formData, default_participant: e.currentTarget.checked })}
        />

        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave}>
            Save
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
};
