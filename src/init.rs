use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::debug;

/// Initialize termsearch for the current shell.
pub fn handle_init() -> Result<()> {
    let zsh_config_dir = get_zsh_config_dir()?;
    debug!("ZSH configuration directory: {:?}", zsh_config_dir);

    let zsh_file_path = zsh_config_dir.join("termsearch.zsh");
    debug!("File path to termsearch.zsh: {:?}", zsh_file_path);

    let zsh_script = include_str!("../termsearch.zsh");
    fs::write(&zsh_file_path, zsh_script)
        .with_context(|| format!("Failed to write to {:?}", zsh_file_path))?;
    debug!("Successfully written termsearch.zsh script");

    let zshrc_path = zsh_config_dir.join(".zshrc");
    let source_command = format!("source {}", zsh_file_path.display());
    append_to_file(&zshrc_path, &source_command)?;
    debug!("Source command appended to .zshrc");

    println!("Successfully initialized termsearch. Restart your terminal to enable it.");

    Ok(())
}

/// Get ZSH configuration directory.
///
/// Returns
///
/// The path to ZSH configuration directory.
///
fn get_zsh_config_dir() -> Result<PathBuf> {
    let zsh_config_dir = std::env::var("ZDOTDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME environment variable not set");
            PathBuf::from(home)
        });

    Ok(zsh_config_dir)
}

/// Append content to a file if it's not already present.
///
/// # Arguments
///
/// * `file_path`: The path to the file.
/// * `content`: The content to append.
///
fn append_to_file(file_path: &PathBuf, content: &str) -> Result<()> {
    debug!("Append content to file: {:?}", file_path);
    if file_path.exists() {
        let existing_content = fs::read_to_string(file_path)?;
        if existing_content.contains(content) {
            debug!("Content already present in file");
            return Ok(());
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .with_context(|| format!("Failed to open or create {:?}", file_path))?;

    writeln!(file, "{}", content)?;
    debug!("Content appended successfully");

    Ok(())
}
