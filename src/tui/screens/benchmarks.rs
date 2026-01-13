//! Benchmarks screen - engine performance analysis

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;

/// Benchmarks screen state
#[derive(Debug, Clone)]
pub struct BenchmarksScreen {
    // TODO: Add benchmark results, history list, etc.
}

impl BenchmarksScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BenchmarksScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for BenchmarksScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Engine Benchmarks")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content placeholder
        let content = Paragraph::new("Benchmark runner coming in Phase 5...")
            .style(Style::default().fg(theme.fg_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(content, chunks[1]);

        // Footer
        let footer = Paragraph::new(" [Esc] Back  [R] Run All  [?] Help")
            .style(Style::default().fg(theme.fg_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_key(&mut self, _key: &KeyEvent) -> Option<Message> {
        None
    }
}
