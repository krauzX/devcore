use anyhow::Result;
use clap::{Parser, Subcommand};
use devcore_core::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "codetrail", about = "Change receipts + blast radius analysis")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize codetrail
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
            let mut analyzer = BlastRadiusAnalyzer::new(&path);
            analyzer.build_graph()?;
            println!("CodeTrail initialized at {}", path.display());
            Ok(())
        }
    }
}
