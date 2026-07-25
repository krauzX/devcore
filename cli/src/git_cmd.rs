use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};
use devcore_core::Store;
use devcore_devtrack::{
    add_xp, analyze_repo, compute_streak, detect_languages, get_progress, init_skill_schema,
    SkillAxis,
};

#[derive(Parser)]
pub struct GitCmd {
    #[command(subcommand)]
    pub action: GitAction,
}

#[derive(Subcommand)]
pub enum GitAction {
    Analyze,
    Streak,
    Skills,
    Xp {
        axis: String,
        amount: u32,
        reason: String,
    },
}

pub fn run(cmd: GitCmd, project_root: &Path) -> Result<()> {
    match cmd.action {
        GitAction::Analyze => {
            let analysis = analyze_repo(project_root)?;
            println!("Repository Analysis:");
            println!("  Commits:    {}", analysis.total_commits);
            println!("  Insertions: +{}", analysis.total_insertions);
            println!("  Deletions:  -{}", analysis.total_deletions);
            println!("  Files:      {}", analysis.unique_files);
            println!("  Authors:    {}", analysis.unique_authors);
            if let Some(first) = analysis.first_commit {
                println!("  First:      {}", first);
            }
            if let Some(last) = analysis.last_commit {
                println!("  Last:       {}", last);
            }
            println!();
            let langs = detect_languages(project_root);
            if !langs.is_empty() {
                println!("Languages:");
                for lang in &langs {
                    println!("  {} — {} files, {} lines", lang.name, lang.files, lang.lines);
                }
            }
        }
        GitAction::Streak => {
            let streak = compute_streak(project_root)?;
            println!("Git Streak:");
            println!("  Current:  {} days", streak.current);
            println!("  Longest:  {} days", streak.longest);
            println!("  Total:    {} days", streak.total_days);
            if let Some(last) = streak.last_commit_date {
                println!("  Last:     {}", last);
            }
        }
        GitAction::Skills => {
            let store = Store::open(project_root)?;
            let conn = store.conn()?;
            init_skill_schema(&conn)?;
            let progress = get_progress(&conn)?;
            println!("Skill Progress:");
            for sp in &progress {
                println!(
                    "  {} — Level {} ({} XP)",
                    sp.axis.as_str(),
                    sp.level,
                    sp.xp
                );
            }
        }
        GitAction::Xp { axis, amount, reason } => {
            let skill_axis: SkillAxis = axis.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let store = Store::open(project_root)?;
            let conn = store.conn()?;
            init_skill_schema(&conn)?;
            let updated = add_xp(&conn, skill_axis, amount, &reason)?;
            println!(
                "Added {} XP to {} → Level {} ({} total XP)",
                amount, axis, updated.level, updated.xp
            );
        }
    }
    Ok(())
}
