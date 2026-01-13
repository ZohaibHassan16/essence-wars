//! Home screen - main menu with recent activity

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{
    AnalysisScreen, ArenaScreen, BenchmarksScreen, HelpScreen, Screen, ScreenWidget,
    TuningScreen, WeightsScreen,
};
use crate::tui::app::Message;
use crate::tui::events::{is_down_key, is_enter_key, is_help_key, is_up_key};
use crate::tui::state::ExperimentInfo;
use crate::tui::theme::Theme;
use crate::tui::widgets::{Menu, MenuItem, StatusBar};
use crate::version;

/// Build the main menu items
fn build_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("Arena Match", "Run bot vs bot matches"),
        MenuItem::new("Tune Weights", "Optimize bot parameters"),
        MenuItem::new("Analysis", "View tuning experiment results"),
        MenuItem::new("Benchmarks", "Engine performance analysis"),
        MenuItem::new("Weight Manager", "Manage weight configurations"),
        MenuItem::new("Documentation", "View guides and references"),
        MenuItem::new("Exit", "Quit application"),
    ]
}

/// Home screen state
#[derive(Debug, Clone)]
pub struct HomeScreen {
    /// Currently selected menu item
    selected: usize,
    /// Menu items
    items: Vec<MenuItem>,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            items: build_menu_items(),
        }
    }

    /// Render recent activity section
    fn render_recent_activity(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        experiments: &[ExperimentInfo],
    ) {
        let block = Block::default()
            .title(" Recent Activity ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.bg_alt));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if experiments.is_empty() {
            let empty_msg = Paragraph::new("  No recent experiments found")
                .style(Style::default().fg(theme.fg_dim));
            frame.render_widget(empty_msg, inner);
            return;
        }

        // Show up to 5 recent experiments
        let lines: Vec<Line> = experiments
            .iter()
            .take(5)
            .map(|exp| {
                let status_style = if exp.best_win_rate.is_some() {
                    Style::default().fg(theme.success)
                } else {
                    Style::default().fg(theme.warning)
                };

                let win_rate_str = exp
                    .best_win_rate
                    .map(|wr| format!("{:.1}% WR", wr))
                    .unwrap_or_else(|| "in progress".to_string());

                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&exp.id, Style::default().fg(theme.fg)),
                    Span::styled(" - ", Style::default().fg(theme.fg_dim)),
                    Span::styled(&exp.mode, Style::default().fg(theme.info)),
                    Span::styled(" (", Style::default().fg(theme.fg_dim)),
                    Span::styled(win_rate_str, status_style),
                    Span::styled(")", Style::default().fg(theme.fg_dim)),
                ])
            })
            .collect();

        let activity = Paragraph::new(lines);
        frame.render_widget(activity, inner);
    }
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for HomeScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        self.render_with_state(frame, area, theme, &[]);
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        if is_up_key(key) {
            self.selected = Menu::select_previous(&self.items, self.selected);
            return None;
        }

        if is_down_key(key) {
            self.selected = Menu::select_next(&self.items, self.selected);
            return None;
        }

        if is_enter_key(key) {
            return match self.selected {
                0 => Some(Message::Navigate(Screen::Arena(ArenaScreen::new()))),
                1 => Some(Message::Navigate(Screen::Tuning(TuningScreen::new()))),
                2 => Some(Message::Navigate(Screen::Analysis(AnalysisScreen::new()))),
                3 => Some(Message::Navigate(Screen::Benchmarks(BenchmarksScreen::new()))),
                4 => Some(Message::Navigate(Screen::Weights(WeightsScreen::new()))),
                5 => Some(Message::Navigate(Screen::Help(HelpScreen::new()))),
                6 => Some(Message::Quit),
                _ => None,
            };
        }

        if is_help_key(key) {
            return Some(Message::Navigate(Screen::Help(HelpScreen::new())));
        }

        // Number keys for quick selection
        if let crossterm::event::KeyCode::Char(c) = key.code {
            if let Some(idx) = c.to_digit(10) {
                let idx = idx as usize;
                if idx >= 1 && idx <= self.items.len() {
                    self.selected = idx - 1;
                    return self.handle_key(&KeyEvent::from(crossterm::event::KeyCode::Enter));
                }
            }
        }

        None
    }
}

impl HomeScreen {
    /// Render with application state (for recent activity)
    pub fn render_with_state(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        experiments: &[ExperimentInfo],
    ) {
        // Main layout: header, content, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let version_str = version::version_string();
        let header = Paragraph::new(" Essence Wars Research Lab")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title_bottom(Line::from(format!(" {} ", version_str)).right_aligned())
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content area - split into prompt + menu + recent activity
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),  // Prompt
                Constraint::Length(9),  // Menu (7 items + border)
                Constraint::Length(1),  // Spacer
                Constraint::Min(5),     // Recent activity
            ])
            .split(chunks[1]);

        // Prompt
        let prompt = Paragraph::new("  What would you like to do?")
            .style(Style::default().fg(theme.fg_dim));
        frame.render_widget(prompt, content_chunks[0]);

        // Menu with horizontal margin
        let menu_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),  // Left margin
                Constraint::Min(50),    // Menu
                Constraint::Length(2),  // Right margin
            ])
            .split(content_chunks[1])[1];

        let menu = Menu::new(&self.items, theme).selected(self.selected);
        frame.render_widget(menu, menu_area);

        // Recent activity with horizontal margin
        let activity_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(50),
                Constraint::Length(2),
            ])
            .split(content_chunks[3])[1];

        self.render_recent_activity(frame, activity_area, theme, experiments);

        // Footer (status bar)
        let hints = StatusBar::home_hints();
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }
}
