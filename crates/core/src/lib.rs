/// AI-generated commit and file content detection.
pub mod ai_detector;
/// Dependency graph analysis for estimating change blast radius.
pub mod blast_radius;
/// Configurable project settings loaded from .devcore/config.toml.
pub mod config;
/// Git repository analysis and commit history extraction.
pub mod git_analyzer;
/// Core data models for change receipts, risks, and workflow events.
pub mod models;
/// Persistent storage for change receipts and workflow events.
pub mod storage;

pub use ai_detector::AiDetector;
pub use blast_radius::BlastRadiusAnalyzer;
pub use git_analyzer::{BlameLine, CommitInfo, GitAnalyzer};
pub use models::*;
pub use storage::Store;
