use serde::{Deserialize, Serialize};
use std::path::Path;

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
    pub fn load(project_root: &Path) -> Self {
        let config_path = project_root.join(".devcore").join("config.toml");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }
}
