//! Post-tuning validation for quick weight verification.
//!
//! Runs a quick validation after tuning completes to verify the
//! tuned weights perform reasonably.

use std::time::Instant;

use crate::bots::{BotType, GreedyWeights};
use crate::cards::CardDatabase;
use crate::decks::DeckRegistry;
use crate::execution::MatchupBuilder;
use crate::validation::{ArchetypeWeights, BalanceAnalyzer, ValidationExecutor};

/// Configuration for post-tuning validation.
pub struct PostTuningValidationConfig {
    /// Games per matchup (default: 20 for speed)
    pub games_per_matchup: usize,
    /// Alpha-Beta search depth (default: 6)
    pub alphabeta_depth: u32,
    /// Random seed
    pub seed: u64,
}

impl Default for PostTuningValidationConfig {
    fn default() -> Self {
        Self {
            games_per_matchup: 20,
            alphabeta_depth: 6,
            seed: 42,
        }
    }
}

/// Results from post-tuning validation.
pub struct PostTuningValidationResult {
    /// Average win rate across all matchups.
    pub avg_win_rate: f64,
    /// Total games played.
    pub total_games: usize,
    /// Time taken.
    pub duration_secs: f64,
}

/// Run a quick post-tuning validation using Alpha-Beta bot.
///
/// This runs validation across inter-faction matchups to verify
/// the tuned weights perform reasonably.
pub fn run_post_tuning_validation(
    card_db: &CardDatabase,
    deck_registry: &DeckRegistry,
    new_weights: &GreedyWeights,
    previous_weights: Option<&GreedyWeights>,
    config: &PostTuningValidationConfig,
) -> Result<PostTuningValidationResult, String> {
    println!();
    println!("Running post-tuning validation ({} games/matchup, Alpha-Beta depth {})...",
        config.games_per_matchup, config.alphabeta_depth);

    let start_time = Instant::now();

    // Build inter-faction matchups
    let builder = MatchupBuilder::new(deck_registry);
    let matchups = builder.build_inter_faction_matchups();

    if matchups.is_empty() {
        return Err("No valid matchups found for validation".to_string());
    }

    // Create archetype weights using the new weights for all archetypes
    let archetype_weights = ArchetypeWeights::from_single(new_weights.clone());

    // Create executor with Alpha-Beta bot
    let executor = ValidationExecutor::new(card_db, 50) // mcts_sims not used for AB
        .with_bot_type(BotType::AlphaBeta)
        .with_alphabeta_depth(config.alphabeta_depth)
        .with_progress(true); // Show progress updates

    // Run validation
    let matchup_results = executor
        .run_all(&matchups, &archetype_weights, config.games_per_matchup, config.seed)
        .map_err(|e| format!("Validation failed: {}", e))?;

    let duration = start_time.elapsed();

    // Analyze results
    let analyzer = BalanceAnalyzer::new();
    let summary = analyzer.analyze(&matchup_results);

    // Print summary
    println!();
    println!("=== Validation Results ===");
    println!("  P1 win rate: {:.1}%", summary.p1_win_rate * 100.0);
    println!("  Status: {}", summary.overall_status);
    println!("  Time: {:.1}s", duration.as_secs_f64());

    // Print weight diff table if we have previous weights
    if let Some(prev) = previous_weights {
        print_weight_diff(prev, new_weights);
    }

    let total_games = matchups.len() * 2 * config.games_per_matchup;

    Ok(PostTuningValidationResult {
        avg_win_rate: summary.p1_win_rate,
        total_games,
        duration_secs: duration.as_secs_f64(),
    })
}

/// Print a table showing weight changes between previous and new weights.
fn print_weight_diff(prev: &GreedyWeights, new: &GreedyWeights) {
    let prev_vec = prev.to_vec();
    let new_vec = new.to_vec();

    // Weight parameter names (matching GreedyWeights order)
    let names = [
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

    // Find significant changes (> 0.01 absolute difference)
    let mut changes: Vec<(usize, f32, f32, f32)> = Vec::new();
    for (i, (p, n)) in prev_vec.iter().zip(new_vec.iter()).enumerate() {
        let delta = n - p;
        if delta.abs() > 0.01 {
            changes.push((i, *p, *n, delta));
        }
    }

    if changes.is_empty() {
        println!("\n  No significant weight changes detected.");
        return;
    }

    // Sort by absolute change (largest first)
    changes.sort_by(|a, b| b.3.abs().partial_cmp(&a.3.abs()).unwrap());

    println!();
    println!("  Weight Changes vs Previous:");
    println!("  {:<22} {:>10} {:>10} {:>10}", "Parameter", "Previous", "New", "Δ");
    println!("  {}", "-".repeat(54));

    for (i, prev_val, new_val, delta) in changes.iter().take(10) {
        let name = names.get(*i).unwrap_or(&"unknown");
        let delta_sign = if *delta > 0.0 { "+" } else { "" };
        println!(
            "  {:<22} {:>10.3} {:>10.3} {:>9}{:.3}",
            name, prev_val, new_val, delta_sign, delta
        );
    }

    if changes.len() > 10 {
        println!("  ... and {} more changes", changes.len() - 10);
    }
}
