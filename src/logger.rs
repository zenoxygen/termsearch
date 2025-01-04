use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Local};
use log::{Level, Log, Metadata, Record};

// A simple logger that writes to a file
pub struct Logger {
    file: Mutex<File>,
}

impl Logger {
    pub fn new(log_file_path: PathBuf) -> Result<Self, std::io::Error> {
        // Create the log file or append to it if it exists
        let log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file_path)?;

        Ok(Logger {
            file: Mutex::new(log_file),
        })
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Enable all messages at or above the configured level
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Get the current time using chrono
            let now: DateTime<Local> = Local::now();
            let formatted_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let log_msg = format!("{} [{}] {}", formatted_time, record.level(), record.args());

            // Write to file
            if let Ok(mut file) = self.file.lock() {
                writeln!(file, "{}", log_msg).ok();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            file.flush().ok();
        }
    }
}
