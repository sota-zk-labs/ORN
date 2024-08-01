use std::cmp::PartialEq;

use clap::CommandFactory;
use clap::{Parser, Subcommand};
use orn::const_values::get_constant_values;

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
        Some(command) => {
            match command {
                Commands::Version => {
                    println!(env!("APP_VERSION"));
                    return;
                }
                Commands::UpdateConst { path: _ } => {
                    update_const().await;
                    return;
                }
            }
        }
        None => {
            Cli::command().print_help().unwrap();
            return;
        }
    }
}

async fn update_const() -> () {
    let constant_values = get_constant_values();
    println!("constant_values = {:#?}", constant_values);
}