import React, { useState, useEffect } from 'react';
import { Modal, TextInput, Textarea, Button, Stack, Group, Select } from '@mantine/core';
import { SlashCommand, CommandType } from '../../types/slash_command';

const COMMAND_TYPE_OPTIONS = [
  { value: 'prompt', label: 'Prompt (template expansion)' },
  { value: 'shell', label: 'Shell (command execution)' },
];

interface SlashCommandEditorModalProps {
  opened: boolean;
  onClose: () => void;
  command: Partial<SlashCommand> | null;
  onSave: (command: SlashCommand) => void;
}

export const SlashCommandEditorModal: React.FC<SlashCommandEditorModalProps> = ({
  opened,
  onClose,
  command,
  onSave,
}) => {
  const [formData, setFormData] = useState<Partial<SlashCommand>>({
    name: '',
    icon: '⚡',
    description: '',
    type: 'prompt',
    content: '',
    workingDir: undefined,
  });

  // Update form data when command prop changes
  useEffect(() => {
    if (command) {
      setFormData({
        name: command.name || '',
        icon: command.icon || '⚡',
        description: command.description || '',
        type: command.type || 'prompt',
        content: command.content || '',
        workingDir: command.workingDir || undefined,
      });
    } else {
      setFormData({
        name: '',
        icon: '⚡',
        description: '',
        type: 'prompt',
        content: '',
        workingDir: undefined,
      });
    }
  }, [command]);

  const handleSave = () => {
    // Validate required fields
    if (!formData.name || !formData.description || !formData.content) {
      alert('Name, Description, and Content are required fields');
      return;
    }

    // Cast to SlashCommand since we've validated required fields
    const validatedCommand: SlashCommand = {
      name: formData.name,
      icon: formData.icon || '⚡',
      description: formData.description,
      type: formData.type as CommandType,
      content: formData.content,
      workingDir: formData.workingDir || undefined,
    };

    onSave(validatedCommand);
  };

  const isEditing = !!command?.name;
  const isShellCommand = formData.type === 'shell';

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={isEditing ? 'Edit Slash Command' : 'Create New Slash Command'}
      size="lg"
    >
      <Stack gap="md">
        <TextInput
          label="Command Name"
          placeholder="my-command"
          description="Used as /command-name in chat (no spaces, lowercase recommended)"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.currentTarget.value })}
          disabled={isEditing}
          required
        />

        <TextInput
          label="Icon"
          placeholder="⚡"
          description="Emoji to display in the UI"
          value={formData.icon}
          onChange={(e) => setFormData({ ...formData, icon: e.currentTarget.value })}
          maxLength={2}
        />

        <TextInput
          label="Description"
          placeholder="Brief description of what this command does"
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.currentTarget.value })}
          required
        />

        <Select
          label="Command Type"
          description="Prompt: expands templates with variables. Shell: executes commands."
          data={COMMAND_TYPE_OPTIONS}
          value={formData.type}
          onChange={(value) => setFormData({ ...formData, type: value as CommandType })}
          allowDeselect={false}
          required
        />

        <Textarea
          label={isShellCommand ? 'Shell Command' : 'Prompt Template'}
          placeholder={
            isShellCommand
              ? 'git status'
              : 'Analyze the files in {workspace}:\n{files}\n\nCurrent branch: {git_branch}'
          }
          description={
            isShellCommand
              ? 'Command to execute. Variables: {workspace}, {workspace_path}, {files}, {git_branch}, {git_status}'
              : 'Prompt template. Variables: {workspace}, {workspace_path}, {files}, {git_branch}, {git_status}'
          }
          value={formData.content}
          onChange={(e) => setFormData({ ...formData, content: e.currentTarget.value })}
          minRows={6}
          autosize
          required
        />

        {isShellCommand && (
          <TextInput
            label="Working Directory (Optional)"
            placeholder="{workspace_path}"
            description="Directory to execute command in. Supports {workspace_path} variable."
            value={formData.workingDir || ''}
            onChange={(e) => setFormData({ ...formData, workingDir: e.currentTarget.value || undefined })}
          />
        )}

        <Group justify="flex-end" gap="sm">
          <Button variant="subtle" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave}>
            {isEditing ? 'Save Changes' : 'Create Command'}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
};
