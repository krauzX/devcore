use anyhow::Result;
use clap::{Parser, Subcommand};
use devcore_core::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "codetrail",
    about = "Change receipts + blast radius analysis for AI-generated code",
    version,
    long_about = "CodeTrail tracks why code was changed, what it depends on, \
                   and what could break. Query any file's history and blast radius."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize CodeTrail in the current project
    Init {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show change history for a file
    History {
        /// File path relative to project root
        file: String,

        /// Number of recent changes to show
        #[arg(short, long, default_value = "10")]
        limit: usize,

        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Explain why a file exists, its decisions, and dependents
    Explain {
        /// File path relative to project root
        file: String,

        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show blast radius for a file
    Blast {
        /// File path relative to project root
        file: String,

        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show all AI-generated change receipts
    AiLog {
        /// Number of receipts to show
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Only show receipts from a specific source (cursor, copilot, claude, etc.)
        #[arg(long)]
        source: Option<String>,
    },

    /// Show risk summary for the project
    Risk {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Find files with highest blast radius (most depended upon)
    Hotspots {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Number of hotspots to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init { path } => cmd_init(&path),
        Commands::History { file, limit, path } => cmd_history(&path, &file, limit),
        Commands::Explain { file, path } => cmd_explain(&path, &file),
        Commands::Blast { file, path } => cmd_blast(&path, &file),
        Commands::AiLog { limit, source } => cmd_ai_log(limit, source.as_deref()),
        Commands::Risk { path } => cmd_risk(&path),
        Commands::Hotspots { path, limit } => cmd_hotspots(&path, limit),
    }
}

fn cmd_init(path: &Path) -> Result<()> {
    let _git = GitAnalyzer::open(path)?;
    let _store = Store::open(path)?;

    let mut analyzer = BlastRadiusAnalyzer::new(path);
    analyzer.build_graph()?;

    println!("CodeTrail initialized at {}", path.display());
    println!();
    println!("Commands:");
    println!("  codetrail history <file>    # Change history for a file");
    println!("  codetrail explain <file>    # Why does this exist? What depends on it?");
    println!("  codetrail blast <file>      # What breaks if I change this?");
    println!("  codetrail ai-log            # All AI-generated change receipts");
    println!("  codetrail risk              # Project risk summary");
    println!("  codetrail hotspots          # Files with highest blast radius");

    Ok(())
}

fn cmd_history(project_root: &Path, file_path: &str, limit: usize) -> Result<()> {
    let git = GitAnalyzer::open(project_root)?;
    let store = Store::open(project_root)?;
    let receipts = store.receipts_for_file(file_path)?;

    println!("Change History: {}", file_path);
    println!("{}", "=".repeat(70));

    if receipts.is_empty() {
        println!("No change receipts found for this file.");
        println!("Run `shipforge receipt` after commits to track changes.");
        return Ok(());
    }

    for r in receipts.iter().take(limit) {
        let short = &r.commit_oid[..12.min(r.commit_oid.len())];
        let ai_marker = if r.is_ai_generated { " [AI]" } else { "" };

        println!(
            "\n  {}{} — {}",
            short,
            ai_marker,
            r.timestamp.format("%Y-%m-%d %H:%M")
        );
        println!("  Intent: {}", r.intent);

        if r.is_ai_generated {
            println!("  Source: {:?}", r.ai_source);
        }

        // Show this file's specific change
        if let Some(fc) = r.files_changed.iter().find(|f| f.path == file_path) {
            let marker = match fc.status {
                ChangeStatus::Added => "+",
                ChangeStatus::Deleted => "-",
                ChangeStatus::Renamed => "~",
                ChangeStatus::Modified => "M",
            };
            println!(
                "  Status: {} ({}+/{}-)",
                marker, fc.insertions, fc.deletions
            );
        }

        if !r.risks.is_empty() {
            println!("  Risks:");
            for risk in &r.risks {
                if risk.file == file_path {
                    println!("    [{:?}] {}", risk.severity, risk.description);
                }
            }
        }
    }

    // Show blame info
    if let Ok(blame) = git.blame_file(file_path) {
        let mut author_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for line in &blame {
            *author_counts.entry(line.author.clone()).or_insert(0) += 1;
        }
        if !author_counts.is_empty() {
            println!("\n--- Blame Summary ---");
            let mut sorted: Vec<_> = author_counts.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            for (author, count) in sorted.iter().take(5) {
                println!("  {:>4} lines  {}", count, author);
            }
        }
    }

    Ok(())
}

fn cmd_explain(project_root: &Path, file_path: &str) -> Result<()> {
    let store = Store::open(project_root)?;
    let receipts = store.recent_receipts(200)?;

    println!("File: {}", file_path);
    println!("{}", "=".repeat(70));

    // Change history
    let relevant: Vec<&ChangeReceipt> = receipts
        .iter()
        .filter(|r| r.files_changed.iter().any(|f| f.path == file_path))
        .collect();

    println!("\n--- Change History ({} commits) ---", relevant.len());
    for r in relevant.iter().take(10) {
        let short = &r.commit_oid[..12.min(r.commit_oid.len())];
        let ai = if r.is_ai_generated { " [AI]" } else { "" };
        println!("  {}{} — {}", short, ai, r.intent);
    }

    // Blast radius
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;
    let br = analyzer.analyze(file_path);

    println!("\n--- Dependents ---");
    if br.direct_dependents.is_empty() {
        println!("  No direct dependents — safe to modify in isolation.");
    } else {
        println!("  Direct ({} files):", br.direct_dependents.len());
        for d in &br.direct_dependents {
            println!("    → {}", d);
        }
    }

    if !br.indirect_dependents.is_empty() {
        println!(
            "  Indirect ({} files, {} hops):",
            br.indirect_dependents.len(),
            br.depth
        );
        for d in &br.indirect_dependents {
            println!("    → {}", d);
        }
    }

    // Risks from receipts
    let all_risks: Vec<&Risk> = relevant
        .iter()
        .flat_map(|r| r.risks.iter())
        .filter(|risk| {
            risk.file == file_path || risk.downstream_files.contains(&file_path.to_string())
        })
        .collect();

    if !all_risks.is_empty() {
        println!("\n--- Known Risks ---");
        for risk in all_risks {
            println!("  [{:?}] {}", risk.severity, risk.description);
            if !risk.downstream_files.is_empty() {
                println!("    Downstream: {}", risk.downstream_files.join(", "));
            }
        }
    }

    Ok(())
}

fn cmd_blast(project_root: &Path, file_path: &str) -> Result<()> {
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;
    let br = analyzer.analyze(file_path);

    println!("Blast Radius: {}", file_path);
    println!("{}", "=".repeat(70));
    println!("Direct dependents:   {}", br.direct_dependents.len());
    println!("Indirect dependents: {}", br.indirect_dependents.len());
    println!("Max depth:           {}", br.depth);

    if !br.direct_dependents.is_empty() {
        println!("\nDirect (would break immediately):");
        for d in &br.direct_dependents {
            println!("  ⚠ {}", d);
        }
    }

    if !br.indirect_dependents.is_empty() {
        println!("\nIndirect (may break transitively):");
        for d in &br.indirect_dependents {
            println!("  ⚡ {}", d);
        }
    }

    let risk_level = match (br.direct_dependents.len(), br.indirect_dependents.len()) {
        (0, 0) => "LOW — isolated file, safe to change",
        (1..=3, 0) => "LOW — few direct dependents",
        (1..=3, _) => "MEDIUM — some transitive impact",
        (4..=10, _) => "HIGH — many dependents, proceed with caution",
        _ => "CRITICAL — heavily depended upon, extreme caution",
    };

    println!("\nRisk Level: {}", risk_level);

    Ok(())
}

fn cmd_ai_log(limit: usize, source_filter: Option<&str>) -> Result<()> {
    let store = Store::open(Path::new("."))?;
    let receipts = store.recent_receipts(limit * 3)?;

    let filtered: Vec<&ChangeReceipt> = receipts
        .iter()
        .filter(|r| r.is_ai_generated)
        .filter(|r| {
            if let Some(src) = source_filter {
                r.ai_source
                    .as_ref()
                    .map(|s| {
                        format!("{:?}", s)
                            .to_lowercase()
                            .contains(&src.to_lowercase())
                    })
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .take(limit)
        .collect();

    let total_commits = store.recent_receipts(limit * 3)?.len();
    let ai_count = filtered.len();
    println!(
        "Showing {} of {} total AI receipts ({} total commits)",
        ai_count,
        store.recent_receipts(1000)?.len(),
        total_commits
    );

    println!("AI-Generated Change Receipts");
    println!("{}", "=".repeat(80));

    if filtered.is_empty() {
        println!("No AI-generated receipts found.");
        return Ok(());
    }

    println!(
        "{:<12}  {:<10}  {:<6}  {:<8}  {:<42}",
        "COMMIT", "SOURCE", "RISK", "FILES", "INTENT"
    );
    println!("{}", "-".repeat(80));

    for r in &filtered {
        let short = &r.commit_oid[..12.min(r.commit_oid.len())];
        let source = r
            .ai_source
            .as_ref()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "?".to_string());
        let risk = match r.risk_score {
            0..=3 => "LOW",
            4..=6 => "MED",
            7..=8 => "HIGH",
            _ => "CRIT",
        };
        let files = format!("{}", r.files_changed.len());
        let intent = if r.intent.len() > 40 {
            format!("{}...", &r.intent[..37])
        } else {
            r.intent.clone()
        };

        println!(
            "{:<12}  {:<10}  {:<6}  {:<8}  {}",
            short, source, risk, files, intent
        );
    }

    Ok(())
}

fn cmd_risk(project_root: &Path) -> Result<()> {
    let store = Store::open(project_root)?;
    let receipts = store.recent_receipts(100)?;

    let ai_count = receipts.iter().filter(|r| r.is_ai_generated).count();
    let human_count = receipts.len() - ai_count;
    let high_risk = receipts.iter().filter(|r| r.risk_score >= 7).count();
    let avg_risk: f64 = if receipts.is_empty() {
        0.0
    } else {
        receipts.iter().map(|r| r.risk_score as f64).sum::<f64>() / receipts.len() as f64
    };

    println!("Project Risk Summary");
    println!("{}", "=".repeat(50));
    println!("Total receipts:     {}", receipts.len());
    println!(
        "AI-generated:       {} ({:.0}%)",
        ai_count,
        if receipts.is_empty() {
            0.0
        } else {
            ai_count as f64 / receipts.len() as f64 * 100.0
        }
    );
    println!("Human-authored:     {}", human_count);
    println!("High-risk (≥7):     {}", high_risk);
    println!("Average risk score: {:.1}/10", avg_risk);

    // Files with most changes
    let mut file_changes: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for r in &receipts {
        for fc in &r.files_changed {
            *file_changes.entry(fc.path.clone()).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<_> = file_changes.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    if !sorted.is_empty() {
        println!("\nMost Changed Files:");
        for (file, count) in sorted.iter().take(10) {
            println!("  {:>4}x  {}", count, file);
        }
    }

    Ok(())
}

fn cmd_hotspots(project_root: &Path, limit: usize) -> Result<()> {
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;

    let files = analyzer.list_all_files();

    let mut scored: Vec<(String, usize, usize)> = files
        .iter()
        .map(|f| {
            let br = analyzer.analyze(f);
            (
                f.clone(),
                br.direct_dependents.len(),
                br.indirect_dependents.len(),
            )
        })
        .filter(|(_, direct, _)| *direct > 0)
        .collect();

    scored.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));

    println!("Hotspots (files with highest blast radius)");
    println!("{}", "=".repeat(70));

    if scored.is_empty() {
        println!("No hotspots found. Run `shipforge receipt` to start tracking.");
        return Ok(());
    }

    for (file, direct, indirect) in scored.iter().take(limit) {
        let total = direct + indirect;
        let bar: String = "█".repeat(total.min(30));
        println!("  {:>3}  {:<50} {}", total, file, bar);
    }

    Ok(())
}
