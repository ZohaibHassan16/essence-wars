//! Text input widget wrapper

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use tui_input::Input;

use crate::tui::theme::Theme;

/// A text input widget
#[derive(Debug, Clone)]
pub struct TextInput {
    /// The underlying input state
    input: Input,
    /// Widget label
    label: String,
    /// Placeholder text
    placeholder: String,
    /// Is this widget focused?
    focused: bool,
}

impl TextInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            input: Input::default(),
            label: label.into(),
            placeholder: String::new(),
            focused: false,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn with_value(mut self, text: impl Into<String>) -> Self {
        self.input = self.input.clone().with_value(text.into());
        self
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.input = self.input.clone().with_value(value.into());
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Handle a key event, returns true if the event was consumed
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        use tui_input::InputRequest;

        match key.code {
            KeyCode::Char(c) => {
                self.input.handle(InputRequest::InsertChar(c));
                true
            }
            KeyCode::Backspace => {
                self.input.handle(InputRequest::DeletePrevChar);
                true
            }
            KeyCode::Delete => {
                self.input.handle(InputRequest::DeleteNextChar);
                true
            }
            KeyCode::Left => {
                self.input.handle(InputRequest::GoToPrevChar);
                true
            }
            KeyCode::Right => {
                self.input.handle(InputRequest::GoToNextChar);
                true
            }
            KeyCode::Home => {
                self.input.handle(InputRequest::GoToStart);
                true
            }
            KeyCode::End => {
                self.input.handle(InputRequest::GoToEnd);
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

        // Render input value or placeholder
        let value = self.input.value();
        let (display_text, style): (&str, Style) = if value.is_empty() && !self.placeholder.is_empty() {
            (self.placeholder.as_str(), Style::default().fg(theme.fg_dim))
        } else {
            (value, Style::default().fg(theme.fg))
        };

        let display = format!(" {}", display_text);
        buf.set_string(inner.x, inner.y, &display, style);

        // Show cursor position if focused
        if self.focused {
            let cursor_pos = self.input.visual_cursor();
            let cursor_x = inner.x + 1 + cursor_pos as u16;
            if cursor_x < inner.x + inner.width {
                if let Some(cell) = buf.cell_mut((cursor_x, inner.y)) {
                    cell.set_style(Style::default().bg(theme.primary).fg(theme.bg));
                }
            }
        }
    }
}
