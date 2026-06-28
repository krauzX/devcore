use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};
use devcore_core::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "shipforge",
    about = "Generate structured change receipts for AI-generated code",
    version,
    long_about = "ShipForge analyzes your git commits, detects AI-generated code, \
                   and generates structured change receipts with intent, blast radius, \
                   and risk scoring. Every commit gets a trail."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ShipForge in the current project
    Init {
        /// Project root directory (default: current dir)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Generate a change receipt for staged/committed changes
    Receipt {
        /// Commit OID (default: HEAD)
        #[arg(short, long)]
        commit: Option<String>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show a stored change receipt
    Show {
        /// Commit OID
        commit: String,

        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },

    /// List recent change receipts
    Log {
        /// Number of recent receipts to show
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Only show AI-generated commits
        #[arg(long)]
        ai_only: bool,
    },

    /// Explain why a file exists and what it depends on
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
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => cmd_init(&path),
        Commands::Receipt { commit, path } => cmd_receipt(&path, commit.as_deref()),
        Commands::Show { commit, format } => cmd_show(&commit, &format),
        Commands::Log { limit, ai_only } => cmd_log(limit, ai_only),
        Commands::Explain { file, path } => cmd_explain(&path, &file),
        Commands::Blast { file, path } => cmd_blast(&path, &file),
    }
}

fn cmd_init(path: &Path) -> Result<()> {
    let git = GitAnalyzer::open(path)?;
    let store = Store::open(path)?;

    let files = git.list_files()?;
    println!("ShipForge initialized for project with {} tracked files.", files.len());

    // Build initial blast radius graph
    let mut analyzer = BlastRadiusAnalyzer::new(path);
    analyzer.build_graph()?;

    println!("Blast radius graph built. ShipForge is ready.");
    println!();
    println!("Usage:");
    println!("  shipforge receipt           # Generate receipt for HEAD commit");
    println!("  shipforge receipt -c <oid>  # Generate receipt for specific commit");
    println!("  shipforge log               # Show recent receipts");
    println!("  shipforge explain <file>    # Why does this file exist?");
    println!("  shipforge blast <file>      # What breaks if I change this?");

    Ok(())
}

fn cmd_receipt(project_root: &Path, commit_oid: Option<&str>) -> Result<()> {
    let git = GitAnalyzer::open(project_root)?;
    let store = Store::open(project_root)?;
    let detector = AiDetector::new();

    let oid = match commit_oid {
        Some(oid) => oid.to_string(),
        None => git.head_oid()?,
    };

    let repo = git2::Repository::open(project_root)?;
    let commit = repo.find_commit(git2::Oid::from_str(&oid)?)?;

    let info = git.commit_info(&commit)?;
    let intent = detector.extract_intent(&info.message);
    let is_ai = info.is_ai_generated;
    let ai_source = info.ai_source.clone();

    // Build blast radius
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;

    let mut all_blast = devcore_core::BlastRadius::default();
    for fc in &info.files_changed {
        let br = analyzer.analyze(&fc.path);
        all_blast.direct_dependents.extend(br.direct_dependents);
        all_blast.indirect_dependents.extend(br.indirect_dependents);
        all_blast.depth = all_blast.depth.max(br.depth);
    }
    all_blast.direct_dependents.sort();
    all_blast.direct_dependents.dedup();
    all_blast.indirect_dependents.sort();
    all_blast.indirect_dependents.dedup();

    // Risk scoring
    let risk_score = calculate_risk_score(&info, &all_blast);
    let risks = identify_risks(&info, &all_blast);

    let receipt = ChangeReceipt {
        id: uuid::Uuid::new_v4().to_string(),
        commit_oid: oid.clone(),
        timestamp: info.timestamp,
        is_ai_generated: is_ai,
        ai_source: ai_source.clone(),
        intent: intent.clone(),
        files_changed: info.files_changed.clone(),
        decisions: vec![Decision {
            timestamp: Utc::now(),
            description: format!("Commit authored by {}", info.author),
            rationale: ai_source.map(|s| format!("Detected as {:?} output", s)),
        }],
        risks,
        blast_radius: all_blast,
        risk_score,
    };

    store.save_receipt(&receipt)?;

    // Print receipt
    print_receipt(&receipt);

    Ok(())
}

fn cmd_show(commit_oid: &str, format: &str) -> Result<()> {
    // Use current dir for now — in production would find .devcore
    let store = Store::open(Path::new("."))?;

    match store.get_receipt(commit_oid)? {
        Some(receipt) => {
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            } else {
                print_receipt(&receipt);
            }
        }
        None => {
            eprintln!("No receipt found for commit {}", commit_oid);
        }
    }

    Ok(())
}

fn cmd_log(limit: usize, ai_only: bool) -> Result<()> {
    let store = Store::open(Path::new("."))?;
    let receipts = store.recent_receipts(limit * 2)?; // fetch extra to filter

    let filtered: Vec<&ChangeReceipt> = receipts
        .iter()
        .filter(|r| !ai_only || r.is_ai_generated)
        .take(limit)
        .collect();

    if filtered.is_empty() {
        println!("No change receipts found.");
        return Ok(());
    }

    println!("{:<12}  {:<8}  {:<6}  {:<50}", "COMMIT", "SOURCE", "RISK", "INTENT");
    println!("{}", "-".repeat(80));

    for r in &filtered {
        let short_oid = &r.commit_oid[..12.min(r.commit_oid.len())];
        let source = r
            .ai_source
            .as_ref()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "human".to_string());
        let risk_bar = match r.risk_score {
            0..=3 => "LOW",
            4..=6 => "MED",
            7..=8 => "HIGH",
            _ => "CRIT",
        };
        let intent = if r.intent.len() > 48 {
            format!("{}...", &r.intent[..45])
        } else {
            r.intent.clone()
        };

        println!("{:<12}  {:<8}  {:<6}  {}", short_oid, source, risk_bar, intent);
    }

    Ok(())
}

fn cmd_explain(project_root: &Path, file_path: &str) -> Result<()> {
    let store = Store::open(project_root)?;
    let receipts = store.recent_receipts(100)?;

    let relevant: Vec<&ChangeReceipt> = receipts
        .iter()
        .filter(|r| r.files_changed.iter().any(|f| f.path == file_path))
        .collect();

    println!("File: {}", file_path);
    println!("{}", "=".repeat(60));

    if relevant.is_empty() {
        println!("No change receipts found for this file.");
        println!("Run `shipforge receipt` after your next commit to track changes.");
        return Ok(());
    }

    println!("\nChange History ({} receipts):", relevant.len());
    for r in &relevant {
        let short = &r.commit_oid[..12.min(r.commit_oid.len())];
        println!("  [{}] {} — {}", r.timestamp.format("%Y-%m-%d %H:%M"), short, r.intent);
        if r.is_ai_generated {
            println!("    Source: {:?}", r.ai_source);
        }
    }

    // Blast radius
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;
    let br = analyzer.analyze(file_path);

    if !br.direct_dependents.is_empty() {
        println!("\nDirect dependents (files that import this):");
        for d in &br.direct_dependents {
            println!("  → {}", d);
        }
    }

    if !br.indirect_dependents.is_empty() {
        println!("\nIndirect dependents (2+ hops):");
        for d in &br.indirect_dependents {
            println!("  → {}", d);
        }
    }

    if br.direct_dependents.is_empty() && br.indirect_dependents.is_empty() {
        println!("\nNo dependents found — safe to modify.");
    }

    Ok(())
}

fn cmd_blast(project_root: &Path, file_path: &str) -> Result<()> {
    let mut analyzer = BlastRadiusAnalyzer::new(project_root);
    analyzer.build_graph()?;
    let br = analyzer.analyze(file_path);

    println!("Blast Radius: {}", file_path);
    println!("{}", "=".repeat(60));
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

    // Risk level
    let risk = match (br.direct_dependents.len(), br.indirect_dependents.len()) {
        (0, 0) => "LOW — isolated file, safe to change",
        (1..=3, 0) => "LOW — few direct dependents",
        (1..=3, _) => "MEDIUM — some transitive impact",
        (4..=10, _) => "HIGH — many dependents, proceed with caution",
        _ => "CRITICAL — heavily depended upon, extreme caution",
    };

    println!("\nRisk Level: {}", risk);

    Ok(())
}

fn calculate_risk_score(info: &CommitInfo, blast: &BlastRadius) -> u8 {
    let mut score: u8 = 0;

    // AI-generated code starts at +2
    if info.is_ai_generated {
        score += 2;
    }

    // Large changes are riskier
    let total_changes = info.insertions + info.deletions;
    if total_changes > 500 {
        score += 3;
    } else if total_changes > 100 {
        score += 2;
    } else if total_changes > 50 {
        score += 1;
    }

    // Many files changed
    if info.files_changed.len() > 10 {
        score += 2;
    } else if info.files_changed.len() > 5 {
        score += 1;
    }

    // Blast radius
    score += (blast.direct_dependents.len() / 3).min(3) as u8;
    score += (blast.indirect_dependents.len() / 5).min(2) as u8;

    score.min(10)
}

fn identify_risks(info: &CommitInfo, blast: &BlastRadius) -> Vec<Risk> {
    let mut risks = Vec::new();

    for file in &info.files_changed {
        let dependents: Vec<String> = blast
            .direct_dependents
            .iter()
            .filter(|d| {
                // Simple heuristic: check if the file is in the import path
                d.contains(&file.path)
            })
            .cloned()
            .collect();

        if !dependents.is_empty() {
            risks.push(Risk {
                severity: if dependents.len() > 5 {
                    RiskSeverity::High
                } else {
                    RiskSeverity::Medium
                },
                file: file.path.clone(),
                line: None,
                description: format!(
                    "{} downstream file(s) depend on this",
                    dependents.len()
                ),
                downstream_files: dependents,
            });
        }
    }

    if info.is_ai_generated && info.deletions > info.insertions {
        risks.push(Risk {
            severity: RiskSeverity::Medium,
            file: "(commit-wide)".to_string(),
            line: None,
            description: "AI commit has more deletions than insertions — possible regressions".to_string(),
            downstream_files: vec![],
        });
    }

    risks
}

fn print_receipt(receipt: &ChangeReceipt) {
    let short_oid = &receipt.commit_oid[..12.min(receipt.commit_oid.len())];
    let risk_bar = match receipt.risk_score {
        0..=3 => "LOW",
        4..=6 => "MEDIUM",
        7..=8 => "HIGH",
        _ => "CRITICAL",
    };

    println!("┌─ Change Receipt ─────────────────────────────────────────┐");
    println!("│ Commit:    {:<47}│", short_oid);
    println!(
        "│ Timestamp: {:<47}│",
        receipt.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "│ Source:    {:<47}│",
        if receipt.is_ai_generated {
            format!("AI ({:?})", receipt.ai_source)
        } else {
            "Human".to_string()
        }
    );
    println!("│ Intent:    {:<47}│", truncate(&receipt.intent, 47));
    println!(
        "│ Risk:      {:<3} ({}){}│",
        risk_bar,
        receipt.risk_score,
        " ".repeat(40 - risk_bar.len() - format!("{}", receipt.risk_score).len())
    );
    println!("├─ Files Changed ──────────────────────────────────────────┤");
    for fc in &receipt.files_changed {
        let marker = match fc.status {
            ChangeStatus::Added => "+",
            ChangeStatus::Deleted => "-",
            ChangeStatus::Renamed => "~",
            ChangeStatus::Modified => "M",
        };
        println!("│  {} {:<54}│", marker, truncate(&fc.path, 54));
    }
    if receipt.files_changed.is_empty() {
        println!("│  (no files changed){:<39}│", "");
    }
    println!("├─ Blast Radius ──────────────────────────────────────────┤");
    println!(
        "│  Direct:   {:<47}│",
        receipt.blast_radius.direct_dependents.len()
    );
    println!(
        "│  Indirect: {:<47}│",
        receipt.blast_radius.indirect_dependents.len()
    );
    if !receipt.risks.is_empty() {
        println!("├─ Risks ─────────────────────────────────────────────────┤");
        for risk in &receipt.risks {
            let sev = format!("[{:?}]", risk.severity);
            println!(
                "│  {} {:<52}│",
                sev,
                truncate(&risk.description, 52)
            );
        }
    }
    println!("└──────────────────────────────────────────────────────────┘");
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
