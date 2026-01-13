//! Weights screen - weight file management

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar, Toast};

/// Parsed weight file data
#[derive(Debug, Clone)]
struct WeightFile {
    name: String,
    path: PathBuf,
    display_name: String,
    weights: Option<ParsedWeights>,
}

#[derive(Debug, Clone)]
struct ParsedWeights {
    name: String,
    version: u32,
    greedy: GreedyWeightsData,
}

#[derive(Debug, Clone, Default)]
struct GreedyWeightsData {
    own_life: f32,
    enemy_life_damage: f32,
    own_creature_attack: f32,
    own_creature_health: f32,
    enemy_creature_attack: f32,
    enemy_creature_health: f32,
    creature_count: f32,
    board_advantage: f32,
    cards_in_hand: f32,
    action_points: f32,
    keyword_guard: f32,
    keyword_lethal: f32,
    keyword_lifesteal: f32,
    keyword_rush: f32,
    keyword_ranged: f32,
    keyword_piercing: f32,
    keyword_shield: f32,
    keyword_quick: f32,
    win_bonus: f32,
    lose_penalty: f32,
}

/// Screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeightsState {
    BrowsingWeights,
    ViewingDetails,
    Comparing,
    ConfirmPromote,
}

/// Weights screen state
#[derive(Debug, Clone)]
pub struct WeightsScreen {
    state: WeightsState,
    weights: Vec<WeightFile>,
    list_state: ListState,
    compare_idx: Option<usize>,
    scroll: u16,
}

impl WeightsScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            state: WeightsState::BrowsingWeights,
            weights: Vec::new(),
            list_state: ListState::default(),
            compare_idx: None,
            scroll: 0,
        };
        screen.load_weights();
        if !screen.weights.is_empty() {
            screen.list_state.select(Some(0));
        }
        screen
    }

    /// Create with comparison mode: new weights vs default
    pub fn with_compare(weights_path: &str) -> Self {
        let mut screen = Self::new();

        // Find the default weights index
        let default_idx = screen.weights.iter().position(|w| {
            w.display_name == "data/weights/default" || w.name == "default"
        });

        // Find the specified weights
        let weights_name = std::path::Path::new(weights_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(weights_path);

        let new_idx = screen.weights.iter().position(|w| {
            w.path.to_string_lossy().contains(weights_path) ||
            w.display_name.contains(weights_name)
        });

        // If we found both, set up comparison
        if let (Some(default), Some(new)) = (default_idx, new_idx) {
            screen.compare_idx = Some(default);
            screen.list_state.select(Some(new));
            screen.state = WeightsState::Comparing;
        } else if let Some(new) = new_idx {
            // Just select the new weights if no default found
            screen.list_state.select(Some(new));
            screen.state = WeightsState::ViewingDetails;
        }

        screen
    }

    fn load_weights(&mut self) {
        let mut weights: Vec<WeightFile> = Vec::new();

        // Load from data/weights/
        let data_weights_dir = PathBuf::from("data/weights");
        if data_weights_dir.exists() {
            self.load_weights_from_dir(&data_weights_dir, &mut weights, "data/weights");
        }

        // Load from experiments (recent experiment weights)
        let experiments_dir = PathBuf::from("experiments/mcts");
        if experiments_dir.exists() {
            if let Ok(entries) = fs::read_dir(&experiments_dir) {
                let mut exp_dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
                // Sort by name (timestamp) descending to get most recent first
                exp_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                // Only show 5 most recent experiment weights
                for entry in exp_dirs.into_iter().take(5) {
                    let weights_path = entry.path().join("weights.toml");
                    if weights_path.exists() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let parsed = Self::parse_weights(&weights_path);
                        weights.push(WeightFile {
                            name: name.clone(),
                            path: weights_path,
                            display_name: format!("experiments/{}", name),
                            weights: parsed,
                        });
                    }
                }
            }
        }

        self.weights = weights;
    }

    fn load_weights_from_dir(&self, dir: &PathBuf, weights: &mut Vec<WeightFile>, prefix: &str) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "toml").unwrap_or(false) {
                    let name = path.file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let parsed = Self::parse_weights(&path);
                    weights.push(WeightFile {
                        name: name.clone(),
                        path,
                        display_name: format!("{}/{}", prefix, name),
                        weights: parsed,
                    });
                }
            }
        }
    }

    fn parse_weights(path: &PathBuf) -> Option<ParsedWeights> {
        let content = fs::read_to_string(path).ok()?;

        let mut name = String::new();
        let mut version = 1;
        let mut greedy = GreedyWeightsData::default();
        let mut in_greedy_section = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("[default.greedy]") {
                in_greedy_section = true;
                continue;
            }
            if line.starts_with('[') {
                in_greedy_section = false;
                continue;
            }

            if in_greedy_section {
                Self::parse_greedy_field(line, &mut greedy);
            } else {
                if let Some(value) = line.strip_prefix("name = ") {
                    name = value.trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("version = ") {
                    version = value.parse().unwrap_or(1);
                }
            }
        }

        Some(ParsedWeights { name, version, greedy })
    }

    fn parse_greedy_field(line: &str, greedy: &mut GreedyWeightsData) {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return;
        }
        let key = parts[0].trim();
        let value: f32 = parts[1].trim().parse().unwrap_or(0.0);

        match key {
            "own_life" => greedy.own_life = value,
            "enemy_life_damage" => greedy.enemy_life_damage = value,
            "own_creature_attack" => greedy.own_creature_attack = value,
            "own_creature_health" => greedy.own_creature_health = value,
            "enemy_creature_attack" => greedy.enemy_creature_attack = value,
            "enemy_creature_health" => greedy.enemy_creature_health = value,
            "creature_count" => greedy.creature_count = value,
            "board_advantage" => greedy.board_advantage = value,
            "cards_in_hand" => greedy.cards_in_hand = value,
            "action_points" => greedy.action_points = value,
            "keyword_guard" => greedy.keyword_guard = value,
            "keyword_lethal" => greedy.keyword_lethal = value,
            "keyword_lifesteal" => greedy.keyword_lifesteal = value,
            "keyword_rush" => greedy.keyword_rush = value,
            "keyword_ranged" => greedy.keyword_ranged = value,
            "keyword_piercing" => greedy.keyword_piercing = value,
            "keyword_shield" => greedy.keyword_shield = value,
            "keyword_quick" => greedy.keyword_quick = value,
            "win_bonus" => greedy.win_bonus = value,
            "lose_penalty" => greedy.lose_penalty = value,
            _ => {}
        }
    }

    fn selected_weight(&self) -> Option<&WeightFile> {
        self.list_state.selected().and_then(|i| self.weights.get(i))
    }

    fn select_next(&mut self) {
        if self.weights.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.weights.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        if self.weights.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn promote_selected(&mut self) -> bool {
        let selected = match self.selected_weight() {
            Some(w) => w.clone(),
            None => return false,
        };

        let dest = PathBuf::from("data/weights/default.toml");

        // Copy the file
        match fs::copy(&selected.path, &dest) {
            Ok(_) => {
                self.load_weights(); // Refresh list
                true
            }
            Err(_) => false,
        }
    }

    fn render_weight_list(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // List
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Weight Manager")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Weight list
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Weight Files ");

        if self.weights.is_empty() {
            let empty = Paragraph::new("\n  No weight files found.\n\n  Run tuning to generate weights!")
                .style(Style::default().fg(theme.fg_dim))
                .block(list_block);
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.weights
                .iter()
                .enumerate()
                .map(|(idx, w)| {
                    let is_default = w.display_name == "data/weights/default";
                    let marker = if is_default { " [DEFAULT]" } else { "" };
                    let compare_marker = if self.compare_idx == Some(idx) { " [COMPARE]" } else { "" };

                    let style = if is_default {
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    ListItem::new(format!("  {}{}{}", w.display_name, marker, compare_marker))
                        .style(style)
                })
                .collect();

            let list = List::new(items)
                .block(list_block)
                .highlight_style(Style::default().fg(theme.primary).bold())
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state.clone());
        }

        // Footer
        let hints = if self.compare_idx.is_some() {
            vec![
                KeyHint::new("↑↓", "Navigate"),
                KeyHint::new("Enter", "Compare"),
                KeyHint::new("Esc", "Cancel Compare"),
            ]
        } else {
            vec![
                KeyHint::new("↑↓", "Navigate"),
                KeyHint::new("Enter", "View"),
                KeyHint::new("C", "Compare"),
                KeyHint::new("P", "Promote"),
                KeyHint::new("R", "Refresh"),
            ]
        };
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn render_weight_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let weight = match self.selected_weight() {
            Some(w) => w,
            None => return,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Details
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(format!(" Weight: {} ", weight.display_name))
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Details
        let details_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Parameters ");
        let inner = details_block.inner(chunks[1]);
        frame.render_widget(details_block, chunks[1]);

        if let Some(ref w) = weight.weights {
            let lines = self.render_weight_params(w, theme);
            let para = Paragraph::new(lines).scroll((self.scroll, 0));
            frame.render_widget(para, inner);
        } else {
            let no_data = Paragraph::new("  Unable to parse weight file")
                .style(Style::default().fg(theme.error));
            frame.render_widget(no_data, inner);
        }

        // Footer
        let hints = vec![
            KeyHint::new("↑↓", "Scroll"),
            KeyHint::new("P", "Promote to Default"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn render_weight_params(&self, w: &ParsedWeights, theme: &Theme) -> Vec<Line<'static>> {
        let g = &w.greedy;
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name: ", Style::default().fg(theme.fg_dim)),
                Span::styled(w.name.clone(), Style::default().fg(theme.info)),
                Span::styled("    Version: ", Style::default().fg(theme.fg_dim)),
                Span::styled(format!("{}", w.version), Style::default().fg(theme.info)),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Life:", Style::default().fg(theme.fg).bold())),
            Self::param_line("    own_life", g.own_life, theme),
            Self::param_line("    enemy_life_damage", g.enemy_life_damage, theme),
            Line::from(""),
            Line::from(Span::styled("  Creatures:", Style::default().fg(theme.fg).bold())),
            Self::param_line("    own_creature_attack", g.own_creature_attack, theme),
            Self::param_line("    own_creature_health", g.own_creature_health, theme),
            Self::param_line("    enemy_creature_attack", g.enemy_creature_attack, theme),
            Self::param_line("    enemy_creature_health", g.enemy_creature_health, theme),
            Line::from(""),
            Line::from(Span::styled("  Board:", Style::default().fg(theme.fg).bold())),
            Self::param_line("    creature_count", g.creature_count, theme),
            Self::param_line("    board_advantage", g.board_advantage, theme),
            Line::from(""),
            Line::from(Span::styled("  Resources:", Style::default().fg(theme.fg).bold())),
            Self::param_line("    cards_in_hand", g.cards_in_hand, theme),
            Self::param_line("    action_points", g.action_points, theme),
            Line::from(""),
            Line::from(Span::styled("  Keywords:", Style::default().fg(theme.fg).bold())),
            Self::param_line("    guard", g.keyword_guard, theme),
            Self::param_line("    lethal", g.keyword_lethal, theme),
            Self::param_line("    lifesteal", g.keyword_lifesteal, theme),
            Self::param_line("    rush", g.keyword_rush, theme),
            Self::param_line("    ranged", g.keyword_ranged, theme),
            Self::param_line("    piercing", g.keyword_piercing, theme),
            Self::param_line("    shield", g.keyword_shield, theme),
            Self::param_line("    quick", g.keyword_quick, theme),
            Line::from(""),
            Line::from(Span::styled("  Terminal:", Style::default().fg(theme.fg).bold())),
            Self::param_line("    win_bonus", g.win_bonus, theme),
            Self::param_line("    lose_penalty", g.lose_penalty, theme),
        ]
    }

    fn param_line(name: &str, value: f32, theme: &Theme) -> Line<'static> {
        let color = if value > 0.0 {
            theme.success
        } else if value < 0.0 {
            theme.error
        } else {
            theme.fg_dim
        };

        Line::from(vec![
            Span::styled(format!("{:<25}", name), Style::default().fg(theme.fg_dim)),
            Span::styled(format!("{:>10.4}", value), Style::default().fg(color)),
        ])
    }

    fn render_compare(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let (weight1, weight2) = match (self.compare_idx, self.list_state.selected()) {
            (Some(i1), Some(i2)) => {
                match (self.weights.get(i1), self.weights.get(i2)) {
                    (Some(w1), Some(w2)) => (w1, w2),
                    _ => return,
                }
            }
            _ => return,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Comparison
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Weight Comparison")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Comparison
        let comp_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(format!(" {} vs {} ", weight1.name, weight2.name));
        let inner = comp_block.inner(chunks[1]);
        frame.render_widget(comp_block, chunks[1]);

        match (&weight1.weights, &weight2.weights) {
            (Some(w1), Some(w2)) => {
                let lines = self.render_weight_diff(&w1.greedy, &w2.greedy, theme);
                let para = Paragraph::new(lines).scroll((self.scroll, 0));
                frame.render_widget(para, inner);
            }
            _ => {
                let err = Paragraph::new("  Unable to parse one or both weight files")
                    .style(Style::default().fg(theme.error));
                frame.render_widget(err, inner);
            }
        }

        // Footer
        let hints = vec![
            KeyHint::new("↑↓", "Scroll"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn render_weight_diff(&self, g1: &GreedyWeightsData, g2: &GreedyWeightsData, theme: &Theme) -> Vec<Line<'static>> {
        vec![
            Line::from(vec![
                Span::styled(format!("{:<25}", "Parameter"), Style::default().fg(theme.fg_dim).bold()),
                Span::styled(format!("{:>10}", "Left"), Style::default().fg(theme.fg_dim).bold()),
                Span::styled(format!("{:>10}", "Right"), Style::default().fg(theme.fg_dim).bold()),
                Span::styled(format!("{:>10}", "Diff"), Style::default().fg(theme.fg_dim).bold()),
            ]),
            Line::from(Span::styled("─".repeat(55), Style::default().fg(theme.border))),
            Self::diff_line("own_life", g1.own_life, g2.own_life, theme),
            Self::diff_line("enemy_life_damage", g1.enemy_life_damage, g2.enemy_life_damage, theme),
            Self::diff_line("own_creature_attack", g1.own_creature_attack, g2.own_creature_attack, theme),
            Self::diff_line("own_creature_health", g1.own_creature_health, g2.own_creature_health, theme),
            Self::diff_line("enemy_creature_attack", g1.enemy_creature_attack, g2.enemy_creature_attack, theme),
            Self::diff_line("enemy_creature_health", g1.enemy_creature_health, g2.enemy_creature_health, theme),
            Self::diff_line("creature_count", g1.creature_count, g2.creature_count, theme),
            Self::diff_line("board_advantage", g1.board_advantage, g2.board_advantage, theme),
            Self::diff_line("cards_in_hand", g1.cards_in_hand, g2.cards_in_hand, theme),
            Self::diff_line("action_points", g1.action_points, g2.action_points, theme),
            Self::diff_line("keyword_guard", g1.keyword_guard, g2.keyword_guard, theme),
            Self::diff_line("keyword_lethal", g1.keyword_lethal, g2.keyword_lethal, theme),
            Self::diff_line("keyword_lifesteal", g1.keyword_lifesteal, g2.keyword_lifesteal, theme),
            Self::diff_line("keyword_rush", g1.keyword_rush, g2.keyword_rush, theme),
            Self::diff_line("keyword_ranged", g1.keyword_ranged, g2.keyword_ranged, theme),
            Self::diff_line("keyword_piercing", g1.keyword_piercing, g2.keyword_piercing, theme),
            Self::diff_line("keyword_shield", g1.keyword_shield, g2.keyword_shield, theme),
            Self::diff_line("keyword_quick", g1.keyword_quick, g2.keyword_quick, theme),
            Self::diff_line("win_bonus", g1.win_bonus, g2.win_bonus, theme),
            Self::diff_line("lose_penalty", g1.lose_penalty, g2.lose_penalty, theme),
        ]
    }

    fn diff_line(name: &str, v1: f32, v2: f32, theme: &Theme) -> Line<'static> {
        let diff = v2 - v1;
        let diff_color = if diff.abs() < 0.001 {
            theme.fg_dim
        } else if diff > 0.0 {
            theme.success
        } else {
            theme.error
        };

        Line::from(vec![
            Span::styled(format!("{:<25}", name), Style::default().fg(theme.fg)),
            Span::styled(format!("{:>10.3}", v1), Style::default().fg(theme.fg)),
            Span::styled(format!("{:>10.3}", v2), Style::default().fg(theme.fg)),
            Span::styled(format!("{:>+10.3}", diff), Style::default().fg(diff_color)),
        ])
    }

    fn render_confirm_promote(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let weight = match self.selected_weight() {
            Some(w) => w,
            None => return,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new(" Confirm Promotion")
            .style(Style::default().fg(theme.fg).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.warning))
                    .style(Style::default().bg(theme.bg_alt)),
            );
        frame.render_widget(header, chunks[0]);

        // Confirmation message
        let msg_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));
        let inner = msg_block.inner(chunks[1]);
        frame.render_widget(msg_block, chunks[1]);

        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  Promote weights to default?", Style::default().fg(theme.fg).bold())),
            Line::from(""),
            Line::from(vec![
                Span::styled("  From: ", Style::default().fg(theme.fg_dim)),
                Span::styled(&weight.display_name, Style::default().fg(theme.info)),
            ]),
            Line::from(vec![
                Span::styled("  To:   ", Style::default().fg(theme.fg_dim)),
                Span::styled("data/weights/default.toml", Style::default().fg(theme.warning)),
            ]),
            Line::from(""),
            Line::from(Span::styled("  This will overwrite the current default weights.", Style::default().fg(theme.warning))),
        ]);
        frame.render_widget(msg, inner);

        // Footer
        let hints = vec![
            KeyHint::new("Y", "Confirm"),
            KeyHint::new("N/Esc", "Cancel"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }
}

impl Default for WeightsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for WeightsScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match self.state {
            WeightsState::BrowsingWeights => self.render_weight_list(frame, area, theme),
            WeightsState::ViewingDetails => self.render_weight_details(frame, area, theme),
            WeightsState::Comparing => self.render_compare(frame, area, theme),
            WeightsState::ConfirmPromote => self.render_confirm_promote(frame, area, theme),
        }
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        match self.state {
            WeightsState::BrowsingWeights => {
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
                        if self.compare_idx.is_some() {
                            // Complete comparison
                            if self.list_state.selected() != self.compare_idx {
                                self.state = WeightsState::Comparing;
                                self.scroll = 0;
                            }
                        } else if self.selected_weight().is_some() {
                            self.state = WeightsState::ViewingDetails;
                            self.scroll = 0;
                        }
                        None
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if self.compare_idx.is_some() {
                            self.compare_idx = None; // Cancel compare
                        } else {
                            self.compare_idx = self.list_state.selected(); // Start compare
                        }
                        None
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        if self.selected_weight().is_some() {
                            self.state = WeightsState::ConfirmPromote;
                        }
                        None
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.load_weights();
                        if !self.weights.is_empty() && self.list_state.selected().is_none() {
                            self.list_state.select(Some(0));
                        }
                        None
                    }
                    KeyCode::Esc => {
                        if self.compare_idx.is_some() {
                            self.compare_idx = None;
                            None
                        } else {
                            None // Let parent handle Esc
                        }
                    }
                    _ => None,
                }
            }
            WeightsState::ViewingDetails => {
                match key.code {
                    KeyCode::Esc => {
                        self.state = WeightsState::BrowsingWeights;
                        None
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.scroll = self.scroll.saturating_sub(1);
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.scroll += 1;
                        None
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.state = WeightsState::ConfirmPromote;
                        None
                    }
                    _ => None,
                }
            }
            WeightsState::Comparing => {
                match key.code {
                    KeyCode::Esc => {
                        self.state = WeightsState::BrowsingWeights;
                        self.compare_idx = None;
                        None
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.scroll = self.scroll.saturating_sub(1);
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.scroll += 1;
                        None
                    }
                    _ => None,
                }
            }
            WeightsState::ConfirmPromote => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        let success = self.promote_selected();
                        self.state = WeightsState::BrowsingWeights;
                        if success {
                            Some(Message::ShowToast(Toast::success("Weights promoted to default")))
                        } else {
                            Some(Message::ShowToast(Toast::error("Failed to promote weights")))
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.state = WeightsState::BrowsingWeights;
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}
