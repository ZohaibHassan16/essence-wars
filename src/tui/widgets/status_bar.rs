//! Status bar widget for bottom of screen

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::theme::Theme;

/// A key hint for the status bar
#[derive(Debug, Clone)]
pub struct KeyHint {
    pub key: String,
    pub action: String,
}

impl KeyHint {
    pub fn new(key: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
        }
    }
}

/// Status bar with key hints
pub struct StatusBar<'a> {
    hints: &'a [KeyHint],
    theme: &'a Theme,
}

impl<'a> StatusBar<'a> {
    pub fn new(hints: &'a [KeyHint], theme: &'a Theme) -> Self {
        Self { hints, theme }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build hint text
        let mut spans = vec![Span::raw(" ")];
        for (i, hint) in self.hints.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Style::default().fg(self.theme.fg_dim)));
            }
            spans.push(Span::styled(
                format!("[{}]", hint.key),
                Style::default().fg(self.theme.primary),
            ));
            spans.push(Span::styled(
                format!(" {}", hint.action),
                Style::default().fg(self.theme.fg_dim),
            ));
        }

        let line = Line::from(spans);
        Paragraph::new(line).render(inner, buf);
    }
}

/// Common status bar configurations
impl<'a> StatusBar<'a> {
    /// Standard hints for home screen
    pub fn home_hints() -> Vec<KeyHint> {
        vec![
            KeyHint::new("↑↓", "Navigate"),
            KeyHint::new("Enter", "Select"),
            KeyHint::new("Q", "Quit"),
            KeyHint::new("?", "Help"),
        ]
    }

    /// Standard hints for sub-screens
    pub fn sub_screen_hints() -> Vec<KeyHint> {
        vec![
            KeyHint::new("Esc", "Back"),
            KeyHint::new("?", "Help"),
        ]
    }
}
