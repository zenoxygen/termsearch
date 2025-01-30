use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use log::debug;
use regex::Regex;

/// A command entry with its command string and timestamp.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
}

/// Read shell history file and returns the last entries.
///
/// # Arguments
///
/// * `num_lines`: The maximum number of history lines to read.
///
/// # Returns
///
/// A vector of `CommandEntry` structs.
///
pub fn read_zsh_history(num_lines: usize) -> Result<Vec<CommandEntry>> {
    let file = File::open(get_zsh_history_file()?)?;
    let reader = BufReader::new(file);

    let timestamp_regex = Regex::new(r"^: (\d+):\d+;(.*)$")?;
    let mut history = VecDeque::with_capacity(num_lines);

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                debug!("Failed to read line {}: {}", line_num + 1, e);
                continue;
            }
        };

        if let Some(caps) = timestamp_regex.captures(&line) {
            if let (Some(timestamp_str), Some(command)) = (caps.get(1), caps.get(2)) {
                let timestamp = match timestamp_str.as_str().parse::<i64>() {
                    Ok(timestamp) => timestamp,
                    Err(e) => {
                        debug!("Failed to parse timestamp on line {}: {}", line_num + 1, e);
                        continue;
                    }
                };

                // Convert Unix timestamp to DateTime<Utc>
                let timestamp = match Utc.timestamp_opt(timestamp, 0).single() {
                    Some(timestamp) => timestamp,
                    None => {
                        debug!("Invalid timestamp on line {}", line_num + 1);
                        continue;
                    }
                };

                let command = command.as_str().trim_end().to_string();

                if !command.is_empty() {
                    if history.len() >= num_lines {
                        history.pop_front();
                    }
                    history.push_back(CommandEntry { command, timestamp });
                }
            }
        } else {
            debug!("Line {} does not match expected format", line_num + 1);
        }
    }

    debug!("Read {} history entries", history.len());
    Ok(history.into())
}

/// Get history file path from environment variables.
///
/// # Returns
///
/// The path to the shell history.
///
fn get_zsh_history_file() -> Result<PathBuf> {
    debug!("Get history file path");

    // Check the `HISTFILE` environment variable
    if let Ok(histfile) = env::var("HISTFILE") {
        let path = PathBuf::from(histfile);
        if path.is_file() {
            debug!("Use HISTFILE environment variable: {:?}", path);
            return Ok(path);
        }
    }

    // Fallback to default ZSH history file path
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let default_path = PathBuf::from(home).join(".zsh_history");

    if default_path.is_file() {
        debug!("Use default ZSH history file path: {:?}", default_path);
        Ok(default_path)
    } else {
        Err(anyhow::anyhow!(
            "ZSH history file not found at default location: {:?}",
            default_path
        ))
    }
}
