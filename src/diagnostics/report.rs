//! Diagnostic report output.
//!
//! Formats and prints diagnostic statistics to the console.

use super::analyzer::AggregatedStats;

/// Print the full diagnostic report to stdout.
pub fn print_report(stats: &AggregatedStats) {
    println!("\n{}", "=".repeat(60));
    println!("P1/P2 ASYMMETRY DIAGNOSTIC REPORT");
    println!("{}\n", "=".repeat(60));

    print_overall_statistics(stats);
    print_win_rate_by_length(stats);
    print_first_blood(stats);
    print_tempo_metrics(stats);
    print_board_advantage(stats);
    print_resource_efficiency(stats);
    print_combat_efficiency(stats);
    print_first_creature_death(stats);
    print_actions_per_game(stats);
    print_statistical_summary(stats);
    print_resource_curves(stats);
    print_essence_curves(stats);
    print_board_health_curves(stats);
    print_notable_games(stats);
    print_analysis_hints(stats);
}

fn print_overall_statistics(stats: &AggregatedStats) {
    println!("=== Overall Statistics ===");
    println!("Total games: {}", stats.total_games);

    // Get statistical analysis for P1 win rate
    let p1_stats = stats.p1_win_rate_stats();

    println!(
        "P1 wins: {} ({}) {}",
        stats.p1_wins,
        p1_stats.format_with_ci(),
        p1_stats.significance.symbol()
    );
    println!(
        "P2 wins: {} ({:.1}%)",
        stats.p2_wins,
        stats.p2_win_rate() * 100.0
    );
    println!(
        "Draws: {} ({:.1}%)",
        stats.draws,
        stats.draw_rate() * 100.0
    );

    // Balance assessment
    let assessment = stats.balance_assessment();
    println!(
        "\nBalance Assessment: {} {}",
        assessment.symbol(),
        assessment.description()
    );

    // Show statistical details if significant
    if p1_stats.significance != super::statistics::SignificanceLevel::NotSignificant {
        println!(
            "  Chi-square: {:.2}, p-value: {:.4} ({})",
            p1_stats.chi_square,
            p1_stats.p_value,
            p1_stats.significance.description()
        );
    }
}

fn print_win_rate_by_length(stats: &AggregatedStats) {
    println!("\n=== P1 Win Rate by Game Length ===");
    if stats.games_early > 0 {
        println!(
            "Early (turns 1-10):  {:.1}% ({}/{})",
            100.0 * stats.p1_wins_early as f64 / stats.games_early as f64,
            stats.p1_wins_early,
            stats.games_early
        );
    }
    if stats.games_mid > 0 {
        println!(
            "Mid (turns 11-20):   {:.1}% ({}/{})",
            100.0 * stats.p1_wins_mid as f64 / stats.games_mid as f64,
            stats.p1_wins_mid,
            stats.games_mid
        );
    }
    if stats.games_late > 0 {
        println!(
            "Late (turns 21-30):  {:.1}% ({}/{})",
            100.0 * stats.p1_wins_late as f64 / stats.games_late as f64,
            stats.p1_wins_late,
            stats.games_late
        );
    }
}

fn print_first_blood(stats: &AggregatedStats) {
    println!("\n=== First Blood (First to Deal Damage) ===");
    let first_blood_total = stats.p1_first_blood + stats.p2_first_blood;
    if first_blood_total > 0 {
        let fb_stats = stats.first_blood_stats();
        println!(
            "P1 first blood: {} ({}) {}",
            stats.p1_first_blood,
            fb_stats.format_with_ci(),
            fb_stats.significance.symbol()
        );
        println!(
            "P2 first blood: {} ({:.1}%)",
            stats.p2_first_blood,
            100.0 * stats.p2_first_blood as f64 / first_blood_total as f64
        );

        if fb_stats.significance != super::statistics::SignificanceLevel::NotSignificant {
            println!(
                "  → First blood advantage is {} (p = {:.4})",
                fb_stats.significance.description(),
                fb_stats.p_value
            );
        }
    }
}

fn print_tempo_metrics(stats: &AggregatedStats) {
    println!("\n=== Tempo Metrics ===");

    // First creature play with confidence interval
    let first_creature_total = stats.p1_first_creature + stats.p2_first_creature;
    if first_creature_total > 0 {
        let fc_stats = stats.first_creature_stats();
        println!(
            "First creature played: P1 {} ({}) {}, P2 {} ({:.1}%)",
            stats.p1_first_creature,
            fc_stats.format_with_ci(),
            fc_stats.significance.symbol(),
            stats.p2_first_creature,
            100.0 * stats.p2_first_creature as f64 / first_creature_total as f64
        );

        if fc_stats.significance != super::statistics::SignificanceLevel::NotSignificant {
            println!(
                "  → First creature advantage is {} (p = {:.4})",
                fc_stats.significance.description(),
                fc_stats.p_value
            );
        }
    }

    // Average first creature turn
    let p1_avg = stats.p1_avg_first_creature_turn();
    let p2_avg = stats.p2_avg_first_creature_turn();
    match (p1_avg, p2_avg) {
        (Some(t1), Some(t2)) => {
            println!("Avg first creature turn: P1 {:.2}, P2 {:.2}", t1, t2);
            let diff = t1 - t2;
            if diff.abs() > 0.5 {
                if diff > 0.0 {
                    println!("  → P2 plays first creature {:.2} turns earlier on average", diff);
                } else {
                    println!(
                        "  → P1 plays first creature {:.2} turns earlier on average",
                        -diff
                    );
                }
            }
        }
        (Some(t1), None) => println!("Avg first creature turn: P1 {:.2}, P2 N/A", t1),
        (None, Some(t2)) => println!("Avg first creature turn: P1 N/A, P2 {:.2}", t2),
        (None, None) => {}
    }
}

fn print_board_advantage(stats: &AggregatedStats) {
    println!("\n=== Board Advantage ===");
    println!(
        "Average board advantage score: {:.2} (positive = P1 ahead)",
        stats.avg_board_advantage()
    );

    let total_turns =
        stats.total_turns_p1_ahead + stats.total_turns_p2_ahead + stats.total_turns_even;
    if total_turns > 0 {
        println!(
            "Turns P1 ahead: {} ({:.1}%)",
            stats.total_turns_p1_ahead,
            stats.pct_turns_p1_ahead() * 100.0
        );
        println!(
            "Turns P2 ahead: {} ({:.1}%)",
            stats.total_turns_p2_ahead,
            stats.pct_turns_p2_ahead() * 100.0
        );
        println!(
            "Turns even: {} ({:.1}%)",
            stats.total_turns_even,
            (1.0 - stats.pct_turns_p1_ahead() - stats.pct_turns_p2_ahead()) * 100.0
        );
    }
}

fn print_resource_efficiency(stats: &AggregatedStats) {
    println!("\n=== Resource Efficiency ===");

    // Essence spent
    println!(
        "Avg essence spent/game: P1 {:.1}, P2 {:.1}",
        stats.p1_avg_essence_spent(),
        stats.p2_avg_essence_spent()
    );

    // Efficiency ratio
    let p1_eff = stats.p1_resource_efficiency();
    let p2_eff = stats.p2_resource_efficiency();
    println!(
        "Board impact per essence: P1 {:.2}, P2 {:.2}",
        p1_eff, p2_eff
    );

    if p1_eff > 0.0 && p2_eff > 0.0 {
        let diff = p1_eff - p2_eff;
        if diff.abs() > 0.1 {
            if diff > 0.0 {
                println!("  → P1 is {:.1}% more efficient with essence", (diff / p2_eff) * 100.0);
            } else {
                println!("  → P2 is {:.1}% more efficient with essence", (-diff / p1_eff) * 100.0);
            }
        }
    }
}

fn print_combat_efficiency(stats: &AggregatedStats) {
    println!("\n=== Combat Efficiency ===");

    // Face damage
    println!(
        "Avg face damage/game: P1 {:.1}, P2 {:.1}",
        stats.p1_avg_face_damage(),
        stats.p2_avg_face_damage()
    );

    // Trade ratios
    let p1_tr = stats.p1_trade_ratio();
    let p2_tr = stats.p2_trade_ratio();
    println!(
        "Trade ratio (kills/losses): P1 {:.2}, P2 {:.2}",
        p1_tr, p2_tr
    );

    // Total creatures killed/lost
    if stats.p1_total_creatures_killed > 0 || stats.p2_total_creatures_killed > 0 {
        println!(
            "Total creatures: P1 killed {}, lost {} | P2 killed {}, lost {}",
            stats.p1_total_creatures_killed,
            stats.p1_total_creatures_lost,
            stats.p2_total_creatures_killed,
            stats.p2_total_creatures_lost
        );
    }
}

fn print_first_creature_death(stats: &AggregatedStats) {
    if let Some(avg) = stats.avg_first_creature_death() {
        println!("\nAvg first creature death: turn {:.1}", avg);
    }
}

fn print_actions_per_game(stats: &AggregatedStats) {
    println!("\n=== Actions Per Game ===");
    println!("P1 avg actions: {:.1}", stats.p1_avg_actions());
    println!("P2 avg actions: {:.1}", stats.p2_avg_actions());
}

fn print_resource_curves(stats: &AggregatedStats) {
    println!("\n=== Average Resources by Turn (at start of P1's turn) ===");
    println!(
        "{:>4} | {:>8} {:>8} | {:>6} {:>6} | {:>5} {:>5} | {:>6} {:>6}",
        "Turn", "P1 Life", "P2 Life", "P1 Crt", "P2 Crt", "P1 Hnd", "P2 Hnd", "P1 Atk", "P2 Atk"
    );
    println!("{}", "-".repeat(80));

    let p1_life = AggregatedStats::avg_curve(&stats.p1_life_by_turn);
    let p2_life = AggregatedStats::avg_curve(&stats.p2_life_by_turn);
    let p1_creatures = AggregatedStats::avg_curve(&stats.p1_creatures_by_turn);
    let p2_creatures = AggregatedStats::avg_curve(&stats.p2_creatures_by_turn);
    let p1_hand = AggregatedStats::avg_curve(&stats.p1_hand_by_turn);
    let p2_hand = AggregatedStats::avg_curve(&stats.p2_hand_by_turn);
    let p1_attack = AggregatedStats::avg_curve(&stats.p1_board_attack_by_turn);
    let p2_attack = AggregatedStats::avg_curve(&stats.p2_board_attack_by_turn);

    for i in 0..p1_life.len().min(15) {
        let turn = p1_life.get(i).map(|(t, _)| *t).unwrap_or(0);
        println!(
            "{:>4} | {:>8.1} {:>8.1} | {:>6.1} {:>6.1} | {:>5.1} {:>5.1} | {:>6.1} {:>6.1}",
            turn,
            p1_life.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_life.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p1_creatures.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_creatures.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p1_hand.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_hand.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p1_attack.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_attack.get(i).map(|(_, v)| *v).unwrap_or(0.0),
        );
    }
}

fn print_essence_curves(stats: &AggregatedStats) {
    println!("\n=== Average Essence by Turn (at start of P1's turn) ===");
    println!(
        "{:>4} | {:>8} {:>8} | {:>8} {:>8}",
        "Turn", "P1 Ess", "P2 Ess", "P1 Max", "P2 Max"
    );
    println!("{}", "-".repeat(50));

    let p1_essence = AggregatedStats::avg_curve(&stats.p1_essence_by_turn);
    let p2_essence = AggregatedStats::avg_curve(&stats.p2_essence_by_turn);
    let p1_max_essence = AggregatedStats::avg_curve(&stats.p1_max_essence_by_turn);
    let p2_max_essence = AggregatedStats::avg_curve(&stats.p2_max_essence_by_turn);

    for i in 0..p1_essence.len().min(15) {
        let turn = p1_essence.get(i).map(|(t, _)| *t).unwrap_or(0);
        println!(
            "{:>4} | {:>8.1} {:>8.1} | {:>8.1} {:>8.1}",
            turn,
            p1_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p1_max_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_max_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
        );
    }
}

fn print_board_health_curves(stats: &AggregatedStats) {
    println!("\n=== Average Board Health by Turn (at start of P1's turn) ===");
    println!("{:>4} | {:>10} {:>10}", "Turn", "P1 Health", "P2 Health");
    println!("{}", "-".repeat(35));

    let p1_board_health = AggregatedStats::avg_curve(&stats.p1_board_health_by_turn);
    let p2_board_health = AggregatedStats::avg_curve(&stats.p2_board_health_by_turn);

    for i in 0..p1_board_health.len().min(15) {
        let turn = p1_board_health.get(i).map(|(t, _)| *t).unwrap_or(0);
        println!(
            "{:>4} | {:>10.1} {:>10.1}",
            turn,
            p1_board_health.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            p2_board_health.get(i).map(|(_, v)| *v).unwrap_or(0.0),
        );
    }
}

fn print_notable_games(stats: &AggregatedStats) {
    println!("\n=== Notable Games (for debugging) ===");
    if let Some((seed, turns)) = stats.earliest_p1_win_seed {
        println!("Fastest P1 win: {} turns (seed {})", turns, seed);
    }
    if let Some((seed, turns)) = stats.earliest_p2_win_seed {
        println!("Fastest P2 win: {} turns (seed {})", turns, seed);
    }
}

fn print_statistical_summary(stats: &AggregatedStats) {
    println!("\n=== Statistical Summary ===");

    // Game length percentiles
    if let Some((p10, p50, p90)) = stats.game_length_percentiles() {
        println!(
            "Game length: P10={:.0}, P50={:.0}, P90={:.0} turns",
            p10, p50, p90
        );
    }

    // Board advantage percentiles
    if let Some((p10, p50, p90)) = stats.board_advantage_percentiles() {
        println!(
            "Board advantage: P10={:.1}, P50={:.1}, P90={:.1} (positive = P1 ahead)",
            p10, p50, p90
        );
    }
}

fn print_analysis_hints(stats: &AggregatedStats) {
    println!("\n=== Analysis Hints ===");
    let p1_stats = stats.p1_win_rate_stats();
    let p1_wr = p1_stats.proportion;

    // Use statistical significance for hints
    if stats.has_significant_imbalance() {
        if p1_wr < 0.5 {
            println!(
                "⚠ P1 win rate ({}) is significantly below 50%",
                p1_stats.format_with_ci()
            );
        } else {
            println!(
                "⚠ P1 win rate ({}) is significantly above 50%",
                p1_stats.format_with_ci()
            );
        }

        // Analyze contributing factors
        let fb_stats = stats.first_blood_stats();
        if fb_stats.significance != super::statistics::SignificanceLevel::NotSignificant {
            if fb_stats.proportion > 0.5 && p1_wr > 0.5 {
                println!("  → P1's first blood advantage correlates with higher win rate");
            } else if fb_stats.proportion < 0.5 && p1_wr < 0.5 {
                println!("  → P2's first blood advantage correlates with higher win rate");
            }
        }

        let fc_stats = stats.first_creature_stats();
        if fc_stats.significance != super::statistics::SignificanceLevel::NotSignificant {
            if fc_stats.proportion > 0.5 && p1_wr > 0.5 {
                println!("  → P1's tempo advantage (first creature) correlates with wins");
            } else if fc_stats.proportion < 0.5 && p1_wr < 0.5 {
                println!("  → P2's tempo advantage (first creature) correlates with wins");
            }
        }

        if stats.games_early > 0
            && (stats.p1_wins_early as f64 / stats.games_early as f64) < 0.4
        {
            println!("  → P1 struggles especially in early game");
        }
    } else {
        println!("✓ No significant P1/P2 imbalance detected");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_report_empty_stats() {
        // Just ensure it doesn't panic with empty stats
        let stats = AggregatedStats::new();
        // Can't easily test stdout, but ensure no panics
        print_report(&stats);
    }
}
