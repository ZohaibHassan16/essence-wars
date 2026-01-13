//! Event handling for the TUI
//!
//! Handles keyboard input, tick events, and converts them to Messages.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use super::app::Message;

/// Event handler that polls for terminal events
pub struct EventHandler {
    /// Tick rate for polling
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate in milliseconds
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for the next event and convert to a Message
    pub fn next(&self) -> io::Result<Option<Message>> {
        if event::poll(self.tick_rate)? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(self.handle_key(key)));
            }
        }
        // Tick event (no key pressed)
        Ok(Some(Message::Tick))
    }

    /// Convert a key event to a Message
    fn handle_key(&self, key: KeyEvent) -> Message {
        // Handle Ctrl+C globally
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Message::Quit;
        }

        Message::KeyPress(key)
    }
}

/// Helper functions for common key checks
pub fn is_quit_key(key: &KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Char('q') | KeyCode::Char('Q')
    ) && key.modifiers.is_empty()
}

pub fn is_back_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Esc && key.modifiers.is_empty()
}

pub fn is_help_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('?') && key.modifiers.is_empty()
}

pub fn is_enter_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Enter && key.modifiers.is_empty()
}

pub fn is_up_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Up && key.modifiers.is_empty()
}

pub fn is_down_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Down && key.modifiers.is_empty()
}

pub fn is_tab_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Tab && key.modifiers.is_empty()
}

pub fn is_shift_tab_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::BackTab
        || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
}
