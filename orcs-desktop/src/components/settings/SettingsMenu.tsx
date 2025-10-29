import { Menu, ActionIcon, Text, Stack } from '@mantine/core';
import {
  IconSettings,
  IconFile,
  IconFolder,
  IconFolderOpen,
  IconInfoCircle,
} from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { notifications } from '@mantine/notifications';
import { useState, useEffect } from 'react';

/**
 * Settings menu component that provides access to configuration files and directories.
 *
 * Features:
 * - Open config file (config.toml)
 * - Open sessions directory
 * - Open workspaces directory
 * - Open personas directory
 * - Open slash commands directory
 * - Display app root directory
 */
export function SettingsMenu() {
  const [rootDir, setRootDir] = useState<string>('');

  useEffect(() => {
    const loadRootDir = async () => {
      try {
        const dir = await invoke<string>('get_root_directory');
        setRootDir(dir);
      } catch (error) {
        console.error('[Settings] Failed to load root directory:', error);
      }
    };
    loadRootDir();
  }, []);

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

  const handleOpenPersonasDir = async () => {
    try {
      const personasDir = await invoke<string>('get_personas_directory');
      console.log('[Settings] Opening personas directory:', personasDir);
      await openPath(personasDir);
    } catch (error) {
      console.error('[Settings] Failed to open personas directory:', error);
      notifications.show({
        title: 'Failed to Open Directory',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleOpenSlashCommandsDir = async () => {
    try {
      const slashCommandsDir = await invoke<string>('get_slash_commands_directory');
      console.log('[Settings] Opening slash commands directory:', slashCommandsDir);
      await openPath(slashCommandsDir);
    } catch (error) {
      console.error('[Settings] Failed to open slash commands directory:', error);
      notifications.show({
        title: 'Failed to Open Directory',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleOpenRootDir = async () => {
    if (!rootDir) {
      notifications.show({
        title: 'Root Directory Not Loaded',
        message: 'Please wait for the root directory to load',
        color: 'yellow',
      });
      return;
    }

    try {
      console.log('[Settings] Opening root directory:', rootDir);
      await openPath(rootDir);
    } catch (error) {
      console.error('[Settings] Failed to open root directory:', error);
      notifications.show({
        title: 'Failed to Open Directory',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleOpenLogsDirectory = async () => {
    try {
      const logsDir = await invoke<string>('get_logs_directory');
      console.log('[Settings] Opening logs directory:', logsDir);
      await openPath(logsDir);
    } catch (error) {
      console.error('[Settings] Failed to open logs directory:', error);
      notifications.show({
        title: 'Failed to Open Logs Directory',
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
        <Menu.Label>Application Info</Menu.Label>
        <Menu.Item
          leftSection={<IconInfoCircle size={16} />}
          onClick={handleOpenRootDir}
        >
          <Stack gap={4}>
            <Text size="xs" c="dimmed">Root Directory:</Text>
            <Text size="xs" style={{ fontFamily: 'monospace', wordBreak: 'break-all' }}>
              {rootDir || 'Loading...'}
            </Text>
          </Stack>
        </Menu.Item>

        <Menu.Divider />

        <Menu.Label>Configuration</Menu.Label>
        <Menu.Item
          leftSection={<IconFile size={16} />}
          onClick={handleOpenConfigFile}
        >
          <Text size="sm">Open Config File</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenLogsDirectory}
        >
          <Text size="sm">Open Logs Directory</Text>
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
        <Menu.Item
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenPersonasDir}
        >
          <Text size="sm">Personas Directory</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenSlashCommandsDir}
        >
          <Text size="sm">Slash Commands Directory</Text>
        </Menu.Item>
      </Menu.Dropdown>
    </Menu>
  );
}
