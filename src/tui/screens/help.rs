//! Help screen - documentation and keyboard shortcuts

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::events::{is_back_key, is_down_key, is_up_key};
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

/// Help screen state
#[derive(Debug, Clone)]
pub struct HelpScreen {
    scroll: u16,
}

impl HelpScreen {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    fn build_help_content(theme: &Theme) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  GLOBAL SHORTCUTS", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Q           ", Style::default().fg(theme.fg)),
                Span::styled("Quit application", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Esc         ", Style::default().fg(theme.fg)),
                Span::styled("Go back / Cancel", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    ?           ", Style::default().fg(theme.fg)),
                Span::styled("Show this help", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  NAVIGATION", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Up/Down     ", Style::default().fg(theme.fg)),
                Span::styled("Move selection up / down", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Enter       ", Style::default().fg(theme.fg)),
                Span::styled("Select / Activate", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Tab         ", Style::default().fg(theme.fg)),
                Span::styled("Next field", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Shift+Tab   ", Style::default().fg(theme.fg)),
                Span::styled("Previous field", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    1-7         ", Style::default().fg(theme.fg)),
                Span::styled("Quick jump to menu item (Home screen)", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ARENA", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Tab         ", Style::default().fg(theme.fg)),
                Span::styled("Cycle through form fields", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Space       ", Style::default().fg(theme.fg)),
                Span::styled("Toggle checkboxes", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Enter       ", Style::default().fg(theme.fg)),
                Span::styled("Start match", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  TUNING", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Space       ", Style::default().fg(theme.fg)),
                Span::styled("Pause / Resume", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    S           ", Style::default().fg(theme.fg)),
                Span::styled("Save checkpoint", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    L           ", Style::default().fg(theme.fg)),
                Span::styled("View log", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  TUNING (After Completion)", Style::default().fg(theme.success).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    A           ", Style::default().fg(theme.fg)),
                Span::styled("View Analysis - open experiment results", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    W           ", Style::default().fg(theme.fg)),
                Span::styled("Compare Weights - diff new vs default", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    P           ", Style::default().fg(theme.fg)),
                Span::styled("Promote Weights - set as new default", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    R           ", Style::default().fg(theme.fg)),
                Span::styled("Run new tuning session", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ANALYSIS", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Up/Down     ", Style::default().fg(theme.fg)),
                Span::styled("Navigate experiments", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    Enter       ", Style::default().fg(theme.fg)),
                Span::styled("View details", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    PgUp/PgDn   ", Style::default().fg(theme.fg)),
                Span::styled("Page through data", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  BENCHMARKS", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    R           ", Style::default().fg(theme.fg)),
                Span::styled("Run all benchmarks", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    C           ", Style::default().fg(theme.fg)),
                Span::styled("Run Criterion only", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    M           ", Style::default().fg(theme.fg)),
                Span::styled("Run MCTS profile only", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    A           ", Style::default().fg(theme.fg)),
                Span::styled("Run Arena throughput only", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  WEIGHTS", Style::default().fg(theme.primary).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    P           ", Style::default().fg(theme.fg)),
                Span::styled("Promote to default", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    C           ", Style::default().fg(theme.fg)),
                Span::styled("Compare with another file", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    V           ", Style::default().fg(theme.fg)),
                Span::styled("View full weights", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(vec![
                Span::styled("    D           ", Style::default().fg(theme.fg)),
                Span::styled("Delete weight file", Style::default().fg(theme.fg_dim)),
            ]),
            Line::from(""),
        ]
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for HelpScreen {
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
        let header = Paragraph::new(" Help & Keyboard Shortcuts")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content
        let help_content = Self::build_help_content(theme);
        let content = Paragraph::new(help_content)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(content, chunks[1]);

        // Footer
        let hints = vec![
            KeyHint::new("Esc", "Back"),
            KeyHint::new("↑↓", "Scroll"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        if is_back_key(key) {
            return Some(Message::GoBack);
        }

        if is_up_key(key) && self.scroll > 0 {
            self.scroll -= 1;
        }

        if is_down_key(key) {
            self.scroll += 1;
        }

        // Page up/down
        if let crossterm::event::KeyCode::PageUp = key.code {
            self.scroll = self.scroll.saturating_sub(10);
        }
        if let crossterm::event::KeyCode::PageDown = key.code {
            self.scroll += 10;
        }

        None
    }
}
