//! Benchmarks screen - engine performance analysis

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

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
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("  Benchmark runner coming in Phase 5..."),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Features planned:", Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("    - Run Criterion benchmarks (core engine ops)", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Run MCTS profiling (nodes/sec, depth)", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Run Arena throughput test (games/sec)", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Browse historical benchmark results", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    - Compare performance across versions", Style::default().fg(theme.fg_dim)),
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
            KeyHint::new("R", "Run All"),
            KeyHint::new("C", "Criterion"),
            KeyHint::new("M", "MCTS"),
            KeyHint::new("?", "Help"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, _key: &KeyEvent) -> Option<Message> {
        None
    }
}
