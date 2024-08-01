use std::cmp::PartialEq;

use clap::CommandFactory;
use clap::{Parser, Subcommand};
use tracing::info;

/// ORN.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(short, long)]
    version: bool,
}
#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Commands {
    /// Print
    Version,
    /// Update constant values in Move files
    UpdateConst {
        /// File paths, can be used multiple times
        #[arg(short, long, default_value = ".")]
        path: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    if args.version {
        println!(env!("APP_VERSION"));
        return;
    }
    match args.command {
        None => {
            Cli::command().print_help().unwrap();
            return;
        }
        Some(c) => {
            match c {
                Commands::Version => {
                    info!(env!("APP_VERSION"));
                    return;
                }
                Commands::UpdateConst { path: _ } => {
                    return;
                }
            }
        }
    }
}
