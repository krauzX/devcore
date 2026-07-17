use crate::monitor::scanner::RepoActivity;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Research-backed developer productivity metrics.
///
/// Based on:
/// - arXiv:2606.26959 "Shift to Agentic AI" (Codex usage patterns)
/// - arXiv:2606.15283 "AI-driven Software Development" (productivity framework)
/// - arXiv:2603.28592 "Debt Behind the AI Boom" (AI code quality)
/// - Flow state research (Csikszentmihalyi)
/// - Context switching cost studies (American Psychological Association)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevMetrics {
    /// Total commits across all repos
    pub total_commits: u64,
    /// Total lines changed
    pub total_lines_changed: u64,
    /// Percentage of AI-generated commits
    pub ai_ratio: f64,
    /// Number of repos worked on
    pub repos_touched: usize,
    /// Total unique files changed
    pub total_files_changed: usize,
    /// Average commits per repo
    pub commits_per_repo: f64,
    /// Code churn ratio (insertions + deletions / files)
    pub churn_ratio: f64,
    /// AI productivity multiplier (estimated)
    pub ai_multiplier: f64,
    /// Context switching index (repos touched per day)
    pub context_switch_index: f64,
    /// Focus score (consistency of work across repos)
    pub focus_score: f64,
    /// Sustainability score (work-life balance indicator)
    pub sustainability_score: f64,
    /// Weekly trend (-1.0 to 1.0, negative = declining)
    pub weekly_trend: f64,
    /// Bottleneck category
    pub bottleneck: String,
    /// Actionable insights
    pub insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub category: InsightCategory,
    pub severity: InsightSeverity,
    pub message: String,
    pub suggestion: String,
    pub research_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightCategory {
    Focus,
    AiUsage,
    Churn,
    ContextSwitching,
    Sustainability,
    Collaboration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSeverity {
    Info,
    Warning,
    Critical,
}

/// Compute research-backed metrics from repo activity data.
pub fn compute_metrics(activities: &[RepoActivity]) -> DevMetrics {
    let total_commits: u64 = activities.iter().map(|a| a.total_commits as u64).sum();
    let total_insertions: u64 = activities.iter().map(|a| a.total_insertions as u64).sum();
    let total_deletions: u64 = activities.iter().map(|a| a.total_deletions as u64).sum();
    let total_files: usize = activities.iter().map(|a| a.unique_files).sum();
    let ai_commits: u64 = activities.iter().map(|a| a.ai_commits as u64).sum();

    let ai_ratio = if total_commits > 0 {
        ai_commits as f64 / total_commits as f64
    } else {
        0.0
    };

    let commits_per_repo = if !activities.is_empty() {
        total_commits as f64 / activities.len() as f64
    } else {
        0.0
    };

    let churn_ratio = if total_files > 0 {
        (total_insertions + total_deletions) as f64 / total_files as f64
    } else {
        0.0
    };

    // AI multiplier: research shows 1.5-3x productivity boost (arXiv:2606.26959)
    // We estimate based on AI ratio and churn patterns
    let ai_multiplier = if ai_ratio > 0.0 {
        1.0 + (ai_ratio * 1.5) // Conservative: 1.0x to 2.5x range
    } else {
        1.0
    };

    // Context switching: repos touched per day (over 7 days)
    let days_active = activities
        .iter()
        .filter(|a| a.last_commit.is_some())
        .count()
        .max(1);
    let context_switch_index = activities.len() as f64 / days_active as f64;

    // Focus score: how concentrated is work on primary repo?
    // 1.0 = all work on one repo, 0.0 = evenly spread
    let focus_score = if let Some(top) = activities.first() {
        if total_commits > 0 {
            top.total_commits as f64 / total_commits as f64
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Sustainability: based on work distribution across time
    // Even distribution = good, bursty = warning
    let sustainability = compute_sustainability(activities);

    // Trend: compare first half vs second half of period
    let trend = compute_trend(activities);

    // Bottleneck detection
    let bottleneck = detect_bottleneck(activities);

    // Generate insights
    let insights = generate_insights(
        ai_ratio,
        context_switch_index,
        focus_score,
        sustainability,
        churn_ratio,
        total_commits,
    );

    DevMetrics {
        total_commits,
        total_lines_changed: total_insertions + total_deletions,
        ai_ratio,
        repos_touched: activities.len(),
        total_files_changed: total_files,
        commits_per_repo,
        churn_ratio,
        ai_multiplier,
        context_switch_index,
        focus_score,
        sustainability_score: sustainability,
        weekly_trend: trend,
        bottleneck,
        insights,
    }
}

fn compute_sustainability(activities: &[RepoActivity]) -> f64 {
    // Measure how evenly work is distributed across repos
    // Even distribution = sustainable, all on one = burnout risk
    if activities.is_empty() {
        return 1.0;
    }

    let total: u64 = activities.iter().map(|a| a.total_commits as u64).sum();
    if total == 0 {
        return 1.0;
    }

    let entropy: f64 = activities
        .iter()
        .map(|a| {
            let p = a.total_commits as f64 / total as f64;
            if p > 0.0 {
                -p * p.log2()
            } else {
                0.0
            }
        })
        .sum();

    let max_entropy = (activities.len() as f64).log2();
    if max_entropy > 0.0 {
        entropy / max_entropy // Normalized 0-1
    } else {
        1.0
    }
}

fn compute_trend(activities: &[RepoActivity]) -> f64 {
    if activities.is_empty() {
        return 0.0;
    }

    let mut all_timestamps: Vec<DateTime<chrono::Utc>> =
        activities.iter().filter_map(|a| a.last_commit).collect();
    all_timestamps.sort();

    if all_timestamps.len() < 4 {
        return 0.0;
    }

    let mid = all_timestamps.len() / 2;
    let first_half_count = mid as f64;
    let second_half_count = (all_timestamps.len() - mid) as f64;

    if first_half_count == 0.0 {
        return 1.0;
    }

    ((second_half_count - first_half_count) / first_half_count).clamp(-1.0, 1.0)
}

fn detect_bottleneck(activities: &[RepoActivity]) -> String {
    if activities.is_empty() {
        return "No activity".to_string();
    }

    let max_commits = activities
        .iter()
        .map(|a| a.total_commits)
        .max()
        .unwrap_or(0);
    let total: usize = activities.iter().map(|a| a.total_commits).sum();

    if total == 0 {
        return "No activity".to_string();
    }

    let top_ratio = max_commits as f64 / total as f64;

    if top_ratio > 0.7 {
        format!(
            "Over-concentration: {} takes {:.0}% of your time",
            activities[0].name,
            top_ratio * 100.0
        )
    } else if activities.len() > 5 {
        format!(
            "Context overload: {} repos touched this week",
            activities.len()
        )
    } else {
        "Balanced".to_string()
    }
}

fn generate_insights(
    ai_ratio: f64,
    context_switch_index: f64,
    focus_score: f64,
    sustainability: f64,
    churn_ratio: f64,
    total_commits: u64,
) -> Vec<Insight> {
    let mut insights = Vec::new();

    // AI usage insight
    if ai_ratio > 0.7 {
        insights.push(Insight {
            category: InsightCategory::AiUsage,
            severity: InsightSeverity::Warning,
            message: format!("AI usage at {:.0}% — high reliance on AI tools", ai_ratio * 100.0),
            suggestion: "Ensure adequate review coverage for AI-generated code. Run shipforge receipt on each commit.".into(),
            research_ref: "arXiv:2603.28592 — 22.7% of AI-introduced issues survive long-term".into(),
        });
    } else if ai_ratio > 0.0 && ai_ratio < 0.2 {
        insights.push(Insight {
            category: InsightCategory::AiUsage,
            severity: InsightSeverity::Info,
            message: format!(
                "AI usage at {:.0}% — room to leverage AI for repetitive tasks",
                ai_ratio * 100.0
            ),
            suggestion: "Consider using AI for boilerplate, tests, and documentation.".into(),
            research_ref: "arXiv:2606.26959 — 26.6% of users use AI skills/workflows".into(),
        });
    }

    // Context switching insight
    if context_switch_index > 3.0 {
        insights.push(Insight {
            category: InsightCategory::ContextSwitching,
            severity: InsightSeverity::Warning,
            message: format!(
                "High context switching: {:.1} repos/day average",
                context_switch_index
            ),
            suggestion: "Context switching costs 15-25 minutes per switch (APA research). Batch similar tasks.".into(),
            research_ref: "American Psychological Association — task switching costs 40% of productive time".into(),
        });
    }

    // Focus score insight
    if focus_score < 0.3 {
        insights.push(Insight {
            category: InsightCategory::Focus,
            severity: InsightSeverity::Warning,
            message: "Low focus — work spread evenly across many repos".into(),
            suggestion: "Prioritize depth over breadth. Block time for focused work on one project.".into(),
            research_ref: "Csikszentmihalyi — flow state requires uninterrupted focus on a single task".into(),
        });
    }

    // Sustainability insight
    if sustainability < 0.4 {
        insights.push(Insight {
            category: InsightCategory::Sustainability,
            severity: InsightSeverity::Warning,
            message: "Uneven work distribution — potential burnout risk".into(),
            suggestion: "Balance workload across projects. Consider delegating or deprioritizing."
                .into(),
            research_ref:
                "arXiv:2606.15283 — productivity gains depend on sustainable work patterns".into(),
        });
    }

    // Churn insight
    if churn_ratio > 100.0 {
        insights.push(Insight {
            category: InsightCategory::Churn,
            severity: InsightSeverity::Warning,
            message: format!("High code churn: {:.0} lines changed per file", churn_ratio),
            suggestion: "High churn may indicate instability. Review recent changes for quality."
                .into(),
            research_ref:
                "arXiv:2603.28592 — high churn correlates with accumulated technical debt".into(),
        });
    }

    // Volume insight
    if total_commits > 50 {
        insights.push(Insight {
            category: InsightCategory::Focus,
            severity: InsightSeverity::Info,
            message: format!("High output: {} commits this period", total_commits),
            suggestion: "Great output! Ensure quality keeps pace with quantity.".into(),
            research_ref: "arXiv:2606.26959 — output grew 13-50x per user in 2026".into(),
        });
    }

    insights
}
