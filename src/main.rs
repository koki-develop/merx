use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "merx", about = "Mermaid flowchart executor", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Mermaid flowchart program
    Run {
        /// Path to the .mmd file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file: _ } => {
            println!("Hello, merx!");
        }
    }
}
