//! UI rendering coordinator
//!
//! Orchestrates rendering of the current screen.

use ratatui::prelude::*;

use super::app::App;
use super::screens::{Screen, ScreenWidget};
use super::theme::Theme;

/// Render the application
pub fn render(frame: &mut Frame, app: &App) {
    let theme = Theme::default();

    // Clear with background color
    let area = frame.area();
    frame.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(theme.bg)),
        area,
    );

    // Render current screen
    match app.screen() {
        Screen::Home(screen) => screen.render(frame, area, &theme),
        Screen::Arena(screen) => screen.render(frame, area, &theme),
        Screen::Tuning(screen) => screen.render(frame, area, &theme),
        Screen::Analysis(screen) => screen.render(frame, area, &theme),
        Screen::Benchmarks(screen) => screen.render(frame, area, &theme),
        Screen::Weights(screen) => screen.render(frame, area, &theme),
        Screen::Help(screen) => screen.render(frame, area, &theme),
    }
}
