mod history;
mod logger;
mod search;
mod ui;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{debug, LevelFilter};

use crate::history::read_zsh_history;
use crate::logger::Logger;
use crate::search::{get_frequent_commands, search_commands};
use crate::ui::TerminalUi;

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

/// Initialize termsearch for the current shell.
pub fn handle_init() -> Result<()> {
    let zsh_script = include_str!("../termsearch.zsh");
    println!("{}", zsh_script);

    Ok(())
}

/// Handle the search command.
///
/// # Arguments
///
/// * `term`: The search term (optional).
/// * `max_history`: Maximum number of history entries to read.
/// * `max_results`: Maximum number of results to display.
/// * `output_file`: File to write the selected command (optional).
///
pub fn handle_search(
    term: Option<String>,
    max_history: usize,
    max_results: usize,
    output_file: Option<String>,
) -> Result<()> {
    // Read ZSH history
    let history = read_zsh_history(max_history)?;
    debug!("Read {} history entries", history.len());

    // Initialize UI
    let mut ui = TerminalUi::new(max_results, history)?;

    // Perform search (display most frequent commands if no term provided)
    let initial_matches = if let Some(term) = &term {
        search_commands(term, &ui.history, max_results)
    } else {
        get_frequent_commands(&ui.history, max_results)
    };

    // Display initial results
    ui.set_initial_results(initial_matches)?;

    // Run the UI and get the selected command
    if let Some(selected_command) = ui.run(term)? {
        debug!("Selected command: {}", selected_command);
        if let Some(output_file) = output_file {
            debug!("Write command to output file: {}", output_file);
            let mut file = File::create(&output_file)?;
            writeln!(file, "commandline\t{}", selected_command)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Get the home directory
    let home_dir = std::env::var("HOME").expect("HOME environment variable not set");

    // Define the log file path in the user's home directory
    let log_file_path = PathBuf::from(home_dir).join(".termsearch.log");

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
        Command::Init => handle_init()?,
        Command::Search {
            term,
            output_file,
            max_history,
            max_results,
        } => {
            handle_search(term, max_history, max_results, output_file)?;
        }
    }

    Ok(())
}
