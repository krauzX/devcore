use crate::error::DevCoreError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Project-level configuration for DevCore tools.
///
/// Loaded from `.devcore/config.toml` in the project root. Falls back to
/// sensible defaults when the file is missing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevCoreConfig {
    #[serde(default = "default_extensions")]
    pub source_extensions: Vec<String>,

    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,

    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,
}

fn default_extensions() -> Vec<String> {
    vec![
        "ts".into(),
        "tsx".into(),
        "js".into(),
        "jsx".into(),
        "rs".into(),
        "go".into(),
        "py".into(),
    ]
}

fn default_exclude_dirs() -> Vec<String> {
    vec![
        "node_modules".into(),
        "target".into(),
        ".git".into(),
        ".devcore".into(),
    ]
}

fn default_max_file_size() -> u64 {
    10 * 1024 * 1024
}

impl Default for DevCoreConfig {
    fn default() -> Self {
        Self {
            source_extensions: default_extensions(),
            exclude_dirs: default_exclude_dirs(),
            max_file_size_bytes: default_max_file_size(),
        }
    }
}

impl DevCoreConfig {
    /// Loads configuration from `.devcore/config.toml`.
    ///
    /// Returns defaults if the file does not exist. Returns an error if the
    /// file exists but cannot be parsed.
    pub fn load(project_root: &Path) -> Result<Self, DevCoreError> {
        let config_path = project_root.join(".devcore").join("config.toml");
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: DevCoreConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Saves this configuration to `.devcore/config.toml`.
    pub fn save(&self, project_root: &Path) -> Result<(), DevCoreError> {
        let config_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| DevCoreError::Config(e.to_string()))?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Validates the configuration and returns warnings for questionable values.
    pub fn validate(&self) -> Result<(), DevCoreError> {
        if self.source_extensions.is_empty() {
            return Err(DevCoreError::Config(
                "source_extensions list is empty — at least one extension is required".into(),
            ));
        }
        if self.max_file_size_bytes == 0 {
            return Err(DevCoreError::Config(
                "max_file_size_bytes is 0 — files will be skipped".into(),
            ));
        }
        Ok(())
    }
}
