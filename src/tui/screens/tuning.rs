//! Tuning screen - weight optimization with live progress

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{AnalysisScreen, Screen, ScreenWidget, WeightsScreen};
use crate::tui::app::Message;
use crate::tui::tasks::{
    spawn_tuning_task, GenerationStats, TuningConfig, TuningModeConfig, TuningProgress,
    TuningResult, TuningTaskHandle,
};
use crate::tui::theme::Theme;
use crate::tui::widgets::{Checkbox, Dropdown, KeyHint, NumberInput, StatusBar, TextInput, Toast};

/// Screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TuningState {
    Configuring,
    Running,
    Paused,
    Completed,
}

/// Form field focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormField {
    Mode,
    Generations,
    Population,
    GamesPerEval,
    Sigma,
    Seed,
    Tag,
    Parallel,
    Deck,
    OpponentDeck,
}

impl FormField {
    fn next(self) -> Self {
        match self {
            FormField::Mode => FormField::Generations,
            FormField::Generations => FormField::Population,
            FormField::Population => FormField::GamesPerEval,
            FormField::GamesPerEval => FormField::Sigma,
            FormField::Sigma => FormField::Seed,
            FormField::Seed => FormField::Tag,
            FormField::Tag => FormField::Parallel,
            FormField::Parallel => FormField::Deck,
            FormField::Deck => FormField::OpponentDeck,
            FormField::OpponentDeck => FormField::Mode,
        }
    }

    fn prev(self) -> Self {
        match self {
            FormField::Mode => FormField::OpponentDeck,
            FormField::Generations => FormField::Mode,
            FormField::Population => FormField::Generations,
            FormField::GamesPerEval => FormField::Population,
            FormField::Sigma => FormField::GamesPerEval,
            FormField::Seed => FormField::Sigma,
            FormField::Tag => FormField::Seed,
            FormField::Parallel => FormField::Tag,
            FormField::Deck => FormField::Parallel,
            FormField::OpponentDeck => FormField::Deck,
        }
    }
}

/// Tuning screen state
///
/// Note: Manual Debug impl because TuningTaskHandle contains JoinHandle which doesn't impl Debug
pub struct TuningScreen {
    state: TuningState,
    focused: FormField,

    // Form fields
    mode_dropdown: Dropdown,
    generations_input: NumberInput,
    population_input: NumberInput,
    games_input: NumberInput,
    sigma_input: TextInput,
    seed_input: NumberInput,
    tag_input: TextInput,
    parallel_checkbox: Checkbox,
    deck_dropdown: Dropdown,
    opponent_dropdown: Dropdown,

    // Available decks
    deck_ids: Vec<String>,

    // Running task
    task_handle: Option<TuningTaskHandle>,

    // Progress tracking
    current_stats: Option<GenerationStats>,
    fitness_history: Vec<f64>,
    result: Option<TuningResult>,
    experiment_dir: Option<String>,
}

impl std::fmt::Debug for TuningScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TuningScreen")
            .field("state", &self.state)
            .field("focused", &self.focused)
            .field("task_running", &self.task_handle.is_some())
            .finish()
    }
}

impl TuningScreen {
    pub fn new() -> Self {
        let modes: Vec<String> = TuningModeConfig::all()
            .iter()
            .map(|m| m.as_str().to_string())
            .collect();

        Self {
            state: TuningState::Configuring,
            focused: FormField::Mode,

            mode_dropdown: Dropdown::new("Mode", modes),
            generations_input: NumberInput::new("Generations").with_value(50).min(1).max(1000),
            population_input: NumberInput::new("Population").with_value(0).min(0).max(100),
            games_input: NumberInput::new("Games/Eval").with_value(50).min(10).max(500),
            sigma_input: TextInput::new("Sigma").with_value("0.3"),
            seed_input: NumberInput::new("Seed").with_value(42).min(0).max(u32::MAX as i64),
            tag_input: TextInput::new("Tag").with_value("default"),
            parallel_checkbox: Checkbox::new("Parallel Evaluation").checked(true),
            deck_dropdown: Dropdown::new("Deck", vec!["(loading...)".to_string()]),
            opponent_dropdown: Dropdown::new("Opponent Deck", vec!["(loading...)".to_string()]),

            deck_ids: Vec::new(),

            task_handle: None,
            current_stats: None,
            fitness_history: Vec::new(),
            result: None,
            experiment_dir: None,
        }
    }

    /// Initialize with available decks
    pub fn with_decks(mut self, deck_ids: &[String]) -> Self {
        self.deck_ids = deck_ids.to_vec();
        if !deck_ids.is_empty() {
            self.deck_dropdown = Dropdown::new("Deck", deck_ids.to_vec());
            self.opponent_dropdown = Dropdown::new("Opponent Deck", deck_ids.to_vec());
        }
        self
    }

    fn start_tuning(&mut self) {
        let mode_idx = self.mode_dropdown.selected_index();
        let mode = TuningModeConfig::all()[mode_idx];

        let sigma: f64 = self.sigma_input.value().parse().unwrap_or(0.3);
        let population = if self.population_input.value() == 0 {
            None
        } else {
            Some(self.population_input.value() as usize)
        };

        let config = TuningConfig {
            mode,
            generations: self.generations_input.value() as u32,
            population,
            games_per_eval: self.games_input.value() as usize,
            sigma,
            min_sigma: 0.001,
            seed: self.seed_input.value() as u64,
            target_win_rate: None,
            mcts_sims: 200,
            parallel: self.parallel_checkbox.is_checked(),
            tag: self.tag_input.value().to_string(),
            experiment_dir: std::path::PathBuf::from("experiments"),
            initial_weights: None,
            deck: if mode == TuningModeConfig::Specialist && !self.deck_ids.is_empty() {
                Some(self.deck_ids[self.deck_dropdown.selected_index()].clone())
            } else {
                None
            },
            opponent_deck: if mode == TuningModeConfig::Specialist && !self.deck_ids.is_empty() {
                Some(self.deck_ids[self.opponent_dropdown.selected_index()].clone())
            } else {
                None
            },
        };

        self.fitness_history.clear();
        self.current_stats = None;
        self.result = None;
        self.experiment_dir = None;
        self.task_handle = Some(spawn_tuning_task(config));
        self.state = TuningState::Running;
    }

    fn toggle_pause(&mut self) {
        if let Some(ref handle) = self.task_handle {
            if self.state == TuningState::Running {
                handle.pause();
                self.state = TuningState::Paused;
            } else if self.state == TuningState::Paused {
                handle.resume();
                self.state = TuningState::Running;
            }
        }
    }

    fn stop_tuning(&mut self) {
        if let Some(ref handle) = self.task_handle {
            handle.stop();
        }
    }

    fn reset(&mut self) {
        self.state = TuningState::Configuring;
        self.task_handle = None;
        self.current_stats = None;
        self.result = None;
        self.experiment_dir = None;
        self.fitness_history.clear();
    }

    fn update_focus(&mut self) {
        // Update focus state on all widgets
        self.mode_dropdown.set_focused(self.focused == FormField::Mode);
        self.generations_input.set_focused(self.focused == FormField::Generations);
        self.population_input.set_focused(self.focused == FormField::Population);
        self.games_input.set_focused(self.focused == FormField::GamesPerEval);
        self.sigma_input.set_focused(self.focused == FormField::Sigma);
        self.seed_input.set_focused(self.focused == FormField::Seed);
        self.tag_input.set_focused(self.focused == FormField::Tag);
        self.parallel_checkbox.set_focused(self.focused == FormField::Parallel);
        self.deck_dropdown.set_focused(self.focused == FormField::Deck);
        self.opponent_dropdown.set_focused(self.focused == FormField::OpponentDeck);
    }

    fn handle_config_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => {
                self.start_tuning();
                None
            }
            KeyCode::Tab | KeyCode::Down => {
                self.focused = self.focused.next();
                self.update_focus();
                None
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focused = self.focused.prev();
                self.update_focus();
                None
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char(_) | KeyCode::Backspace => {
                // Delegate to focused widget
                match self.focused {
                    FormField::Mode => {
                        if key.code == KeyCode::Left {
                            self.mode_dropdown.previous();
                        } else if key.code == KeyCode::Right {
                            self.mode_dropdown.next();
                        }
                    }
                    FormField::Generations => {
                        self.generations_input.handle_key(key);
                    }
                    FormField::Population => {
                        self.population_input.handle_key(key);
                    }
                    FormField::GamesPerEval => {
                        self.games_input.handle_key(key);
                    }
                    FormField::Sigma => {
                        self.sigma_input.handle_key(*key);
                    }
                    FormField::Seed => {
                        self.seed_input.handle_key(key);
                    }
                    FormField::Tag => {
                        self.tag_input.handle_key(*key);
                    }
                    FormField::Parallel => {
                        if matches!(key.code, KeyCode::Left | KeyCode::Right | KeyCode::Char(' ')) {
                            self.parallel_checkbox.toggle();
                        }
                    }
                    FormField::Deck => {
                        if key.code == KeyCode::Left {
                            self.deck_dropdown.previous();
                        } else if key.code == KeyCode::Right {
                            self.deck_dropdown.next();
                        }
                    }
                    FormField::OpponentDeck => {
                        if key.code == KeyCode::Left {
                            self.opponent_dropdown.previous();
                        } else if key.code == KeyCode::Right {
                            self.opponent_dropdown.next();
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn handle_running_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.toggle_pause();
                None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.stop_tuning();
                None
            }
            _ => None,
        }
    }

    fn handle_completed_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter | KeyCode::Char('r') | KeyCode::Char('R') => {
                self.reset();
                None
            }
            // View Analysis - navigate to analysis screen with this experiment
            KeyCode::Char('a') | KeyCode::Char('A') => {
                if let Some(ref result) = self.result {
                    let exp_path = result.experiment_dir.display().to_string();
                    Some(Message::Navigate(Box::new(Screen::Analysis(
                        AnalysisScreen::with_experiment(&exp_path)
                    ))))
                } else {
                    None
                }
            }
            // Compare Weights - navigate to weights screen in compare mode
            KeyCode::Char('w') | KeyCode::Char('W') => {
                if let Some(ref result) = self.result {
                    let weights_path = result.weights_path.display().to_string();
                    Some(Message::Navigate(Box::new(Screen::Weights(
                        WeightsScreen::with_compare(&weights_path)
                    ))))
                } else {
                    None
                }
            }
            // Promote Weights - copy to default
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if let Some(ref result) = self.result {
                    let dest = PathBuf::from("data/weights/default.toml");
                    match fs::copy(&result.weights_path, &dest) {
                        Ok(_) => Some(Message::ShowToast(
                            Toast::success("Weights promoted to default!")
                        )),
                        Err(e) => Some(Message::ShowToast(
                            Toast::error(format!("Failed to promote: {}", e))
                        )),
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn render_config(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(20),    // Form
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Weight Tuning - Configuration")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Form content
        let form_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" CMA-ES Parameters ");
        let inner = form_block.inner(chunks[1]);
        frame.render_widget(form_block, chunks[1]);

        // Form layout
        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Mode
                Constraint::Length(1), // Mode description
                Constraint::Length(2), // Generations & Population
                Constraint::Length(2), // Games & Sigma
                Constraint::Length(2), // Seed & Tag
                Constraint::Length(2), // Parallel
                Constraint::Length(2), // Deck (specialist)
                Constraint::Length(2), // Opponent (specialist)
                Constraint::Min(0),    // Spacer
            ])
            .split(inner);

        // Mode dropdown
        self.mode_dropdown.render(form_chunks[0], frame.buffer_mut(), theme);

        // Mode description
        let mode_idx = self.mode_dropdown.selected_index();
        let mode = TuningModeConfig::all()[mode_idx];
        let desc = Paragraph::new(format!("  {}", mode.description()))
            .style(Style::default().fg(theme.fg_dim));
        frame.render_widget(desc, form_chunks[1]);

        // Row 2: Generations & Population
        let row2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(form_chunks[2]);
        self.generations_input.render(row2[0], frame.buffer_mut(), theme);
        self.population_input.render(row2[1], frame.buffer_mut(), theme);

        // Row 3: Games & Sigma
        let row3 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(form_chunks[3]);
        self.games_input.render(row3[0], frame.buffer_mut(), theme);
        self.sigma_input.render(row3[1], frame.buffer_mut(), theme);

        // Row 4: Seed & Tag
        let row4 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(form_chunks[4]);
        self.seed_input.render(row4[0], frame.buffer_mut(), theme);
        self.tag_input.render(row4[1], frame.buffer_mut(), theme);

        // Parallel checkbox
        self.parallel_checkbox.render(form_chunks[5], frame.buffer_mut(), theme);

        // Specialist mode deck selection
        if mode == TuningModeConfig::Specialist {
            self.deck_dropdown.render(form_chunks[6], frame.buffer_mut(), theme);
            self.opponent_dropdown.render(form_chunks[7], frame.buffer_mut(), theme);
        }

        // Footer
        let hints = vec![
            KeyHint::new("Enter", "Start"),
            KeyHint::new("Tab", "Next field"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn render_running(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(5),  // Stats
                Constraint::Min(10),    // Chart
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header with status
        let status_str = if self.state == TuningState::Paused {
            "PAUSED"
        } else {
            "RUNNING"
        };
        let header_text = format!(" Weight Tuning - {} ", status_str);
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if self.state == TuningState::Paused {
                        theme.warning
                    } else {
                        theme.success
                    }))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Stats panel
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Progress ");
        let stats_inner = stats_block.inner(chunks[1]);
        frame.render_widget(stats_block, chunks[1]);

        if let Some(ref stats) = self.current_stats {
            let progress_pct = (stats.generation as f64 / stats.total_generations as f64) * 100.0;
            let eta_secs = if stats.generation > 0 {
                let avg_gen_time = stats.elapsed_secs / stats.generation as f64;
                let remaining_gens = stats.total_generations - stats.generation;
                avg_gen_time * remaining_gens as f64
            } else {
                0.0
            };

            let stats_text = vec![
                Line::from(vec![
                    Span::styled("  Generation: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{}/{}", stats.generation, stats.total_generations),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled(
                        format!(" ({:.1}%)", progress_pct),
                        Style::default().fg(theme.info),
                    ),
                    Span::styled("  |  Best Fitness: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.2}", stats.best_fitness),
                        Style::default().fg(theme.success),
                    ),
                    Span::styled("  |  Win Rate: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.1}%", stats.best_win_rate * 100.0),
                        Style::default().fg(theme.primary),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Sigma: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{:.4}", stats.sigma), Style::default().fg(theme.fg)),
                    Span::styled("  |  Gen Time: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.1}s", stats.gen_time_secs),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled("  |  Total: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.0}s", stats.elapsed_secs),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled("  |  ETA: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{:.0}s", eta_secs), Style::default().fg(theme.fg)),
                ]),
            ];
            let stats_para = Paragraph::new(stats_text);
            frame.render_widget(stats_para, stats_inner);
        } else {
            let loading = Paragraph::new("  Starting tuning...")
                .style(Style::default().fg(theme.fg_dim));
            frame.render_widget(loading, stats_inner);
        }

        // Fitness history chart
        self.render_fitness_chart(frame, chunks[2], theme);

        // Footer
        let hints = vec![
            KeyHint::new("P", if self.state == TuningState::Paused { "Resume" } else { "Pause" }),
            KeyHint::new("S", "Stop"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[3]);
    }

    fn render_fitness_chart(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Fitness History ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.fitness_history.is_empty() {
            let empty = Paragraph::new("  Waiting for data...")
                .style(Style::default().fg(theme.fg_dim));
            frame.render_widget(empty, inner);
            return;
        }

        // Find min/max for scaling
        let min_fit = self.fitness_history.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_fit = self.fitness_history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_fit - min_fit).max(0.01); // Avoid division by zero

        let chart_height = (inner.height as usize).saturating_sub(2);
        let chart_width = (inner.width as usize).saturating_sub(10);

        if chart_height == 0 || chart_width == 0 {
            return;
        }

        // Create ASCII chart
        let mut chart_lines: Vec<Line> = Vec::new();

        // Y-axis will be shown inline

        for row in 0..chart_height {
            let row_ratio = row as f64 / chart_height as f64;
            let threshold = max_fit - row_ratio * range;

            let mut line_spans = vec![
                Span::styled(
                    format!("{:>7.1} |", if row == 0 {
                        max_fit
                    } else if row == chart_height - 1 {
                        min_fit
                    } else {
                        threshold
                    }),
                    Style::default().fg(theme.fg_dim),
                ),
            ];

            // Sample data points to fit width
            let data_len = self.fitness_history.len();
            let mut row_chars = String::new();

            for col in 0..chart_width.min(data_len) {
                let data_idx = if data_len <= chart_width {
                    col
                } else {
                    (col * data_len) / chart_width
                };

                if data_idx < self.fitness_history.len() {
                    let value = self.fitness_history[data_idx];
                    let value_ratio = (max_fit - value) / range;
                    let value_row = (value_ratio * chart_height as f64) as usize;

                    if value_row == row {
                        row_chars.push('*');
                    } else {
                        row_chars.push(' ');
                    }
                }
            }

            line_spans.push(Span::styled(row_chars, Style::default().fg(theme.success)));
            chart_lines.push(Line::from(line_spans));
        }

        // X-axis
        let x_axis = format!("{:>8}+{}", "", "-".repeat(chart_width));
        chart_lines.push(Line::from(Span::styled(x_axis, Style::default().fg(theme.fg_dim))));

        let x_label = format!("{:>8} Gen 0{:>width$}Gen {}", "", "", self.fitness_history.len(), width = chart_width.saturating_sub(10));
        chart_lines.push(Line::from(Span::styled(x_label, Style::default().fg(theme.fg_dim))));

        let chart = Paragraph::new(chart_lines);
        frame.render_widget(chart, inner);
    }

    fn render_completed(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(15),    // Results
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Weight Tuning - Complete")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.success))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Results
        let results_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Results ");
        let inner = results_block.inner(chunks[1]);
        frame.render_widget(results_block, chunks[1]);

        if let Some(ref result) = self.result {
            let results_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Stop Reason:     ", Style::default().fg(theme.fg_dim)),
                    Span::styled(&result.stop_reason, Style::default().fg(theme.fg)),
                ]),
                Line::from(vec![
                    Span::styled("  Generations:     ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{}", result.total_generations),
                        Style::default().fg(theme.fg),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Evaluations:     ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{}", result.total_evaluations),
                        Style::default().fg(theme.fg),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Best Fitness:    ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.2}", result.best_fitness),
                        Style::default().fg(theme.success),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Best Win Rate:   ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.1}%", result.best_win_rate * 100.0),
                        Style::default().fg(theme.primary),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Total Time:      ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        format!("{:.1}s", result.elapsed_secs),
                        Style::default().fg(theme.fg),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Weights saved:   ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        result.weights_path.display().to_string(),
                        Style::default().fg(theme.info),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Experiment dir:  ", Style::default().fg(theme.fg_dim)),
                    Span::styled(
                        result.experiment_dir.display().to_string(),
                        Style::default().fg(theme.info),
                    ),
                ]),
            ];

            let results = Paragraph::new(results_text);
            frame.render_widget(results, inner);
        }

        // Footer with post-tuning actions
        let hints = vec![
            KeyHint::new("A", "View Analysis"),
            KeyHint::new("W", "Compare Weights"),
            KeyHint::new("P", "Promote"),
            KeyHint::new("R", "New Tuning"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }
}

impl Default for TuningScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for TuningScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match self.state {
            TuningState::Configuring => self.render_config(frame, area, theme),
            TuningState::Running | TuningState::Paused => self.render_running(frame, area, theme),
            TuningState::Completed => self.render_completed(frame, area, theme),
        }
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match self.state {
            TuningState::Configuring => self.handle_config_key(key),
            TuningState::Running | TuningState::Paused => self.handle_running_key(key),
            TuningState::Completed => self.handle_completed_key(key),
        }
    }

    fn tick(&mut self) -> Option<Message> {
        // Process progress updates from background task
        let mut toast: Option<Toast> = None;

        if let Some(ref handle) = self.task_handle {
            while let Some(progress) = handle.try_recv() {
                match progress {
                    TuningProgress::Started(dir) => {
                        self.experiment_dir = Some(dir.display().to_string());
                    }
                    TuningProgress::Generation(stats) => {
                        self.fitness_history.push(stats.best_fitness);
                        self.current_stats = Some(stats);
                    }
                    TuningProgress::Completed(result) => {
                        let win_rate = result.best_win_rate;
                        self.result = Some(result);
                        self.state = TuningState::Completed;
                        toast = Some(Toast::success(format!("Tuning complete! Win rate: {:.1}%", win_rate * 100.0)));
                    }
                    TuningProgress::Stopped(result) => {
                        self.result = Some(result);
                        self.state = TuningState::Completed;
                        toast = Some(Toast::info("Tuning stopped by user"));
                    }
                    TuningProgress::Paused => {
                        self.state = TuningState::Paused;
                    }
                    TuningProgress::Resumed => {
                        self.state = TuningState::Running;
                    }
                    TuningProgress::Error(e) => {
                        // For now, just complete with error
                        self.result = Some(TuningResult {
                            total_generations: self.current_stats.as_ref().map(|s| s.generation).unwrap_or(0),
                            total_evaluations: 0,
                            best_fitness: self.current_stats.as_ref().map(|s| s.best_fitness).unwrap_or(0.0),
                            best_win_rate: self.current_stats.as_ref().map(|s| s.best_win_rate).unwrap_or(0.0),
                            elapsed_secs: self.current_stats.as_ref().map(|s| s.elapsed_secs).unwrap_or(0.0),
                            stop_reason: format!("Error: {}", e),
                            weights_path: std::path::PathBuf::new(),
                            experiment_dir: std::path::PathBuf::new(),
                        });
                        self.state = TuningState::Completed;
                        toast = Some(Toast::error(format!("Tuning error: {}", e)));
                    }
                }
            }
        }

        toast.map(Message::ShowToast)
    }
}

impl Clone for TuningScreen {
    fn clone(&self) -> Self {
        // Create new screen with same config but no running task
        let mut screen = Self::new();
        screen.state = TuningState::Configuring;
        screen.deck_ids = self.deck_ids.clone();
        screen.mode_dropdown = self.mode_dropdown.clone();
        screen.generations_input = self.generations_input.clone();
        screen.population_input = self.population_input.clone();
        screen.games_input = self.games_input.clone();
        screen.sigma_input = self.sigma_input.clone();
        screen.seed_input = self.seed_input.clone();
        screen.tag_input = self.tag_input.clone();
        screen.parallel_checkbox = self.parallel_checkbox.clone();
        screen.deck_dropdown = self.deck_dropdown.clone();
        screen.opponent_dropdown = self.opponent_dropdown.clone();
        screen
    }
}
