mod academic_cmd;
mod challenges_cmd;
mod git_cmd;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use academic_cmd::AcademicCmd;
use challenges_cmd::DsaCmd;
use git_cmd::GitCmd;

#[derive(Parser)]
#[command(name = "devcore", about = "DevCore CLI — academic tracker, git forge, DSA challenges")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Tui,
    Tray,
    #[command(name = "academic")]
    Academic(AcademicCmd),
    #[command(name = "git")]
    Git(GitCmd),
    #[command(name = "dsa")]
    Dsa(DsaCmd),
}

fn project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tui => {
            let root = project_root();
            let mut app = devcore_tui::App::new(&root)?;
            app.run()?;
        }
        Commands::Tray => {
            let root = project_root();
            let app = devcore_tray::TrayApp::new(&root)?;
            app.run()?;
        }
        Commands::Academic(cmd) => academic_cmd::run(cmd, &project_root())?,
        Commands::Git(cmd) => git_cmd::run(cmd, &project_root())?,
        Commands::Dsa(cmd) => challenges_cmd::run(cmd, &project_root())?,
    }

    Ok(())
}
