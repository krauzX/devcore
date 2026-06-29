use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Identifies the AI tool or assistant that authored or assisted with a commit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiSource {
    /// Cursor AI coding assistant
    Cursor,
    /// GitHub Copilot
    Copilot,
    /// Anthropic Claude Code
    ClaudeCode,
    /// Windsurf (Codeium) AI assistant
    Windsurf,
    /// Aider AI pair programmer
    Aider,
    /// Unknown AI source
    Unknown,
}

impl AiSource {
    /// Attempts to identify the AI source from a commit message by scanning for known keywords.
    /// Returns `None` if no known AI tool is detected.
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

/// A structured record of what changed in a commit, why, and what risks it carries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReceipt {
    /// Unique receipt identifier (UUID)
    pub id: String,
    /// Git commit OID this receipt describes
    pub commit_oid: String,
    /// Timestamp of the commit
    pub timestamp: DateTime<Utc>,
    /// Whether the commit was authored or assisted by AI
    pub is_ai_generated: bool,
    /// Detected AI source, if any
    pub ai_source: Option<AiSource>,
    /// Human-readable summary of the change intent
    pub intent: String,
    /// List of files modified in this commit
    pub files_changed: Vec<FileChange>,
    /// Key architectural or design decisions made in this commit
    pub decisions: Vec<Decision>,
    /// Identified risks introduced by this commit
    pub risks: Vec<Risk>,
    /// Estimated downstream impact of this commit
    pub blast_radius: BlastRadius,
    /// Aggregate risk score from 0 (safe) to 100 (critical)
    pub risk_score: u8,
}

/// Represents a single file-level change within a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Relative path of the changed file
    pub path: String,
    /// Nature of the change (added, modified, etc.)
    pub status: ChangeStatus,
    /// Number of lines inserted
    pub insertions: u32,
    /// Number of lines deleted
    pub deletions: u32,
}

/// Describes the kind of change applied to a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeStatus {
    /// File was newly created
    Added,
    /// Existing file was modified
    Modified,
    /// File was removed
    Deleted,
    /// File was renamed or moved
    Renamed,
}

/// A notable decision recorded during development.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// When the decision was made
    pub timestamp: DateTime<Utc>,
    /// Description of the decision
    pub description: String,
    /// Reasoning behind the decision, if provided
    pub rationale: Option<String>,
}

/// A risk identified in a commit that may require review or mitigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    /// How severe the risk is
    pub severity: RiskSeverity,
    /// File where the risk originates
    pub file: String,
    /// Specific line number, if identifiable
    pub line: Option<u32>,
    /// Description of the risk
    pub description: String,
    /// Files that could be affected by this risk
    pub downstream_files: Vec<String>,
}

/// Severity level for a identified risk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskSeverity {
    /// Minor risk, unlikely to cause issues
    Low,
    /// Moderate risk that warrants attention
    Medium,
    /// Significant risk that should be reviewed before merge
    High,
    /// Critical risk that likely breaks functionality
    Critical,
}

/// Estimated downstream impact of a change, measured by dependent files and depth.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlastRadius {
    /// Files that directly import or depend on the changed file
    pub direct_dependents: Vec<String>,
    /// Files that transitively depend on the changed file
    pub indirect_dependents: Vec<String>,
    /// Maximum depth of the dependency chain explored
    pub depth: u32,
}

/// A single event captured from the developer's workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Unique event identifier (UUID)
    pub id: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Category of the event
    pub event_type: EventType,
    /// Arbitrary event-specific payload
    pub details: serde_json::Value,
}

/// Categories of workflow events that can be tracked.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// A git commit was created
    GitCommit,
    /// A file was edited in the editor
    FileEdit,
    /// A terminal command was executed
    TerminalCommand,
    /// A build was triggered
    BuildRun,
    /// Tests were executed
    TestRun,
    /// An interaction with an AI assistant occurred
    AiInteraction,
    /// No significant activity detected
    Idle,
}

/// A summary report of developer workflow activity over a time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowReport {
    /// Start of the reporting period
    pub period_start: DateTime<Utc>,
    /// End of the reporting period
    pub period_end: DateTime<Utc>,
    /// Total number of events in the period
    pub total_events: u64,
    /// Time spent in each activity category (in minutes)
    pub time_by_category: std::collections::HashMap<String, f64>,
    /// Identified workflow bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
    /// Suggestions for improving workflow efficiency
    pub suggestions: Vec<String>,
}

/// A workflow bottleneck where the developer spent disproportionate time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Activity category (e.g., "BuildRun", "TestRun")
    pub category: String,
    /// Time spent in this category (in minutes)
    pub time_minutes: f64,
    /// Percentage of total workflow time
    pub percentage: f64,
    /// Suggested action to reduce this bottleneck
    pub suggestion: String,
}
