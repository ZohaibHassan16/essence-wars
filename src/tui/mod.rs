//! Essence Wars TUI - Terminal User Interface for research workflows
//!
//! A full-featured TUI built with ratatui, featuring:
//! - Arena match configuration and execution
//! - Weight tuning with live progress
//! - Experiment analysis viewer
//! - Engine benchmark runner
//! - Weight file management

mod app;
mod config;
mod events;
mod state;
mod theme;
mod ui;

pub mod screens;
pub mod tasks;
pub mod widgets;

pub use app::{App, Message};
pub use config::TuiConfig;
pub use events::EventHandler;
pub use state::AppState;
pub use theme::Theme;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

/// Run the TUI application
pub fn run() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let mut event_handler = EventHandler::new(250); // 250ms tick rate

    let result = run_app(&mut terminal, &mut app, &mut event_handler);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> io::Result<()> {
    loop {
        // Render
        terminal.draw(|frame| ui::render(frame, app))?;

        // Handle events
        if let Some(msg) = event_handler.next()? {
            app.update(msg);
        }

        // Check for quit
        if app.should_quit() {
            // Save config on exit
            app.save_config();
            return Ok(());
        }
    }
}
