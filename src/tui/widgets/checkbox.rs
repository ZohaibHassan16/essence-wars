//! Checkbox widget

use ratatui::prelude::*;

use crate::tui::theme::Theme;

/// A checkbox widget
#[derive(Debug, Clone)]
pub struct Checkbox {
    /// Widget label
    label: String,
    /// Is checked?
    checked: bool,
    /// Is this widget focused?
    focused: bool,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            focused: false,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let checkbox = if self.checked { "[x]" } else { "[ ]" };

        let style = if self.focused {
            Style::default().fg(theme.primary)
        } else {
            Style::default().fg(theme.fg)
        };

        let label_style = if self.focused {
            Style::default().fg(theme.fg)
        } else {
            Style::default().fg(theme.fg_dim)
        };

        // Render checkbox part
        buf.set_string(area.x, area.y, checkbox, style);
        // Render label part
        buf.set_string(area.x + 4, area.y, &self.label, label_style);
    }
}
