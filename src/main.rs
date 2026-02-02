use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use merx::parser;

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

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file.display(), e);
                    return ExitCode::from(2);
                }
            };

            let flowchart = match parser::parse(&content) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    return ExitCode::from(2);
                }
            };

            match serde_json::to_string_pretty(&flowchart) {
                Ok(json) => {
                    println!("{}", json);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("JSON serialization error: {}", e);
                    ExitCode::from(1)
                }
            }
        }
    }
}
