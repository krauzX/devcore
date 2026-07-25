use crate::error::DevCoreError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevCoreConfig {
    pub institution: String,
    pub program: String,
    pub batch: String,
    pub total_semesters: u8,
}

impl Default for DevCoreConfig {
    fn default() -> Self {
        Self {
            institution: String::new(),
            program: String::new(),
            batch: String::new(),
            total_semesters: 8,
        }
    }
}

impl DevCoreConfig {
    pub fn load(project_root: &Path) -> Result<Self, DevCoreError> {
        let config_path = project_root.join(".devcore").join("config.json");
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: DevCoreConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, project_root: &Path) -> Result<(), DevCoreError> {
        let config_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
