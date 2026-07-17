/// Unified error type for the DevCore core library.
#[derive(Debug, thiserror::Error)]
pub enum DevCoreError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("not found: {0}")]
    NotFound(String),
}

impl From<toml::de::Error> for DevCoreError {
    fn from(err: toml::de::Error) -> Self {
        DevCoreError::Config(err.to_string())
    }
}
