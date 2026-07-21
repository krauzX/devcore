use crate::monitor::metrics::{self, DevMetrics, InsightCategory, InsightSeverity};
use crate::monitor::scanner::{RepoActivity, SystemScanner};
use anyhow::Result;
use chrono::{Duration, Utc};
use colored::{Color, Colorize};
use std::path::Path;

#[allow(dead_code)]
pub struct WeeklyReport {
    scanner: SystemScanner,
    activities: Vec<RepoActivity>,
    metrics: DevMetrics,
}

impl WeeklyReport {
    pub fn generate(project_root: Option<&Path>) -> Result<Self> {
        let mut scanner = if let Some(root) = project_root {
            SystemScanner::with_dirs(vec![root.to_path_buf()])
        } else {
            SystemScanner::new()
        };

        let _repos = scanner.scan();
        let since = Utc::now() - Duration::days(7);
        let activities = scanner.collect_activity(since)?;
        let metrics = metrics::compute_metrics(&activities);

        Ok(Self {
            scanner,
            activities,
            metrics,
        })
    }

    pub fn print(&self) {
        self.print_header();
        self.print_summary();
        self.print_repo_breakdown();
        self.print_insights();
        self.print_footer();
    }

    fn print_header(&self) {
        println!();
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════════╗"
                .cyan()
                .bold()
        );
        println!(
            "{}  {:<54}  {}",
            "║".cyan().bold(),
            "DevCore Weekly Productivity Report".white().bold(),
            "║".cyan().bold()
        );
        println!(
            "{}  {:<54}  {}",
            "║".cyan().bold(),
            format!("{}", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")).dimmed(),
            "║".cyan().bold()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════════╝"
                .cyan()
                .bold()
        );
        println!();
    }

    fn print_summary(&self) {
        let m = &self.metrics;

        println!("  {}", "SUMMARY".white().bold());
        println!("  {}", "-".repeat(50));

        println!(
            "  {:<25} {:>10}",
            "Total commits:".dimmed(),
            m.total_commits.to_string().white().bold()
        );
        println!(
            "  {:<25} {:>10}",
            "Lines changed:".dimmed(),
            m.total_lines_changed.to_string().white().bold()
        );
        println!(
            "  {:<25} {:>10}",
            "Repos touched:".dimmed(),
            m.repos_touched.to_string().white().bold()
        );
        println!(
            "  {:<25} {:>10}",
            "Files changed:".dimmed(),
            m.total_files_changed.to_string().white().bold()
        );
        println!(
            "  {:<25} {:>10}",
            "AI usage:".dimmed(),
            format!("{:.0}%", m.ai_ratio * 100.0).yellow().bold()
        );
        println!(
            "  {:<25} {:>10}",
            "AI multiplier:".dimmed(),
            format!("{:.1}x", m.ai_multiplier).green().bold()
        );
        println!();

        println!("  {}", "KEY METRICS".white().bold());
        println!("  {}", "-".repeat(50));

        let focus_color = if m.focus_score > 0.6 {
            Color::Green
        } else if m.focus_score > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };
        println!(
            "  {:<25} {:>10}",
            "Focus score:".dimmed(),
            format!("{:.0}%", m.focus_score * 100.0)
                .color(focus_color)
                .bold()
        );

        let ctx_color = if m.context_switch_index < 2.0 {
            Color::Green
        } else if m.context_switch_index < 4.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        println!(
            "  {:<25} {:>10}",
            "Context switches:".dimmed(),
            format!("{:.1}/day", m.context_switch_index)
                .color(ctx_color)
                .bold()
        );

        let sust_color = if m.sustainability_score > 0.6 {
            Color::Green
        } else if m.sustainability_score > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };
        println!(
            "  {:<25} {:>10}",
            "Sustainability:".dimmed(),
            format!("{:.0}%", m.sustainability_score * 100.0)
                .color(sust_color)
                .bold()
        );

        let trend_str = if m.weekly_trend > 0.1 {
            format!("+{:.0}%", m.weekly_trend * 100.0).green()
        } else if m.weekly_trend < -0.1 {
            format!("{:.0}%", m.weekly_trend * 100.0).red()
        } else {
            "stable".dimmed()
        };
        println!(
            "  {:<25} {:>10}",
            "Weekly trend:".dimmed(),
            trend_str.bold()
        );

        println!(
            "  {:<25} {:>10}",
            "Bottleneck:".dimmed(),
            m.bottleneck.yellow()
        );
        println!();
    }

    fn print_repo_breakdown(&self) {
        if self.activities.is_empty() {
            return;
        }

        println!("  {}", "REPO ACTIVITY".white().bold());
        println!("  {}", "-".repeat(50));

        for (i, activity) in self.activities.iter().take(10).enumerate() {
            let bar_len = if self.metrics.total_commits > 0 {
                (activity.total_commits as f64 / self.metrics.total_commits as f64 * 30.0) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_len);
            let ai_marker = if activity.ai_commits > 0 {
                format!(" ({})", activity.ai_commits).yellow().to_string()
            } else {
                String::new()
            };

            println!(
                "  {:<3} {:<20} {:>4}commits{}  {}",
                format!("{}.", i + 1).dimmed(),
                truncate_str(&activity.name, 20),
                activity.total_commits,
                ai_marker,
                bar.cyan()
            );
            println!(
                "      {:<20} {:>4}files  {:>6} lines",
                "",
                activity.unique_files,
                format!(
                    "{:+}",
                    activity.total_insertions as i64 - activity.total_deletions as i64
                )
            );
        }

        if self.activities.len() > 10 {
            println!("      ... and {} more repos", self.activities.len() - 10);
        }
        println!();
    }

    fn print_insights(&self) {
        if self.metrics.insights.is_empty() {
            return;
        }

        println!("  {}", "INSIGHTS & SUGGESTIONS".white().bold());
        println!("  {}", "-".repeat(50));

        for insight in &self.metrics.insights {
            let icon = match insight.severity {
                InsightSeverity::Critical => "🔴".to_string(),
                InsightSeverity::Warning => "🟡".to_string(),
                InsightSeverity::Info => "🟢".to_string(),
            };

            let cat = match insight.category {
                InsightCategory::Focus => "FOCUS".cyan(),
                InsightCategory::AiUsage => "AI".yellow(),
                InsightCategory::Churn => "CHURN".red(),
                InsightCategory::ContextSwitching => "CTX".magenta(),
                InsightCategory::Sustainability => "SUSTAIN".green(),
                InsightCategory::Collaboration => "COLLAB".blue(),
            };

            println!("  {} [{}] {}", icon, cat, insight.message.white());
            println!("    💡 {}", insight.suggestion.dimmed());
            println!("    📚 {}", insight.research_ref.dimmed());
            println!();
        }
    }

    fn print_footer(&self) {
        println!(
            "{}",
            "══════════════════════════════════════════════════════════════".cyan()
        );
        println!(
            "  {} Powered by research: arXiv:2606.26959, arXiv:2606.15283, arXiv:2603.28592",
            "📊".dimmed()
        );
        println!(
            "  {} Run `devpulse dashboard` for interactive TUI",
            "🖥️ ".dimmed()
        );
        println!(
            "  {} Run `devpulse report` for per-project breakdown",
            "📋".dimmed()
        );
        println!();
    }

    #[allow(dead_code)]
    pub fn metrics(&self) -> &DevMetrics {
        &self.metrics
    }

    #[allow(dead_code)]
    pub fn activities(&self) -> &[RepoActivity] {
        &self.activities
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
