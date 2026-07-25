use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};
use devcore_challenges::{ChallengeEngine, Difficulty, LeetCodeClient, ProjectEngine};

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
    #[command(name = "leetcode")]
    Leetcode(LeetcodeCmd),
    #[command(name = "project")]
    Project(ProjectCmd),
    Browse {
        #[arg(long)]
        difficulty: Option<String>,
        #[arg(long, default_value_t = 1)]
        page: usize,
        #[arg(long, default_value_t = 20)]
        per_page: usize,
    },
}

#[derive(Parser)]
pub struct ProjectCmd {
    #[command(subcommand)]
    pub action: ProjectAction,
}

#[derive(Subcommand)]
pub enum ProjectAction {
    List,
    Install {
        project_id: String,
    },
    Remove {
        project_id: String,
    },
    Show {
        project_id: String,
    },
    Progress {
        project_id: String,
    },
    Complete {
        project_id: String,
        stage: usize,
    },
    Stage {
        project_id: String,
        stage: usize,
    },
    View {
        project_id: String,
    },
}

#[derive(Parser)]
pub struct LeetcodeCmd {
    #[command(subcommand)]
    pub action: LeetcodeAction,
}

#[derive(Subcommand)]
pub enum LeetcodeAction {
    List {
        #[arg(long, default_value = "all")]
        difficulty: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        tags: Option<Vec<String>>,
    },
    Daily,
    Show {
        title_slug: String,
    },
}

pub fn run(cmd: DsaCmd, project_root: &Path) -> Result<()> {
    let mut engine = ChallengeEngine::new(&project_root.join(".devcore"));

    match cmd.action {
        DsaAction::Leetcode(lc_cmd) => return run_leetcode(lc_cmd),
        DsaAction::Project(proj_cmd) => return run_project(proj_cmd, project_root),
        DsaAction::Browse {
            difficulty,
            page,
            per_page,
        } => {
            let diff = difficulty.as_deref();
            let result = engine.list_offline(diff, page, per_page);
            println!(
                "Offline Problems (page {}/{}, showing {}-{} of {})",
                result.page,
                result.total_pages,
                (result.page.saturating_sub(1) * result.per_page + 1).min(result.total),
                (result.page * result.per_page).min(result.total),
                result.total
            );
            println!();
            println!(
                "{:<6} {:<50} {:<10} {:<10}",
                "ID", "Title", "Diff", "Accept%"
            );
            println!("{}", "-".repeat(76));
            for p in &result.problems {
                println!(
                    "{:<6} {:<50} {:<10} {:<9.1}%",
                    p.fid, p.title, p.difficulty, p.acceptance
                );
            }
            println!();
            println!(
                "Page {}/{} — use --page N to navigate",
                result.page, result.total_pages
            );
        }
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
            let difficulty: Difficulty = level.parse().map_err(|e: String| anyhow::anyhow!(e))?;
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

fn run_leetcode(cmd: LeetcodeCmd) -> Result<()> {
    let client = LeetCodeClient::new().map_err(|e| anyhow::anyhow!(e))?;

    match cmd.action {
        LeetcodeAction::List {
            difficulty,
            limit,
            tags,
        } => {
            let diff = if difficulty == "all" {
                None
            } else {
                Some(difficulty.as_str())
            };
            let tag_refs = tags.as_deref();
            let problems = client.fetch_problem_list(diff, tag_refs, limit)?;
            println!(
                "{}",
                devcore_challenges::leetcode::format_problem_list(&problems)
            );
        }
        LeetcodeAction::Daily => {
            let problem = client.fetch_daily_challenge()?;
            println!(
                "{}",
                devcore_challenges::leetcode::format_problem(&problem)
            );
        }
        LeetcodeAction::Show { title_slug } => {
            let problem = client.fetch_problem(&title_slug)?;
            println!(
                "{}",
                devcore_challenges::leetcode::format_problem(&problem)
            );
        }
    }
    Ok(())
}

fn run_project(cmd: ProjectCmd, project_root: &Path) -> Result<()> {
    let data_dir = project_root.join(".devcore");
    let mut engine = ProjectEngine::new(&data_dir);

    match cmd.action {
        ProjectAction::List => {
            let available = engine.list_available();
            let installed = engine.list_installed();
            println!("Available Projects:");
            for p in available {
                let is_installed = installed.iter().any(|i| i.id == p.id);
                let status = if is_installed { " [installed]" } else { "" };
                let progress_info = engine
                    .get_progress(&p.id)
                    .map(|prog| {
                        format!(
                            " ({}/{}, {:.0}%)",
                            prog.completed_stages.len(),
                            p.stages.len(),
                            prog.percentage()
                        )
                    })
                    .unwrap_or_default();
                println!(
                    "  {} — {} ({}, {} stages){}{}",
                    p.id, p.name, p.difficulty, p.stages.len(), progress_info, status
                );
            }
        }
        ProjectAction::Install { project_id } => {
            engine.install_project(&project_id).map_err(|e| anyhow::anyhow!(e))?;
            println!("Installed project '{}'.", project_id);
        }
        ProjectAction::Remove { project_id } => {
            engine.remove_project(&project_id).map_err(|e| anyhow::anyhow!(e))?;
            println!("Removed project '{}'.", project_id);
        }
        ProjectAction::Show { project_id } => {
            if let Some(p) = engine.get_project(&project_id) {
                println!("{}", p.name);
                println!("{}", "=".repeat(p.name.len()));
                println!();
                println!("{}", p.description);
                println!();
                println!("Language: {}", p.language);
                println!("Difficulty: {}", p.difficulty);
                println!("Stages: {}", p.stages.len());

                if let Some(prog) = engine.get_progress(&project_id) {
                    println!("Progress: {}/{} ({:.0}%)", prog.completed_stages.len(), p.stages.len(), prog.percentage());
                    if prog.is_complete() {
                        println!("Status: Complete");
                    } else {
                        println!("Status: In Progress (current stage: {})", prog.current_stage + 1);
                    }
                }

                println!();
                for (i, stage) in p.stages.iter().enumerate() {
                    let marker = engine
                        .get_progress(&project_id)
                        .and_then(|prog| {
                            if prog.completed_stages.contains(&i) {
                                Some("[x]")
                            } else if Some(i) == prog.current_stage.checked_sub(0) {
                                Some("[>]")
                            } else {
                                Some("[ ]")
                            }
                        })
                        .unwrap_or("[ ]");
                    println!("  {}. {} {} — {}", i + 1, marker, stage.name, stage.description);
                }
                println!();
                println!("Readme:");
                println!("{}", p.readme);
            } else {
                println!("Project '{}' not found.", project_id);
            }
        }
        ProjectAction::Progress { project_id } => {
            if let Some(p) = engine.get_project(&project_id) {
                match engine.get_progress(&project_id) {
                    Some(prog) => {
                        println!("Project: {}", p.name);
                        println!("Progress: {}/{} ({:.0}%)", prog.completed_stages.len(), p.stages.len(), prog.percentage());
                        if prog.is_complete() {
                            println!("Status: Complete");
                            if let Some(ref completed_at) = prog.completed_at {
                                println!("Completed: {}", completed_at);
                            }
                        } else {
                            println!("Status: In Progress");
                            println!("Current Stage: {} — {}", prog.current_stage + 1, p.stages[prog.current_stage].name);
                        }
                        if let Some(ref started_at) = prog.started_at {
                            println!("Started: {}", started_at);
                        }
                        println!();
                        println!("Stage Details:");
                        for (i, stage) in p.stages.iter().enumerate() {
                            let marker = if prog.completed_stages.contains(&i) {
                                "[x]"
                            } else if i == prog.current_stage {
                                "[>]"
                            } else {
                                "[ ]"
                            };
                            println!("  {}. {} {} — {}", i + 1, marker, stage.name, stage.description);
                        }
                        println!();
                        println!("Progress Bar:");
                        let filled = (prog.percentage() / 100.0 * 30.0) as usize;
                        let empty = 30 - filled;
                        println!("[{}{}]", "=".repeat(filled), "-".repeat(empty));
                    }
                    None => {
                        println!("No progress recorded for project '{}'.", project_id);
                        println!("Start working on the project to track progress.");
                    }
                }
            } else {
                println!("Project '{}' not found.", project_id);
            }
        }
        ProjectAction::Complete { project_id, stage } => {
            let stage_idx = stage.saturating_sub(1);
            engine.set_stage_completed(&project_id, stage_idx).map_err(|e| anyhow::anyhow!(e))?;
            if let Some(p) = engine.get_project(&project_id) {
                let stage_name = p.stages.get(stage_idx).map(|s| s.name.as_str()).unwrap_or("unknown");
                println!("Marked stage {} ({}) as complete for project '{}'.", stage, stage_name, project_id);
                if let Some(prog) = engine.get_progress(&project_id) {
                    println!("Progress: {}/{} ({:.0}%)", prog.completed_stages.len(), p.stages.len(), prog.percentage());
                    if prog.is_complete() {
                        println!("Congratulations! Project '{}' is now complete!", project_id);
                    }
                }
            }
        }
        ProjectAction::Stage { project_id, stage } => {
            let stage_idx = stage.saturating_sub(1);
            if let Some(p) = engine.get_project(&project_id) {
                if let Some(stage_data) = p.stages.get(stage_idx) {
                    println!("Project: {} — Stage {}/{}", p.name, stage, p.stages.len());
                    println!("Stage: {}", stage_data.name);
                    println!("{}", "=".repeat(50));
                    println!();
                    println!("{}", stage_data.description);
                    println!();

                    if !stage_data.files.is_empty() {
                        println!("Skeleton Files:");
                        println!("{}", "-".repeat(50));
                        for file in &stage_data.files {
                            println!("--- {} ---", file.path);
                            println!("{}", file.content);
                            println!();
                        }
                    }

                    if !stage_data.tests.is_empty() {
                        println!("Tests:");
                        println!("{}", "-".repeat(50));
                        for file in &stage_data.tests {
                            println!("--- {} ---", file.path);
                            println!("{}", file.content);
                            println!();
                        }
                    }

                    if !stage_data.solution_files.is_empty() {
                        println!("Solution Files (available in .solutions/):");
                        println!("{}", "-".repeat(50));
                        for file in &stage_data.solution_files {
                            println!("  {}", file.path);
                        }
                        println!();
                    }
                } else {
                    println!("Stage {} not found in project '{}' (total: {} stages).", stage, project_id, p.stages.len());
                }
            } else {
                println!("Project '{}' not found.", project_id);
            }
        }
        ProjectAction::View { project_id } => {
            if let Some(p) = engine.get_project(&project_id) {
                println!("{}", p.name);
                println!("{}", "=".repeat(p.name.len()));
                println!();
                println!("{}", p.description);
                println!();
                println!("Language: {}", p.language);
                println!("Difficulty: {}", p.difficulty);
                println!("Stages: {}", p.stages.len());

                let progress = engine.get_progress(&project_id);
                if let Some(ref prog) = progress {
                    println!("Progress: {}/{} ({:.0}%)", prog.completed_stages.len(), p.stages.len(), prog.percentage());
                    if prog.is_complete() {
                        println!("Status: Complete");
                    } else {
                        println!("Status: In Progress (current stage: {})", prog.current_stage + 1);
                    }
                }

                println!();
                for (i, stage) in p.stages.iter().enumerate() {
                    let marker = progress.as_ref().map(|prog| {
                        if prog.completed_stages.contains(&i) {
                            "[x]"
                        } else if i == prog.current_stage {
                            "[>]"
                        } else {
                            "[ ]"
                        }
                    }).unwrap_or("[ ]");

                    println!();
                    println!("Stage {}: {} {}", i + 1, marker, stage.name);
                    println!("{}", "-".repeat(60));
                    println!("{}", stage.description);
                    println!();

                    if !stage.files.is_empty() {
                        println!("Skeleton Files:");
                        for file in &stage.files {
                            println!("--- {} ---", file.path);
                            println!("{}", file.content);
                            println!();
                        }
                    }

                    if !stage.tests.is_empty() {
                        println!("Tests:");
                        for file in &stage.tests {
                            println!("--- {} ---", file.path);
                            println!("{}", file.content);
                            println!();
                        }
                    }

                    if !stage.solution_files.is_empty() {
                        println!("Solution Files (available in .solutions/):");
                        for file in &stage.solution_files {
                            println!("  {}", file.path);
                        }
                        println!();
                    }
                }

                println!();
                println!("Readme:");
                println!("{}", p.readme);
            } else {
                println!("Project '{}' not found.", project_id);
            }
        }
    }
    Ok(())
}
