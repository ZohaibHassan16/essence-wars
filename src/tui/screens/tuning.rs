//! Tuning screen - weight optimization with live progress

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

/// Tuning screen state
#[derive(Debug, Clone)]
pub struct TuningScreen {
    // TODO: Add tuning config, progress state, etc.
}

impl TuningScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TuningScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for TuningScreen {
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
        let header = Paragraph::new(" Weight Tuning")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content placeholder
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("  Tuning wizard coming in Phase 4..."),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Features planned:", Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("    - Tuning mode selection (vs-random, vs-greedy, multi-opponent)", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - CMA-ES parameter configuration", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Live fitness/win-rate progress", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - ASCII fitness history chart", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Pause/resume/stop controls", Style::default().fg(theme.fg_dim)),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        );
        frame.render_widget(content, chunks[1]);

        // Footer
        let hints = vec![
            KeyHint::new("Esc", "Back"),
            KeyHint::new("?", "Help"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, _key: &KeyEvent) -> Option<Message> {
        None
    }
}
