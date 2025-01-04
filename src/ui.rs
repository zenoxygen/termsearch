use std::io::{stdout, Stdout, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::debug;

use crate::history::CommandEntry;
use crate::search::{get_frequent_commands, search_commands};

/// Actions after handling a key event.
enum KeyAction {
    /// Select a command and return it.
    Select(String),
    /// Continue the event loop.
    Continue,
    /// Exit the program.
    Exit,
}

/// Manage the terminal UI state.
pub struct TerminalUi {
    /// The full history of commands.
    pub history: Vec<CommandEntry>,
    /// The list of commands matching the current search term.
    matches: Vec<CommandEntry>,
    /// The current search term entered by the user.
    input: String,
    /// The index of the currently selected command in the matches list.
    selected_index: usize,
    /// The current search term (optional, used for initial search).
    term: Option<String>,
    /// The maximum number of results to display.
    num_results: usize,
    /// The standard output handle for rendering the UI.
    stdout: Stdout,
}

impl TerminalUi {
    /// Create a new `TerminalUi` instance.
    ///
    /// # Arguments
    ///
    /// * `num_results`: Maximum number of results to display.
    /// * `history`: Vector of command entries from shell history.
    ///
    pub fn new(num_results: usize, history: Vec<CommandEntry>) -> Result<Self> {
        debug!("Initialize UI");

        terminal::enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, Hide)
            .context("Failed to enter alternate screen and hide cursor")?;

        Ok(Self {
            stdout,
            history,
            matches: Vec::new(),
            input: String::new(),
            selected_index: 0,
            term: None,
            num_results,
        })
    }

    /// Set the initial search results and update the UI.
    ///
    /// # Arguments
    ///
    /// * `initial_matches`: Vector of initial command entries to display.
    ///
    pub fn set_initial_results(&mut self, initial_matches: Vec<CommandEntry>) -> Result<()> {
        debug!("Set initial results, count: {}", initial_matches.len());
        self.matches = initial_matches;
        self.selected_index = 0;
        self.draw_matches()
    }

    /// Clean up the terminal UI state.
    pub fn cleanup(&mut self) -> Result<()> {
        debug!("Cleanup UI");
        terminal::disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(self.stdout, Show, ResetColor, LeaveAlternateScreen)
            .context("Failed to restore terminal state")?;
        Ok(())
    }

    /// Run the terminal UI and return the selected command if any.
    ///
    /// # Arguments
    ///
    /// * `initial_term`: Optional initial search term.
    ///
    pub fn run(&mut self, initial_term: Option<String>) -> Result<Option<String>> {
        debug!("Run UI");

        if let Some(term) = initial_term {
            self.input = term;
            self.term = Some(self.input.clone());
        }

        // Initial UI setup
        self.draw_input_buffer()?;
        self.draw_matches()?;

        // Main event loop
        loop {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match self.handle_key_event(key_event)? {
                        KeyAction::Select(command) => return Ok(Some(command)),
                        KeyAction::Continue => {}
                        KeyAction::Exit => {
                            self.cleanup()?;
                            return Ok(None);
                        }
                    }
                }
            }
            // Add a small delay to reduce CPU usage
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Handle a key event and return the appropriate action.
    ///
    /// # Arguments
    ///
    /// * `key_event`: The key event to handle.
    ///
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<KeyAction> {
        match key_event.code {
            // Exit handling
            KeyCode::Esc => {
                debug!("Escape key pressed");
                Ok(KeyAction::Exit)
            }
            KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                debug!("Ctrl+C pressed");
                Ok(KeyAction::Exit)
            }
            KeyCode::Char('d') if key_event.modifiers == KeyModifiers::CONTROL => {
                debug!("Ctrl+D pressed");
                Ok(KeyAction::Exit)
            }

            // Character input
            KeyCode::Char(c) => {
                debug!("Character '{}' pressed", c);
                self.input.push(c);
                self.term = Some(self.input.clone());
                self.update_matches();
                self.draw_matches()?;
                Ok(KeyAction::Continue)
            }

            // Backspace handling
            KeyCode::Backspace => {
                debug!("Backspace pressed");
                self.input.pop();
                self.term = Some(self.input.clone());
                self.update_matches();
                self.draw_matches()?;
                Ok(KeyAction::Continue)
            }

            // Navigation down
            KeyCode::Down | KeyCode::Tab => {
                debug!("Down/Tab key pressed");
                if self.selected_index >= self.matches.len().saturating_sub(1) {
                    self.selected_index = 0;
                } else {
                    self.selected_index += 1;
                }
                self.draw_matches()?;
                Ok(KeyAction::Continue)
            }

            // Navigation up
            KeyCode::Up | KeyCode::BackTab => {
                debug!("Up/Shift+Tab key pressed");
                if self.selected_index == 0 {
                    self.selected_index = self.matches.len().saturating_sub(1);
                } else {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
                self.draw_matches()?;
                Ok(KeyAction::Continue)
            }

            // Command selection
            KeyCode::Enter => {
                debug!("Enter key pressed");
                if let Some(command_entry) = self.matches.get(self.selected_index) {
                    Ok(KeyAction::Select(command_entry.command.clone()))
                } else {
                    Ok(KeyAction::Continue)
                }
            }

            // Ignore other keys
            _ => {
                debug!("Other key pressed");
                Ok(KeyAction::Continue)
            }
        }
    }

    /// Update the matches based on the current search term.
    fn update_matches(&mut self) {
        debug!("Update matches");

        self.matches = if let Some(term) = &self.term {
            if !term.is_empty() {
                search_commands(term, &self.history, self.num_results)
            } else {
                get_frequent_commands(&self.history, self.num_results)
            }
        } else {
            get_frequent_commands(&self.history, self.num_results)
        };

        self.selected_index = 0;
    }

    /// Draw the input buffer with the current search term.
    fn draw_input_buffer(&mut self) -> Result<()> {
        debug!("Draw input buffer");
        let (width, _) = terminal::size()?;

        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print(format!(
                "{:width$}",
                format!("> {}", self.input),
                width = width as usize
            ))
        )?;
        self.stdout.flush()?;

        Ok(())
    }

    /// Draw the matches in the terminal with highlighting.
    fn draw_matches(&mut self) -> Result<()> {
        debug!("Draw matches");
        let (_, height) = terminal::size()?;

        // Clear existing matches
        for i in 0..height {
            queue!(
                self.stdout,
                cursor::MoveTo(0, 1 + i),
                terminal::Clear(terminal::ClearType::CurrentLine)
            )?;
        }

        // Draw matches with highlighting
        for (i, command_entry) in self.matches.iter().enumerate() {
            queue!(
                self.stdout,
                cursor::MoveTo(0, (i + 1) as u16),
                SetForegroundColor(if i == self.selected_index {
                    Color::Black
                } else {
                    Color::Reset
                }),
                SetBackgroundColor(if i == self.selected_index {
                    Color::White
                } else {
                    Color::Reset
                }),
            )?;

            // If there's a search term, highlight matching parts
            if let Some(term) = &self.term {
                let command = &command_entry.command;
                if let Some(match_start) = command.to_lowercase().find(&term.to_lowercase()) {
                    let match_end = match_start + term.len();

                    // Print before match
                    queue!(self.stdout, Print(&command[..match_start]))?;

                    // Print match with highlight
                    queue!(
                        self.stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(&command[match_start..match_end]),
                        SetForegroundColor(if i == self.selected_index {
                            Color::Black
                        } else {
                            Color::Reset
                        }),
                    )?;

                    // Print after match
                    queue!(self.stdout, Print(&command[match_end..]))?;
                } else {
                    queue!(self.stdout, Print(&command_entry.command))?;
                }
            } else {
                queue!(self.stdout, Print(&command_entry.command))?;
            }

            queue!(self.stdout, ResetColor)?;
        }

        // Redraw input buffer
        self.draw_input_buffer()?;
        self.stdout.flush()?;

        Ok(())
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
