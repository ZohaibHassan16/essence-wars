//! Analysis screen - experiment results viewer

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

/// Parsed experiment data
#[derive(Debug, Clone)]
struct ExperimentData {
    id: String,
    path: PathBuf,
    summary: Option<ExperimentSummary>,
    stats: Vec<GenerationStat>,
    version: Option<VersionInfo>,
}

#[derive(Debug, Clone)]
struct ExperimentSummary {
    mode: String,
    best_fitness: f64,
    best_win_rate: f64,
    total_time: f64,
    generations: u32,
    stop_reason: String,
}

#[derive(Debug, Clone)]
struct GenerationStat {
    generation: u32,
    fitness: f64,
    win_rate: f64,
    sigma: f64,
    gen_time: f64,
}

#[derive(Debug, Clone)]
struct VersionInfo {
    name: String,
    version: String,
    git_hash: String,
}

/// Screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalysisState {
    BrowsingExperiments,
    ViewingDetails,
}

/// Analysis screen state
#[derive(Debug, Clone)]
pub struct AnalysisScreen {
    state: AnalysisState,
    experiments: Vec<ExperimentData>,
    list_state: ListState,
    stats_scroll: u16,
}

impl AnalysisScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            state: AnalysisState::BrowsingExperiments,
            experiments: Vec::new(),
            list_state: ListState::default(),
            stats_scroll: 0,
        };
        screen.load_experiments();
        if !screen.experiments.is_empty() {
            screen.list_state.select(Some(0));
        }
        screen
    }

    /// Create with a specific experiment pre-selected and open details
    pub fn with_experiment(experiment_path: &str) -> Self {
        let mut screen = Self::new();

        // Find the experiment by path or ID
        let experiment_name = std::path::Path::new(experiment_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(experiment_path);

        // Try to find and select the experiment
        for (i, exp) in screen.experiments.iter().enumerate() {
            if exp.id == experiment_name || exp.path.to_string_lossy().contains(experiment_name) {
                screen.list_state.select(Some(i));
                screen.state = AnalysisState::ViewingDetails;
                break;
            }
        }

        screen
    }

    fn load_experiments(&mut self) {
        let experiments_dir = PathBuf::from("experiments/mcts");
        if !experiments_dir.exists() {
            return;
        }

        let mut experiments: Vec<ExperimentData> = Vec::new();

        if let Ok(entries) = fs::read_dir(&experiments_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let id = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let summary = Self::parse_summary(&path);
                    let stats = Self::parse_stats(&path);
                    let version = Self::parse_version(&path);

                    experiments.push(ExperimentData {
                        id,
                        path,
                        summary,
                        stats,
                        version,
                    });
                }
            }
        }

        // Sort by ID (which includes timestamp) - newest first
        experiments.sort_by(|a, b| b.id.cmp(&a.id));
        self.experiments = experiments;
    }

    fn parse_summary(path: &PathBuf) -> Option<ExperimentSummary> {
        let summary_path = path.join("summary.txt");
        let content = fs::read_to_string(summary_path).ok()?;

        let mut mode = String::new();
        let mut best_fitness = 0.0;
        let mut best_win_rate = 0.0;
        let mut total_time = 0.0;
        let mut generations = 0;
        let mut stop_reason = String::new();

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Mode: ") {
                mode = value.to_string();
            } else if let Some(value) = line.strip_prefix("Best Fitness: ") {
                best_fitness = value.parse().unwrap_or(0.0);
            } else if let Some(value) = line.strip_prefix("Best Win Rate: ") {
                best_win_rate = value.trim_end_matches('%').parse().unwrap_or(0.0);
            } else if let Some(value) = line.strip_prefix("Total Time: ") {
                total_time = value.trim_end_matches('s').parse().unwrap_or(0.0);
            } else if let Some(value) = line.strip_prefix("Generations: ") {
                generations = value.parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("Stop Reason: ") {
                stop_reason = value.to_string();
            }
        }

        Some(ExperimentSummary {
            mode,
            best_fitness,
            best_win_rate,
            total_time,
            generations,
            stop_reason,
        })
    }

    fn parse_stats(path: &PathBuf) -> Vec<GenerationStat> {
        let stats_path = path.join("stats.csv");
        let content = match fs::read_to_string(stats_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut stats = Vec::new();
        for line in content.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                stats.push(GenerationStat {
                    generation: parts[0].parse().unwrap_or(0),
                    fitness: parts[1].parse().unwrap_or(0.0),
                    win_rate: parts[2].parse().unwrap_or(0.0),
                    sigma: parts[3].parse().unwrap_or(0.0),
                    gen_time: parts[4].parse().unwrap_or(0.0),
                });
            }
        }
        stats
    }

    fn parse_version(path: &PathBuf) -> Option<VersionInfo> {
        let version_path = path.join("version.toml");
        let content = fs::read_to_string(version_path).ok()?;

        let mut name = String::new();
        let mut version = String::new();
        let mut git_hash = String::new();

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("name = ") {
                name = value.trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("version = ") {
                version = value.trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("git_hash = ") {
                git_hash = value.trim_matches('"').to_string();
            }
        }

        Some(VersionInfo { name, version, git_hash })
    }

    fn selected_experiment(&self) -> Option<&ExperimentData> {
        self.list_state.selected().and_then(|i| self.experiments.get(i))
    }

    fn select_next(&mut self) {
        if self.experiments.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.experiments.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        if self.experiments.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn render_experiment_list(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // List
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Experiment Analysis")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Experiment list
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Experiments ");

        if self.experiments.is_empty() {
            let empty = Paragraph::new("\n  No experiments found in experiments/mcts/\n\n  Run some tuning first!")
                .style(Style::default().fg(theme.fg_dim))
                .block(list_block);
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.experiments
                .iter()
                .map(|exp| {
                    let summary_str = exp.summary.as_ref().map(|s| {
                        format!(" | {} | {:.1}% | {:.0}s", s.mode, s.best_win_rate, s.total_time)
                    }).unwrap_or_default();

                    ListItem::new(format!("  {} {}", exp.id, summary_str))
                })
                .collect();

            let list = List::new(items)
                .block(list_block)
                .highlight_style(Style::default().fg(theme.primary).bold())
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state.clone());
        }

        // Footer
        let hints = vec![
            KeyHint::new("↑↓", "Navigate"),
            KeyHint::new("Enter", "View Details"),
            KeyHint::new("R", "Refresh"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn render_experiment_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let exp = match self.selected_experiment() {
            Some(e) => e,
            None => return,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Length(12),  // Summary
                Constraint::Min(10),     // Stats table
                Constraint::Length(3),   // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(format!(" Experiment: {} ", exp.id))
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Summary panel
        let summary_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Summary ");
        let summary_inner = summary_block.inner(chunks[1]);
        frame.render_widget(summary_block, chunks[1]);

        let mut summary_lines = vec![Line::from("")];

        if let Some(ref s) = exp.summary {
            summary_lines.extend(vec![
                Line::from(vec![
                    Span::styled("  Mode:        ", Style::default().fg(theme.fg_dim)),
                    Span::styled(&s.mode, Style::default().fg(theme.fg)),
                    Span::styled("     Generations: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{}", s.generations), Style::default().fg(theme.fg)),
                ]),
                Line::from(vec![
                    Span::styled("  Best Fitness: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{:.2}", s.best_fitness), Style::default().fg(theme.success)),
                    Span::styled("   Best Win Rate: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{:.1}%", s.best_win_rate), Style::default().fg(theme.primary)),
                ]),
                Line::from(vec![
                    Span::styled("  Total Time:  ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{:.1}s", s.total_time), Style::default().fg(theme.fg)),
                    Span::styled("    Stop Reason: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(&s.stop_reason, Style::default().fg(theme.fg)),
                ]),
            ]);
        }

        if let Some(ref v) = exp.version {
            summary_lines.push(Line::from(""));
            summary_lines.push(Line::from(vec![
                Span::styled("  Engine:      ", Style::default().fg(theme.fg_dim)),
                Span::styled(format!("{} v{}", v.name, v.version), Style::default().fg(theme.info)),
                Span::styled("  (", Style::default().fg(theme.fg_dim)),
                Span::styled(&v.git_hash[..8.min(v.git_hash.len())], Style::default().fg(theme.fg_dim)),
                Span::styled(")", Style::default().fg(theme.fg_dim)),
            ]));
        }

        summary_lines.push(Line::from(vec![
            Span::styled("  Weights:     ", Style::default().fg(theme.fg_dim)),
            Span::styled(exp.path.join("weights.toml").display().to_string(), Style::default().fg(theme.info)),
        ]));

        let summary = Paragraph::new(summary_lines);
        frame.render_widget(summary, summary_inner);

        // Stats table
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(format!(" Generation Stats ({} entries) ", exp.stats.len()));
        let stats_inner = stats_block.inner(chunks[2]);
        frame.render_widget(stats_block, chunks[2]);

        if exp.stats.is_empty() {
            let no_stats = Paragraph::new("  No stats.csv found")
                .style(Style::default().fg(theme.fg_dim));
            frame.render_widget(no_stats, stats_inner);
        } else {
            // Header row
            let mut table_lines = vec![
                Line::from(vec![
                    Span::styled("  Gen    ", Style::default().fg(theme.fg_dim).bold()),
                    Span::styled("Fitness  ", Style::default().fg(theme.fg_dim).bold()),
                    Span::styled("Win Rate  ", Style::default().fg(theme.fg_dim).bold()),
                    Span::styled("Sigma    ", Style::default().fg(theme.fg_dim).bold()),
                    Span::styled("Time", Style::default().fg(theme.fg_dim).bold()),
                ]),
                Line::from(Span::styled("  ─────────────────────────────────────────────", Style::default().fg(theme.border))),
            ];

            // Data rows with scrolling
            let visible_rows = (stats_inner.height as usize).saturating_sub(3);
            let scroll = self.stats_scroll as usize;
            let start = scroll.min(exp.stats.len().saturating_sub(visible_rows));

            for stat in exp.stats.iter().skip(start).take(visible_rows) {
                table_lines.push(Line::from(vec![
                    Span::styled(format!("  {:>4}   ", stat.generation), Style::default().fg(theme.fg)),
                    Span::styled(format!("{:>6.2}   ", stat.fitness), Style::default().fg(theme.success)),
                    Span::styled(format!("{:>6.1}%   ", stat.win_rate), Style::default().fg(theme.primary)),
                    Span::styled(format!("{:>.4}   ", stat.sigma), Style::default().fg(theme.fg)),
                    Span::styled(format!("{:.1}s", stat.gen_time), Style::default().fg(theme.fg_dim)),
                ]));
            }

            let table = Paragraph::new(table_lines);
            frame.render_widget(table, stats_inner);

            // Scrollbar if needed
            if exp.stats.len() > visible_rows {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(exp.stats.len())
                    .position(scroll);
                frame.render_stateful_widget(
                    scrollbar,
                    chunks[2].inner(Margin { horizontal: 0, vertical: 1 }),
                    &mut scrollbar_state,
                );
            }
        }

        // Footer
        let hints = vec![
            KeyHint::new("↑↓", "Scroll Stats"),
            KeyHint::new("W", "View Weights"),
            KeyHint::new("Esc", "Back to List"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[3]);
    }
}

impl Default for AnalysisScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for AnalysisScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match self.state {
            AnalysisState::BrowsingExperiments => self.render_experiment_list(frame, area, theme),
            AnalysisState::ViewingDetails => self.render_experiment_details(frame, area, theme),
        }
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match self.state {
            AnalysisState::BrowsingExperiments => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.select_previous();
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.select_next();
                        None
                    }
                    KeyCode::Enter => {
                        if self.selected_experiment().is_some() {
                            self.state = AnalysisState::ViewingDetails;
                            self.stats_scroll = 0;
                        }
                        None
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.load_experiments();
                        if !self.experiments.is_empty() && self.list_state.selected().is_none() {
                            self.list_state.select(Some(0));
                        }
                        None
                    }
                    _ => None,
                }
            }
            AnalysisState::ViewingDetails => {
                match key.code {
                    KeyCode::Esc => {
                        self.state = AnalysisState::BrowsingExperiments;
                        None
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.stats_scroll = self.stats_scroll.saturating_sub(1);
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(exp) = self.selected_experiment() {
                            let max_scroll = exp.stats.len().saturating_sub(5) as u16;
                            self.stats_scroll = (self.stats_scroll + 1).min(max_scroll);
                        }
                        None
                    }
                    KeyCode::PageUp => {
                        self.stats_scroll = self.stats_scroll.saturating_sub(10);
                        None
                    }
                    KeyCode::PageDown => {
                        if let Some(exp) = self.selected_experiment() {
                            let max_scroll = exp.stats.len().saturating_sub(5) as u16;
                            self.stats_scroll = (self.stats_scroll + 10).min(max_scroll);
                        }
                        None
                    }
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        // TODO: Navigate to weights screen with this experiment's weights
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}
