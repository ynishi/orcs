import React, { useState, useEffect } from 'react';
import { Modal, TextInput, Textarea, Switch, Button, Stack, Group } from '@mantine/core';
import { PersonaConfig } from '../../types/agent';

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
  const [formData, setFormData] = useState<Partial<PersonaConfig>>({
    id: '',
    name: '',
    role: '',
    background: '',
    communication_style: '',
    default_participant: false,
  });

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
      });
    } else {
      setFormData({
        id: '',
        name: '',
        role: '',
        background: '',
        communication_style: '',
        default_participant: false,
      });
    }
  }, [persona]);

  const handleSave = () => {
    // Validate required fields
    if (!formData.id || !formData.name) {
      alert('ID and Name are required fields');
      return;
    }

    // Cast to PersonaConfig since we've validated required fields
    const validatedPersona: PersonaConfig = {
      id: formData.id,
      name: formData.name,
      role: formData.role || '',
      background: formData.background || '',
      communication_style: formData.communication_style || '',
      default_participant: formData.default_participant || false,
    };

    onSave(validatedPersona);
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
        <TextInput
          label="ID"
          placeholder="unique-persona-id"
          value={formData.id}
          onChange={(e) => setFormData({ ...formData, id: e.currentTarget.value })}
          disabled={isEditing}
          required
        />

        <TextInput
          label="Name"
          placeholder="Persona Name"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.currentTarget.value })}
          required
        />

        <TextInput
          label="Role"
          placeholder="e.g., World-Class UX Engineer"
          value={formData.role}
          onChange={(e) => setFormData({ ...formData, role: e.currentTarget.value })}
        />

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
