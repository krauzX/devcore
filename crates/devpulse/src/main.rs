use anyhow::Result;
use chrono::{DateTime, Duration, Timelike, Utc};
use clap::{Parser, Subcommand};
use devcore_core::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "devpulse",
    about = "Developer workflow analyzer — find where your time actually goes",
    version,
    long_about = "DevPulse monitors your development workflow and generates reports \
                   showing exactly where time is spent: coding, reviewing, searching, \
                   building, or context-switching."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize DevPulse in the current project
    Init {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Generate a workflow report for a time period
    Report {
        /// Time period: day, week, month
        #[arg(long, default_value = "week")]
        period: String,

        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Record a manual event (for testing or manual tracking)
    Event {
        /// Event type: coding, review, build, search, meeting, ai
        #[arg(short, long)]
        kind: String,

        /// Duration in minutes
        #[arg(short, long)]
        minutes: u32,

        /// Description
        #[arg(short, long, default_value = "")]
        description: String,

        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show suggestions for reducing time waste
    Suggest {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show time breakdown as a visual chart
    Chart {
        #[arg(long, default_value = "week")]
        period: String,

        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => cmd_init(&path),
        Commands::Report { period, path } => cmd_report(&path, &period),
        Commands::Event {
            kind,
            minutes,
            description,
            path,
        } => cmd_record_event(&path, &kind, minutes, &description),
        Commands::Suggest { path } => cmd_suggest(&path),
        Commands::Chart { period, path } => cmd_chart(&path, &period),
    }
}

fn cmd_init(path: &Path) -> Result<()> {
    let _store = Store::open(path)?;
    println!("DevPulse initialized at {}", path.display());
    println!();
    println!("Commands:");
    println!("  devpulse report --period week    # Weekly workflow report");
    println!("  devpulse event -k coding -m 30   # Record 30min of coding");
    println!("  devpulse suggest                 # Get productivity suggestions");
    println!("  devpulse chart --period day      # Visual time breakdown");
    println!();
    println!("Tip: Use `devpulse event` to manually track time blocks,");
    println!("or integrate with git commits to auto-detect workflow patterns.");

    Ok(())
}

fn cmd_report(project_root: &Path, period: &str) -> Result<()> {
    let store = Store::open(project_root)?;

    let (since, label) = parse_period(period);

    let events = store.events_since(since)?;
    let git = GitAnalyzer::open(project_root).ok();

    // Analyze git activity as workflow signal
    let mut time_by_category: HashMap<String, f64> = HashMap::new();
    let mut total_minutes = 0.0;

    // Process recorded events
    for event in &events {
        let cat = format!("{:?}", event.event_type);
        let mins = event
            .details
            .get("minutes")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        *time_by_category.entry(cat).or_insert(0.0) += mins;
        total_minutes += mins;
    }

    // If no events, analyze git history as proxy
    if events.is_empty() {
        if let Some(git_analyzer) = &git {
            let since_dt = Utc::now() - Duration::hours(parse_period_hours(period));
            if let Ok(commits) = git_analyzer.commits_since(since_dt, 100) {
                // Estimate time from commit patterns
                let ai_commits: Vec<_> = commits.iter().filter(|c| c.is_ai_generated).collect();
                let human_commits: Vec<_> = commits.iter().filter(|c| !c.is_ai_generated).collect();

                // Rough heuristic: each commit represents ~15-30 min of work
                let ai_minutes = ai_commits.len() as f64 * 20.0;
                let human_minutes = human_commits.len() as f64 * 25.0;
                let review_minutes = human_commits.len() as f64 * 15.0; // review time

                time_by_category.insert("AI_Coding".to_string(), ai_minutes);
                time_by_category.insert("Human_Coding".to_string(), human_minutes);
                time_by_category.insert("Review".to_string(), review_minutes);

                total_minutes = ai_minutes + human_minutes + review_minutes;

                println!("Git-Activity Based Estimate (no manual events recorded)");
            }
        }
    }

    println!("\nDevPulse Report: {}", label);
    println!("{}", "=".repeat(60));

    if total_minutes == 0.0 {
        println!("No activity recorded for this period.");
        println!("Use `devpulse event` to record time blocks.");
        return Ok(());
    }

    let hours = total_minutes / 60.0;
    println!(
        "Total time tracked: {:.1}h ({:.0} minutes)",
        hours, total_minutes
    );
    println!();

    // Sort by time
    let mut sorted: Vec<_> = time_by_category.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("{:<20} {:>8}  {:>6}  BAR", "CATEGORY", "TIME", "PCT");
    println!("{}", "-".repeat(60));

    for (cat, mins) in &sorted {
        let pct = (mins / total_minutes) * 100.0;
        let bar_len = (pct / 100.0 * 40.0) as usize;
        let bar = "█".repeat(bar_len);
        let hours_mins = if *mins >= 60.0 {
            format!("{:.1}h", mins / 60.0)
        } else {
            format!("{:.0}m", mins)
        };
        println!("{:<20} {:>8}  {:>5.1}%  {}", cat, hours_mins, pct, bar);
    }

    // Bottleneck detection
    if let Some((top_cat, top_mins)) = sorted.first() {
        let pct = (top_mins / total_minutes) * 100.0;
        if pct > 40.0 {
            println!("\n⚠ Bottleneck: {} takes {:.0}% of your time", top_cat, pct);
            match top_cat.as_str() {
                "Search" | "SEARCH" => {
                    println!(
                        "  Suggestion: Consider creating an index or using `codetrail explain`"
                    );
                }
                "Review" | "REVIEW" => {
                    println!("  Suggestion: Break tasks into smaller PRs to reduce review surface");
                }
                "AI_Coding" => {
                    println!("  Suggestion: Good use of AI — ensure review quality stays high");
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn cmd_record_event(
    project_root: &Path,
    kind: &str,
    minutes: u32,
    description: &str,
) -> Result<()> {
    let store = Store::open(project_root)?;

    let event_type = match kind.to_lowercase().as_str() {
        "coding" | "code" => EventType::GitCommit,
        "review" | "reviewing" => EventType::FileEdit,
        "build" | "deploy" => EventType::BuildRun,
        "test" | "testing" => EventType::TestRun,
        "search" | "looking" => EventType::FileEdit,
        "meeting" | "slack" => EventType::AiInteraction,
        "ai" | "cursor" | "copilot" => EventType::AiInteraction,
        _ => EventType::FileEdit,
    };

    let details = serde_json::json!({
        "minutes": minutes,
        "description": description,
        "category": kind,
    });

    let event = WorkflowEvent {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type,
        details,
    };

    store.save_event(&event)?;

    println!(
        "Recorded: {} minutes of {} — {}",
        minutes, kind, description
    );

    Ok(())
}

fn cmd_suggest(project_root: &Path) -> Result<()> {
    let _store = Store::open(project_root)?;
    let git = GitAnalyzer::open(project_root)?;

    println!("DevPulse Suggestions");
    println!("{}", "=".repeat(60));

    // Analyze recent commits for patterns
    let since = Utc::now() - Duration::days(7);
    let commits = git.commits_since(since, 100)?;

    let ai_count = commits.iter().filter(|c| c.is_ai_generated).count();
    let human_count = commits.len() - ai_count;

    if commits.is_empty() {
        println!("No recent commits to analyze.");
        return Ok(());
    }

    println!(
        "Last 7 days: {} commits ({} AI, {} human)",
        commits.len(),
        ai_count,
        human_count
    );
    println!();

    // Analyze blast radius of recent changes
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;

    let mut high_blast_files = Vec::new();
    for commit in &commits {
        for fc in &commit.files_changed {
            let br = analyzer.analyze(&fc.path);
            if br.direct_dependents.len() >= 3 {
                high_blast_files.push((fc.path.clone(), br.direct_dependents.len()));
            }
        }
    }
    high_blast_files.sort_by(|a, b| b.1.cmp(&a.1));
    high_blast_files.dedup();

    if !high_blast_files.is_empty() {
        println!("High-Risk Files Changed Recently:");
        for (file, deps) in &high_blast_files {
            println!("  ⚠ {} ({} dependents)", file, deps);
        }
        println!(
            "  → Consider using `codetrail blast {}` before next change",
            high_blast_files[0].0
        );
        println!();
    }

    // AI usage analysis
    if ai_count > 0 {
        let ai_ratio = ai_count as f64 / commits.len() as f64 * 100.0;
        println!("AI Usage: {:.0}% of commits are AI-generated", ai_ratio);

        if ai_ratio > 70.0 {
            println!("  ⚠ High AI usage — ensure adequate review coverage");
            println!("  → Run `shipforge receipt` on each commit to track change quality");
        } else if ai_ratio > 30.0 {
            println!("  ✓ Balanced AI usage");
        } else {
            println!("  Low AI usage — consider using AI for repetitive tasks");
        }
        println!();
    }

    // General suggestions
    println!("General Suggestions:");
    if commits.len() > 20 {
        println!("  • Consider batching related changes into fewer commits");
    }
    let large_commits: Vec<_> = commits
        .iter()
        .filter(|c| (c.insertions + c.deletions) > 200)
        .collect();
    if !large_commits.is_empty() {
        println!(
            "  • {} large commits (>200 changes) — break into smaller pieces",
            large_commits.len()
        );
    }
    println!("  • Use `codetrail explain <file>` before modifying high-traffic files");
    println!("  • Record time blocks with `devpulse event` for better insights");

    Ok(())
}

fn cmd_chart(project_root: &Path, period: &str) -> Result<()> {
    let store = Store::open(project_root)?;

    let (since, label) = parse_period(period);
    let events = store.events_since(since)?;

    println!("DevPulse Chart: {}", label);
    println!("{}", "=".repeat(60));

    if events.is_empty() {
        println!("No events recorded. Use `devpulse event` to start tracking.");
        return Ok(());
    }

    // Build hourly histogram
    let mut hourly: HashMap<u32, u32> = HashMap::new();
    for event in &events {
        let hour = event.timestamp.hour();
        let mins = event
            .details
            .get("minutes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        *hourly.entry(hour).or_insert(0) += mins;
    }

    let max_val = hourly.values().copied().max().unwrap_or(1);

    println!("\nHour of Day  Minutes  Activity");
    println!("{}", "-".repeat(50));

    for hour in 0..24 {
        let mins = hourly.get(&hour).copied().unwrap_or(0);
        let bar_len = if max_val > 0 {
            (mins as f64 / max_val as f64 * 30.0) as usize
        } else {
            0
        };
        let bar = "█".repeat(bar_len);
        let marker = if mins > 0 { "●" } else { " " };
        println!("{:>2}:00  {:>6}m  {} {}", hour, mins, bar, marker);
    }

    Ok(())
}

fn parse_period(period: &str) -> (DateTime<Utc>, String) {
    match period.to_lowercase().as_str() {
        "day" | "today" => (
            Utc::now() - Duration::hours(24),
            "Last 24 hours".to_string(),
        ),
        "week" | "7d" => (Utc::now() - Duration::days(7), "Last 7 days".to_string()),
        "month" | "30d" => (Utc::now() - Duration::days(30), "Last 30 days".to_string()),
        _ => (Utc::now() - Duration::days(7), "Last 7 days".to_string()),
    }
}

fn parse_period_hours(period: &str) -> i64 {
    match period.to_lowercase().as_str() {
        "day" | "today" => 24,
        "week" | "7d" => 168,
        "month" | "30d" => 720,
        _ => 168,
    }
}
