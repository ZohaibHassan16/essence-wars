//! Help screen - documentation and keyboard shortcuts

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::events::is_back_key;
use crate::tui::theme::Theme;

const HELP_TEXT: &str = r#"
Essence Wars Research Lab - Help

GLOBAL SHORTCUTS
  Q           Quit application
  Esc         Go back / Cancel
  ?           Show this help

NAVIGATION
  ↑ / ↓       Move selection up / down
  Enter       Select / Activate
  Tab         Next field
  Shift+Tab   Previous field
  1-7         Quick jump to menu item (Home screen)

ARENA
  Tab         Cycle through form fields
  Space       Toggle checkboxes
  Enter       Start match

TUNING
  Space       Pause / Resume
  S           Save checkpoint
  L           View log

ANALYSIS
  ↑ / ↓       Navigate experiments
  Enter       View details
  PgUp/PgDn   Page through data

BENCHMARKS
  R           Run all benchmarks
  C           Run Criterion only
  M           Run MCTS profile only
  A           Run Arena throughput only

WEIGHTS
  P           Promote to default
  C           Compare with another file
  V           View full weights
  D           Delete weight file
"#;

/// Help screen state
#[derive(Debug, Clone)]
pub struct HelpScreen {
    scroll: u16,
}

impl HelpScreen {
    pub fn new() -> Self {
        Self { scroll: 0 }
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
        let header = Paragraph::new(" Help & Documentation")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content
        let content = Paragraph::new(HELP_TEXT)
            .style(Style::default().fg(theme.fg))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(content, chunks[1]);

        // Footer
        let footer = Paragraph::new(" [Esc] Back  [↑↓] Scroll")
            .style(Style::default().fg(theme.fg_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        use crate::tui::events::{is_down_key, is_up_key};

        if is_back_key(key) {
            return Some(Message::GoBack);
        }

        if is_up_key(key) && self.scroll > 0 {
            self.scroll -= 1;
        }

        if is_down_key(key) {
            self.scroll += 1;
        }

        None
    }
}
