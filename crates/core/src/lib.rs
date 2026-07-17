/// AI-generated commit and file content detection.
pub mod ai_detector;
/// Dependency graph analysis for estimating change blast radius.
pub mod blast_radius;
/// Configurable project settings loaded from .devcore/config.toml.
pub mod config;
/// Unified error type for the DevCore core library.
pub mod error;
/// Git repository analysis and commit history extraction.
pub mod git_analyzer;
/// Core data models for change receipts, risks, and workflow events.
pub mod models;
/// Persistent storage for change receipts and workflow events.
pub mod storage;

pub use ai_detector::AiDetector;
pub use blast_radius::BlastRadiusAnalyzer;
pub use config::DevCoreConfig;
pub use error::DevCoreError;
pub use git_analyzer::{BlameLine, CommitInfo, GitAnalyzer};
pub use models::*;
pub use storage::Store;

pub mod prelude {
    pub use crate::ai_detector::AiDetector;
    pub use crate::blast_radius::BlastRadiusAnalyzer;
    pub use crate::config::DevCoreConfig;
    pub use crate::error::DevCoreError;
    pub use crate::git_analyzer::GitAnalyzer;
    pub use crate::models::*;
    pub use crate::storage::Store;
}
