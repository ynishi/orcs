//! Agent configuration structures.

use std::collections::HashMap;
use std::path::PathBuf;

/// Workspace-specific configuration for agent execution.
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    /// Root directory of the workspace
    pub workspace_root: PathBuf,
    /// Enhanced PATH environment variable
    pub enhanced_path: String,
}

impl WorkspaceConfig {
    /// Creates a new WorkspaceConfig with enhanced PATH.
    ///
    /// # Example
    /// ```
    /// use orcs_core::agent::WorkspaceConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = WorkspaceConfig::new(PathBuf::from("/path/to/workspace"));
    /// assert!(!config.enhanced_path.is_empty());
    /// ```
    pub fn new(workspace_root: PathBuf) -> Self {
        let enhanced_path = crate::agent::build_enhanced_path(&workspace_root);
        Self {
            workspace_root,
            enhanced_path,
        }
    }

    /// Creates a WorkspaceConfig with custom PATH.
    pub fn with_custom_path(workspace_root: PathBuf, enhanced_path: String) -> Self {
        Self {
            workspace_root,
            enhanced_path,
        }
    }
}

/// Comprehensive agent configuration.
#[derive(Debug, Clone, Default)]
pub struct AgentConfig {
    /// Working directory for agent execution
    pub cwd: Option<PathBuf>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Workspace configuration (if applicable)
    pub workspace: Option<WorkspaceConfig>,
}

impl AgentConfig {
    /// Creates a new empty AgentConfig.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an AgentConfig from workspace configuration.
    ///
    /// # Example
    /// ```
    /// use orcs_core::agent::AgentConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = AgentConfig::from_workspace(PathBuf::from("/path/to/workspace"));
    /// assert!(config.cwd.is_some());
    /// assert!(config.env_vars.contains_key("PATH"));
    /// ```
    pub fn from_workspace(workspace_root: PathBuf) -> Self {
        let workspace = WorkspaceConfig::new(workspace_root.clone());
        let mut env_vars = HashMap::new();
        env_vars.insert("PATH".to_string(), workspace.enhanced_path.clone());

        Self {
            cwd: Some(workspace_root),
            env_vars,
            workspace: Some(workspace),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_new() {
        let workspace = PathBuf::from("/test/workspace");
        let config = WorkspaceConfig::new(workspace.clone());

        assert_eq!(config.workspace_root, workspace);
        assert!(!config.enhanced_path.is_empty());
    }

    #[test]
    fn test_workspace_config_with_custom_path() {
        let workspace = PathBuf::from("/test/workspace");
        let custom_path = "/custom/bin:/usr/bin".to_string();
        let config = WorkspaceConfig::with_custom_path(workspace.clone(), custom_path.clone());

        assert_eq!(config.workspace_root, workspace);
        assert_eq!(config.enhanced_path, custom_path);
    }

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new();

        assert!(config.cwd.is_none());
        assert!(config.env_vars.is_empty());
        assert!(config.workspace.is_none());
    }

    #[test]
    fn test_agent_config_from_workspace() {
        let workspace = PathBuf::from("/test/workspace");
        let config = AgentConfig::from_workspace(workspace.clone());

        assert_eq!(config.cwd, Some(workspace));
        assert!(config.env_vars.contains_key("PATH"));
        assert!(config.workspace.is_some());
    }
}
