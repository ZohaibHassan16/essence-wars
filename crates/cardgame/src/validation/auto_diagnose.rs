//! Auto-diagnosis of outlier decks after validation.
//!
//! When a deck shows significant imbalance (win rate outside 40-60%),
//! this module runs detailed diagnostics to understand why.

use std::fs;
use std::io;
use std::path::Path;

use crate::bots::{AlphaBetaConfig, BotType, MctsConfig};
use crate::cards::CardDatabase;
use crate::decks::{DeckDefinition, DeckRegistry};
use crate::diagnostics::{
    analyze_critical_turns, export_json, AggregatedStats, CriticalTurnConfig, DiagnosticConfig,
    DiagnosticRunner,
};

use super::BalanceSummary;

/// Information about an outlier deck that needs diagnosis.
#[derive(Debug, Clone)]
pub struct OutlierDeck {
    /// Deck ID.
    pub deck_id: String,
    /// Full deck definition.
    pub deck: DeckDefinition,
    /// Overall win rate from validation.
    pub win_rate: f64,
    /// Deck ID of worst matchup opponent.
    pub worst_matchup_id: Option<String>,
    /// Win rate against worst matchup.
    pub worst_matchup_rate: Option<f64>,
}

/// Configuration for auto-diagnosis runs.
#[derive(Debug, Clone)]
pub struct AutoDiagnoseConfig {
    /// Number of games per diagnostic run.
    pub games: usize,
    /// Bot type to use.
    pub bot_type: BotType,
    /// Alpha-beta search depth.
    pub alphabeta_depth: u32,
    /// MCTS simulations per move.
    pub mcts_sims: u32,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl Default for AutoDiagnoseConfig {
    fn default() -> Self {
        Self {
            games: 50,
            bot_type: BotType::AlphaBeta,
            alphabeta_depth: 6,
            mcts_sims: 100,
            seed: 42,
        }
    }
}

/// Find outlier decks from validation results.
///
/// An outlier is a deck with win rate outside [0.5 - threshold, 0.5 + threshold].
/// Default threshold of 0.10 means decks with win rate <40% or >60% are outliers.
pub fn find_outliers(
    summary: &BalanceSummary,
    deck_registry: &DeckRegistry,
    threshold: f64,
) -> Vec<OutlierDeck> {
    summary
        .deck_stats
        .iter()
        .filter(|d| (d.win_rate - 0.5).abs() > threshold)
        .filter_map(|d| {
            let deck = deck_registry.get(&d.deck_id)?;
            Some(OutlierDeck {
                deck_id: d.deck_id.clone(),
                deck: deck.clone(),
                win_rate: d.win_rate,
                worst_matchup_id: d.worst_matchup.as_ref().map(|(id, _)| id.clone()),
                worst_matchup_rate: d.worst_matchup.as_ref().map(|(_, rate)| *rate),
            })
        })
        .collect()
}

/// Run diagnostics on outlier decks.
///
/// For each outlier:
/// 1. Mirror match: Deck vs itself (pure P1/P2 asymmetry analysis)
/// 2. Worst matchup: Deck vs its worst opponent (understand why it's weak/strong)
///
/// Results are saved to `{output_dir}/diagnostics/{deck_id}/`.
pub fn run_outlier_diagnostics(
    outliers: &[OutlierDeck],
    deck_registry: &DeckRegistry,
    card_db: &CardDatabase,
    config: &AutoDiagnoseConfig,
    output_dir: &Path,
    show_progress: bool,
) -> io::Result<()> {
    if outliers.is_empty() {
        return Ok(());
    }

    let diagnostics_dir = output_dir.join("diagnostics");
    fs::create_dir_all(&diagnostics_dir)?;

    let runner = DiagnosticRunner::new(card_db);
    let mcts_config = MctsConfig {
        simulations: config.mcts_sims,
        exploration: 1.414,
        max_rollout_depth: 100,
        parallel_trees: 1,
        leaf_rollouts: 1,
    };
    let alphabeta_config = AlphaBetaConfig::with_depth(config.alphabeta_depth);
    let critical_turn_config = CriticalTurnConfig::default();

    for (idx, outlier) in outliers.iter().enumerate() {
        let deck_dir = diagnostics_dir.join(&outlier.deck_id);
        fs::create_dir_all(&deck_dir)?;

        let win_pct = outlier.win_rate * 100.0;
        let status = if outlier.win_rate < 0.5 { "weak" } else { "strong" };

        println!(
            "\n[{}/{}] Diagnosing: {} (win rate: {:.1}%, {})",
            idx + 1,
            outliers.len(),
            outlier.deck_id,
            win_pct,
            status
        );

        // 1. Mirror match diagnosis
        print!("  Running mirror match ({} games)...", config.games);
        if show_progress {
            println!();
        }

        let mirror_config = DiagnosticConfig::new(outlier.deck.clone(), config.games)
            .with_decks(outlier.deck.clone(), outlier.deck.clone())
            .with_bots(config.bot_type.clone(), config.bot_type.clone())
            .with_mcts_config(mcts_config.clone())
            .with_alphabeta_config(alphabeta_config.clone())
            .with_seed(config.seed)
            .with_progress(show_progress);

        match runner.run(&mirror_config) {
            Ok(games) => {
                let stats = AggregatedStats::analyze(&games);
                let ct_stats = analyze_critical_turns(&games, &critical_turn_config);

                // Export stats
                let stats_path = deck_dir.join("mirror_stats.json");
                export_json(&stats_path, &stats, &games, false)?;

                // Write summary
                let summary_path = deck_dir.join("mirror_summary.txt");
                write_diagnostic_summary(
                    &summary_path,
                    &outlier.deck_id,
                    None,
                    &stats,
                    &ct_stats,
                    config.games,
                )?;

                if !show_progress {
                    println!(" done");
                }
            }
            Err(e) => {
                println!(" failed: {}", e);
            }
        }

        // 2. Worst matchup diagnosis (if available)
        if let Some(ref opponent_id) = outlier.worst_matchup_id {
            if let Some(opponent_deck) = deck_registry.get(opponent_id) {
                let matchup_rate = outlier.worst_matchup_rate.unwrap_or(0.0) * 100.0;
                print!(
                    "  Running vs {} ({} games, {:.1}% win rate)...",
                    opponent_id, config.games, matchup_rate
                );
                if show_progress {
                    println!();
                }

                let matchup_config = DiagnosticConfig::new(outlier.deck.clone(), config.games)
                    .with_decks(outlier.deck.clone(), opponent_deck.clone())
                    .with_bots(config.bot_type.clone(), config.bot_type.clone())
                    .with_mcts_config(mcts_config.clone())
                    .with_alphabeta_config(alphabeta_config.clone())
                    .with_seed(config.seed + 1000) // Different seed for matchup
                    .with_progress(show_progress);

                match runner.run(&matchup_config) {
                    Ok(games) => {
                        let stats = AggregatedStats::analyze(&games);
                        let ct_stats = analyze_critical_turns(&games, &critical_turn_config);

                        // Export stats
                        let filename = format!("vs_{}_stats.json", opponent_id);
                        let stats_path = deck_dir.join(&filename);
                        export_json(&stats_path, &stats, &games, false)?;

                        // Write summary
                        let summary_filename = format!("vs_{}_summary.txt", opponent_id);
                        let summary_path = deck_dir.join(&summary_filename);
                        write_diagnostic_summary(
                            &summary_path,
                            &outlier.deck_id,
                            Some(opponent_id),
                            &stats,
                            &ct_stats,
                            config.games,
                        )?;

                        if !show_progress {
                            println!(" done");
                        }
                    }
                    Err(e) => {
                        println!(" failed: {}", e);
                    }
                }
            }
        }

        println!("  Saved to {}", deck_dir.display());
    }

    println!("\nAuto-diagnosis complete. See diagnostics/ for detailed reports.");
    Ok(())
}

/// Write a human-readable summary of diagnostic results.
fn write_diagnostic_summary(
    path: &Path,
    deck_id: &str,
    opponent_id: Option<&str>,
    stats: &AggregatedStats,
    ct_stats: &crate::diagnostics::CriticalTurnStats,
    games: usize,
) -> io::Result<()> {
    use std::io::Write;

    let mut file = fs::File::create(path)?;

    // Header
    writeln!(file, "=== Diagnostic Summary ===")?;
    if let Some(opp) = opponent_id {
        writeln!(file, "Matchup: {} vs {}", deck_id, opp)?;
    } else {
        writeln!(file, "Mirror Match: {}", deck_id)?;
    }
    writeln!(file, "Games: {}", games)?;
    writeln!(file)?;

    // Win rates
    writeln!(file, "=== Win Rates ===")?;
    writeln!(file, "P1 wins: {}", stats.p1_wins)?;
    writeln!(file, "P2 wins: {}", stats.p2_wins)?;
    writeln!(file, "Draws: {}", stats.draws)?;
    let p1_rate = if stats.total_games > 0 {
        stats.p1_wins as f64 / stats.total_games as f64 * 100.0
    } else {
        0.0
    };
    writeln!(file, "P1 win rate: {:.1}%", p1_rate)?;
    writeln!(file, "Balance: {:?}", stats.balance_assessment())?;
    writeln!(file)?;

    // Game length
    writeln!(file, "=== Game Length ===")?;
    if let Some((p10, p50, p90)) = stats.game_length_percentiles() {
        writeln!(file, "Game length (P10/P50/P90): {:.0}/{:.0}/{:.0}", p10, p50, p90)?;
    }
    writeln!(file)?;

    // First blood
    writeln!(file, "=== First Blood ===")?;
    writeln!(file, "P1 first blood: {}", stats.p1_first_blood)?;
    writeln!(file, "P2 first blood: {}", stats.p2_first_blood)?;
    if let Some(corr) = stats.first_blood_win_correlation() {
        writeln!(file, "First blood → win correlation: {:.2}", corr)?;
    }
    writeln!(file)?;

    // Board advantage
    writeln!(file, "=== Board Advantage ===")?;
    writeln!(file, "Avg board advantage: {:.2}", stats.avg_board_advantage())?;
    if let Some(corr) = stats.board_advantage_win_correlation() {
        writeln!(file, "Board advantage → win correlation: {:.2}", corr)?;
    }
    writeln!(file)?;

    // Critical turns
    writeln!(file, "=== Critical Turns ===")?;
    writeln!(file, "Total critical turns: {}", ct_stats.total_critical_turns)?;
    writeln!(
        file,
        "Avg critical turns per game: {:.1}",
        ct_stats.avg_per_game
    )?;
    writeln!(
        file,
        "Advantaged player win rate: {:.1}%",
        ct_stats.advantaged_player_win_rate * 100.0
    )?;
    writeln!(file)?;

    // Resource efficiency
    writeln!(file, "=== Resource Efficiency ===")?;
    writeln!(
        file,
        "P1 avg essence spent: {:.1}",
        stats.p1_avg_essence_spent()
    )?;
    writeln!(
        file,
        "P2 avg essence spent: {:.1}",
        stats.p2_avg_essence_spent()
    )?;
    writeln!(file)?;

    // Combat efficiency
    writeln!(file, "=== Combat Efficiency ===")?;
    writeln!(
        file,
        "P1 avg face damage: {:.1}",
        stats.p1_avg_face_damage()
    )?;
    writeln!(
        file,
        "P2 avg face damage: {:.1}",
        stats.p2_avg_face_damage()
    )?;

    Ok(())
}
