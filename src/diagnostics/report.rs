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
    print_first_creature_death(stats);
    print_actions_per_game(stats);
    print_resource_curves(stats);
    print_essence_curves(stats);
    print_board_health_curves(stats);
    print_notable_games(stats);
    print_analysis_hints(stats);
}

fn print_overall_statistics(stats: &AggregatedStats) {
    println!("=== Overall Statistics ===");
    println!("Total games: {}", stats.total_games);
    println!(
        "P1 wins: {} ({:.1}%)",
        stats.p1_wins,
        stats.p1_win_rate() * 100.0
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
        println!(
            "P1 first blood: {} ({:.1}%)",
            stats.p1_first_blood,
            100.0 * stats.p1_first_blood as f64 / first_blood_total as f64
        );
        println!(
            "P2 first blood: {} ({:.1}%)",
            stats.p2_first_blood,
            100.0 * stats.p2_first_blood as f64 / first_blood_total as f64
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

fn print_analysis_hints(stats: &AggregatedStats) {
    println!("\n=== Analysis Hints ===");
    let p1_wr = stats.p1_win_rate();
    if p1_wr < 0.45 {
        println!(
            "⚠ P1 win rate ({:.1}%) is below expected 50%",
            p1_wr * 100.0
        );

        if stats.p2_first_blood > stats.p1_first_blood {
            println!("  → P2 gets first blood more often - possible tempo advantage");
        }

        if stats.games_early > 0
            && (stats.p1_wins_early as f64 / stats.games_early as f64) < 0.4
        {
            println!("  → P1 struggles especially in early game");
        }

        if stats.p2_actions_total > stats.p1_actions_total {
            println!("  → P2 takes more actions on average");
        }
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
