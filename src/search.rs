use std::collections::HashMap;

use chrono::{DateTime, Utc};
use log::debug;

use crate::history::CommandEntry;

/// Weight for recency.
const RECENCY_WEIGHT: f32 = 0.6;
/// Weight for frequency.
const FREQUENCY_WEIGHT: f32 = 0.4;

/// Search commands based on a term.
///
/// # Arguments
///
/// * `term`: The search term.
/// * `history`: The list of command entries from the history.
/// * `max_results`: Maximum number of results to return.
///
/// # Returns
///
/// A vector of `CommandEntry` structs, sorted by their weighted score.
pub fn search_commands(
    term: &str,
    history: &[CommandEntry],
    max_results: usize,
) -> Vec<CommandEntry> {
    debug!("Search commands with term: {}", term);

    let term = term.to_lowercase();

    // Store the best score for each unique command
    let mut command_scores: HashMap<String, f32> = HashMap::new();

    // Calculate scores for each command
    for entry in history.iter() {
        // Calculate match score based on the search term
        let match_score = match entry.command.to_lowercase().find(&term) {
            Some(0) => 1.0, // Exact match at the start
            Some(pos) => 0.5 - pos as f32 / entry.command.len() as f32, // Partial match
            None => 0.0,    // No match
        };

        if match_score > 0.0 {
            // Calculate recency weight (more recent = higher weight)
            let seconds_ago = (Utc::now() - entry.timestamp).num_seconds() as f32;
            let recency_weight = 1.0 / (1.0 + seconds_ago.log10());

            // Calculate frequency weight (more frequent = higher weight)
            let frequency_weight = command_scores
                .get(&entry.command)
                .map_or(1.0, |&score| score + 1.0);

            // Combine scores with weights
            let total_score = match_score
                * (RECENCY_WEIGHT * recency_weight + FREQUENCY_WEIGHT * frequency_weight);

            // Update the best score for the command
            command_scores
                .entry(entry.command.clone())
                .and_modify(|e| *e = f32::max(*e, total_score))
                .or_insert(total_score);
        }
    }

    // Convert to a sorted vector
    let mut sorted_commands: Vec<_> = command_scores.into_iter().collect();
    sorted_commands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take the top results
    sorted_commands
        .into_iter()
        .take(max_results)
        .map(|(cmd, _)| CommandEntry {
            command: cmd,
            timestamp: DateTime::<Utc>::default(), // Timestamp not needed
        })
        .collect()
}

/// Get the most frequent commands.
///
/// * `history`: The list of command entries from the history.
/// * `max_results`: Maximum number of results to return.
///
/// # Returns
///
/// A vector of `CommandEntry` structs, sorted by their weighted score.
pub fn get_frequent_commands(history: &[CommandEntry], max_results: usize) -> Vec<CommandEntry> {
    debug!("Get frequent commands");

    // Store the frequency and most recent timestamp for each command
    let mut command_data: HashMap<String, (usize, DateTime<Utc>)> = HashMap::new();

    // Calculate frequency and recency
    for entry in history.iter() {
        command_data
            .entry(entry.command.clone())
            .and_modify(|(count, timestamp)| {
                *count += 1;
                if entry.timestamp > *timestamp {
                    *timestamp = entry.timestamp;
                }
            })
            .or_insert((1, entry.timestamp));
    }

    // Convert to a vector and calculate weighted scores
    let mut scored_commands: Vec<_> = command_data
        .into_iter()
        .map(|(cmd, (count, timestamp))| {
            // Calculate recency weight (more recent = higher weight)
            let seconds_ago = (Utc::now() - timestamp).num_seconds() as f32;
            let recency_weight = 1.0 / (1.0 + seconds_ago.log10());

            // Calculate frequency weight (more frequent = higher weight)
            let frequency_weight = count as f32;

            // Combine scores with weights
            let total_score = RECENCY_WEIGHT * recency_weight + FREQUENCY_WEIGHT * frequency_weight;

            (cmd, total_score)
        })
        .collect();

    // Sort by total score (descending)
    scored_commands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take the top results
    scored_commands
        .into_iter()
        .take(max_results)
        .map(|(cmd, _)| CommandEntry {
            command: cmd,
            timestamp: DateTime::<Utc>::default(), // Timestamp not needed
        })
        .collect()
}
