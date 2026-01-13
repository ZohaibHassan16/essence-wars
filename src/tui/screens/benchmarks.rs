//! Benchmarks screen - engine performance analysis

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::ScreenWidget;
use crate::bots::{GreedyBot, RandomBot, Bot};
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar, Toast};
use crate::types::CardId;

/// Format number with thousands separators
fn format_number(n: f64) -> String {
    let s = format!("{:.0}", n);
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

/// Benchmark result
#[derive(Debug, Clone)]
struct BenchmarkResult {
    timestamp: String,
    random_games_per_sec: f64,
    greedy_games_per_sec: f64,
    random_total_games: u32,
    greedy_total_games: u32,
    random_elapsed: f64,
    greedy_elapsed: f64,
}

/// Progress updates from benchmark task
#[derive(Debug, Clone)]
enum BenchmarkProgress {
    Started,
    RandomComplete(f64, u32, f64), // games/sec, total, elapsed
    GreedyComplete(f64, u32, f64),
    Complete(BenchmarkResult),
    Error(String),
}

/// Benchmark task handle
struct BenchmarkTaskHandle {
    receiver: Receiver<BenchmarkProgress>,
    _handle: JoinHandle<()>,
}

/// Screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BenchmarkState {
    Idle,
    Running,
    Complete,
}

/// Benchmarks screen state
pub struct BenchmarksScreen {
    state: BenchmarkState,
    results: Vec<BenchmarkResult>,
    current_result: Option<BenchmarkResult>,
    list_state: ListState,
    task_handle: Option<BenchmarkTaskHandle>,
    random_progress: Option<(f64, u32, f64)>,
    greedy_progress: Option<(f64, u32, f64)>,
    error_message: Option<String>,
}

impl BenchmarksScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            state: BenchmarkState::Idle,
            results: Vec::new(),
            current_result: None,
            list_state: ListState::default(),
            task_handle: None,
            random_progress: None,
            greedy_progress: None,
            error_message: None,
        };
        screen.load_history();
        screen
    }

    fn load_history(&mut self) {
        let history_dir = PathBuf::from("benchmark_results");
        if !history_dir.exists() {
            return;
        }

        let mut results = Vec::new();
        if let Ok(entries) = fs::read_dir(&history_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "txt").unwrap_or(false) {
                    if let Some(result) = Self::parse_result_file(&path) {
                        results.push(result);
                    }
                }
            }
        }

        // Sort by timestamp descending
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.results = results;

        if !self.results.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn parse_result_file(path: &PathBuf) -> Option<BenchmarkResult> {
        let content = fs::read_to_string(path).ok()?;
        let filename = path.file_stem()?.to_str()?;

        let mut random_games_per_sec = 0.0;
        let mut greedy_games_per_sec = 0.0;
        let mut random_total_games = 0;
        let mut greedy_total_games = 0;
        let mut random_elapsed = 0.0;
        let mut greedy_elapsed = 0.0;

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Random games/sec: ") {
                random_games_per_sec = value.parse().unwrap_or(0.0);
            } else if let Some(value) = line.strip_prefix("Greedy games/sec: ") {
                greedy_games_per_sec = value.parse().unwrap_or(0.0);
            } else if let Some(value) = line.strip_prefix("Random total games: ") {
                random_total_games = value.parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("Greedy total games: ") {
                greedy_total_games = value.parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("Random elapsed: ") {
                random_elapsed = value.trim_end_matches('s').parse().unwrap_or(0.0);
            } else if let Some(value) = line.strip_prefix("Greedy elapsed: ") {
                greedy_elapsed = value.trim_end_matches('s').parse().unwrap_or(0.0);
            }
        }

        Some(BenchmarkResult {
            timestamp: filename.to_string(),
            random_games_per_sec,
            greedy_games_per_sec,
            random_total_games,
            greedy_total_games,
            random_elapsed,
            greedy_elapsed,
        })
    }

    fn start_benchmark(&mut self) {
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            run_benchmark(sender);
        });

        self.task_handle = Some(BenchmarkTaskHandle {
            receiver,
            _handle: handle,
        });
        self.state = BenchmarkState::Running;
        self.random_progress = None;
        self.greedy_progress = None;
        self.error_message = None;
        self.current_result = None;
    }

    fn save_result(&self, result: &BenchmarkResult) {
        let dir = PathBuf::from("benchmark_results");
        let _ = fs::create_dir_all(&dir);

        let path = dir.join(format!("{}.txt", result.timestamp));
        let content = format!(
            "Benchmark Results\n\
             =================\n\
             Timestamp: {}\n\
             \n\
             Random vs Random:\n\
             Random games/sec: {:.0}\n\
             Random total games: {}\n\
             Random elapsed: {:.2}s\n\
             \n\
             Greedy vs Greedy:\n\
             Greedy games/sec: {:.0}\n\
             Greedy total games: {}\n\
             Greedy elapsed: {:.2}s\n",
            result.timestamp,
            result.random_games_per_sec,
            result.random_total_games,
            result.random_elapsed,
            result.greedy_games_per_sec,
            result.greedy_total_games,
            result.greedy_elapsed
        );
        let _ = fs::write(path, content);
    }

    fn select_next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.results.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn render_idle(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Length(10),  // Current/Latest results
                Constraint::Min(5),      // History
                Constraint::Length(3),   // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Engine Benchmarks")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Current results
        let results_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Latest Results ");
        let inner = results_block.inner(chunks[1]);
        frame.render_widget(results_block, chunks[1]);

        let result = self.current_result.as_ref().or_else(|| self.results.first());
        if let Some(r) = result {
            let lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Random vs Random:  ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{} games/sec", format_number(r.random_games_per_sec)), Style::default().fg(theme.success)),
                    Span::styled(format!("  ({} games in {:.2}s)", r.random_total_games, r.random_elapsed), Style::default().fg(theme.fg_dim)),
                ]),
                Line::from(vec![
                    Span::styled("  Greedy vs Greedy:  ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{} games/sec", format_number(r.greedy_games_per_sec)), Style::default().fg(theme.primary)),
                    Span::styled(format!("  ({} games in {:.2}s)", r.greedy_total_games, r.greedy_elapsed), Style::default().fg(theme.fg_dim)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Timestamp: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(&r.timestamp, Style::default().fg(theme.info)),
                ]),
            ];
            let para = Paragraph::new(lines);
            frame.render_widget(para, inner);
        } else if let Some(ref err) = self.error_message {
            let error_text = Paragraph::new(format!("\n  Error: {}\n\n  Press R to retry.", err))
                .style(Style::default().fg(theme.error));
            frame.render_widget(error_text, inner);
        } else {
            let no_results = Paragraph::new("\n  No benchmark results yet.\n\n  Press R to run benchmarks!")
                .style(Style::default().fg(theme.fg_dim));
            frame.render_widget(no_results, inner);
        }

        // History
        let history_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" History ");

        if self.results.is_empty() {
            let empty = Paragraph::new("  No history")
                .style(Style::default().fg(theme.fg_dim))
                .block(history_block);
            frame.render_widget(empty, chunks[2]);
        } else {
            let items: Vec<ListItem> = self.results
                .iter()
                .map(|r| {
                    ListItem::new(format!(
                        "  {}  |  Random: {}/s  |  Greedy: {}/s",
                        r.timestamp, format_number(r.random_games_per_sec), format_number(r.greedy_games_per_sec)
                    ))
                })
                .collect();

            let list = List::new(items)
                .block(history_block)
                .highlight_style(Style::default().fg(theme.primary).bold())
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[2], &mut self.list_state.clone());
        }

        // Footer
        let hints = vec![
            KeyHint::new("R", "Run Benchmark"),
            KeyHint::new("↑↓", "Navigate History"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[3]);
    }

    fn render_running(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Running Benchmarks...")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.warning))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Progress
        let progress_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Progress ");
        let inner = progress_block.inner(chunks[1]);
        frame.render_widget(progress_block, chunks[1]);

        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled("  Running performance benchmarks...", Style::default().fg(theme.fg))),
            Line::from(""),
        ];

        if let Some((games_per_sec, total, elapsed)) = self.random_progress {
            lines.push(Line::from(vec![
                Span::styled("  Random vs Random: ", Style::default().fg(theme.fg_dim)),
                Span::styled(format!("{} games/sec", format_number(games_per_sec)), Style::default().fg(theme.success)),
                Span::styled(format!("  ({} games in {:.2}s)", total, elapsed), Style::default().fg(theme.fg_dim)),
            ]));
            lines.push(Line::from(""));

            if let Some((greedy_gps, greedy_total, greedy_elapsed)) = self.greedy_progress {
                lines.push(Line::from(vec![
                    Span::styled("  Greedy vs Greedy: ", Style::default().fg(theme.fg_dim)),
                    Span::styled(format!("{} games/sec", format_number(greedy_gps)), Style::default().fg(theme.primary)),
                    Span::styled(format!("  ({} games in {:.2}s)", greedy_total, greedy_elapsed), Style::default().fg(theme.fg_dim)),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("  Finalizing...", Style::default().fg(theme.fg))));
            } else {
                lines.push(Line::from(Span::styled("  Running Greedy vs Greedy...", Style::default().fg(theme.fg))));
            }
        } else {
            lines.push(Line::from(Span::styled("  Running Random vs Random...", Style::default().fg(theme.fg))));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);

        // Footer
        let hints = vec![KeyHint::new("", "Please wait...")];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }
}

impl Default for BenchmarksScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BenchmarksScreen {
    fn clone(&self) -> Self {
        let mut screen = Self::new();
        screen.state = BenchmarkState::Idle;
        screen.results = self.results.clone();
        screen.current_result = self.current_result.clone();
        screen
    }
}

impl std::fmt::Debug for BenchmarksScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BenchmarksScreen")
            .field("state", &self.state)
            .field("results_count", &self.results.len())
            .finish()
    }
}

impl ScreenWidget for BenchmarksScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match self.state {
            BenchmarkState::Idle | BenchmarkState::Complete => self.render_idle(frame, area, theme),
            BenchmarkState::Running => self.render_running(frame, area, theme),
        }
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match self.state {
            BenchmarkState::Idle | BenchmarkState::Complete => {
                match key.code {
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.start_benchmark();
                        None
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.select_previous();
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.select_next();
                        None
                    }
                    _ => None,
                }
            }
            BenchmarkState::Running => None, // No input while running
        }
    }

    fn tick(&mut self) -> Option<Message> {
        // Collect progress updates first to avoid borrow issues
        let mut updates = Vec::new();
        if let Some(ref handle) = self.task_handle {
            while let Ok(progress) = handle.receiver.try_recv() {
                updates.push(progress);
            }
        }

        let mut toast: Option<Toast> = None;

        // Process updates
        for progress in updates {
            match progress {
                BenchmarkProgress::Started => {}
                BenchmarkProgress::RandomComplete(games_per_sec, total, elapsed) => {
                    self.random_progress = Some((games_per_sec, total, elapsed));
                }
                BenchmarkProgress::GreedyComplete(games_per_sec, total, elapsed) => {
                    self.greedy_progress = Some((games_per_sec, total, elapsed));
                }
                BenchmarkProgress::Complete(result) => {
                    let random_gps = result.random_games_per_sec;
                    self.save_result(&result);
                    self.current_result = Some(result);
                    self.state = BenchmarkState::Complete;
                    self.load_history(); // Refresh history
                    toast = Some(Toast::success(format!("Benchmark complete! {} games/sec", format_number(random_gps))));
                }
                BenchmarkProgress::Error(msg) => {
                    self.error_message = Some(msg.clone());
                    self.state = BenchmarkState::Idle;
                    toast = Some(Toast::error(format!("Benchmark error: {}", msg)));
                }
            }
        }

        toast.map(Message::ShowToast)
    }
}

/// Run the benchmark in a background thread
fn run_benchmark(sender: Sender<BenchmarkProgress>) {
    let _ = sender.send(BenchmarkProgress::Started);

    // Load card database
    let card_db = match CardDatabase::load_from_directory("data/cards") {
        Ok(db) => db,
        Err(e) => {
            let _ = sender.send(BenchmarkProgress::Error(format!("Failed to load cards: {}", e)));
            return;
        }
    };

    // Default deck for benchmarks
    let deck: Vec<CardId> = vec![
        CardId(1), CardId(1),
        CardId(3), CardId(3),
        CardId(6), CardId(6),
        CardId(8), CardId(8),
        CardId(11), CardId(11),
        CardId(12), CardId(12),
        CardId(16), CardId(16),
        CardId(20), CardId(20),
        CardId(34), CardId(34),
    ];

    // Benchmark Random vs Random
    let random_games = 10000;
    let start = Instant::now();

    for i in 0..random_games {
        let seed = 42u64.wrapping_add(i as u64);
        run_random_game(&card_db, &deck, seed);
    }

    let random_elapsed = start.elapsed().as_secs_f64();
    let random_games_per_sec = random_games as f64 / random_elapsed;
    let _ = sender.send(BenchmarkProgress::RandomComplete(
        random_games_per_sec,
        random_games,
        random_elapsed,
    ));

    // Benchmark Greedy vs Greedy
    let greedy_games = 1000;
    let start = Instant::now();

    for i in 0..greedy_games {
        let seed = 42u64.wrapping_add(i as u64);
        run_greedy_game(&card_db, &deck, seed);
    }

    let greedy_elapsed = start.elapsed().as_secs_f64();
    let greedy_games_per_sec = greedy_games as f64 / greedy_elapsed;
    let _ = sender.send(BenchmarkProgress::GreedyComplete(
        greedy_games_per_sec,
        greedy_games,
        greedy_elapsed,
    ));

    // Create result
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string();
    let result = BenchmarkResult {
        timestamp,
        random_games_per_sec,
        greedy_games_per_sec,
        random_total_games: random_games,
        greedy_total_games: greedy_games,
        random_elapsed,
        greedy_elapsed,
    };

    let _ = sender.send(BenchmarkProgress::Complete(result));
}

fn run_random_game(card_db: &CardDatabase, deck: &[CardId], seed: u64) {
    let mut bot1 = RandomBot::new(seed);
    let mut bot2 = RandomBot::new(seed.wrapping_add(1000));

    let mut engine = GameEngine::new(card_db);
    engine.start_game(deck.to_vec(), deck.to_vec(), seed);

    let mut action_count = 0;
    while !engine.is_game_over() && action_count < 500 {
        let state_tensor = engine.get_state_tensor();
        let legal_mask = engine.get_legal_action_mask();
        let legal_actions = engine.get_legal_actions();

        let action = if engine.current_player().0 == 0 {
            bot1.select_action(&state_tensor, &legal_mask, &legal_actions)
        } else {
            bot2.select_action(&state_tensor, &legal_mask, &legal_actions)
        };

        let _ = engine.apply_action(action);
        action_count += 1;
    }
}

fn run_greedy_game(card_db: &CardDatabase, deck: &[CardId], seed: u64) {
    let mut bot1 = GreedyBot::new(card_db, seed);
    let mut bot2 = GreedyBot::new(card_db, seed.wrapping_add(1000));

    let mut engine = GameEngine::new(card_db);
    engine.start_game(deck.to_vec(), deck.to_vec(), seed);

    let mut action_count = 0;
    while !engine.is_game_over() && action_count < 500 {
        let action = if engine.current_player().0 == 0 {
            bot1.select_action_with_engine(&engine)
        } else {
            bot2.select_action_with_engine(&engine)
        };

        let _ = engine.apply_action(action);
        action_count += 1;
    }
}
