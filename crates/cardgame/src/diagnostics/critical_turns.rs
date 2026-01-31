//! Critical Turn Identification
//!
//! Detects game-deciding turns based on life swings, board advantage changes,
//! and leadership changes.

use crate::types::PlayerId;

use super::GameDiagnostics;

/// Configuration for critical turn detection
#[derive(Debug, Clone)]
pub struct CriticalTurnConfig {
    /// Minimum life swing to trigger (default: 5)
    pub life_swing_threshold: i32,
    /// Minimum board advantage delta to trigger (default: 3.0)
    pub board_swing_threshold: f64,
}

impl Default for CriticalTurnConfig {
    fn default() -> Self {
        Self {
            life_swing_threshold: 5,
            board_swing_threshold: 3.0,
        }
    }
}

impl CriticalTurnConfig {
    /// Create a new config with custom thresholds
    pub fn new(life_swing: i32, board_swing: f64) -> Self {
        Self {
            life_swing_threshold: life_swing,
            board_swing_threshold: board_swing,
        }
    }
}

/// Trigger type for a critical turn
#[derive(Debug, Clone)]
pub enum CriticalTurnTrigger {
    /// Life swing exceeding threshold
    LifeSwing { player: PlayerId, amount: i32 },
    /// Board advantage change exceeding threshold
    BoardAdvantageSwing { delta: f64 },
    /// Leadership changed between turns
    LeadershipChange,
}

/// A detected critical turn
#[derive(Debug, Clone)]
pub struct CriticalTurn {
    /// Which game this occurred in
    pub game_index: usize,
    /// Turn number
    pub turn_number: u32,
    /// What triggered this critical turn
    pub trigger: CriticalTurnTrigger,
    /// Who eventually won the game
    pub eventual_winner: Option<PlayerId>,
    /// Which player was advantaged after this turn
    pub advantaged_player: Option<PlayerId>,
}

/// Aggregated critical turn statistics
#[derive(Debug, Clone)]
pub struct CriticalTurnStats {
    /// Total critical turns detected
    pub total_critical_turns: usize,
    /// Number of games analyzed
    pub total_games: usize,
    /// Critical turns per game on average
    pub avg_per_game: f64,
    /// Distribution by turn number ranges
    pub by_turn_range: TurnRangeDistribution,
    /// Trigger type distribution
    pub by_trigger: TriggerDistribution,
    /// Win correlation: percentage of times the advantaged player won
    pub advantaged_player_win_rate: f64,
    /// All detected critical turns (for detailed analysis)
    pub critical_turns: Vec<CriticalTurn>,
}

/// Distribution of critical turns by turn number range
#[derive(Debug, Clone, Default)]
pub struct TurnRangeDistribution {
    pub early: usize,    // Turns 1-5
    pub mid_early: usize, // Turns 6-8
    pub mid: usize,      // Turns 9-11
    pub late: usize,     // Turns 12+
}

impl TurnRangeDistribution {
    fn add(&mut self, turn: u32) {
        match turn {
            1..=5 => self.early += 1,
            6..=8 => self.mid_early += 1,
            9..=11 => self.mid += 1,
            _ => self.late += 1,
        }
    }

    fn total(&self) -> usize {
        self.early + self.mid_early + self.mid + self.late
    }
}

/// Distribution by trigger type
#[derive(Debug, Clone, Default)]
pub struct TriggerDistribution {
    pub life_swing: usize,
    pub board_swing: usize,
    pub leadership_change: usize,
}

impl TriggerDistribution {
    fn add(&mut self, trigger: &CriticalTurnTrigger) {
        match trigger {
            CriticalTurnTrigger::LifeSwing { .. } => self.life_swing += 1,
            CriticalTurnTrigger::BoardAdvantageSwing { .. } => self.board_swing += 1,
            CriticalTurnTrigger::LeadershipChange => self.leadership_change += 1,
        }
    }
}

/// Analyze games for critical turns
pub fn analyze_critical_turns(
    games: &[GameDiagnostics],
    config: &CriticalTurnConfig,
) -> CriticalTurnStats {
    let mut critical_turns = Vec::new();

    for (game_idx, game) in games.iter().enumerate() {
        let game_critical = detect_critical_turns(game_idx, game, config);
        critical_turns.extend(game_critical);
    }

    // Compute statistics
    let total_critical_turns = critical_turns.len();
    let total_games = games.len();
    let avg_per_game = if total_games > 0 {
        total_critical_turns as f64 / total_games as f64
    } else {
        0.0
    };

    // Distribution by turn range
    let mut by_turn_range = TurnRangeDistribution::default();
    for ct in &critical_turns {
        by_turn_range.add(ct.turn_number);
    }

    // Distribution by trigger
    let mut by_trigger = TriggerDistribution::default();
    for ct in &critical_turns {
        by_trigger.add(&ct.trigger);
    }

    // Win correlation: how often did the advantaged player win?
    let mut advantaged_wins = 0;
    let mut advantaged_total = 0;
    for ct in &critical_turns {
        if let (Some(advantaged), Some(winner)) = (ct.advantaged_player, ct.eventual_winner) {
            advantaged_total += 1;
            if advantaged == winner {
                advantaged_wins += 1;
            }
        }
    }
    let advantaged_player_win_rate = if advantaged_total > 0 {
        advantaged_wins as f64 / advantaged_total as f64
    } else {
        0.0
    };

    CriticalTurnStats {
        total_critical_turns,
        total_games,
        avg_per_game,
        by_turn_range,
        by_trigger,
        advantaged_player_win_rate,
        critical_turns,
    }
}

/// Detect critical turns in a single game
fn detect_critical_turns(
    game_index: usize,
    game: &GameDiagnostics,
    config: &CriticalTurnConfig,
) -> Vec<CriticalTurn> {
    let mut critical = Vec::new();

    let snapshots = &game.snapshots;
    if snapshots.len() < 2 {
        return critical;
    }

    for i in 1..snapshots.len() {
        let prev = &snapshots[i - 1];
        let curr = &snapshots[i];

        // Skip if same turn (multiple snapshots per turn)
        if prev.turn == curr.turn {
            continue;
        }

        // Check for life swing
        let p1_life_change = curr.p1_life - prev.p1_life;
        let p2_life_change = curr.p2_life - prev.p2_life;

        // P1 took significant damage
        if p1_life_change <= -config.life_swing_threshold {
            critical.push(CriticalTurn {
                game_index,
                turn_number: curr.turn,
                trigger: CriticalTurnTrigger::LifeSwing {
                    player: PlayerId::PLAYER_ONE,
                    amount: -p1_life_change,
                },
                eventual_winner: game.winner,
                advantaged_player: Some(PlayerId::PLAYER_TWO),
            });
        }

        // P2 took significant damage
        if p2_life_change <= -config.life_swing_threshold {
            critical.push(CriticalTurn {
                game_index,
                turn_number: curr.turn,
                trigger: CriticalTurnTrigger::LifeSwing {
                    player: PlayerId::PLAYER_TWO,
                    amount: -p2_life_change,
                },
                eventual_winner: game.winner,
                advantaged_player: Some(PlayerId::PLAYER_ONE),
            });
        }

        // Check for board advantage swing
        let prev_advantage = board_advantage(prev.p1_total_attack, prev.p1_total_health,
                                             prev.p2_total_attack, prev.p2_total_health);
        let curr_advantage = board_advantage(curr.p1_total_attack, curr.p1_total_health,
                                             curr.p2_total_attack, curr.p2_total_health);
        let advantage_delta = (curr_advantage - prev_advantage).abs();

        if advantage_delta >= config.board_swing_threshold {
            let advantaged = if curr_advantage > 0.0 {
                Some(PlayerId::PLAYER_ONE)
            } else if curr_advantage < 0.0 {
                Some(PlayerId::PLAYER_TWO)
            } else {
                None
            };

            critical.push(CriticalTurn {
                game_index,
                turn_number: curr.turn,
                trigger: CriticalTurnTrigger::BoardAdvantageSwing { delta: advantage_delta },
                eventual_winner: game.winner,
                advantaged_player: advantaged,
            });
        }

        // Check for leadership change (life leader changed)
        let prev_leader = life_leader(prev.p1_life, prev.p2_life);
        let curr_leader = life_leader(curr.p1_life, curr.p2_life);

        if prev_leader != curr_leader {
            critical.push(CriticalTurn {
                game_index,
                turn_number: curr.turn,
                trigger: CriticalTurnTrigger::LeadershipChange,
                eventual_winner: game.winner,
                advantaged_player: curr_leader,
            });
        }
    }

    critical
}

/// Compute board advantage score (positive = P1 advantage)
fn board_advantage(p1_attack: i32, p1_health: i32, p2_attack: i32, p2_health: i32) -> f64 {
    let p1_score = p1_attack as f64 * 1.5 + p1_health as f64;
    let p2_score = p2_attack as f64 * 1.5 + p2_health as f64;
    p1_score - p2_score
}

/// Determine who is leading in life (None = tied)
fn life_leader(p1_life: i32, p2_life: i32) -> Option<PlayerId> {
    if p1_life > p2_life {
        Some(PlayerId::PLAYER_ONE)
    } else if p2_life > p1_life {
        Some(PlayerId::PLAYER_TWO)
    } else {
        None
    }
}

/// Print critical turn analysis report
pub fn print_critical_turns_report(stats: &CriticalTurnStats, config: &CriticalTurnConfig) {
    println!();
    println!("=== Critical Turn Analysis ===");
    println!(
        "Detection: Life swing >= {}, Board swing >= {:.1}",
        config.life_swing_threshold, config.board_swing_threshold
    );
    println!();

    println!(
        "Critical Turns: {} across {} games ({:.1} per game avg)",
        stats.total_critical_turns, stats.total_games, stats.avg_per_game
    );
    println!();

    // Turn range distribution
    println!("By Turn Number:");
    let total = stats.by_turn_range.total().max(1) as f64;
    let bar_scale = 40.0;

    let ranges = [
        ("Turn 1-5:  ", stats.by_turn_range.early),
        ("Turn 6-8:  ", stats.by_turn_range.mid_early),
        ("Turn 9-11: ", stats.by_turn_range.mid),
        ("Turn 12+:  ", stats.by_turn_range.late),
    ];

    for (label, count) in ranges {
        let pct = count as f64 / total * 100.0;
        let bar_len = (pct / 100.0 * bar_scale) as usize;
        let bar = "#".repeat(bar_len);
        println!("  {} {:>4} ({:5.1}%)  {}", label, count, pct, bar);
    }
    println!();

    // Trigger distribution
    println!("Trigger Distribution:");
    let trigger_total = (stats.by_trigger.life_swing
        + stats.by_trigger.board_swing
        + stats.by_trigger.leadership_change)
        .max(1) as f64;

    let triggers = [
        ("Life swing:       ", stats.by_trigger.life_swing),
        ("Board swing:      ", stats.by_trigger.board_swing),
        ("Leadership change:", stats.by_trigger.leadership_change),
    ];

    for (label, count) in triggers {
        let pct = count as f64 / trigger_total * 100.0;
        println!("  {} {:>4} ({:5.1}%)", label, count, pct);
    }
    println!();

    // Outcome correlation
    println!("Outcome Correlation:");
    println!(
        "  Advantage-gaining player won: {:.1}%",
        stats.advantaged_player_win_rate * 100.0
    );
}
