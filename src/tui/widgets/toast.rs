//! Toast notification widget
//!
//! Displays temporary messages to the user that automatically expire.

use std::time::{Duration, Instant};

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::theme::Theme;

/// Toast notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Success,
    Error,
    Warning,
    Info,
}

/// A toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    /// Create a new toast with default 3 second duration
    pub fn new(message: impl Into<String>, level: ToastLevel) -> Self {
        Self {
            message: message.into(),
            level,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    /// Create a success toast
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Success)
    }

    /// Create an error toast
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Error).with_duration(Duration::from_secs(5))
    }

    /// Create a warning toast
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Warning)
    }

    /// Create an info toast
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Info)
    }

    /// Set custom duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Check if the toast has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Get remaining time as a fraction (1.0 = full, 0.0 = expired)
    pub fn remaining_fraction(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        (1.0 - elapsed / total).max(0.0)
    }
}

/// Toast container that manages multiple toasts
#[derive(Debug, Clone, Default)]
pub struct ToastContainer {
    toasts: Vec<Toast>,
}

impl ToastContainer {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    /// Add a toast
    pub fn push(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    /// Remove expired toasts
    pub fn cleanup(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    /// Check if there are any toasts to display
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Get the toasts
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    /// Render toasts in the top-right corner
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.toasts.is_empty() {
            return;
        }

        // Calculate toast area (top-right, stacked vertically)
        let toast_width = 50.min(area.width.saturating_sub(4));
        let mut y = 1;

        for toast in &self.toasts {
            if y >= area.height.saturating_sub(3) {
                break; // Don't overflow
            }

            let toast_height = 3;
            let x = area.width.saturating_sub(toast_width + 2);

            let toast_area = Rect::new(x, y, toast_width, toast_height);

            // Get colors based on level
            let (border_color, icon) = match toast.level {
                ToastLevel::Success => (theme.success, "✓"),
                ToastLevel::Error => (theme.error, "✗"),
                ToastLevel::Warning => (theme.warning, "⚠"),
                ToastLevel::Info => (theme.info, "ℹ"),
            };

            // Clear the area behind the toast
            frame.render_widget(Clear, toast_area);

            // Render toast
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme.bg_alt));

            let inner = block.inner(toast_area);
            frame.render_widget(block, toast_area);

            // Render message with icon
            let text = format!(" {} {}", icon, toast.message);
            let para = Paragraph::new(text)
                .style(Style::default().fg(theme.fg));
            frame.render_widget(para, inner);

            y += toast_height + 1;
        }
    }
}
