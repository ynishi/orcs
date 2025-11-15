//! Fluent API builder for creating configured agents.

use super::AgentConfig;
use std::path::PathBuf;

/// Fluent builder for creating workspace-aware agents.
///
/// # Example
/// ```
/// use orcs_core::agent::AgentBuilder;
/// use std::path::PathBuf;
///
/// let config = AgentBuilder::new()
///     .with_workspace(PathBuf::from("/path/to/project"))
///     .build();
///
/// assert!(config.cwd.is_some());
/// assert!(config.env_vars.contains_key("PATH"));
/// ```
#[derive(Debug, Default)]
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    /// Creates a new AgentBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the workspace root directory.
    ///
    /// Automatically configures:
    /// - CWD to workspace_root
    /// - Enhanced PATH environment variable
    ///
    /// # Example
    /// ```
    /// use orcs_core::agent::AgentBuilder;
    /// use std::path::PathBuf;
    ///
    /// let builder = AgentBuilder::new()
    ///     .with_workspace(PathBuf::from("/path/to/project"));
    /// ```
    pub fn with_workspace(mut self, workspace_root: PathBuf) -> Self {
        self.config = AgentConfig::from_workspace(workspace_root);
        self
    }

    /// Sets custom working directory.
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.config.cwd = Some(cwd);
        self
    }

    /// Adds an environment variable.
    ///
    /// # Example
    /// ```
    /// use orcs_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new()
    ///     .with_env("CUSTOM_VAR", "custom_value");
    /// ```
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env_vars.insert(key.into(), value.into());
        self
    }

    /// Builds the final AgentConfig.
    pub fn build(self) -> AgentConfig {
        self.config
    }
}

impl Clone for AgentBuilder {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder_new() {
        let builder = AgentBuilder::new();
        let config = builder.build();

        assert!(config.cwd.is_none());
        assert!(config.env_vars.is_empty());
    }

    #[test]
    fn test_agent_builder_with_workspace() {
        let workspace = PathBuf::from("/test/workspace");
        let config = AgentBuilder::new()
            .with_workspace(workspace.clone())
            .build();

        assert_eq!(config.cwd, Some(workspace));
        assert!(config.env_vars.contains_key("PATH"));
        assert!(config.workspace.is_some());
    }

    #[test]
    fn test_agent_builder_with_cwd() {
        let cwd = PathBuf::from("/custom/cwd");
        let config = AgentBuilder::new().with_cwd(cwd.clone()).build();

        assert_eq!(config.cwd, Some(cwd));
    }

    #[test]
    fn test_agent_builder_with_env() {
        let config = AgentBuilder::new()
            .with_env("CUSTOM_VAR", "custom_value")
            .build();

        assert_eq!(
            config.env_vars.get("CUSTOM_VAR"),
            Some(&"custom_value".to_string())
        );
    }

    #[test]
    fn test_agent_builder_clone() {
        let builder = AgentBuilder::new()
            .with_workspace(PathBuf::from("/test/workspace"));

        let cloned = builder.clone();
        let config1 = builder.build();
        let config2 = cloned.build();

        assert_eq!(config1.cwd, config2.cwd);
        assert_eq!(config1.env_vars, config2.env_vars);
    }
}
