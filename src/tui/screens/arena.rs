//! Arena screen - match configuration and execution

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::events::{is_enter_key, is_tab_key, is_shift_tab_key};
use crate::tui::tasks::{
    spawn_arena_task, ArenaConfig, ArenaProgress, ArenaResult, ArenaTaskHandle, BotType,
};
use crate::tui::theme::Theme;
use crate::tui::widgets::{Checkbox, KeyHint, ProgressDisplay, Select, StatusBar, TextInput, Toast};

/// Arena screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArenaState {
    /// Configuring match settings
    Configuring,
    /// Running matches
    Running,
    /// Showing results
    Results,
}

/// Which form field is focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedField {
    Bot1,
    Bot2,
    Deck1,
    Deck2,
    Games,
    Seed,
    Verbose,
    StartButton,
}

impl FocusedField {
    fn next(self) -> Self {
        match self {
            Self::Bot1 => Self::Bot2,
            Self::Bot2 => Self::Deck1,
            Self::Deck1 => Self::Deck2,
            Self::Deck2 => Self::Games,
            Self::Games => Self::Seed,
            Self::Seed => Self::Verbose,
            Self::Verbose => Self::StartButton,
            Self::StartButton => Self::Bot1,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Bot1 => Self::StartButton,
            Self::Bot2 => Self::Bot1,
            Self::Deck1 => Self::Bot2,
            Self::Deck2 => Self::Deck1,
            Self::Games => Self::Deck2,
            Self::Seed => Self::Games,
            Self::Verbose => Self::Seed,
            Self::StartButton => Self::Verbose,
        }
    }
}

/// Arena screen
pub struct ArenaScreen {
    state: ArenaState,
    focused: FocusedField,

    // Form fields
    bot1_select: Select,
    bot2_select: Select,
    deck1_select: Select,
    deck2_select: Select,
    games_input: TextInput,
    seed_input: TextInput,
    verbose_checkbox: Checkbox,

    // Task state
    task_handle: Option<ArenaTaskHandle>,
    progress: ProgressDisplay,
    result: Option<ArenaResult>,
    error: Option<String>,
}

impl ArenaScreen {
    pub fn new() -> Self {
        let bot_options: Vec<String> = BotType::all().iter().map(|b| b.as_str().to_string()).collect();

        Self {
            state: ArenaState::Configuring,
            focused: FocusedField::Bot1,

            bot1_select: Select::new("Bot 1", bot_options.clone()),
            bot2_select: Select::new("Bot 2", bot_options),
            deck1_select: Select::new("Deck 1", vec!["(loading...)".to_string()]),
            deck2_select: Select::new("Deck 2", vec!["(loading...)".to_string()]),
            games_input: TextInput::new("Games").placeholder("100"),
            seed_input: TextInput::new("Seed").placeholder("random"),
            verbose_checkbox: Checkbox::new("Verbose logging"),

            task_handle: None,
            progress: ProgressDisplay::new("Games", 100),
            result: None,
            error: None,
        }
    }

    /// Initialize with available decks from app state
    pub fn with_decks(mut self, decks: &[String]) -> Self {
        if !decks.is_empty() {
            self.deck1_select = Select::new("Deck 1", decks.to_vec());
            self.deck2_select = Select::new("Deck 2", decks.to_vec());
        }
        self
    }

    fn update_focus(&mut self) {
        self.bot1_select.set_focused(self.focused == FocusedField::Bot1);
        self.bot2_select.set_focused(self.focused == FocusedField::Bot2);
        self.deck1_select.set_focused(self.focused == FocusedField::Deck1);
        self.deck2_select.set_focused(self.focused == FocusedField::Deck2);
        self.games_input.set_focused(self.focused == FocusedField::Games);
        self.seed_input.set_focused(self.focused == FocusedField::Seed);
        self.verbose_checkbox.set_focused(self.focused == FocusedField::Verbose);
    }

    fn build_config(&self) -> ArenaConfig {
        let bot_types = BotType::all();

        ArenaConfig {
            bot1_type: bot_types[self.bot1_select.selected_index()],
            bot2_type: bot_types[self.bot2_select.selected_index()],
            deck1: self.deck1_select.selected_value().to_string(),
            deck2: self.deck2_select.selected_value().to_string(),
            weights1: None, // TODO: Add weight selection
            weights2: None,
            games: self.games_input.value().parse().unwrap_or(100),
            seed: self.seed_input.value().parse().ok(),
            verbose: self.verbose_checkbox.is_checked(),
        }
    }

    fn start_match(&mut self) {
        let config = self.build_config();
        self.progress = ProgressDisplay::new("Games", config.games);
        self.progress.start();
        self.task_handle = Some(spawn_arena_task(config));
        self.state = ArenaState::Running;
        self.error = None;
        self.result = None;
    }

    fn render_config(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Player 1 row
                Constraint::Length(3), // Player 2 row
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Settings row 1
                Constraint::Length(3), // Settings row 2
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Button row
                Constraint::Min(1),    // Rest
            ])
            .split(area);

        // Player 1 row: Bot + Deck
        let p1_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);

        self.bot1_select.render(p1_chunks[0], frame.buffer_mut(), theme);
        self.deck1_select.render(p1_chunks[1], frame.buffer_mut(), theme);

        // Player 2 row: Bot + Deck
        let p2_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        self.bot2_select.render(p2_chunks[0], frame.buffer_mut(), theme);
        self.deck2_select.render(p2_chunks[1], frame.buffer_mut(), theme);

        // Settings row 1: Games + Seed
        let settings1_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[3]);

        self.games_input.render(settings1_chunks[0], frame.buffer_mut(), theme);
        self.seed_input.render(settings1_chunks[1], frame.buffer_mut(), theme);

        // Settings row 2: Verbose checkbox
        let checkbox_area = Rect {
            x: chunks[4].x + 1,
            y: chunks[4].y + 1,
            width: chunks[4].width.saturating_sub(2),
            height: 1,
        };
        self.verbose_checkbox.render(checkbox_area, frame.buffer_mut(), theme);

        // Start button
        let button_style = if self.focused == FocusedField::StartButton {
            Style::default().fg(theme.bg).bg(theme.primary).bold()
        } else {
            Style::default().fg(theme.primary)
        };

        let button = Paragraph::new("  [ Start Match ]  ")
            .style(button_style)
            .alignment(Alignment::Center);
        frame.render_widget(button, chunks[6]);
    }

    fn render_running(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Info
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Progress
                Constraint::Min(1),    // Rest
            ])
            .split(area);

        // Match info
        let config = self.build_config();
        let info = Paragraph::new(format!(
            "Running: {} vs {} | {} games",
            config.bot1_type.as_str(),
            config.bot2_type.as_str(),
            config.games
        ))
        .style(Style::default().fg(theme.fg))
        .alignment(Alignment::Center);
        frame.render_widget(info, chunks[0]);

        // Progress bar
        let progress_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(chunks[2])[1];

        self.progress.render(progress_area, frame.buffer_mut(), theme);
    }

    fn render_results(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(1), // Spacer
                Constraint::Min(10),   // Results
                Constraint::Length(2), // Instructions
            ])
            .split(area);

        if let Some(ref error) = self.error {
            let error_msg = Paragraph::new(format!("Error: {}", error))
                .style(Style::default().fg(theme.error))
                .alignment(Alignment::Center);
            frame.render_widget(error_msg, chunks[0]);
        } else if let Some(ref result) = self.result {
            // Title
            let title = Paragraph::new("Match Complete!")
                .style(Style::default().fg(theme.success).bold())
                .alignment(Alignment::Center);
            frame.render_widget(title, chunks[0]);

            // Results
            let results_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Results ");

            let inner = results_block.inner(chunks[2]);
            frame.render_widget(results_block, chunks[2]);

            let results_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Total Games:    ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{}", result.total_games),
                        Style::default().fg(theme.fg),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Player 1 Wins:  ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{} ({:.1}%)", result.player1_wins, result.player1_win_rate * 100.0),
                        Style::default().fg(theme.success),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Player 2 Wins:  ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!(
                            "{} ({:.1}%)",
                            result.player2_wins,
                            (1.0 - result.player1_win_rate - (result.draws as f64 / result.total_games as f64)) * 100.0
                        ),
                        Style::default().fg(theme.error),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Draws:          ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{}", result.draws),
                        Style::default().fg(theme.warning),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Time:           ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.2}s", result.elapsed_secs),
                        Style::default().fg(theme.fg),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Throughput:     ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        if result.games_per_sec >= 1.0 {
                            format!("{:.0} games/sec", result.games_per_sec)
                        } else {
                            format!("{:.2} games/sec", result.games_per_sec)
                        },
                        Style::default().fg(theme.info),
                    ),
                ]),
            ];

            let results = Paragraph::new(results_text);
            frame.render_widget(results, inner);

            // Instructions
            let instructions = Paragraph::new("Press Enter to run again, Esc to go back")
                .style(Style::default().fg(theme.fg_dim))
                .alignment(Alignment::Center);
            frame.render_widget(instructions, chunks[3]);
        }
    }
}

impl Clone for ArenaScreen {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            focused: self.focused,
            bot1_select: self.bot1_select.clone(),
            bot2_select: self.bot2_select.clone(),
            deck1_select: self.deck1_select.clone(),
            deck2_select: self.deck2_select.clone(),
            games_input: self.games_input.clone(),
            seed_input: self.seed_input.clone(),
            verbose_checkbox: self.verbose_checkbox.clone(),
            task_handle: None, // Can't clone the handle
            progress: self.progress.clone(),
            result: self.result.clone(),
            error: self.error.clone(),
        }
    }
}

impl std::fmt::Debug for ArenaScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArenaScreen")
            .field("state", &self.state)
            .field("focused", &self.focused)
            .finish()
    }
}

impl Default for ArenaScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for ArenaScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let header_text = match self.state {
            ArenaState::Configuring => " Arena Match Builder",
            ArenaState::Running => " Running Arena Match...",
            ArenaState::Results => " Arena Match Results",
        };
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Content
        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));
        let content_inner = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);

        match self.state {
            ArenaState::Configuring => self.render_config(frame, content_inner, theme),
            ArenaState::Running => self.render_running(frame, content_inner, theme),
            ArenaState::Results => self.render_results(frame, content_inner, theme),
        }

        // Footer
        let hints = match self.state {
            ArenaState::Configuring => vec![
                KeyHint::new("Tab", "Next"),
                KeyHint::new("↑↓", "Change"),
                KeyHint::new("Enter", "Start"),
                KeyHint::new("Esc", "Back"),
            ],
            ArenaState::Running => vec![
                KeyHint::new("Esc", "Cancel"),
            ],
            ArenaState::Results => vec![
                KeyHint::new("Enter", "New Match"),
                KeyHint::new("Esc", "Back"),
            ],
        };
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match self.state {
            ArenaState::Configuring => self.handle_config_key(key),
            ArenaState::Running => {
                // Allow canceling with Esc (task will complete in background)
                if key.code == KeyCode::Esc {
                    self.task_handle = None;
                    self.state = ArenaState::Configuring;
                }
                None
            }
            ArenaState::Results => {
                if is_enter_key(key) {
                    self.state = ArenaState::Configuring;
                }
                None
            }
        }
    }

    fn tick(&mut self) -> Option<Message> {
        // Poll for task updates - collect messages first to avoid borrow issues
        let messages: Vec<_> = if let Some(ref handle) = self.task_handle {
            std::iter::from_fn(|| handle.try_recv()).collect()
        } else {
            Vec::new()
        };

        let mut toast: Option<Toast> = None;

        // Process collected messages
        for progress in messages {
            match progress {
                ArenaProgress::Started => {}
                ArenaProgress::Progress(current, _total) => {
                    self.progress.set_current(current);
                }
                ArenaProgress::Completed(result) => {
                    let p1_wins = result.player1_wins;
                    let total = result.total_games;
                    self.result = Some(result);
                    self.state = ArenaState::Results;
                    self.task_handle = None;
                    toast = Some(Toast::success(format!("Match complete! P1: {}/{} wins", p1_wins, total)));
                }
                ArenaProgress::Error(err) => {
                    self.error = Some(err.clone());
                    self.state = ArenaState::Results;
                    self.task_handle = None;
                    toast = Some(Toast::error(format!("Arena error: {}", err)));
                }
            }
        }

        toast.map(Message::ShowToast)
    }
}

impl ArenaScreen {
    fn handle_config_key(&mut self, key: &KeyEvent) -> Option<Message> {
        // Tab navigation
        if is_tab_key(key) {
            self.focused = self.focused.next();
            self.update_focus();
            return None;
        }
        if is_shift_tab_key(key) {
            self.focused = self.focused.prev();
            self.update_focus();
            return None;
        }

        // Enter to start match
        if is_enter_key(key) && self.focused == FocusedField::StartButton {
            self.start_match();
            return None;
        }

        // Handle field-specific input
        match self.focused {
            FocusedField::Bot1 => self.handle_select_key(key, &mut self.bot1_select.clone()),
            FocusedField::Bot2 => self.handle_select_key(key, &mut self.bot2_select.clone()),
            FocusedField::Deck1 => self.handle_select_key(key, &mut self.deck1_select.clone()),
            FocusedField::Deck2 => self.handle_select_key(key, &mut self.deck2_select.clone()),
            FocusedField::Games => {
                let mut input = self.games_input.clone();
                input.handle_key(*key);
                self.games_input = input;
            }
            FocusedField::Seed => {
                let mut input = self.seed_input.clone();
                input.handle_key(*key);
                self.seed_input = input;
            }
            FocusedField::Verbose => {
                if key.code == KeyCode::Char(' ') || is_enter_key(key) {
                    self.verbose_checkbox.toggle();
                }
            }
            FocusedField::StartButton => {}
        }

        // Sync changes back
        match self.focused {
            FocusedField::Bot1 => {
                if key.code == KeyCode::Up {
                    self.bot1_select.previous();
                } else if key.code == KeyCode::Down {
                    self.bot1_select.next();
                }
            }
            FocusedField::Bot2 => {
                if key.code == KeyCode::Up {
                    self.bot2_select.previous();
                } else if key.code == KeyCode::Down {
                    self.bot2_select.next();
                }
            }
            FocusedField::Deck1 => {
                if key.code == KeyCode::Up {
                    self.deck1_select.previous();
                } else if key.code == KeyCode::Down {
                    self.deck1_select.next();
                }
            }
            FocusedField::Deck2 => {
                if key.code == KeyCode::Up {
                    self.deck2_select.previous();
                } else if key.code == KeyCode::Down {
                    self.deck2_select.next();
                }
            }
            _ => {}
        }

        None
    }

    fn handle_select_key(&mut self, _key: &KeyEvent, _select: &mut Select) {
        // Handled in the match block above now
    }
}
