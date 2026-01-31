//! Dataset Analysis CLI - Statistics on MCTS training datasets.
//!
//! Analyzes JSONL.gz dataset files used for ML training.
//!
//! Usage:
//!   cargo run --release --bin dataset_stats -- \
//!     --input data/datasets/mcts_100k_sims100.jsonl.gz

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process;

use clap::Parser;
use flate2::read::GzDecoder;
use serde::Deserialize;

/// Dataset Analysis - Statistics on MCTS training datasets
#[derive(Parser, Debug)]
#[command(name = "dataset-stats")]
#[command(about = "Analyze MCTS training dataset files")]
struct Args {
    /// Path to the dataset (JSONL or JSONL.gz)
    #[arg(long, short = 'i')]
    input: PathBuf,

    /// Maximum games to analyze (for quick sampling)
    #[arg(long)]
    max_games: Option<usize>,

    /// Number of top actions to show
    #[arg(long, default_value = "10")]
    top_actions: usize,
}

/// Minimal game record - only deserialize what we need
#[derive(Debug, Deserialize)]
struct GameRecord {
    #[allow(dead_code)]
    game_id: String,
    deck1: String,
    deck2: String,
    winner: u8, // 0 = P1, 1 = P2
    moves: Vec<MoveRecord>,
}

/// Minimal move record
#[derive(Debug, Deserialize)]
struct MoveRecord {
    turn: u32,
    #[allow(dead_code)]
    player: u8,
    action: u8,
}

/// Accumulated statistics
struct DatasetStats {
    total_games: usize,
    total_positions: usize,
    p1_wins: usize,
    p2_wins: usize,
    draws: usize,
    game_lengths: Vec<u32>,
    action_counts: [u64; 256],
    deck_usage: HashMap<String, usize>,
}

impl DatasetStats {
    fn new() -> Self {
        Self {
            total_games: 0,
            total_positions: 0,
            p1_wins: 0,
            p2_wins: 0,
            draws: 0,
            game_lengths: Vec::new(),
            action_counts: [0; 256],
            deck_usage: HashMap::new(),
        }
    }

    fn add_game(&mut self, game: &GameRecord) {
        self.total_games += 1;
        self.total_positions += game.moves.len();

        // Track winner
        match game.winner {
            0 => self.p1_wins += 1,
            1 => self.p2_wins += 1,
            _ => self.draws += 1,
        }

        // Track game length (max turn number in moves)
        let max_turn = game.moves.iter().map(|m| m.turn).max().unwrap_or(0);
        self.game_lengths.push(max_turn);

        // Track actions
        for m in &game.moves {
            self.action_counts[m.action as usize] += 1;
        }

        // Track deck usage
        *self.deck_usage.entry(game.deck1.clone()).or_insert(0) += 1;
        *self.deck_usage.entry(game.deck2.clone()).or_insert(0) += 1;
    }

    fn avg_moves_per_game(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.total_positions as f64 / self.total_games as f64
        }
    }

    fn percentile(&self, p: usize) -> u32 {
        if self.game_lengths.is_empty() {
            return 0;
        }
        let mut sorted = self.game_lengths.clone();
        sorted.sort_unstable();
        let idx = (p * sorted.len() / 100).min(sorted.len() - 1);
        sorted[idx]
    }
}

fn main() {
    let args = Args::parse();

    // Validate input exists
    if !args.input.exists() {
        eprintln!("Error: Input file not found: {:?}", args.input);
        process::exit(1);
    }

    // Open file (with optional gzip decompression)
    let file = match File::open(&args.input) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            process::exit(1);
        }
    };

    let is_gzipped = args
        .input
        .extension()
        .is_some_and(|ext| ext == "gz");

    // Process file
    let stats = if is_gzipped {
        let decoder = GzDecoder::new(file);
        let reader = BufReader::new(decoder);
        process_lines(reader, args.max_games)
    } else {
        let reader = BufReader::new(file);
        process_lines(reader, args.max_games)
    };

    // Print results
    print_report(&args, &stats);
}

fn process_lines<R: BufRead>(reader: R, max_games: Option<usize>) -> DatasetStats {
    let mut stats = DatasetStats::new();
    let mut line_num = 0;
    let mut errors = 0;

    for line in reader.lines() {
        if let Some(max) = max_games {
            if stats.total_games >= max {
                break;
            }
        }

        line_num += 1;
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line {}: {}", line_num, e);
                errors += 1;
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<GameRecord>(&line) {
            Ok(game) => stats.add_game(&game),
            Err(e) => {
                if errors < 5 {
                    eprintln!("Parse error on line {}: {}", line_num, e);
                }
                errors += 1;
            }
        }

        // Progress indicator for large files
        if stats.total_games > 0 && stats.total_games.is_multiple_of(10000) {
            eprint!("\rProcessed {} games...", stats.total_games);
        }
    }

    if stats.total_games >= 10000 {
        eprintln!(); // Clear progress line
    }

    if errors > 0 {
        eprintln!("Warning: {} parse errors encountered", errors);
    }

    stats
}

fn print_report(args: &Args, stats: &DatasetStats) {
    let filename = args
        .input
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    println!("=== Dataset Analysis ===");
    println!("File: {}", filename);
    println!();

    // Overview
    println!("=== Overview ===");
    println!("Total games:     {:>10}", format_number(stats.total_games));
    println!(
        "Total positions: {:>10}",
        format_number(stats.total_positions)
    );
    println!("Avg moves/game:  {:>10.1}", stats.avg_moves_per_game());
    println!();

    // Win distribution
    println!("=== Win Distribution ===");
    let total = stats.total_games as f64;
    if total > 0.0 {
        println!(
            "P1 wins: {:>8} ({:5.1}%)",
            format_number(stats.p1_wins),
            stats.p1_wins as f64 / total * 100.0
        );
        println!(
            "P2 wins: {:>8} ({:5.1}%)",
            format_number(stats.p2_wins),
            stats.p2_wins as f64 / total * 100.0
        );
        if stats.draws > 0 {
            println!(
                "Draws:   {:>8} ({:5.1}%)",
                format_number(stats.draws),
                stats.draws as f64 / total * 100.0
            );
        }
    }
    println!();

    // Game length
    println!("=== Game Length ===");
    println!(
        "P10: {} | P50: {} | P90: {} turns",
        stats.percentile(10),
        stats.percentile(50),
        stats.percentile(90)
    );
    println!();

    // Top actions
    println!("=== Top {} Actions ===", args.top_actions);
    let mut action_vec: Vec<(usize, u64)> = stats
        .action_counts
        .iter()
        .enumerate()
        .filter(|(_, &count)| count > 0)
        .map(|(idx, &count)| (idx, count))
        .collect();
    action_vec.sort_by(|a, b| b.1.cmp(&a.1));

    let total_actions: u64 = stats.action_counts.iter().sum();
    for (idx, count) in action_vec.iter().take(args.top_actions) {
        let pct = *count as f64 / total_actions as f64 * 100.0;
        let action_name = action_name(*idx);
        println!("  {:>3}  {:<16} {:>5.1}%", idx, action_name, pct);
    }
    println!();

    // Deck usage
    println!("=== Deck Usage ===");
    let mut deck_vec: Vec<(&String, &usize)> = stats.deck_usage.iter().collect();
    deck_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (deck, count) in deck_vec.iter().take(12) {
        let pct = **count as f64 / (stats.total_games * 2) as f64 * 100.0;
        println!("  {:<28} {:>6} ({:5.1}%)", deck, count, pct);
    }
}

fn action_name(idx: usize) -> &'static str {
    match idx {
        0..=99 => "PlayCard",
        100..=149 => "Attack",
        150..=249 => "UseAbility",
        255 => "EndTurn",
        _ => "Unknown",
    }
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
