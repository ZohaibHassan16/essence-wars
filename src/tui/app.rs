//! Main application state and message handling
//!
//! Implements the Elm architecture: Input -> Message -> Update -> Render

use crossterm::event::KeyEvent;

use super::events::{is_back_key, is_quit_key};
use super::screens::{HomeScreen, Screen};
use super::state::AppState;

/// Application messages for state updates
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(Screen),
    GoBack,
    Quit,

    // Input events
    KeyPress(KeyEvent),
    Tick,
}

/// Main application state
pub struct App {
    /// Current active screen
    screen: Screen,

    /// Navigation history for back button
    history: Vec<Screen>,

    /// Shared application state
    pub state: AppState,

    /// Should quit?
    quit: bool,
}

impl App {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            screen: Screen::Home(HomeScreen::new()),
            history: Vec::new(),
            state: AppState::new(),
            quit: false,
        }
    }

    /// Process a message and update state
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Navigate(screen) => {
                // Initialize screen with app state if needed
                let screen = self.initialize_screen(screen);
                let old = std::mem::replace(&mut self.screen, screen);
                self.history.push(old);
            }
            Message::GoBack => {
                if let Some(screen) = self.history.pop() {
                    self.screen = screen;
                }
            }
            Message::Quit => {
                self.quit = true;
            }
            Message::KeyPress(key) => {
                // Global key handling
                if is_quit_key(&key) {
                    self.quit = true;
                    return;
                }
                if is_back_key(&key) && !self.history.is_empty() {
                    self.update(Message::GoBack);
                    return;
                }

                // Delegate to current screen
                if let Some(msg) = self.screen.handle_key(&key) {
                    self.update(msg);
                }
            }
            Message::Tick => {
                // Handle background task updates, etc.
                self.screen.tick();
            }
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Get the current screen
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// Get mutable access to the current screen
    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// Initialize a screen with app state data
    fn initialize_screen(&self, screen: Screen) -> Screen {
        match screen {
            Screen::Arena(arena) => {
                let deck_ids: Vec<String> = self.state.decks.iter().map(|d| d.id.clone()).collect();
                Screen::Arena(arena.with_decks(&deck_ids))
            }
            // Other screens that need initialization can be added here
            other => other,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
