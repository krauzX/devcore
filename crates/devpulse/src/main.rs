use anyhow::Result;
use clap::{Parser, Subcommand};
use devcore_core::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "devpulse", about = "Developer workflow analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize devpulse
    Init {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path } => {
            let store = Store::open(&path)?;
            println!("DevPulse initialized at {}", path.display());
            Ok(())
        }
    }
}
