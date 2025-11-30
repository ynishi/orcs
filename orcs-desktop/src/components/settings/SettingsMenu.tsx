import { Menu, ActionIcon, Text, Stack, Switch } from '@mantine/core';
import {
  IconSettings,
  IconFile,
  IconFolder,
  IconFolderOpen,
  IconInfoCircle,
  IconCopy,
  IconWand,
  IconUser,
  IconBug,
} from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { notifications } from '@mantine/notifications';
import { useState, useEffect } from 'react';
import { PERSONA_CONFIG_TEMPLATE, CONFIG_TOML_TEMPLATE, SECRETS_TEMPLATE, generateConfigSessionPrompt } from '../../utils/configTemplates';
import type { PersonaConfig } from '../../types/agent';
import type { Session } from '../../types/session';
import { useDebugStore } from '../../stores/debugStore';

interface SettingsMenuProps {
  onSelectSession?: (session: Session) => void;
}

interface UserProfile {
  nickname: string;
  background: string;
}

/**
 * Settings menu component that provides access to configuration files and directories.
 *
 * Features:
 * - Display user profile (nickname)
 * - Open config file (config.toml)
 * - Open sessions directory
 * - Open workspaces directory
 * - Open personas directory
 * - Open slash commands directory
 * - Display app root directory
 * - Start Config Session (AI-assisted configuration)
 */
export function SettingsMenu({
  onSelectSession,
}: SettingsMenuProps = {}) {
  const [rootDir, setRootDir] = useState<string>('');
  const [defaultWorkspace, setDefaultWorkspace] = useState<string>('');
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);

  const { debugSettings, isLoaded, initialize, toggleDebugMode } = useDebugStore();

  useEffect(() => {
    const loadRootDir = async () => {
      try {
        const dir = await invoke<string>('get_root_pathectory');
        setRootDir(dir);
      } catch (error) {
        console.error('[Settings] Failed to load root directory:', error);
      }
    };
    loadRootDir();
  }, []);

  useEffect(() => {
    const loadDefaultWorkspace = async () => {
      try {
        const workspace = await invoke<string>('get_default_workspace_path');
        setDefaultWorkspace(workspace);
      } catch (error) {
        console.error('[Settings] Failed to load default workspace:', error);
      }
    };
    loadDefaultWorkspace();
  }, []);

  useEffect(() => {
    const loadUserProfile = async () => {
      try {
        const profile = await invoke<UserProfile>('get_user_profile');
        setUserProfile(profile);
      } catch (error) {
        console.error('[Settings] Failed to load user profile:', error);
      }
    };
    loadUserProfile();
  }, []);

  useEffect(() => {
    if (!isLoaded) {
      initialize();
    }
  }, [isLoaded, initialize]);

  const handleToggleDebugMode = async () => {
    try {
      await toggleDebugMode();
      notifications.show({
        title: debugSettings?.enableLlmDebug ? 'Debug Mode Disabled' : 'Debug Mode Enabled',
        message: debugSettings?.enableLlmDebug
          ? 'Debug mode has been disabled. Log level reset to info.'
          : 'Debug mode enabled! Menu bar will show debug indicator. Log level set to trace.',
        color: debugSettings?.enableLlmDebug ? 'gray' : 'orange',
        autoClose: 3000,
      });
    } catch (error) {
      notifications.show({
        title: 'Failed to Toggle Debug Mode',
        message: String(error),
        color: 'red',
      });
    }
  };

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

  const handleOpenWorkspacesRepositoryDir = async () => {
    try {
      const workspacesDir = await invoke<string>('get_workspaces_repository_directory');
      console.log('[Settings] Opening workspaces repository directory:', workspacesDir);
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

  const handleOpenTasksDir = async () => {
    try {
      const tasksDir = await invoke<string>('get_tasks_directory');
      console.log('[Settings] Opening tasks directory:', tasksDir);
      await openPath(tasksDir);
    } catch (error) {
      console.error('[Settings] Failed to open tasks directory:', error);
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

  const handleOpenDefaultWorkspace = async () => {
    if (!defaultWorkspace) {
      notifications.show({
        title: 'Default Workspace Not Loaded',
        message: 'Please wait for the default workspace to load',
        color: 'yellow',
      });
      return;
    }

    try {
      console.log('[Settings] Opening default workspace:', defaultWorkspace);
      await openPath(defaultWorkspace);
    } catch (error) {
      console.error('[Settings] Failed to open default workspace:', error);
      notifications.show({
        title: 'Failed to Open Default Workspace',
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

  const handleOpenSecretFile = async () => {
    try {
      const secretPath = await invoke<string>('get_secret_path');
      console.log('[Settings] Opening secret file:', secretPath);
      await openPath(secretPath);
    } catch (error) {
      console.error('[Settings] Failed to open secret file:', error);
      notifications.show({
        title: 'Failed to Open Secret File',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleCopyPersonaTemplate = async () => {
    try {
      await navigator.clipboard.writeText(PERSONA_CONFIG_TEMPLATE);
      notifications.show({
        title: 'Template Copied!',
        message: 'Persona configuration guide copied to clipboard. Paste it into a chat session for help!',
        color: 'green',
        autoClose: 5000,
      });
    } catch (error) {
      console.error('[Settings] Failed to copy template:', error);
      notifications.show({
        title: 'Failed to Copy Template',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleCopyConfigTemplate = async () => {
    try {
      await navigator.clipboard.writeText(CONFIG_TOML_TEMPLATE);
      notifications.show({
        title: 'Template Copied!',
        message: 'Config template copied to clipboard',
        color: 'green',
        autoClose: 3000,
      });
    } catch (error) {
      console.error('[Settings] Failed to copy template:', error);
      notifications.show({
        title: 'Failed to Copy Template',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleCopySecretsTemplate = async () => {
    try {
      await navigator.clipboard.writeText(SECRETS_TEMPLATE);
      notifications.show({
        title: 'Template Copied!',
        message: 'Secrets template copied to clipboard',
        color: 'green',
        autoClose: 3000,
      });
    } catch (error) {
      console.error('[Settings] Failed to copy template:', error);
      notifications.show({
        title: 'Failed to Copy Template',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleStartConfigSession = async () => {
    if (!onSelectSession) {
      notifications.show({
        title: 'Feature Not Available',
        message: 'Config Session feature is not initialized',
        color: 'yellow',
      });
      return;
    }

    try {
      // 1. Get ORCS config directory (will use as Admin Workspace root)
      const orcsConfigDir = await invoke<string>('get_config_path');
      const orcsDir = orcsConfigDir.replace('/config.toml', '');

      // 2. Gather configuration information
      const personas = await invoke<PersonaConfig[]>('get_personas');
      const configPath = await invoke<string>('get_config_path');
      const personasDir = await invoke<string>('get_personas_directory');
      const slashCommandsDir = await invoke<string>('get_slash_commands_directory');
      const secretsPath = await invoke<string>('get_secret_path');

      // 3. Generate Config Session system prompt
      const systemPrompt = generateConfigSessionPrompt({
        personas,
        configPath,
        personasDir,
        slashCommandsDir,
        secretsPath,
        orcsConfigDir: orcsDir,
      });

      // 4. Create Config Session (backend handles everything)
      console.log('[Settings] Creating config session at:', orcsDir);
      const session = await invoke<Session>('create_config_session', {
        workspaceRootPath: orcsDir,
        systemPrompt,
      });

      console.log('[Settings] Config session created:', session);

      // 5. Select the session
      onSelectSession(session);

      notifications.show({
        title: '🛠️ Config Session Started',
        message: 'Ask me anything about ORCS configuration!',
        color: 'blue',
        autoClose: 5000,
      });

    } catch (error) {
      console.error('[Settings] Failed to start config session:', error);
      notifications.show({
        title: 'Failed to Start Config Session',
        message: String(error),
        color: 'red',
        autoClose: 10000,
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

      <Menu.Dropdown style={{ maxHeight: '80vh', overflowY: 'auto' }}>
        <Menu.Label>User Profile</Menu.Label>
        <Menu.Item
          leftSection={<IconUser size={16} />}
          closeMenuOnClick={false}
        >
          <Stack gap={4}>
            <Text size="sm" fw={600}>{userProfile?.nickname || 'Loading...'}</Text>
            {userProfile?.background && (
              <Text size="xs" c="dimmed" lineClamp={2}>
                {userProfile.background}
              </Text>
            )}
          </Stack>
        </Menu.Item>

        <Menu.Divider />

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
        <Menu.Item
          leftSection={<IconFolderOpen size={16} />}
          onClick={handleOpenDefaultWorkspace}
        >
          <Stack gap={4}>
            <Text size="xs" c="dimmed">Default Workspace:</Text>
            <Text size="xs" style={{ fontFamily: 'monospace', wordBreak: 'break-all' }}>
              {defaultWorkspace || 'Loading...'}
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
          leftSection={<IconFile size={16} />}
          onClick={handleOpenSecretFile}
        >
          <Text size="sm">Open Secret File</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenLogsDirectory}
        >
          <Text size="sm">Open Logs Directory</Text>
        </Menu.Item>

        <Menu.Divider />

        <Menu.Label>AI Assistant</Menu.Label>
        <Menu.Item
          leftSection={<IconWand size={16} />}
          onClick={handleStartConfigSession}
        >
          <Text size="sm">Start Config Session</Text>
        </Menu.Item>

        <Menu.Divider />

        <Menu.Label>Config Templates</Menu.Label>
        <Menu.Item
          leftSection={<IconCopy size={16} />}
          onClick={handleCopyPersonaTemplate}
        >
          <Text size="sm">Copy Persona Guide</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconCopy size={16} />}
          onClick={handleCopyConfigTemplate}
        >
          <Text size="sm">Copy Config Template</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconCopy size={16} />}
          onClick={handleCopySecretsTemplate}
        >
          <Text size="sm">Copy Secrets Template</Text>
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
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenWorkspacesRepositoryDir}
        >
          <Text size="sm">Workspaces Setting Directory</Text>
        </Menu.Item>
        <Menu.Item
          leftSection={<IconFolderOpen size={16} />}
          onClick={handleOpenWorkspacesDir}
        >
          <Text size="sm">Workspaces Data Directory</Text>
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
        <Menu.Item
          leftSection={<IconFolder size={16} />}
          onClick={handleOpenTasksDir}
        >
          <Text size="sm">Tasks Directory</Text>
        </Menu.Item>

        <Menu.Divider />

        <Menu.Label>Debug Settings</Menu.Label>
        <Menu.Item
          leftSection={<IconBug size={16} />}
          closeMenuOnClick={false}
        >
          <Stack gap={4}>
            <Switch
              label="Debug Mode"
              description="Show LLM prompts/responses & trace logs"
              checked={debugSettings?.enableLlmDebug ?? false}
              onChange={handleToggleDebugMode}
              disabled={!isLoaded}
              color="orange"
            />
          </Stack>
        </Menu.Item>
      </Menu.Dropdown>
    </Menu>
  );
}
