//! Weight Diff CLI - Compare two weight files side-by-side.
//!
//! Usage:
//!   cargo run --release --bin weight-diff -- --file1 data/weights/default.toml --file2 data/weights/generalist.toml
//!   cargo run --release --bin weight-diff -- --file1 weights_a.toml --file2 weights_b.toml --all
//!   cargo run --release --bin weight-diff -- --file1 weights_a.toml --file2 weights_b.toml --threshold 0.1

use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::bots::BotWeights;

/// Weight Diff - Compare two weight files
#[derive(Parser, Debug)]
#[command(name = "weight-diff")]
#[command(about = "Compare two weight files and show differences", long_about = None)]
struct Args {
    /// First weight file to compare
    #[arg(long)]
    file1: PathBuf,

    /// Second weight file to compare
    #[arg(long)]
    file2: PathBuf,

    /// Minimum delta to show (default: 0.01)
    #[arg(long, default_value = "0.01")]
    threshold: f32,

    /// Show all parameters (including unchanged)
    #[arg(long)]
    all: bool,
}

/// Weight parameter names (matching GreedyWeights::to_vec order)
const PARAM_NAMES: [&str; 28] = [
    "own_life",
    "enemy_life_damage",
    "own_creature_value",
    "enemy_creature_damage",
    "own_attack_power",
    "enemy_attack_power",
    "own_health_value",
    "enemy_health_damage",
    "own_hand_size",
    "enemy_hand_size",
    "own_essence",
    "enemy_essence",
    "own_support_value",
    "enemy_support_damage",
    "action_points",
    "keyword_guard",
    "keyword_lethal",
    "keyword_lifesteal",
    "keyword_rush",
    "keyword_ranged",
    "keyword_piercing",
    "keyword_shield",
    "keyword_quick",
    "terminal_win",
    "terminal_loss",
    "terminal_draw",
    "turn_penalty",
    "empty_slot_penalty",
];

fn main() {
    let args = Args::parse();

    // Load both weight files
    let weights1 = match BotWeights::load(&args.file1) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading {:?}: {}", args.file1, e);
            process::exit(1);
        }
    };

    let weights2 = match BotWeights::load(&args.file2) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading {:?}: {}", args.file2, e);
            process::exit(1);
        }
    };

    // Extract GreedyWeights and convert to vectors
    let vec1 = weights1.default.greedy.to_vec();
    let vec2 = weights2.default.greedy.to_vec();

    // Get file names for display
    let name1 = args
        .file1
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file1".to_string());
    let name2 = args
        .file2
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file2".to_string());

    // Print header
    println!("Weight Comparison: {} vs {}", name1, name2);
    println!("{}", "=".repeat(76));
    println!(
        "{:<24} {:>10} {:>10} {:>10} {:>10}",
        "Parameter", &name1[..name1.len().min(10)], &name2[..name2.len().min(10)], "Delta", "Change"
    );
    println!("{}", "-".repeat(76));

    // Collect comparisons
    let mut comparisons: Vec<(usize, f32, f32, f32, f32)> = Vec::new();
    let mut largest_increase: Option<(usize, f32)> = None;
    let mut largest_decrease: Option<(usize, f32)> = None;

    for (i, (v1, v2)) in vec1.iter().zip(vec2.iter()).enumerate() {
        let delta = v2 - v1;
        let pct_change = if v1.abs() > 0.0001 {
            (delta / v1.abs()) * 100.0
        } else if delta.abs() > 0.0001 {
            f32::INFINITY * delta.signum()
        } else {
            0.0
        };

        comparisons.push((i, *v1, *v2, delta, pct_change));

        // Track largest changes
        if delta > 0.0
            && (largest_increase.is_none() || delta > largest_increase.unwrap().1)
        {
            largest_increase = Some((i, delta));
        } else if delta < 0.0
            && (largest_decrease.is_none() || delta < largest_decrease.unwrap().1)
        {
            largest_decrease = Some((i, delta));
        }
    }

    // Sort by absolute delta (largest first)
    comparisons.sort_by(|a, b| b.3.abs().partial_cmp(&a.3.abs()).unwrap_or(std::cmp::Ordering::Equal));

    // Print comparisons
    let mut changes_count = 0;
    for (i, v1, v2, delta, pct_change) in &comparisons {
        let name = PARAM_NAMES.get(*i).unwrap_or(&"unknown");

        // Skip if below threshold (unless --all)
        if !args.all && delta.abs() < args.threshold {
            continue;
        }

        changes_count += 1;

        let delta_str = if *delta >= 0.0 {
            format!("+{:.3}", delta)
        } else {
            format!("{:.3}", delta)
        };

        let pct_str = if pct_change.is_infinite() {
            if *pct_change > 0.0 {
                "+inf".to_string()
            } else {
                "-inf".to_string()
            }
        } else if *pct_change >= 0.0 {
            format!("+{:.1}%", pct_change)
        } else {
            format!("{:.1}%", pct_change)
        };

        println!(
            "{:<24} {:>10.3} {:>10.3} {:>10} {:>10}",
            name, v1, v2, delta_str, pct_str
        );
    }

    // Print footer
    println!("{}", "-".repeat(76));

    let significant_changes = comparisons
        .iter()
        .filter(|(_, _, _, delta, _)| delta.abs() >= args.threshold)
        .count();

    println!(
        "Total changes: {} of {} parameters differ (>{} threshold)",
        significant_changes,
        PARAM_NAMES.len(),
        args.threshold
    );

    if let Some((idx, delta)) = largest_increase {
        let name = PARAM_NAMES.get(idx).unwrap_or(&"unknown");
        let pct = comparisons
            .iter()
            .find(|(i, _, _, _, _)| *i == idx)
            .map(|(_, _, _, _, p)| *p)
            .unwrap_or(0.0);
        if pct.is_infinite() {
            println!("Largest increase: {} (+{:.3})", name, delta);
        } else {
            println!("Largest increase: {} (+{:.3}, +{:.1}%)", name, delta, pct);
        }
    }

    if let Some((idx, delta)) = largest_decrease {
        let name = PARAM_NAMES.get(idx).unwrap_or(&"unknown");
        let pct = comparisons
            .iter()
            .find(|(i, _, _, _, _)| *i == idx)
            .map(|(_, _, _, _, p)| *p)
            .unwrap_or(0.0);
        if pct.is_infinite() {
            println!("Largest decrease: {} ({:.3})", name, delta);
        } else {
            println!("Largest decrease: {} ({:.3}, {:.1}%)", name, delta, pct);
        }
    }

    if changes_count == 0 && !args.all {
        println!("\nNo changes above threshold. Use --all to see all parameters.");
    }
}
