import React, { useState, useEffect } from 'react';
import { Modal, TextInput, Textarea, Button, Stack, Group, Select, Checkbox } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import { SlashCommand, CommandType } from '../../types/slash_command';
import type { PersonaConfig } from '../../types/agent';

/** Get default value for includeInSystemPrompt based on command type */
function getDefaultIncludeInSystemPrompt(type: CommandType): boolean {
  return type !== 'task'; // Task defaults to false, all others default to true
}

const COMMAND_TYPE_OPTIONS = [
  { value: 'prompt', label: 'Prompt (template expansion)' },
  { value: 'shell', label: 'Shell (command execution)' },
  { value: 'task', label: 'Task (orchestrated workflow)' },
  { value: 'action', label: 'Action (AI execution with session context)' },
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
    argsDescription: undefined,
    actionConfig: undefined,
    includeInSystemPrompt: true, // Default for prompt type
  });
  const [personas, setPersonas] = useState<PersonaConfig[]>([]);

  // Load personas for Action type
  useEffect(() => {
    const loadPersonas = async () => {
      try {
        const loadedPersonas = await invoke<PersonaConfig[]>('get_personas');
        setPersonas(loadedPersonas);
      } catch (error) {
        console.error('Failed to load personas:', error);
      }
    };
    loadPersonas();
  }, []);

  // Update form data when command prop changes
  useEffect(() => {
    if (command) {
      const cmdType = command.type || 'prompt';
      setFormData({
        name: command.name || '',
        icon: command.icon || '⚡',
        description: command.description || '',
        type: cmdType,
        content: command.content || '',
        workingDir: command.workingDir || undefined,
        argsDescription: command.argsDescription || undefined,
        actionConfig: command.actionConfig || undefined,
        includeInSystemPrompt: command.includeInSystemPrompt ?? getDefaultIncludeInSystemPrompt(cmdType),
      });
    } else {
      setFormData({
        name: '',
        icon: '⚡',
        description: '',
        type: 'prompt',
        content: '',
        workingDir: undefined,
        argsDescription: undefined,
        actionConfig: undefined,
        includeInSystemPrompt: true,
      });
    }
  }, [command]);

  const handleSave = () => {
    // Validate required fields
    if (!formData.name || !formData.description || !formData.content) {
      alert('Name, Description, and Content are required fields');
      return;
    }

    const cmdType = formData.type as CommandType;

    // Cast to SlashCommand since we've validated required fields
    const validatedCommand: SlashCommand = {
      name: formData.name,
      icon: formData.icon || '⚡',
      description: formData.description,
      type: cmdType,
      content: formData.content,
      workingDir: formData.workingDir || undefined,
      argsDescription: formData.argsDescription || undefined,
      actionConfig: formData.type === 'action' ? formData.actionConfig : undefined,
      includeInSystemPrompt: formData.includeInSystemPrompt ?? getDefaultIncludeInSystemPrompt(cmdType),
    };

    onSave(validatedCommand);
  };

  const isEditing = !!command?.name;
  const isShellCommand = formData.type === 'shell';
  const isTaskCommand = formData.type === 'task';
  const isActionCommand = formData.type === 'action';

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
          description="Prompt: expands templates. Shell: executes commands. Task: runs orchestrated workflows."
          data={COMMAND_TYPE_OPTIONS}
          value={formData.type}
          onChange={(value) => {
            const newType = value as CommandType;
            // When type changes, update includeInSystemPrompt to the new default
            // only if the user hasn't explicitly set it (i.e., it's currently at the old default)
            const currentDefault = getDefaultIncludeInSystemPrompt(formData.type as CommandType);
            const newDefault = getDefaultIncludeInSystemPrompt(newType);
            const shouldUpdateInclude = formData.includeInSystemPrompt === currentDefault;

            setFormData({
              ...formData,
              type: newType,
              includeInSystemPrompt: shouldUpdateInclude ? newDefault : formData.includeInSystemPrompt,
            });
          }}
          allowDeselect={false}
          required
        />

        <Textarea
          label={
            isShellCommand
              ? 'Shell Command'
              : isTaskCommand
                ? 'Task Description'
                : isActionCommand
                  ? 'Action Prompt Template'
                  : 'Prompt Template'
          }
          placeholder={
            isShellCommand
              ? 'git status'
              : isTaskCommand
                ? 'Create a comprehensive README file with project overview, setup instructions, and usage examples'
                : isActionCommand
                  ? '{session_all}\n\nSummarize the above conversation in 3 bullet points.'
                  : 'Analyze the files in {workspace}:\n{files}\n\nCurrent branch: {git_branch}'
          }
          description={
            isShellCommand
              ? 'Command to execute. Variables: {workspace}, {workspace_path}, {files}, {git_branch}, {git_status}, {args}'
              : isTaskCommand
                ? 'Task description that will be executed using ParallelOrchestrator. Can use {args} for runtime arguments.'
                : isActionCommand
                  ? 'Prompt template sent to AI. Variables: {session_all}, {session_recent}, {workspace}, {workspace_path}, {files}, {git_branch}, {git_status}, {args}'
                  : 'Prompt template. Variables: {workspace}, {workspace_path}, {files}, {git_branch}, {git_status}, {args}'
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

        {isActionCommand && (
          <Select
            label="Persona (Optional)"
            placeholder="Select a persona..."
            description="Use a persona's background, style, and backend settings for this action."
            data={[
              { value: '__none__', label: '(None - use default)' },
              ...personas.map((p) => ({
                value: p.id,
                label: `${p.icon || '👤'} ${p.name} - ${p.role}`,
              })),
            ]}
            value={formData.actionConfig?.personaId || '__none__'}
            onChange={(value) =>
              setFormData({
                ...formData,
                actionConfig: value === '__none__'
                  ? undefined
                  : { ...formData.actionConfig, personaId: value || undefined },
              })
            }
            clearable={false}
          />
        )}

        <Textarea
          label="Argument Description (Optional)"
          placeholder="Describe how to provide arguments (e.g., 'Pass file path')"
          description="Shown when running the command to remind you what {args} should contain."
          value={formData.argsDescription || ''}
          onChange={(e) => setFormData({ ...formData, argsDescription: e.currentTarget.value || undefined })}
          minRows={2}
          autosize
        />

        <Checkbox
          label="Include in system prompt"
          description={`When enabled, this command will be visible to AI agents. ${isTaskCommand ? '(Task commands default to disabled)' : ''}`}
          checked={formData.includeInSystemPrompt ?? getDefaultIncludeInSystemPrompt(formData.type as CommandType)}
          onChange={(e) => setFormData({ ...formData, includeInSystemPrompt: e.currentTarget.checked })}
        />

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
