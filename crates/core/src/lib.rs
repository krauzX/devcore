/// Git repository analysis and commit history extraction.
pub mod git_analyzer;
/// Persistent storage for change receipts and workflow events.
pub mod storage;
/// AI-generated commit and file content detection.
pub mod ai_detector;
/// Dependency graph analysis for estimating change blast radius.
pub mod blast_radius;
/// Core data models for change receipts, risks, and workflow events.
pub mod models;

pub use models::*;
pub use git_analyzer::{GitAnalyzer, CommitInfo, BlameLine};
pub use storage::Store;
pub use ai_detector::AiDetector;
pub use blast_radius::BlastRadiusAnalyzer;
