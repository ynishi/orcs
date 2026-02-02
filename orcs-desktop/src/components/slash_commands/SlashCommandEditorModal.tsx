import React, { useState, useEffect } from 'react';
import { Modal, TextInput, Textarea, Button, Stack, Group, Select, Checkbox } from '@mantine/core';
import { notifications } from '@mantine/notifications';
import { invoke } from '@tauri-apps/api/core';
import { SlashCommand, CommandType, PipelineConfig, PipelineStep } from '../../types/slash_command';
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
  { value: 'pipeline', label: 'Pipeline (chain multiple commands)' },
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
    pipelineConfig: undefined,
    includeInSystemPrompt: true, // Default for prompt type
  });
  const [availableCommands, setAvailableCommands] = useState<SlashCommand[]>([]);
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

  // Load available commands for Pipeline type
  useEffect(() => {
    const loadCommands = async () => {
      try {
        const commands = await invoke<SlashCommand[]>('list_slash_commands');
        // Filter out pipeline commands to avoid circular references
        setAvailableCommands(commands.filter((c) => c.type !== 'pipeline'));
      } catch (error) {
        console.error('Failed to load slash commands:', error);
      }
    };
    if (formData.type === 'pipeline') {
      loadCommands();
    }
  }, [formData.type]);

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
        pipelineConfig: command.pipelineConfig || undefined,
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
        pipelineConfig: undefined,
        includeInSystemPrompt: true,
      });
    }
  }, [command]);

  const handleSave = () => {
    const cmdType = formData.type as CommandType;
    const isPipeline = cmdType === 'pipeline';

    // Validate required fields (content not required for pipeline)
    if (!formData.name || !formData.description) {
      notifications.show({
        title: 'Validation Error',
        message: 'Name and Description are required fields',
        color: 'red',
        autoClose: 5000,
      });
      return;
    }

    if (!isPipeline && !formData.content) {
      notifications.show({
        title: 'Validation Error',
        message: 'Content is required',
        color: 'red',
        autoClose: 5000,
      });
      return;
    }

    // Validate pipeline has at least one step
    if (isPipeline && (!formData.pipelineConfig?.steps || formData.pipelineConfig.steps.length === 0)) {
      notifications.show({
        title: 'Validation Error',
        message: 'Pipeline must have at least one step',
        color: 'red',
        autoClose: 5000,
      });
      return;
    }

    // Cast to SlashCommand since we've validated required fields
    const validatedCommand: SlashCommand = {
      name: formData.name,
      icon: formData.icon || '⚡',
      description: formData.description,
      type: cmdType,
      content: isPipeline ? '' : (formData.content || ''),
      workingDir: formData.workingDir || undefined,
      argsDescription: formData.argsDescription || undefined,
      actionConfig: formData.type === 'action' ? formData.actionConfig : undefined,
      pipelineConfig: isPipeline ? formData.pipelineConfig : undefined,
      includeInSystemPrompt: formData.includeInSystemPrompt ?? getDefaultIncludeInSystemPrompt(cmdType),
    };

    onSave(validatedCommand);
  };

  const isEditing = !!command?.name;
  const isShellCommand = formData.type === 'shell';
  const isTaskCommand = formData.type === 'task';
  const isActionCommand = formData.type === 'action';
  const isPipelineCommand = formData.type === 'pipeline';

  // Pipeline step management
  const addPipelineStep = () => {
    const currentConfig = formData.pipelineConfig || { steps: [], failOnError: true, chainOutput: true };
    setFormData({
      ...formData,
      pipelineConfig: {
        ...currentConfig,
        steps: [...currentConfig.steps, { commandName: '' }],
      },
    });
  };

  const removePipelineStep = (index: number) => {
    if (!formData.pipelineConfig) return;
    const newSteps = formData.pipelineConfig.steps.filter((_, i) => i !== index);
    setFormData({
      ...formData,
      pipelineConfig: {
        ...formData.pipelineConfig,
        steps: newSteps,
      },
    });
  };

  const updatePipelineStep = (index: number, updates: Partial<PipelineStep>) => {
    if (!formData.pipelineConfig) return;
    const newSteps = formData.pipelineConfig.steps.map((step, i) =>
      i === index ? { ...step, ...updates } : step
    );
    setFormData({
      ...formData,
      pipelineConfig: {
        ...formData.pipelineConfig,
        steps: newSteps,
      },
    });
  };

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

        {!isPipelineCommand && (
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
        )}

        {isPipelineCommand && (
          <Stack gap="sm">
            <Group justify="space-between">
              <span style={{ fontWeight: 500, fontSize: '14px' }}>Pipeline Steps</span>
              <Button size="xs" variant="light" onClick={addPipelineStep}>
                + Add Step
              </Button>
            </Group>
            {formData.pipelineConfig?.steps.map((step, index) => (
              <Group key={index} gap="sm" align="flex-end">
                <Select
                  label={`Step ${index + 1}`}
                  placeholder="Select command..."
                  data={availableCommands.map((c) => ({
                    value: c.name,
                    label: `${c.icon} /${c.name}`,
                  }))}
                  value={step.commandName}
                  onChange={(value) => updatePipelineStep(index, { commandName: value || '' })}
                  style={{ flex: 1 }}
                />
                <TextInput
                  label="Args"
                  placeholder="Optional args"
                  value={step.args || ''}
                  onChange={(e) => updatePipelineStep(index, { args: e.currentTarget.value || undefined })}
                  style={{ flex: 1 }}
                />
                <Button
                  size="sm"
                  variant="subtle"
                  color="red"
                  onClick={() => removePipelineStep(index)}
                  style={{ marginBottom: 2 }}
                >
                  ✕
                </Button>
              </Group>
            ))}
            {(!formData.pipelineConfig?.steps || formData.pipelineConfig.steps.length === 0) && (
              <span style={{ color: 'var(--mantine-color-dimmed)', fontSize: '13px' }}>
                No steps added. Click "+ Add Step" to add commands to the pipeline.
              </span>
            )}
            <Group gap="lg" mt="xs">
              <Checkbox
                label="Stop on error"
                description="Stop pipeline if any step fails"
                checked={formData.pipelineConfig?.failOnError ?? true}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    pipelineConfig: {
                      steps: formData.pipelineConfig?.steps || [],
                      chainOutput: formData.pipelineConfig?.chainOutput ?? true,
                      failOnError: e.currentTarget.checked,
                    },
                  })
                }
              />
              <Checkbox
                label="Chain output"
                description="Pass output of each step as input to the next"
                checked={formData.pipelineConfig?.chainOutput ?? true}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    pipelineConfig: {
                      steps: formData.pipelineConfig?.steps || [],
                      failOnError: formData.pipelineConfig?.failOnError ?? true,
                      chainOutput: e.currentTarget.checked,
                    },
                  })
                }
              />
            </Group>
          </Stack>
        )}

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
