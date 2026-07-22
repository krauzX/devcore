use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};
use devcore_challenges::{ChallengeEngine, Difficulty};

#[derive(Parser)]
pub struct DsaCmd {
    #[command(subcommand)]
    pub action: DsaAction,
}

#[derive(Subcommand)]
pub enum DsaAction {
    List,
    Install {
        pack_id: String,
    },
    Remove {
        pack_id: String,
    },
    Problems {
        pack_id: String,
    },
    Show {
        pack_id: String,
        problem_id: String,
    },
    ByDifficulty {
        level: String,
    },
}

pub fn run(cmd: DsaCmd, project_root: &Path) -> Result<()> {
    let mut engine = ChallengeEngine::new(project_root);

    match cmd.action {
        DsaAction::List => {
            let available = engine.list_available();
            println!("Available Packs:");
            for pack in available {
                let status = if pack.installed { " [installed]" } else { "" };
                println!(
                    "  {} — {} ({}, {} problems){}",
                    pack.id,
                    pack.name,
                    pack.difficulty,
                    pack.problems.len(),
                    status
                );
            }
            let installed = engine.list_installed();
            if !installed.is_empty() {
                println!();
                println!("Installed Packs:");
                for pack in &installed {
                    println!(
                        "  {} — {} ({}, {} problems)",
                        pack.id,
                        pack.name,
                        pack.difficulty,
                        pack.problems.len()
                    );
                }
            }
        }
        DsaAction::Install { pack_id } => {
            engine.install_pack(&pack_id).map_err(|e| anyhow::anyhow!(e))?;
            println!("Installed pack '{}'.", pack_id);
        }
        DsaAction::Remove { pack_id } => {
            engine.remove_pack(&pack_id).map_err(|e| anyhow::anyhow!(e))?;
            println!("Removed pack '{}'.", pack_id);
        }
        DsaAction::Problems { pack_id } => {
            let problems = engine.list_problems(&pack_id);
            if problems.is_empty() {
                println!("No problems found for pack '{}'.", pack_id);
            } else {
                println!("Problems in '{}':", pack_id);
                for p in &problems {
                    println!(
                        "  {} — {} [{}]",
                        p.id, p.name, p.difficulty
                    );
                }
            }
        }
        DsaAction::Show { pack_id, problem_id } => {
            match engine.get_problem(&pack_id, &problem_id) {
                Some(p) => {
                    println!("{} ({})", p.name, p.difficulty);
                    println!("Category: {}", p.category);
                    println!("Tags: {}", p.tags.join(", "));
                    println!();
                    println!("{}", p.description);
                    if !p.hints.is_empty() {
                        println!();
                        println!("Hints:");
                        for (i, h) in p.hints.iter().enumerate() {
                            println!("  {}. {}", i + 1, h);
                        }
                    }
                    if !p.test_cases.is_empty() {
                        println!();
                        println!("Test Cases:");
                        for tc in &p.test_cases {
                            println!(
                                "  {} — Input: {}, Expected: {}",
                                tc.description, tc.input, tc.expected
                            );
                        }
                    }
                }
                None => {
                    println!("Problem '{}' not found in pack '{}'.", problem_id, pack_id);
                }
            }
        }
        DsaAction::ByDifficulty { level } => {
            let difficulty = match level.to_lowercase().as_str() {
                "easy" => Difficulty::Easy,
                "medium" => Difficulty::Medium,
                "hard" => Difficulty::Hard,
                _ => {
                    anyhow::bail!(
                        "Invalid difficulty '{}'. Valid: easy, medium, hard",
                        level
                    );
                }
            };
            let problems = engine.problems_by_difficulty(difficulty);
            if problems.is_empty() {
                println!("No {} problems found.", difficulty);
            } else {
                println!("{} Problems:", difficulty);
                for (pack_id, p) in &problems {
                    println!(
                        "  [{}] {} — {} ({})",
                        pack_id, p.id, p.name, p.category
                    );
                }
            }
        }
    }
    Ok(())
}
