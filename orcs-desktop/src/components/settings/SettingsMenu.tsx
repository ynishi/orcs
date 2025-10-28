import { Menu, ActionIcon, Text } from '@mantine/core';
import {
  IconSettings,
  IconFile,
  IconFolder,
  IconFolderOpen,
} from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { notifications } from '@mantine/notifications';

/**
 * Settings menu component that provides access to configuration files and directories.
 *
 * Features:
 * - Open config file (config.toml)
 * - Open sessions directory
 * - Open workspaces directory
 */
export function SettingsMenu() {
  const handleOpenConfigFile = async () => {
    try {
      const configPath = await invoke<string>('get_config_path');
      console.log('[Settings] Opening config file:', configPath);
      await openPath(configPath);
    } catch (error) {
      console.error('[Settings] Failed to open config file:', error);
      notifications.show({
        title: 'Failed to Open Config',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleOpenSessionsDir = async () => {
    try {
      const sessionsDir = await invoke<string>('get_sessions_directory');
      console.log('[Settings] Opening sessions directory:', sessionsDir);
      await openPath(sessionsDir);
    } catch (error) {
      console.error('[Settings] Failed to open sessions directory:', error);
      notifications.show({
        title: 'Failed to Open Directory',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleOpenWorkspacesDir = async () => {
    try {
      const workspacesDir = await invoke<string>('get_workspaces_directory');
      console.log('[Settings] Opening workspaces directory:', workspacesDir);
      await openPath(workspacesDir);
    } catch (error) {
      console.error('[Settings] Failed to open workspaces directory:', error);
      notifications.show({
        title: 'Failed to Open Directory',
        message: String(error),
        color: 'red',
      });
    }
  };

  return (
    <Menu shadow="md" width={250}>
      <Menu.Target>
        <ActionIcon variant="subtle" color="gray" size="lg">
          <IconSettings size={20} />
        </ActionIcon>
      </Menu.Target>

      <Menu.Dropdown>
        <Menu.Label>Configuration</Menu.Label>
        <Menu.Item
          leftSection={<IconFile size={16} />}
          onClick={handleOpenConfigFile}
        >
          <Text size="sm">Open Config File</Text>
        </Menu.Item>

        <Menu.Divider />

        <Menu.Label>Data Directories</Menu.Label>
        <Menu.Item
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenSessionsDir}
        >
          <Text size="sm">Sessions Directory</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconFolderOpen size={16} />}
          onClick={handleOpenWorkspacesDir}
        >
          <Text size="sm">Workspaces Directory</Text>
        </Menu.Item>
      </Menu.Dropdown>
    </Menu>
  );
}
