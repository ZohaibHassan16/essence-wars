//! Progress bar widget with ETA

use std::time::{Duration, Instant};

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

use crate::tui::theme::Theme;

/// Progress bar with ETA calculation
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Current progress (0.0 to 1.0)
    progress: f64,
    /// Label text
    label: String,
    /// Start time for ETA calculation
    start_time: Option<Instant>,
    /// Show percentage?
    show_percentage: bool,
    /// Show ETA?
    show_eta: bool,
}

impl ProgressBar {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            progress: 0.0,
            label: label.into(),
            start_time: None,
            show_percentage: true,
            show_eta: true,
        }
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
        if self.start_time.is_none() && progress > 0.0 {
            self.start_time = Some(Instant::now());
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.progress = 0.0;
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.start_time = None;
    }

    /// Calculate estimated time remaining
    pub fn eta(&self) -> Option<Duration> {
        let start = self.start_time?;
        if self.progress <= 0.0 || self.progress >= 1.0 {
            return None;
        }

        let elapsed = start.elapsed();
        let total_estimated = elapsed.as_secs_f64() / self.progress;
        let remaining = total_estimated - elapsed.as_secs_f64();

        if remaining > 0.0 {
            Some(Duration::from_secs_f64(remaining))
        } else {
            None
        }
    }

    /// Format duration as human-readable string
    fn format_duration(d: Duration) -> String {
        let secs = d.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        // Build label with percentage and ETA
        let mut label_parts = vec![self.label.clone()];

        if self.show_percentage {
            label_parts.push(format!("{:.1}%", self.progress * 100.0));
        }

        if self.show_eta {
            if let Some(eta) = self.eta() {
                label_parts.push(format!("ETA: {}", Self::format_duration(eta)));
            }
        }

        let label = label_parts.join(" | ");

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(theme.primary).bg(theme.bg_alt))
            .ratio(self.progress)
            .label(Span::styled(label, Style::default().fg(theme.fg)));

        gauge.render(area, buf);
    }

    /// Render with a border and title
    pub fn render_with_border(&self, area: Rect, buf: &mut Buffer, theme: &Theme, title: &str) {
        let block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = block.inner(area);
        block.render(area, buf);

        self.render(inner, buf, theme);
    }
}

/// Simple progress display for current/total format
#[derive(Debug, Clone)]
pub struct ProgressDisplay {
    pub current: u32,
    pub total: u32,
    pub label: String,
    start_time: Option<Instant>,
}

impl ProgressDisplay {
    pub fn new(label: impl Into<String>, total: u32) -> Self {
        Self {
            current: 0,
            total,
            label: label.into(),
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.current = 0;
    }

    pub fn set_current(&mut self, current: u32) {
        self.current = current.min(self.total);
    }

    pub fn increment(&mut self) {
        self.current = (self.current + 1).min(self.total);
    }

    pub fn progress(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.current as f64 / self.total as f64
        }
    }

    pub fn eta(&self) -> Option<Duration> {
        let start = self.start_time?;
        if self.current == 0 || self.current >= self.total {
            return None;
        }

        let elapsed = start.elapsed();
        let rate = self.current as f64 / elapsed.as_secs_f64();
        let remaining = (self.total - self.current) as f64 / rate;

        if remaining > 0.0 && remaining.is_finite() {
            Some(Duration::from_secs_f64(remaining))
        } else {
            None
        }
    }

    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let progress = self.progress();

        // Build display text
        let mut text = format!("{}: {}/{}", self.label, self.current, self.total);

        if let Some(eta) = self.eta() {
            let secs = eta.as_secs();
            if secs < 60 {
                text.push_str(&format!(" (ETA: {}s)", secs));
            } else {
                text.push_str(&format!(" (ETA: {}m {}s)", secs / 60, secs % 60));
            }
        }

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(theme.primary).bg(theme.bg_alt))
            .ratio(progress)
            .label(Span::styled(text, Style::default().fg(theme.fg)));

        gauge.render(area, buf);
    }
}
