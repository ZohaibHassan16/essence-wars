//! Number input widget

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use crate::tui::theme::Theme;

/// A number input widget
#[derive(Debug, Clone)]
pub struct NumberInput {
    /// Current value
    value: i64,
    /// Minimum value
    min: i64,
    /// Maximum value
    max: i64,
    /// Widget label
    label: String,
    /// Is this widget focused?
    focused: bool,
}

impl NumberInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            value: 0,
            min: i64::MIN,
            max: i64::MAX,
            label: label.into(),
            focused: false,
        }
    }

    pub fn with_value(mut self, val: i64) -> Self {
        self.value = val.clamp(self.min, self.max);
        self
    }

    pub fn min(mut self, min: i64) -> Self {
        self.min = min;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    pub fn max(mut self, max: i64) -> Self {
        self.max = max;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn set_value(&mut self, value: i64) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn decrement(&mut self) {
        if self.value > self.min {
            self.value -= 1;
        }
    }

    pub fn increment_by(&mut self, amount: i64) {
        self.value = (self.value + amount).clamp(self.min, self.max);
    }

    pub fn decrement_by(&mut self, amount: i64) {
        self.value = (self.value - amount).clamp(self.min, self.max);
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                self.decrement();
                true
            }
            KeyCode::Right => {
                self.increment();
                true
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.increment();
                true
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.decrement();
                true
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Append digit to value
                let digit = c.to_digit(10).unwrap() as i64;
                let new_value = self.value.saturating_mul(10).saturating_add(digit);
                self.value = new_value.clamp(self.min, self.max);
                true
            }
            KeyCode::Backspace => {
                // Remove last digit
                self.value /= 10;
                true
            }
            _ => false,
        }
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

        // Show value with increment/decrement hints if focused
        let display = if self.focused {
            format!(" < {} >", self.value)
        } else {
            format!(" {}", self.value)
        };

        let style = if self.focused {
            Style::default().fg(theme.primary)
        } else {
            Style::default().fg(theme.fg)
        };

        buf.set_string(inner.x, inner.y, &display, style);
    }
}
