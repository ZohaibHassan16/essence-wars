//! Screen implementations

mod analysis;
mod arena;
mod benchmarks;
mod help;
mod home;
mod tuning;
mod weights;

pub use analysis::AnalysisScreen;
pub use arena::ArenaScreen;
pub use benchmarks::BenchmarksScreen;
pub use help::HelpScreen;
pub use home::HomeScreen;
pub use tuning::TuningScreen;
pub use weights::WeightsScreen;

use crossterm::event::KeyEvent;

use super::app::Message;
use super::theme::Theme;
use ratatui::prelude::*;

/// All possible screens in the application
#[derive(Debug, Clone)]
pub enum Screen {
    Home(HomeScreen),
    Arena(Box<ArenaScreen>),
    Tuning(Box<TuningScreen>),
    Analysis(AnalysisScreen),
    Benchmarks(BenchmarksScreen),
    Weights(WeightsScreen),
    Help(HelpScreen),
}

impl Screen {
    /// Handle a key event and optionally return a message
    pub fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match self {
            Screen::Home(s) => s.handle_key(key),
            Screen::Arena(s) => s.handle_key(key),
            Screen::Tuning(s) => s.handle_key(key),
            Screen::Analysis(s) => s.handle_key(key),
            Screen::Benchmarks(s) => s.handle_key(key),
            Screen::Weights(s) => s.handle_key(key),
            Screen::Help(s) => s.handle_key(key),
        }
    }

    /// Handle a tick event (for animations, background tasks, etc.)
    pub fn tick(&mut self) -> Option<Message> {
        match self {
            Screen::Home(s) => s.tick(),
            Screen::Arena(s) => s.tick(),
            Screen::Tuning(s) => s.tick(),
            Screen::Analysis(s) => s.tick(),
            Screen::Benchmarks(s) => s.tick(),
            Screen::Weights(s) => s.tick(),
            Screen::Help(s) => s.tick(),
        }
    }
}

/// Trait for screen implementations
pub trait ScreenWidget {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme);
    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message>;
    fn tick(&mut self) -> Option<Message> { None }
}
