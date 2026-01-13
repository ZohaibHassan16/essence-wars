//! Main application state and message handling
//!
//! Implements the Elm architecture: Input -> Message -> Update -> Render

use crossterm::event::KeyEvent;

use crossterm::event::KeyCode;

use super::config::TuiConfig;
use super::events::{is_back_key, is_quit_key};
use super::screens::{HelpScreen, HomeScreen, Screen};
use super::state::AppState;
use super::widgets::{Toast, ToastContainer};

/// Application messages for state updates
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(Box<Screen>),
    GoBack,
    Quit,

    // Input events
    KeyPress(KeyEvent),
    Tick,

    // Toast notifications
    ShowToast(Toast),
}

/// Main application state
pub struct App {
    /// Current active screen
    screen: Screen,

    /// Navigation history for back button
    history: Vec<Screen>,

    /// Shared application state
    pub state: AppState,

    /// User configuration (persisted)
    pub config: TuiConfig,

    /// Toast notifications
    pub toasts: ToastContainer,

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
            config: TuiConfig::load(),
            toasts: ToastContainer::new(),
            quit: false,
        }
    }

    /// Save configuration to disk
    pub fn save_config(&self) {
        if let Err(e) = self.config.save() {
            eprintln!("Warning: Failed to save config: {}", e);
        }
    }

    /// Process a message and update state
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Navigate(screen) => {
                // Initialize screen with app state if needed
                let screen = self.initialize_screen(*screen);
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

                // Global '?' for help (unless already on help screen)
                if key.code == KeyCode::Char('?')
                    && !matches!(self.screen, Screen::Help(_)) {
                    self.update(Message::Navigate(Box::new(Screen::Help(HelpScreen::new()))));
                    return;
                }

                // Delegate to current screen
                if let Some(msg) = self.screen.handle_key(&key) {
                    self.update(msg);
                }
            }
            Message::Tick => {
                // Handle background task updates, etc.
                if let Some(msg) = self.screen.tick() {
                    self.update(msg);
                }
                // Cleanup expired toasts
                self.toasts.cleanup();
            }
            Message::ShowToast(toast) => {
                self.toasts.push(toast);
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
        let deck_ids: Vec<String> = self.state.decks.iter().map(|d| d.id.clone()).collect();
        match screen {
            Screen::Arena(arena) => {
                Screen::Arena(Box::new(arena.with_decks(&deck_ids)))
            }
            Screen::Tuning(tuning) => {
                Screen::Tuning(Box::new(tuning.with_decks(&deck_ids)))
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
