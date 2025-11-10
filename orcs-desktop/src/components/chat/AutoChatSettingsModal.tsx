import React, { useState, useEffect } from 'react';
import { Modal, NumberInput, Select, Switch, Button, Stack, Text, Divider, Group } from '@mantine/core';
import type { AutoChatConfig, StopCondition } from '../../types/session';

interface AutoChatSettingsModalProps {
  opened: boolean;
  onClose: () => void;
  config: AutoChatConfig | null;
  onSave: (config: AutoChatConfig) => void;
}

const STOP_CONDITION_OPTIONS = [
  { value: 'iteration_count', label: 'Stop after N iterations' },
  { value: 'user_interrupt', label: 'Continue until stopped manually' },
];

const DEFAULT_CONFIG: AutoChatConfig = {
  max_iterations: 5,
  stop_condition: 'iteration_count',
  web_search_enabled: true,
};

export const AutoChatSettingsModal: React.FC<AutoChatSettingsModalProps> = ({
  opened,
  onClose,
  config,
  onSave,
}) => {
  const [formData, setFormData] = useState<AutoChatConfig>(DEFAULT_CONFIG);

  // Update form data when config prop changes
  useEffect(() => {
    if (config) {
      setFormData(config);
    } else {
      setFormData(DEFAULT_CONFIG);
    }
  }, [config, opened]);

  const handleSave = () => {
    onSave(formData);
    onClose();
  };

  const handleCancel = () => {
    // Reset to original config
    if (config) {
      setFormData(config);
    } else {
      setFormData(DEFAULT_CONFIG);
    }
    onClose();
  };

  return (
    <Modal
      opened={opened}
      onClose={handleCancel}
      title="AutoChat Settings"
      size="md"
    >
      <Stack gap="md">
        <NumberInput
          label="Max Iterations"
          description="Maximum rounds of automatic dialogue"
          min={1}
          max={20}
          value={formData.max_iterations}
          onChange={(value) =>
            setFormData({ ...formData, max_iterations: value as number })
          }
          required
        />

        <Select
          label="Stop Condition"
          description="When to stop the automatic dialogue"
          data={STOP_CONDITION_OPTIONS}
          value={formData.stop_condition}
          onChange={(value) =>
            setFormData({ ...formData, stop_condition: value as StopCondition })
          }
          required
        />

        <Switch
          label="Enable WebSearch"
          description="Agents can search the web during discussion"
          checked={formData.web_search_enabled}
          onChange={(event) =>
            setFormData({ ...formData, web_search_enabled: event.currentTarget.checked })
          }
        />

        <Divider label="Inherited Settings" />

        <Text size="sm" c="dimmed">
          The following settings are inherited from the current session:
        </Text>
        <Text size="sm" c="dimmed" pl="md">
          • <strong>Execution Strategy</strong> (broadcast/sequential/mentioned)
        </Text>
        <Text size="sm" c="dimmed" pl="md">
          • <strong>Talk Style</strong> (brainstorm/debate/etc.)
        </Text>
        <Text size="sm" c="dimmed" pl="md">
          • <strong>Active Participants</strong> (which agents participate)
        </Text>
        <Text size="sm" c="dimmed">
          Change these in the StatusBar settings if needed.
        </Text>

        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={handleCancel}>
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
