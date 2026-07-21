use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiSource {
    Cursor,
    Copilot,
    ClaudeCode,
    Windsurf,
    Aider,
    Unknown,
}

impl fmt::Display for AiSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiSource::Cursor => write!(f, "Cursor"),
            AiSource::Copilot => write!(f, "GitHub Copilot"),
            AiSource::ClaudeCode => write!(f, "Claude Code"),
            AiSource::Windsurf => write!(f, "Windsurf"),
            AiSource::Aider => write!(f, "Aider"),
            AiSource::Unknown => write!(f, "Unknown AI"),
        }
    }
}

/// A structured record of what changed in a commit, why, and what risks it carries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReceipt {
    pub id: String,
    pub commit_oid: String,
    pub timestamp: DateTime<Utc>,
    pub is_ai_generated: bool,
    pub ai_source: Option<AiSource>,
    pub intent: String,
    pub files_changed: Vec<FileChange>,
    pub decisions: Vec<Decision>,
    pub risks: Vec<Risk>,
    pub blast_radius: BlastRadius,
    /// Aggregate risk score from 0 (safe) to 100 (critical)
    pub risk_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: ChangeStatus,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl fmt::Display for ChangeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeStatus::Added => write!(f, "Added"),
            ChangeStatus::Modified => write!(f, "Modified"),
            ChangeStatus::Deleted => write!(f, "Deleted"),
            ChangeStatus::Renamed => write!(f, "Renamed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub severity: RiskSeverity,
    pub file: String,
    pub line: Option<u32>,
    pub description: String,
    pub downstream_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskSeverity::Low => write!(f, "Low"),
            RiskSeverity::Medium => write!(f, "Medium"),
            RiskSeverity::High => write!(f, "High"),
            RiskSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Estimated downstream impact of a change, measured by dependent files and depth.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlastRadius {
    pub direct_dependents: Vec<String>,
    pub indirect_dependents: Vec<String>,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    GitCommit,
    FileEdit,
    BuildRun,
    TestRun,
    AiInteraction,
}
