//! Weights screen - weight file management

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

/// Weights screen state
#[derive(Debug, Clone)]
pub struct WeightsScreen {
    // TODO: Add weight list, selected weight, preview data, etc.
}

impl WeightsScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WeightsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for WeightsScreen {
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
        let header = Paragraph::new(" Weight Manager")
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
            Line::from("  Weight browser coming in Phase 5..."),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Features planned:", Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("    - Browse data/weights/ directory", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Preview weight parameters", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Compare two weight files", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Promote weights to default", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Delete weight files", Style::default().fg(theme.fg_dim)),
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
            KeyHint::new("↑↓", "Navigate"),
            KeyHint::new("P", "Promote"),
            KeyHint::new("C", "Compare"),
            KeyHint::new("?", "Help"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, _key: &KeyEvent) -> Option<Message> {
        None
    }
}
