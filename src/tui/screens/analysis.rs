//! Analysis screen - experiment results viewer

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

/// Analysis screen state
#[derive(Debug, Clone)]
pub struct AnalysisScreen {
    // TODO: Add experiment list, selected experiment, parsed data, etc.
}

impl AnalysisScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AnalysisScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for AnalysisScreen {
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
        let header = Paragraph::new(" Experiment Analysis")
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
            Line::from("  Experiment browser coming in Phase 5..."),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Features planned:", Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("    - Browse experiments/mcts/ directory", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Parse and display stats.csv data", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Show summary info from summary.txt", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Display version info for reproducibility", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - View generation-by-generation stats", Style::default().fg(theme.fg_dim)),
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
            KeyHint::new("Enter", "View"),
            KeyHint::new("?", "Help"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, _key: &KeyEvent) -> Option<Message> {
        None
    }
}
