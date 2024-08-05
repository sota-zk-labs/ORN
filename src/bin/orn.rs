use std::cmp::PartialEq;

use clap::CommandFactory;
use clap::{Parser, Subcommand};
use orn::const_values::get_constant_values;
use orn::file_manager::FileManager;
use orn::gen_const::gen_consts;

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
        /// File paths, can be used multiple times, accept glob patterns
        #[arg(short, long = "path", default_value = "**/*.move")]
        paths: Vec<String>,
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
        Some(command) => match command {
            Commands::Version => {
                println!(env!("APP_VERSION"));
                return;
            }
            Commands::UpdateConst { paths } => {
                update_const(&paths).await;
                return;
            }
        },
        None => {
            Cli::command().print_help().unwrap();
            return;
        }
    }
}

async fn update_const(paths: &Vec<String>) {
    let constant_values = get_constant_values();
    println!("constant_values = {:#?}", constant_values);

    let file_manager = FileManager::load(paths).unwrap();
    file_manager
        .update(|file_content| gen_consts(&file_content, &constant_values))
        .unwrap()
}
