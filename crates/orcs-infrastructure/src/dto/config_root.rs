//! ConfigRoot DTOs and migrations
//!
//! This module defines versioned DTOs for the root configuration file (config.toml).
//! The configuration structure has evolved to separate concerns:
//!
//! - V1.0.0: Initial version with personas, user_profile, workspaces
//! - V1.1.0: Added app_state field
//! - V2.0.0: Simplified to only user_profile (personas/workspaces/app_state now managed separately)

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, MigratesTo, Versioned};

use super::{AppStateDTO, UserProfileDTO, WorkspaceV1};
use orcs_core::config::{
    ClaudeModelConfig, DebugSettings, EnvSettings, GeminiModelConfig, MemorySyncSettings,
    ModelSettings, OpenAIModelConfig, RootConfig, TerminalSettings,
};

// ============================================================================
// ModelSettings DTOs
// ============================================================================

/// DTO for ModelSettings.
///
/// This is a simple passthrough DTO since ModelSettings is already well-structured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettingsDTO {
    #[serde(default)]
    pub claude: Option<ClaudeModelConfigDTO>,
    #[serde(default)]
    pub gemini: Option<GeminiModelConfigDTO>,
    #[serde(default)]
    pub openai: Option<OpenAIModelConfigDTO>,
}

impl Default for ModelSettingsDTO {
    fn default() -> Self {
        Self {
            claude: Some(ClaudeModelConfigDTO::default()),
            gemini: Some(GeminiModelConfigDTO::default()),
            openai: Some(OpenAIModelConfigDTO::default()),
        }
    }
}

impl ModelSettingsDTO {
    fn into_domain(self) -> ModelSettings {
        ModelSettings {
            claude: self.claude.map(|c| c.into_domain()),
            gemini: self.gemini.map(|g| g.into_domain()),
            openai: self.openai.map(|o| o.into_domain()),
        }
    }

    fn from_domain(settings: ModelSettings) -> Self {
        Self {
            claude: settings.claude.map(ClaudeModelConfigDTO::from_domain),
            gemini: settings.gemini.map(GeminiModelConfigDTO::from_domain),
            openai: settings.openai.map(OpenAIModelConfigDTO::from_domain),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeModelConfigDTO {
    pub model_name: String,
}

impl Default for ClaudeModelConfigDTO {
    fn default() -> Self {
        Self {
            model_name: "claude-sonnet-4-6".to_string(),
        }
    }
}

impl ClaudeModelConfigDTO {
    fn into_domain(self) -> ClaudeModelConfig {
        ClaudeModelConfig {
            model_name: self.model_name,
        }
    }

    fn from_domain(config: ClaudeModelConfig) -> Self {
        Self {
            model_name: config.model_name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiModelConfigDTO {
    pub model_name: String,
}

impl Default for GeminiModelConfigDTO {
    fn default() -> Self {
        Self {
            model_name: "gemini-2.5-flash".to_string(),
        }
    }
}

impl GeminiModelConfigDTO {
    fn into_domain(self) -> GeminiModelConfig {
        GeminiModelConfig {
            model_name: self.model_name,
        }
    }

    fn from_domain(config: GeminiModelConfig) -> Self {
        Self {
            model_name: config.model_name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModelConfigDTO {
    pub model_name: String,
}

impl Default for OpenAIModelConfigDTO {
    fn default() -> Self {
        Self {
            model_name: "gpt-5".to_string(),
        }
    }
}

impl OpenAIModelConfigDTO {
    fn into_domain(self) -> OpenAIModelConfig {
        OpenAIModelConfig {
            model_name: self.model_name,
        }
    }

    fn from_domain(config: OpenAIModelConfig) -> Self {
        Self {
            model_name: config.model_name,
        }
    }
}

// ============================================================================
// EnvSettings DTOs
// ============================================================================

/// DTO for EnvSettings.
///
/// This is a simple passthrough DTO since EnvSettings is already well-structured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSettingsDTO {
    #[serde(default)]
    pub additional_paths: Vec<String>,
    #[serde(default = "default_auto_detect_tool_managers")]
    pub auto_detect_tool_managers: bool,
}

fn default_auto_detect_tool_managers() -> bool {
    true
}

impl Default for EnvSettingsDTO {
    fn default() -> Self {
        Self {
            additional_paths: Vec::new(),
            auto_detect_tool_managers: true,
        }
    }
}

impl EnvSettingsDTO {
    fn into_domain(self) -> EnvSettings {
        EnvSettings {
            additional_paths: self.additional_paths,
            auto_detect_tool_managers: self.auto_detect_tool_managers,
        }
    }

    fn from_domain(settings: EnvSettings) -> Self {
        Self {
            additional_paths: settings.additional_paths,
            auto_detect_tool_managers: settings.auto_detect_tool_managers,
        }
    }
}

// ============================================================================
// DebugSettings DTOs
// ============================================================================

/// DTO for DebugSettings.
///
/// This is a simple passthrough DTO since DebugSettings is already well-structured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSettingsDTO {
    #[serde(default)]
    pub enable_llm_debug: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_level")]
    pub memory_sync_log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for DebugSettingsDTO {
    fn default() -> Self {
        Self {
            enable_llm_debug: false,
            log_level: "info".to_string(),
            memory_sync_log_level: "info".to_string(),
        }
    }
}

impl DebugSettingsDTO {
    fn into_domain(self) -> DebugSettings {
        DebugSettings {
            enable_llm_debug: self.enable_llm_debug,
            log_level: self.log_level,
            memory_sync_log_level: self.memory_sync_log_level,
        }
    }

    fn from_domain(settings: DebugSettings) -> Self {
        Self {
            enable_llm_debug: settings.enable_llm_debug,
            log_level: settings.log_level,
            memory_sync_log_level: settings.memory_sync_log_level,
        }
    }
}

// ============================================================================
// MemorySyncSettings DTOs
// ============================================================================

/// DTO for MemorySyncSettings.
///
/// Controls background synchronization of session messages to external memory stores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySyncSettingsDTO {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_memory_sync_interval")]
    pub interval_secs: u64,
}

fn default_memory_sync_interval() -> u64 {
    60
}

impl Default for MemorySyncSettingsDTO {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 60,
        }
    }
}

impl MemorySyncSettingsDTO {
    fn into_domain(self) -> MemorySyncSettings {
        MemorySyncSettings {
            enabled: self.enabled,
            interval_secs: self.interval_secs,
        }
    }

    fn from_domain(settings: MemorySyncSettings) -> Self {
        Self {
            enabled: settings.enabled,
            interval_secs: settings.interval_secs,
        }
    }
}

// ============================================================================
// TerminalSettings DTOs
// ============================================================================

/// DTO for TerminalSettings.
///
/// Controls terminal application used when opening terminal from workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalSettingsDTO {
    /// Custom terminal application name (macOS) or command (Linux/Windows).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_app: Option<String>,
}

impl TerminalSettingsDTO {
    fn into_domain(self) -> TerminalSettings {
        TerminalSettings {
            custom_app: self.custom_app,
        }
    }

    fn from_domain(settings: TerminalSettings) -> Self {
        Self {
            custom_app: settings.custom_app,
        }
    }
}

// ============================================================================
// ConfigRoot DTOs
// ============================================================================

/// Root configuration structure V1.0.0 for the application config file (legacy).
///
/// This version contained personas, workspaces, and user_profile in a single file.
/// Now deprecated in favor of separate storage:
/// - Personas: DataDir/personas/
/// - Workspaces: DataDir/content/workspaces/
/// - AppState: PrefDir/state.toml
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
#[derive(Default)]
pub struct ConfigRootV1_0_0 {
    /// Persona configurations (each has its own version field).
    /// Stored as serde_json::Value (intermediate format) to allow version-migrate to handle migration.
    #[serde(rename = "persona", default)]
    pub personas: Vec<serde_json::Value>,

    /// User profile configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_profile: Option<UserProfileDTO>,

    /// Workspace configurations (each has its own version field).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<WorkspaceV1>,
}

/// Root configuration structure V1.1.0 for the application config file (legacy).
///
/// Added app_state field (now moved to separate state.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
#[derive(Default)]
pub struct ConfigRootV1_1_0 {
    /// Persona configurations (each has its own version field).
    /// Stored as serde_json::Value (intermediate format) to allow version-migrate to handle migration.
    #[serde(rename = "persona", default)]
    pub personas: Vec<serde_json::Value>,

    /// User profile configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_profile: Option<UserProfileDTO>,

    /// Application state configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_state: Option<AppStateDTO>,

    /// Workspace configurations (each has its own version field).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<WorkspaceV1>,
}

/// Root configuration structure V2.0.0 for the application config file (legacy).
///
/// Simplified to only contain user_profile.
/// Other data now managed separately:
/// - Personas: DataDir/personas/ (AsyncDirPersonaRepository)
/// - Workspaces: DataDir/content/workspaces/ (AsyncDirRepository)
/// - AppState: PrefDir/state.toml (separate file for frequent updates)
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.0.0")]
#[derive(Default)]
pub struct ConfigRootV2_0_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
}

/// Root configuration structure V2.1.0 for the application config file.
///
/// Added model_settings field to separate model configuration from secrets.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.1.0")]
#[derive(Default)]
pub struct ConfigRootV2_1_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
    /// LLM model settings (non-sensitive configuration).
    #[serde(default)]
    pub model_settings: ModelSettingsDTO,
}

/// Root configuration structure V2.2.0 for the application config file.
///
/// Added env_settings field to configure PATH for CLI-based agents.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.2.0")]
#[derive(Default)]
pub struct ConfigRootV2_2_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
    /// LLM model settings (non-sensitive configuration).
    #[serde(default)]
    pub model_settings: ModelSettingsDTO,
    /// Environment PATH configuration for CLI tools.
    #[serde(default)]
    pub env_settings: EnvSettingsDTO,
}

/// Root configuration structure V2.3.0 for the application config file.
///
/// Added debug_settings field to enable LLM debug logging.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.3.0")]
#[derive(Default)]
pub struct ConfigRootV2_3_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
    /// LLM model settings (non-sensitive configuration).
    #[serde(default)]
    pub model_settings: ModelSettingsDTO,
    /// Environment PATH configuration for CLI tools.
    #[serde(default)]
    pub env_settings: EnvSettingsDTO,
    /// Debug settings for LLM interactions.
    #[serde(default)]
    pub debug_settings: DebugSettingsDTO,
}

/// Root configuration structure V2.4.0 for the application config file.
///
/// Added memory_sync_settings field for RAG integration.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.4.0")]
#[derive(Default)]
pub struct ConfigRootV2_4_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
    /// LLM model settings (non-sensitive configuration).
    #[serde(default)]
    pub model_settings: ModelSettingsDTO,
    /// Environment PATH configuration for CLI tools.
    #[serde(default)]
    pub env_settings: EnvSettingsDTO,
    /// Debug settings for LLM interactions.
    #[serde(default)]
    pub debug_settings: DebugSettingsDTO,
    /// Memory synchronization settings for RAG integration.
    #[serde(default)]
    pub memory_sync_settings: MemorySyncSettingsDTO,
}

/// Root configuration structure V2.5.0 for the application config file (current).
///
/// Added terminal_settings field for custom terminal application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "2.5.0")]
#[derive(Default)]
pub struct ConfigRootV2_5_0 {
    /// User profile configuration (name, background, etc.).
    #[serde(default)]
    pub user_profile: UserProfileDTO,
    /// LLM model settings (non-sensitive configuration).
    #[serde(default)]
    pub model_settings: ModelSettingsDTO,
    /// Environment PATH configuration for CLI tools.
    #[serde(default)]
    pub env_settings: EnvSettingsDTO,
    /// Debug settings for LLM interactions.
    #[serde(default)]
    pub debug_settings: DebugSettingsDTO,
    /// Memory synchronization settings for RAG integration.
    #[serde(default)]
    pub memory_sync_settings: MemorySyncSettingsDTO,
    /// Terminal settings for workspace terminal launch.
    #[serde(default)]
    pub terminal_settings: TerminalSettingsDTO,
}

/// Type alias for the latest ConfigRoot version.
pub type ConfigRoot = ConfigRootV2_5_0;

// ============================================================================
// Default implementations
// ============================================================================

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from ConfigRootV1_0_0 to ConfigRootV1_1_0.
/// Adds app_state field with default value.
impl MigratesTo<ConfigRootV1_1_0> for ConfigRootV1_0_0 {
    fn migrate(self) -> ConfigRootV1_1_0 {
        ConfigRootV1_1_0 {
            personas: self.personas,
            user_profile: self.user_profile,
            app_state: None, // Default: no app_state
            workspaces: self.workspaces,
        }
    }
}

/// Migration from ConfigRootV1_1_0 to ConfigRootV2_0_0.
/// Removes personas, workspaces, and app_state (now managed separately).
/// Only keeps user_profile.
impl MigratesTo<ConfigRootV2_0_0> for ConfigRootV1_1_0 {
    fn migrate(self) -> ConfigRootV2_0_0 {
        ConfigRootV2_0_0 {
            user_profile: self.user_profile.unwrap_or_default(),
        }
    }
}

/// Migration from ConfigRootV2_0_0 to ConfigRootV2_1_0.
/// Adds model_settings field with default values.
impl MigratesTo<ConfigRootV2_1_0> for ConfigRootV2_0_0 {
    fn migrate(self) -> ConfigRootV2_1_0 {
        ConfigRootV2_1_0 {
            user_profile: self.user_profile,
            model_settings: ModelSettingsDTO::default(),
        }
    }
}

/// Migration from ConfigRootV2_1_0 to ConfigRootV2_2_0.
/// Adds env_settings field with default values (auto-detect enabled).
impl MigratesTo<ConfigRootV2_2_0> for ConfigRootV2_1_0 {
    fn migrate(self) -> ConfigRootV2_2_0 {
        ConfigRootV2_2_0 {
            user_profile: self.user_profile,
            model_settings: self.model_settings,
            env_settings: EnvSettingsDTO::default(),
        }
    }
}

/// Migration from ConfigRootV2_2_0 to ConfigRootV2_3_0.
/// Adds debug_settings field with default values (debug disabled).
impl MigratesTo<ConfigRootV2_3_0> for ConfigRootV2_2_0 {
    fn migrate(self) -> ConfigRootV2_3_0 {
        ConfigRootV2_3_0 {
            user_profile: self.user_profile,
            model_settings: self.model_settings,
            env_settings: self.env_settings,
            debug_settings: DebugSettingsDTO::default(),
        }
    }
}

/// Migration from ConfigRootV2_3_0 to ConfigRootV2_4_0.
/// Adds memory_sync_settings field with default values (sync disabled).
impl MigratesTo<ConfigRootV2_4_0> for ConfigRootV2_3_0 {
    fn migrate(self) -> ConfigRootV2_4_0 {
        ConfigRootV2_4_0 {
            user_profile: self.user_profile,
            model_settings: self.model_settings,
            env_settings: self.env_settings,
            debug_settings: self.debug_settings,
            memory_sync_settings: MemorySyncSettingsDTO::default(),
        }
    }
}

/// Migration from ConfigRootV2_4_0 to ConfigRootV2_5_0.
/// Adds terminal_settings field with default values.
impl MigratesTo<ConfigRootV2_5_0> for ConfigRootV2_4_0 {
    fn migrate(self) -> ConfigRootV2_5_0 {
        ConfigRootV2_5_0 {
            user_profile: self.user_profile,
            model_settings: self.model_settings,
            env_settings: self.env_settings,
            debug_settings: self.debug_settings,
            memory_sync_settings: self.memory_sync_settings,
            terminal_settings: TerminalSettingsDTO::default(),
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// IntoDomain implementation for ConfigRootV2_5_0.
/// Converts DTO to domain RootConfig.
impl IntoDomain<RootConfig> for ConfigRootV2_5_0 {
    fn into_domain(self) -> RootConfig {
        RootConfig {
            user_profile: self.user_profile.into_domain(),
            model_settings: self.model_settings.into_domain(),
            env_settings: self.env_settings.into_domain(),
            debug_settings: self.debug_settings.into_domain(),
            memory_sync_settings: self.memory_sync_settings.into_domain(),
            terminal_settings: self.terminal_settings.into_domain(),
        }
    }
}

/// FromDomain implementation for ConfigRootV2_5_0.
/// Converts domain RootConfig to DTO for persistence.
impl version_migrate::FromDomain<RootConfig> for ConfigRootV2_5_0 {
    fn from_domain(config: RootConfig) -> Self {
        ConfigRootV2_5_0 {
            user_profile: UserProfileDTO::from_domain(config.user_profile),
            model_settings: ModelSettingsDTO::from_domain(config.model_settings),
            env_settings: EnvSettingsDTO::from_domain(config.env_settings),
            debug_settings: DebugSettingsDTO::from_domain(config.debug_settings),
            memory_sync_settings: MemorySyncSettingsDTO::from_domain(config.memory_sync_settings),
            terminal_settings: TerminalSettingsDTO::from_domain(config.terminal_settings),
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for ConfigRoot.
///
/// Handles automatic schema migration through multiple versions.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Adds `app_state` field with default value (None)
/// - V1.1.0 → V2.0.0: Removes `personas`, `workspaces`, `app_state` (now managed separately)
/// - V2.0.0 → V2.1.0: Adds `model_settings` field with default values
/// - V2.1.0 → V2.2.0: Adds `env_settings` field with default values (auto-detect enabled)
/// - V2.2.0 → V2.3.0: Adds `debug_settings` field with default values (debug disabled)
/// - V2.3.0 → V2.4.0: Adds `memory_sync_settings` field with default values (sync disabled)
/// - V2.4.0 → V2.5.0: Adds `terminal_settings` field with default values
/// - V2.5.0 → RootConfig: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_config_root_migrator();
/// let config: RootConfig = migrator.load_flat_from("config_root", toml_value)?;
/// ```
pub fn create_config_root_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("config_root" => [
        ConfigRootV1_0_0,
        ConfigRootV1_1_0,
        ConfigRootV2_0_0,
        ConfigRootV2_1_0,
        ConfigRootV2_2_0,
        ConfigRootV2_3_0,
        ConfigRootV2_4_0,
        ConfigRootV2_5_0,
        RootConfig
    ], save = true)
    .expect("Failed to create config_root migrator")
}
