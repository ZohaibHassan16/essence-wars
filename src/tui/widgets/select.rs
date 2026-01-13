//! Select widget (dropdown-style selector)

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use crate::tui::theme::Theme;

/// A select/dropdown widget
#[derive(Debug, Clone)]
pub struct Select {
    /// Available options
    options: Vec<String>,
    /// Currently selected index
    selected: usize,
    /// Widget label
    label: String,
    /// Is this widget focused?
    focused: bool,
}

impl Select {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            options,
            selected: 0,
            label: label.into(),
            focused: false,
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_value(&self) -> &str {
        self.options.get(self.selected).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected = index;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn next(&mut self) {
        if self.selected < self.options.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn options(&self) -> &[String] {
        &self.options
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let border_color = if self.focused {
            theme.border_focused
        } else {
            theme.border
        };

        let block = Block::default()
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render selected value with dropdown indicator
        let display_value = format!(
            " {} {}",
            self.selected_value(),
            if self.focused { "▼" } else { "▾" }
        );

        let style = if self.focused {
            Style::default().fg(theme.primary)
        } else {
            Style::default().fg(theme.fg)
        };

        buf.set_string(inner.x, inner.y, &display_value, style);
    }
}
