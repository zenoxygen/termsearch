mod history;
mod init;
mod logger;
mod search;
mod ui;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{debug, LevelFilter};

use crate::logger::Logger;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A minimalist and super fast terminal history search tool."
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize for the current shell.
    Init,
    /// Search through the shell history.
    Search {
        /// The search term (optional).
        term: Option<String>,
        /// The output file (optional).
        #[arg(short = 'o')]
        output_file: Option<String>,
        /// Maximum number of history lines to read.
        #[arg(short = 'm', long = "max-history", default_value = "10000")]
        max_history: usize,
        /// Maximum number of results to display.
        #[arg(short = 'r', long = "max-results", default_value = "10")]
        max_results: usize,
    },
}

fn main() -> Result<()> {
    // Get the home directory
    let home_dir = std::env::var("HOME").expect("HOME environment variable not set");

    // Define the log file path in the user's home directory
    let log_file_path = PathBuf::from(home_dir).join("termsearch.log");

    // Set the log level based on the TERMSEARCH_LOG environment variable (default to INFO)
    let file_log_level = std::env::var("TERMSEARCH_LOG")
        .map(|val| match val.to_uppercase().as_str() {
            "TRACE" => LevelFilter::Trace,
            "DEBUG" => LevelFilter::Debug,
            "WARN" => LevelFilter::Warn,
            "ERROR" => LevelFilter::Error,
            _ => LevelFilter::Info,
        })
        .unwrap_or(LevelFilter::Info);

    // Initialize the logger with the specified file path and a stdout level of Off
    let logger = Logger::new(log_file_path)?;
    log::set_boxed_logger(Box::new(logger)).map(|()| log::set_max_level(file_log_level))?;

    // Get the version from Cargo at compile time
    let version = env!("CARGO_PKG_VERSION");
    debug!("Start termsearch v{}", version);

    let args = Args::parse();

    match args.command {
        Command::Init => init::handle_init()?,
        Command::Search {
            term,
            output_file,
            max_history,
            max_results,
        } => {
            search::handle_search(term, max_history, max_results, output_file)?;
        }
    }

    Ok(())
}
