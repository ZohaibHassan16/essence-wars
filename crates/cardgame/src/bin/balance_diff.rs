//! Balance Diff CLI - Compare two validation result files.
//!
//! Shows changes in balance metrics between validation runs, highlighting
//! significant improvements or regressions.
//!
//! Usage:
//!   cargo run --release --bin balance_diff -- \
//!     --old experiments/validation/run1/results.json \
//!     --new experiments/validation/run2/results.json

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;

use clap::Parser;

use cardgame::validation::{DeckStats, ValidationResults};

/// Balance Diff - Compare validation results
#[derive(Parser, Debug)]
#[command(name = "balance-diff")]
#[command(about = "Compare two validation result files and show balance changes")]
struct Args {
    /// Path to the old (baseline) validation results JSON
    #[arg(long)]
    old: PathBuf,

    /// Path to the new validation results JSON
    #[arg(long)]
    new: PathBuf,

    /// Minimum win rate delta to display (default: 0.05 = 5%)
    #[arg(long, default_value = "0.05")]
    threshold: f64,

    /// Show all decks (including those below threshold)
    #[arg(long)]
    all: bool,
}

/// Direction of change for a deck
#[derive(Debug, Clone, Copy)]
enum ChangeDirection {
    Improved,
    Regressed,
    Unchanged,
}

/// Comparison data for a single deck
struct DeckChange {
    #[allow(dead_code)]
    deck_id: String,
    commander_name: String,
    faction: String,
    old_win_rate: f64,
    new_win_rate: f64,
    delta: f64,
    #[allow(dead_code)]
    old_ci: (f64, f64),
    #[allow(dead_code)]
    new_ci: (f64, f64),
    significant: bool,
    direction: ChangeDirection,
}

fn main() {
    let args = Args::parse();

    // Load old results
    let old_results = match load_results(&args.old) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading old results from {:?}: {}", args.old, e);
            process::exit(1);
        }
    };

    // Load new results
    let new_results = match load_results(&args.new) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading new results from {:?}: {}", args.new, e);
            process::exit(1);
        }
    };

    // Compare and display results
    print_comparison(&old_results, &new_results, args.threshold, args.all);
}

fn load_results(path: &PathBuf) -> Result<ValidationResults, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let results: ValidationResults = serde_json::from_reader(reader)?;
    Ok(results)
}

fn print_comparison(old: &ValidationResults, new: &ValidationResults, threshold: f64, show_all: bool) {
    // Extract timestamps for display
    let old_ts = extract_run_id(&old.timestamp);
    let new_ts = extract_run_id(&new.timestamp);

    println!("Balance Comparison");
    println!("==================");
    println!("Old: {} | New: {}", old_ts, new_ts);
    println!();

    // Global metrics comparison
    println!("=== Global Metrics ===");

    // P1 win rate
    let p1_old = old.summary.p1_win_rate;
    let p1_new = new.summary.p1_win_rate;
    let p1_delta = p1_new - p1_old;
    println!(
        "P1 Win Rate:    {:5.1}% -> {:5.1}% ({:+.1}%)",
        p1_old * 100.0,
        p1_new * 100.0,
        p1_delta * 100.0
    );

    // Max faction delta
    let faction_delta_old = old.summary.max_faction_delta;
    let faction_delta_new = new.summary.max_faction_delta;
    let faction_improvement = faction_delta_new < faction_delta_old;
    println!(
        "Max Faction Δ:  {:5.1}% -> {:5.1}% ({:+.1}%) {}",
        faction_delta_old * 100.0,
        faction_delta_new * 100.0,
        (faction_delta_new - faction_delta_old) * 100.0,
        if faction_improvement { "[IMPROVED]" } else { "" }
    );
    println!();

    // Faction win rates
    println!("=== Faction Win Rates ===");
    let mut factions: Vec<_> = old.summary.faction_win_rates.keys().collect();
    factions.sort();
    for faction in factions {
        let old_rate = old.summary.faction_win_rates.get(faction).copied().unwrap_or(0.5);
        let new_rate = new.summary.faction_win_rates.get(faction).copied().unwrap_or(0.5);
        let delta = new_rate - old_rate;
        println!(
            "  {:12} {:5.1}% -> {:5.1}% ({:+.1}%)",
            capitalize(faction),
            old_rate * 100.0,
            new_rate * 100.0,
            delta * 100.0
        );
    }
    println!();

    // Per-deck changes
    let deck_changes = compute_deck_changes(old, new);

    // Filter and sort by magnitude
    let mut filtered: Vec<_> = deck_changes
        .into_iter()
        .filter(|d| show_all || d.delta.abs() >= threshold)
        .collect();
    filtered.sort_by(|a, b| b.delta.abs().partial_cmp(&a.delta.abs()).unwrap());

    // Count significant changes
    let sig_improved = filtered.iter().filter(|d| d.significant && matches!(d.direction, ChangeDirection::Improved)).count();
    let sig_regressed = filtered.iter().filter(|d| d.significant && matches!(d.direction, ChangeDirection::Regressed)).count();

    if filtered.is_empty() {
        println!("No deck changes above {:.0}% threshold.", threshold * 100.0);
        println!("Use --all to see all decks.");
    } else {
        println!("=== Deck Changes ===");
        println!(
            "{:<28} {:>10} {:>6} {:>6} {:>7}  Status",
            "Commander", "Faction", "Old%", "New%", "Delta"
        );
        println!("{}", "-".repeat(78));

        for change in &filtered {
            let direction_str = match change.direction {
                ChangeDirection::Improved => "▲",
                ChangeDirection::Regressed => "▼",
                ChangeDirection::Unchanged => " ",
            };
            let status = if change.significant {
                match change.direction {
                    ChangeDirection::Improved => "IMPROVED*",
                    ChangeDirection::Regressed => "REGRESSED*",
                    ChangeDirection::Unchanged => "",
                }
            } else {
                match change.direction {
                    ChangeDirection::Improved => "improved",
                    ChangeDirection::Regressed => "regressed",
                    ChangeDirection::Unchanged => "",
                }
            };

            println!(
                "{:<28} {:>10} {:5.1}% {:5.1}% {:+6.1}%  {} {}",
                truncate(&change.commander_name, 28),
                capitalize(&change.faction),
                change.old_win_rate * 100.0,
                change.new_win_rate * 100.0,
                change.delta * 100.0,
                direction_str,
                status
            );
        }
        println!("{}", "-".repeat(78));
        println!("* Statistically significant (95% CIs do not overlap)");
        println!();
        println!(
            "Summary: {} significant improvements, {} significant regressions",
            sig_improved, sig_regressed
        );
    }
}

fn compute_deck_changes(old: &ValidationResults, new: &ValidationResults) -> Vec<DeckChange> {
    // Build lookup by deck_id for old results
    let old_decks: HashMap<&str, &DeckStats> = old
        .summary
        .deck_stats
        .iter()
        .map(|d| (d.deck_id.as_str(), d))
        .collect();

    let mut changes = Vec::new();

    for new_deck in &new.summary.deck_stats {
        if let Some(old_deck) = old_decks.get(new_deck.deck_id.as_str()) {
            let delta = new_deck.win_rate - old_deck.win_rate;
            let old_ci = (old_deck.win_rate_ci_lower, old_deck.win_rate_ci_upper);
            let new_ci = (new_deck.win_rate_ci_lower, new_deck.win_rate_ci_upper);

            // CIs overlap if max(low1, low2) <= min(high1, high2)
            let overlap = old_ci.0.max(new_ci.0) <= old_ci.1.min(new_ci.1);
            let significant = !overlap;

            let direction = if delta > 0.001 {
                ChangeDirection::Improved
            } else if delta < -0.001 {
                ChangeDirection::Regressed
            } else {
                ChangeDirection::Unchanged
            };

            changes.push(DeckChange {
                deck_id: new_deck.deck_id.clone(),
                commander_name: new_deck.commander_name.clone(),
                faction: new_deck.faction.clone(),
                old_win_rate: old_deck.win_rate,
                new_win_rate: new_deck.win_rate,
                delta,
                old_ci,
                new_ci,
                significant,
                direction,
            });
        }
    }

    changes
}

fn extract_run_id(timestamp: &str) -> String {
    // Try to extract YYYY-MM-DD from ISO timestamp
    if timestamp.len() >= 10 {
        timestamp[..10].to_string()
    } else {
        timestamp.to_string()
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
