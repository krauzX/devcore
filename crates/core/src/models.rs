use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiSource {
    Cursor,
    Copilot,
    ClaudeCode,
    Windsurf,
    Aider,
    Unknown,
}

impl AiSource {
    pub fn from_commit_message(msg: &str) -> Option<Self> {
        let lower = msg.to_lowercase();
        if lower.contains("cursor") || lower.contains("ai-generated") {
            Some(Self::Cursor)
        } else if lower.contains("copilot") {
            Some(Self::Copilot)
        } else if lower.contains("claude") || lower.contains("anthropic") {
            Some(Self::ClaudeCode)
        } else if lower.contains("windsurf") || lower.contains("codeium") {
            Some(Self::Windsurf)
        } else if lower.contains("aider") {
            Some(Self::Aider)
        } else {
            None
        }
    }
}

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
    TerminalCommand,
    BuildRun,
    TestRun,
    AiInteraction,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u64,
    pub time_by_category: std::collections::HashMap<String, f64>,
    pub bottlenecks: Vec<Bottleneck>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub category: String,
    pub time_minutes: f64,
    pub percentage: f64,
    pub suggestion: String,
}
