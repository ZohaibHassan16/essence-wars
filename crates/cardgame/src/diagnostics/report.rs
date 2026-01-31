//! Diagnostic report output.
//!
//! Formats and prints diagnostic statistics to the console.

use super::analyzer::AggregatedStats;

/// Print the full diagnostic report to stdout.
pub fn print_report(stats: &AggregatedStats) {
    println!("\n{}", "=".repeat(60));
    println!("P1/P2 ASYMMETRY DIAGNOSTIC REPORT");
    println!("{}\n", "=".repeat(60));
    print_report_body(stats);
}

/// Print a comparative report for deck vs deck analysis.
pub fn print_comparative_report(stats: &AggregatedStats, deck1_name: &str, deck2_name: &str) {
    println!("\n{}", "=".repeat(60));
    println!("DECK COMPARISON: {} vs {}", deck1_name, deck2_name);
    println!("{}\n", "=".repeat(60));

    // Matchup summary
    print_comparative_summary(stats, deck1_name, deck2_name);

    // Tempo comparison
    print_comparative_tempo(stats, deck1_name, deck2_name);

    // Board metrics
    print_comparative_board(stats, deck1_name, deck2_name);

    // Combat efficiency
    print_comparative_combat(stats, deck1_name, deck2_name);

    // Action breakdown
    print_comparative_actions(stats, deck1_name, deck2_name);

    // Resource curves
    print_resource_curves(stats);
}

fn print_comparative_summary(stats: &AggregatedStats, deck1_name: &str, deck2_name: &str) {
    println!("=== Matchup Summary ({} games) ===", stats.total_games);

    let p1_stats = stats.p1_win_rate_stats();

    println!(
        "{} (P1): {} wins ({}) {}",
        deck1_name,
        stats.p1_wins,
        p1_stats.format_with_ci(),
        p1_stats.significance.symbol()
    );
    println!(
        "{} (P2): {} wins ({:.1}%)",
        deck2_name,
        stats.p2_wins,
        stats.p2_win_rate() * 100.0
    );
    if stats.draws > 0 {
        println!(
            "Draws: {} ({:.1}%)",
            stats.draws,
            stats.draw_rate() * 100.0
        );
    }
}

fn print_comparative_tempo(stats: &AggregatedStats, deck1_name: &str, deck2_name: &str) {
    println!("\n=== Tempo Comparison ===");

    // Abbreviate names for table
    let d1 = abbreviate_name(deck1_name, 12);
    let d2 = abbreviate_name(deck2_name, 12);

    println!(
        "{:>18}  {:>12}  {:>12}  {:>10}",
        "", d1, d2, "Delta"
    );

    // First creature turn
    let p1_avg = stats.p1_avg_first_creature_turn();
    let p2_avg = stats.p2_avg_first_creature_turn();
    if let (Some(t1), Some(t2)) = (p1_avg, p2_avg) {
        let delta = t1 - t2;
        let delta_str = format_delta(delta);
        println!(
            "{:>18}: {:>10.1}   {:>10.1}   {:>10}",
            "First creature", t1, t2, delta_str
        );
    }

    // First blood
    let total_fb = stats.p1_first_blood + stats.p2_first_blood;
    if total_fb > 0 {
        let p1_fb_rate = stats.p1_first_blood as f64 / total_fb as f64;
        let p2_fb_rate = stats.p2_first_blood as f64 / total_fb as f64;
        let delta = (p1_fb_rate - p2_fb_rate) * 100.0;
        let delta_str = format_delta_pct(delta);
        println!(
            "{:>18}: {:>10.1}%  {:>10.1}%  {:>10}",
            "First blood rate", p1_fb_rate * 100.0, p2_fb_rate * 100.0, delta_str
        );
    }
}

fn print_comparative_board(stats: &AggregatedStats, deck1_name: &str, deck2_name: &str) {
    println!("\n=== Board Metrics ===");

    let d1 = abbreviate_name(deck1_name, 12);
    let d2 = abbreviate_name(deck2_name, 12);

    println!(
        "{:>18}  {:>12}  {:>12}  {:>10}",
        "", d1, d2, "Delta"
    );

    // Average creatures (approximate from kills/losses)
    // Use board advantage percentage
    let p1_ahead_pct = stats.pct_turns_p1_ahead() * 100.0;
    let p2_ahead_pct = stats.pct_turns_p2_ahead() * 100.0;
    let delta = p1_ahead_pct - p2_ahead_pct;
    let delta_str = format_delta_pct(delta);
    println!(
        "{:>18}: {:>10.1}%  {:>10.1}%  {:>10}",
        "Turns ahead", p1_ahead_pct, p2_ahead_pct, delta_str
    );

    // Resource efficiency
    let p1_eff = stats.p1_resource_efficiency();
    let p2_eff = stats.p2_resource_efficiency();
    let delta = p1_eff - p2_eff;
    let delta_str = format_delta(delta);
    println!(
        "{:>18}: {:>10.2}   {:>10.2}   {:>10}",
        "Board impact/ess", p1_eff, p2_eff, delta_str
    );
}

fn print_comparative_combat(stats: &AggregatedStats, deck1_name: &str, deck2_name: &str) {
    println!("\n=== Combat Efficiency ===");

    let d1 = abbreviate_name(deck1_name, 12);
    let d2 = abbreviate_name(deck2_name, 12);

    println!(
        "{:>18}  {:>12}  {:>12}  {:>10}",
        "", d1, d2, "Delta"
    );

    // Face damage
    let p1_fd = stats.p1_avg_face_damage();
    let p2_fd = stats.p2_avg_face_damage();
    let delta = p1_fd - p2_fd;
    let delta_str = format_delta(delta);
    println!(
        "{:>18}: {:>10.1}   {:>10.1}   {:>10}",
        "Face dmg/game", p1_fd, p2_fd, delta_str
    );

    // Trade ratio
    let p1_tr = stats.p1_trade_ratio().unwrap_or(0.0);
    let p2_tr = stats.p2_trade_ratio().unwrap_or(0.0);
    let delta = p1_tr - p2_tr;
    let delta_str = format_delta(delta);
    println!(
        "{:>18}: {:>10.2}   {:>10.2}   {:>10}",
        "Trade ratio", p1_tr, p2_tr, delta_str
    );

    // Creatures killed
    let p1_kills = if stats.total_games > 0 {
        stats.p1_total_creatures_killed as f64 / stats.total_games as f64
    } else {
        0.0
    };
    let p2_kills = if stats.total_games > 0 {
        stats.p2_total_creatures_killed as f64 / stats.total_games as f64
    } else {
        0.0
    };
    let delta = p1_kills - p2_kills;
    let delta_str = format_delta(delta);
    println!(
        "{:>18}: {:>10.1}   {:>10.1}   {:>10}",
        "Kills/game", p1_kills, p2_kills, delta_str
    );
}

fn print_comparative_actions(stats: &AggregatedStats, deck1_name: &str, deck2_name: &str) {
    let total = stats.total_actions_all();
    if total == 0 || stats.total_games == 0 {
        return;
    }

    println!("\n=== Action Breakdown (avg/game) ===");

    let d1 = abbreviate_name(deck1_name, 12);
    let d2 = abbreviate_name(deck2_name, 12);

    println!("{:>12}  {:>12}  {:>12}", "", d1, d2);

    let games = stats.total_games as f64;

    // PlayCard
    let p1_pc = stats.p1_total_play_card as f64 / games;
    let p2_pc = stats.p2_total_play_card as f64 / games;
    println!("{:>12}: {:>10.1}   {:>10.1}", "PlayCard", p1_pc, p2_pc);

    // Attack
    let p1_atk = stats.p1_total_attack as f64 / games;
    let p2_atk = stats.p2_total_attack as f64 / games;
    println!("{:>12}: {:>10.1}   {:>10.1}", "Attack", p1_atk, p2_atk);

    // Abilities
    let p1_ab = stats.p1_total_ability as f64 / games;
    let p2_ab = stats.p2_total_ability as f64 / games;
    println!("{:>12}: {:>10.1}   {:>10.1}", "Abilities", p1_ab, p2_ab);

    // EndTurn
    let p1_et = stats.p1_total_end_turn as f64 / games;
    let p2_et = stats.p2_total_end_turn as f64 / games;
    println!("{:>12}: {:>10.1}   {:>10.1}", "EndTurn", p1_et, p2_et);
}

/// Abbreviate a deck/commander name for table display.
fn abbreviate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}

/// Format a delta value with +/- sign.
fn format_delta(delta: f64) -> String {
    if delta >= 0.0 {
        format!("+{:.2}", delta)
    } else {
        format!("{:.2}", delta)
    }
}

/// Format a delta percentage with +/- sign.
fn format_delta_pct(delta: f64) -> String {
    if delta >= 0.0 {
        format!("+{:.1}%", delta)
    } else {
        format!("{:.1}%", delta)
    }
}

fn print_report_body(stats: &AggregatedStats) {

    print_overall_statistics(stats);
    print_win_rate_by_length(stats);
    print_first_blood(stats);
    print_tempo_metrics(stats);
    print_board_advantage(stats);
    print_resource_efficiency(stats);
    print_combat_efficiency(stats);
    print_first_creature_death(stats);
    print_actions_per_game(stats);
    print_action_breakdown(stats);
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
    fn format_trade_ratio(ratio: Option<f64>) -> String {
        match ratio {
            None => "N/A".to_string(),
            Some(r) if r.is_infinite() => "∞".to_string(),
            Some(r) => format!("{:.2}", r),
        }
    }
    println!(
        "Trade ratio (kills/losses): P1 {}, P2 {}",
        format_trade_ratio(stats.p1_trade_ratio()),
        format_trade_ratio(stats.p2_trade_ratio())
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

fn print_action_breakdown(stats: &AggregatedStats) {
    let total = stats.total_actions_all();
    if total == 0 {
        return;
    }

    let p1_total = stats.p1_total_actions_by_type();
    let p2_total = stats.p2_total_actions_by_type();

    println!("\n=== Action Breakdown ({} games) ===", stats.total_games);
    println!(
        "{:>12}  {:>10} {:>10} {:>12}",
        "", "P1", "P2", "Total"
    );

    // PlayCard
    let play_total = stats.p1_total_play_card + stats.p2_total_play_card;
    let play_pct = if total > 0 { 100.0 * play_total as f64 / total as f64 } else { 0.0 };
    println!(
        "{:>12}: {:>10} {:>10} {:>10} ({:>4.1}%)",
        "PlayCard",
        format_with_commas(stats.p1_total_play_card),
        format_with_commas(stats.p2_total_play_card),
        format_with_commas(play_total),
        play_pct
    );

    // Attack
    let attack_total = stats.p1_total_attack + stats.p2_total_attack;
    let attack_pct = if total > 0 { 100.0 * attack_total as f64 / total as f64 } else { 0.0 };
    println!(
        "{:>12}: {:>10} {:>10} {:>10} ({:>4.1}%)",
        "Attack",
        format_with_commas(stats.p1_total_attack),
        format_with_commas(stats.p2_total_attack),
        format_with_commas(attack_total),
        attack_pct
    );

    // Abilities
    let ability_total = stats.p1_total_ability + stats.p2_total_ability;
    let ability_pct = if total > 0 { 100.0 * ability_total as f64 / total as f64 } else { 0.0 };
    println!(
        "{:>12}: {:>10} {:>10} {:>10} ({:>4.1}%)",
        "Abilities",
        format_with_commas(stats.p1_total_ability),
        format_with_commas(stats.p2_total_ability),
        format_with_commas(ability_total),
        ability_pct
    );

    // EndTurn
    let end_total = stats.p1_total_end_turn + stats.p2_total_end_turn;
    let end_pct = if total > 0 { 100.0 * end_total as f64 / total as f64 } else { 0.0 };
    println!(
        "{:>12}: {:>10} {:>10} {:>10} ({:>4.1}%)",
        "EndTurn",
        format_with_commas(stats.p1_total_end_turn),
        format_with_commas(stats.p2_total_end_turn),
        format_with_commas(end_total),
        end_pct
    );

    // Separator and totals
    println!("{}", "-".repeat(55));
    println!(
        "{:>12}: {:>10} {:>10} {:>10}",
        "Total",
        format_with_commas(p1_total),
        format_with_commas(p2_total),
        format_with_commas(total)
    );

    // Average per game
    let avg_per_game = stats.avg_actions_per_game();
    let p1_avg = if stats.total_games > 0 { p1_total as f64 / stats.total_games as f64 } else { 0.0 };
    let p2_avg = if stats.total_games > 0 { p2_total as f64 / stats.total_games as f64 } else { 0.0 };
    println!(
        "{:>12}: {:>10.1} {:>10.1} {:>10.1}",
        "Avg/game",
        p1_avg,
        p2_avg,
        avg_per_game
    );
}

/// Format a number with comma separators for thousands.
fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
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
