//! Home screen - main menu

use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{
    AnalysisScreen, ArenaScreen, BenchmarksScreen, HelpScreen, Screen, ScreenWidget,
    TuningScreen, WeightsScreen,
};
use crate::tui::app::Message;
use crate::tui::events::{is_down_key, is_enter_key, is_help_key, is_up_key};
use crate::tui::theme::Theme;
use crate::version;

/// Menu item definition
struct MenuItem {
    label: &'static str,
    description: &'static str,
}

const MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Arena Match",
        description: "Run bot vs bot matches",
    },
    MenuItem {
        label: "Tune Weights",
        description: "Optimize bot parameters",
    },
    MenuItem {
        label: "Analysis",
        description: "View tuning experiment results",
    },
    MenuItem {
        label: "Benchmarks",
        description: "Engine performance analysis",
    },
    MenuItem {
        label: "Weight Manager",
        description: "Manage weight configurations",
    },
    MenuItem {
        label: "Documentation",
        description: "View guides and references",
    },
    MenuItem {
        label: "Exit",
        description: "Quit application",
    },
];

/// Home screen state
#[derive(Debug, Clone)]
pub struct HomeScreen {
    /// Currently selected menu item
    selected: usize,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for HomeScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
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
        let header = Paragraph::new(format!(" Essence Wars Research Lab"))
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title_bottom(Line::from(format!(" {} ", version_str)).right_aligned())
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content area
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Prompt
                Constraint::Min(8),    // Menu
            ])
            .split(chunks[1]);

        // Prompt
        let prompt = Paragraph::new("What would you like to do?")
            .style(Style::default().fg(theme.fg_dim));
        frame.render_widget(prompt, content_chunks[0]);

        // Menu
        let menu_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4),  // Left margin
                Constraint::Min(50),    // Menu
                Constraint::Length(4),  // Right margin
            ])
            .split(content_chunks[1])[1];

        let menu_items: Vec<Line> = MENU_ITEMS
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == self.selected { "> " } else { "  " };
                let style = if i == self.selected {
                    Style::default().fg(theme.primary).bold()
                } else {
                    Style::default().fg(theme.fg)
                };

                Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(format!("{:<18}", item.label), style),
                    Span::styled(
                        item.description,
                        Style::default().fg(if i == self.selected {
                            theme.fg
                        } else {
                            theme.fg_dim
                        }),
                    ),
                ])
            })
            .collect();

        let menu = Paragraph::new(menu_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_alt)),
        );
        frame.render_widget(menu, menu_area);

        // Footer (status bar)
        let footer = Paragraph::new(" [↑↓] Navigate  [Enter] Select  [Q] Quit  [?] Help")
            .style(Style::default().fg(theme.fg_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        if is_up_key(key) {
            if self.selected > 0 {
                self.selected -= 1;
            }
            return None;
        }

        if is_down_key(key) {
            if self.selected < MENU_ITEMS.len() - 1 {
                self.selected += 1;
            }
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
                if idx >= 1 && idx <= MENU_ITEMS.len() {
                    self.selected = idx - 1;
                    return self.handle_key(&KeyEvent::from(crossterm::event::KeyCode::Enter));
                }
            }
        }

        None
    }
}
