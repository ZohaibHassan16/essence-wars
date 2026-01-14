//! P1/P2 Asymmetry Diagnostic Tool
//!
//! Collects detailed per-turn statistics to analyze why P1 has a lower win rate.
//! Outputs data for analysis of resource curves, tempo, and game progression.

use std::collections::HashMap;

use cardgame::bots::{Bot, GreedyBot};
use cardgame::cards::CardDatabase;
use cardgame::decks::DeckRegistry;
use cardgame::engine::GameEngine;
use cardgame::types::{CardId, PlayerId};

/// Snapshot of game state at a specific point
#[derive(Clone, Debug)]
struct TurnSnapshot {
    turn: u32,
    active_player: PlayerId,
    // Life totals
    p1_life: i32,
    p2_life: i32,
    // Board state
    p1_creatures: usize,
    p2_creatures: usize,
    p1_total_attack: i32,
    p2_total_attack: i32,
    p1_total_health: i32,
    p2_total_health: i32,
    // Resources
    p1_hand_size: usize,
    p2_hand_size: usize,
    p1_essence: u8,
    p2_essence: u8,
    p1_max_essence: u8,
    p2_max_essence: u8,
    _p1_action_points: u8,
    _p2_action_points: u8,
}

/// Statistics for a single game
#[derive(Clone, Debug)]
struct GameDiagnostics {
    seed: u64,
    winner: Option<PlayerId>,
    total_turns: u32,
    snapshots: Vec<TurnSnapshot>,
    // Key events
    first_damage_to_p1_turn: Option<u32>,
    first_damage_to_p2_turn: Option<u32>,
    first_creature_death_turn: Option<u32>,
    p1_actions: usize,
    p2_actions: usize,
}

/// Aggregated statistics across all games
#[derive(Default)]
struct AggregatedStats {
    total_games: usize,
    p1_wins: usize,
    p2_wins: usize,
    draws: usize,

    // Win rate by game length
    p1_wins_early: usize,  // turns 1-10
    p1_wins_mid: usize,    // turns 11-20
    p1_wins_late: usize,   // turns 21-30
    games_early: usize,
    games_mid: usize,
    games_late: usize,

    // First blood statistics
    p1_first_blood: usize,
    p2_first_blood: usize,

    // Resource curves by turn (turn -> (sum, count))
    p1_life_by_turn: HashMap<u32, (i64, usize)>,
    p2_life_by_turn: HashMap<u32, (i64, usize)>,
    p1_creatures_by_turn: HashMap<u32, (i64, usize)>,
    p2_creatures_by_turn: HashMap<u32, (i64, usize)>,
    p1_hand_by_turn: HashMap<u32, (i64, usize)>,
    p2_hand_by_turn: HashMap<u32, (i64, usize)>,
    p1_board_attack_by_turn: HashMap<u32, (i64, usize)>,
    p2_board_attack_by_turn: HashMap<u32, (i64, usize)>,
    p1_essence_by_turn: HashMap<u32, (i64, usize)>,
    p2_essence_by_turn: HashMap<u32, (i64, usize)>,
    p1_max_essence_by_turn: HashMap<u32, (i64, usize)>,
    p2_max_essence_by_turn: HashMap<u32, (i64, usize)>,
    p1_board_health_by_turn: HashMap<u32, (i64, usize)>,
    p2_board_health_by_turn: HashMap<u32, (i64, usize)>,

    // Actions per turn
    p1_actions_total: usize,
    p2_actions_total: usize,

    // First creature death
    first_creature_death_turns: Vec<u32>,

    // Notable game seeds for reproduction
    earliest_p1_win_seed: Option<(u64, u32)>,
    earliest_p2_win_seed: Option<(u64, u32)>,
}

impl AggregatedStats {
    fn record_game(&mut self, diag: &GameDiagnostics) {
        self.total_games += 1;

        // Record winner
        match diag.winner {
            Some(PlayerId::PLAYER_ONE) => self.p1_wins += 1,
            Some(PlayerId::PLAYER_TWO) => self.p2_wins += 1,
            _ => self.draws += 1,
        }

        // Win rate by game length
        let length_bucket = if diag.total_turns <= 10 {
            self.games_early += 1;
            if diag.winner == Some(PlayerId::PLAYER_ONE) {
                self.p1_wins_early += 1;
            }
            "early"
        } else if diag.total_turns <= 20 {
            self.games_mid += 1;
            if diag.winner == Some(PlayerId::PLAYER_ONE) {
                self.p1_wins_mid += 1;
            }
            "mid"
        } else {
            self.games_late += 1;
            if diag.winner == Some(PlayerId::PLAYER_ONE) {
                self.p1_wins_late += 1;
            }
            "late"
        };
        let _ = length_bucket; // suppress warning

        // First blood
        match (diag.first_damage_to_p1_turn, diag.first_damage_to_p2_turn) {
            (Some(t1), Some(t2)) if t2 < t1 => self.p1_first_blood += 1,
            (Some(t1), Some(t2)) if t1 < t2 => self.p2_first_blood += 1,
            (None, Some(_)) => self.p1_first_blood += 1,
            (Some(_), None) => self.p2_first_blood += 1,
            _ => {}
        }

        // Actions
        self.p1_actions_total += diag.p1_actions;
        self.p2_actions_total += diag.p2_actions;

        // First creature death
        if let Some(turn) = diag.first_creature_death_turn {
            self.first_creature_death_turns.push(turn);
        }

        // Track notable game seeds
        if diag.winner == Some(PlayerId::PLAYER_ONE) {
            if self.earliest_p1_win_seed.is_none() || diag.total_turns < self.earliest_p1_win_seed.unwrap().1 {
                self.earliest_p1_win_seed = Some((diag.seed, diag.total_turns));
            }
        } else if diag.winner == Some(PlayerId::PLAYER_TWO) {
            if self.earliest_p2_win_seed.is_none() || diag.total_turns < self.earliest_p2_win_seed.unwrap().1 {
                self.earliest_p2_win_seed = Some((diag.seed, diag.total_turns));
            }
        }

        // Resource curves - record at start of each turn
        for snapshot in &diag.snapshots {
            let turn = snapshot.turn;

            // Only record at start of P1's turn for consistent comparison
            if snapshot.active_player == PlayerId::PLAYER_ONE {
                Self::add_to_curve(&mut self.p1_life_by_turn, turn, snapshot.p1_life as i64);
                Self::add_to_curve(&mut self.p2_life_by_turn, turn, snapshot.p2_life as i64);
                Self::add_to_curve(&mut self.p1_creatures_by_turn, turn, snapshot.p1_creatures as i64);
                Self::add_to_curve(&mut self.p2_creatures_by_turn, turn, snapshot.p2_creatures as i64);
                Self::add_to_curve(&mut self.p1_hand_by_turn, turn, snapshot.p1_hand_size as i64);
                Self::add_to_curve(&mut self.p2_hand_by_turn, turn, snapshot.p2_hand_size as i64);
                Self::add_to_curve(&mut self.p1_board_attack_by_turn, turn, snapshot.p1_total_attack as i64);
                Self::add_to_curve(&mut self.p2_board_attack_by_turn, turn, snapshot.p2_total_attack as i64);
                Self::add_to_curve(&mut self.p1_essence_by_turn, turn, snapshot.p1_essence as i64);
                Self::add_to_curve(&mut self.p2_essence_by_turn, turn, snapshot.p2_essence as i64);
                Self::add_to_curve(&mut self.p1_max_essence_by_turn, turn, snapshot.p1_max_essence as i64);
                Self::add_to_curve(&mut self.p2_max_essence_by_turn, turn, snapshot.p2_max_essence as i64);
                Self::add_to_curve(&mut self.p1_board_health_by_turn, turn, snapshot.p1_total_health as i64);
                Self::add_to_curve(&mut self.p2_board_health_by_turn, turn, snapshot.p2_total_health as i64);
            }
        }
    }

    fn add_to_curve(map: &mut HashMap<u32, (i64, usize)>, turn: u32, value: i64) {
        let entry = map.entry(turn).or_insert((0, 0));
        entry.0 += value;
        entry.1 += 1;
    }

    fn avg_curve(map: &HashMap<u32, (i64, usize)>) -> Vec<(u32, f64)> {
        let mut result: Vec<_> = map.iter()
            .map(|(&turn, &(sum, count))| (turn, sum as f64 / count as f64))
            .collect();
        result.sort_by_key(|(turn, _)| *turn);
        result
    }

    fn print_report(&self) {
        println!("\n{}", "=".repeat(60));
        println!("P1/P2 ASYMMETRY DIAGNOSTIC REPORT");
        println!("{}\n", "=".repeat(60));

        // Overall stats
        println!("=== Overall Statistics ===");
        println!("Total games: {}", self.total_games);
        println!("P1 wins: {} ({:.1}%)", self.p1_wins, 100.0 * self.p1_wins as f64 / self.total_games as f64);
        println!("P2 wins: {} ({:.1}%)", self.p2_wins, 100.0 * self.p2_wins as f64 / self.total_games as f64);
        println!("Draws: {} ({:.1}%)", self.draws, 100.0 * self.draws as f64 / self.total_games as f64);

        // Win rate by game length
        println!("\n=== P1 Win Rate by Game Length ===");
        if self.games_early > 0 {
            println!("Early (turns 1-10):  {:.1}% ({}/{})",
                100.0 * self.p1_wins_early as f64 / self.games_early as f64,
                self.p1_wins_early, self.games_early);
        }
        if self.games_mid > 0 {
            println!("Mid (turns 11-20):   {:.1}% ({}/{})",
                100.0 * self.p1_wins_mid as f64 / self.games_mid as f64,
                self.p1_wins_mid, self.games_mid);
        }
        if self.games_late > 0 {
            println!("Late (turns 21-30):  {:.1}% ({}/{})",
                100.0 * self.p1_wins_late as f64 / self.games_late as f64,
                self.p1_wins_late, self.games_late);
        }

        // First blood
        println!("\n=== First Blood (First to Deal Damage) ===");
        let first_blood_total = self.p1_first_blood + self.p2_first_blood;
        if first_blood_total > 0 {
            println!("P1 first blood: {} ({:.1}%)", self.p1_first_blood,
                100.0 * self.p1_first_blood as f64 / first_blood_total as f64);
            println!("P2 first blood: {} ({:.1}%)", self.p2_first_blood,
                100.0 * self.p2_first_blood as f64 / first_blood_total as f64);
        }

        // First creature death
        if !self.first_creature_death_turns.is_empty() {
            let avg_death_turn = self.first_creature_death_turns.iter().sum::<u32>() as f64 
                / self.first_creature_death_turns.len() as f64;
            println!("\nAvg first creature death: turn {:.1}", avg_death_turn);
        }

        // Actions
        println!("\n=== Actions Per Game ===");
        println!("P1 avg actions: {:.1}", self.p1_actions_total as f64 / self.total_games as f64);
        println!("P2 avg actions: {:.1}", self.p2_actions_total as f64 / self.total_games as f64);

        // Resource curves
        println!("\n=== Average Resources by Turn (at start of P1's turn) ===");
        println!("{:>4} | {:>8} {:>8} | {:>6} {:>6} | {:>5} {:>5} | {:>6} {:>6}",
            "Turn", "P1 Life", "P2 Life", "P1 Crt", "P2 Crt", "P1 Hnd", "P2 Hnd", "P1 Atk", "P2 Atk");
        println!("{}", "-".repeat(80));

        let p1_life = Self::avg_curve(&self.p1_life_by_turn);
        let p2_life = Self::avg_curve(&self.p2_life_by_turn);
        let p1_creatures = Self::avg_curve(&self.p1_creatures_by_turn);
        let p2_creatures = Self::avg_curve(&self.p2_creatures_by_turn);
        let p1_hand = Self::avg_curve(&self.p1_hand_by_turn);
        let p2_hand = Self::avg_curve(&self.p2_hand_by_turn);
        let p1_attack = Self::avg_curve(&self.p1_board_attack_by_turn);
        let p2_attack = Self::avg_curve(&self.p2_board_attack_by_turn);

        for i in 0..p1_life.len().min(15) {
            let turn = p1_life.get(i).map(|(t, _)| *t).unwrap_or(0);
            println!("{:>4} | {:>8.1} {:>8.1} | {:>6.1} {:>6.1} | {:>5.1} {:>5.1} | {:>6.1} {:>6.1}",
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

        // Essence curves
        println!("\n=== Average Essence by Turn (at start of P1's turn) ===");
        println!("{:>4} | {:>8} {:>8} | {:>8} {:>8}",
            "Turn", "P1 Ess", "P2 Ess", "P1 Max", "P2 Max");
        println!("{}", "-".repeat(50));

        let p1_essence = Self::avg_curve(&self.p1_essence_by_turn);
        let p2_essence = Self::avg_curve(&self.p2_essence_by_turn);
        let p1_max_essence = Self::avg_curve(&self.p1_max_essence_by_turn);
        let p2_max_essence = Self::avg_curve(&self.p2_max_essence_by_turn);

        for i in 0..p1_essence.len().min(15) {
            let turn = p1_essence.get(i).map(|(t, _)| *t).unwrap_or(0);
            println!("{:>4} | {:>8.1} {:>8.1} | {:>8.1} {:>8.1}",
                turn,
                p1_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
                p2_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
                p1_max_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
                p2_max_essence.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            );
        }

        // Board health curves
        println!("\n=== Average Board Health by Turn (at start of P1's turn) ===");
        println!("{:>4} | {:>10} {:>10}",
            "Turn", "P1 Health", "P2 Health");
        println!("{}", "-".repeat(35));

        let p1_board_health = Self::avg_curve(&self.p1_board_health_by_turn);
        let p2_board_health = Self::avg_curve(&self.p2_board_health_by_turn);

        for i in 0..p1_board_health.len().min(15) {
            let turn = p1_board_health.get(i).map(|(t, _)| *t).unwrap_or(0);
            println!("{:>4} | {:>10.1} {:>10.1}",
                turn,
                p1_board_health.get(i).map(|(_, v)| *v).unwrap_or(0.0),
                p2_board_health.get(i).map(|(_, v)| *v).unwrap_or(0.0),
            );
        }

        // Notable games for reproduction
        println!("\n=== Notable Games (for debugging) ===");
        if let Some((seed, turns)) = self.earliest_p1_win_seed {
            println!("Fastest P1 win: {} turns (seed {})", turns, seed);
        }
        if let Some((seed, turns)) = self.earliest_p2_win_seed {
            println!("Fastest P2 win: {} turns (seed {})", turns, seed);
        }

        // Analysis hints
        println!("\n=== Analysis Hints ===");
        let p1_wr = self.p1_wins as f64 / self.total_games as f64;
        if p1_wr < 0.45 {
            println!("⚠ P1 win rate ({:.1}%) is below expected 50%", p1_wr * 100.0);

            if self.p2_first_blood > self.p1_first_blood {
                println!("  → P2 gets first blood more often - possible tempo advantage");
            }

            if self.games_early > 0 && (self.p1_wins_early as f64 / self.games_early as f64) < 0.4 {
                println!("  → P1 struggles especially in early game");
            }

            if self.p2_actions_total > self.p1_actions_total {
                println!("  → P2 takes more actions on average");
            }
        }
    }
}

fn capture_snapshot(engine: &GameEngine) -> TurnSnapshot {
    let state = &engine.state;
    let p1 = &state.players[0];
    let p2 = &state.players[1];

    TurnSnapshot {
        turn: engine.turn_number() as u32,
        active_player: state.active_player,
        p1_life: p1.life as i32,
        p2_life: p2.life as i32,
        p1_creatures: p1.creatures.len(),
        p2_creatures: p2.creatures.len(),
        p1_total_attack: p1.creatures.iter().map(|c| c.attack as i32).sum(),
        p2_total_attack: p2.creatures.iter().map(|c| c.attack as i32).sum(),
        p1_total_health: p1.creatures.iter().map(|c| c.current_health as i32).sum(),
        p2_total_health: p2.creatures.iter().map(|c| c.current_health as i32).sum(),
        p1_hand_size: p1.hand.len(),
        p2_hand_size: p2.hand.len(),
        p1_essence: p1.current_essence,
        p2_essence: p2.current_essence,
        p1_max_essence: p1.max_essence,
        p2_max_essence: p2.max_essence,
        _p1_action_points: p1.action_points,
        _p2_action_points: p2.action_points,
    }
}

fn run_diagnostic_game(
    card_db: &CardDatabase,
    deck1: Vec<CardId>,
    deck2: Vec<CardId>,
    seed: u64,
) -> GameDiagnostics {
    let mut bot1 = GreedyBot::new(card_db, seed);
    let mut bot2 = GreedyBot::new(card_db, seed + 1000);

    let mut engine = GameEngine::new(card_db);
    engine.start_game(deck1, deck2, seed);

    let mut snapshots = Vec::new();
    let mut first_damage_to_p1_turn = None;
    let mut first_damage_to_p2_turn = None;
    let mut first_creature_death_turn = None;
    let mut p1_actions = 0;
    let mut p2_actions = 0;

    let initial_p1_life = engine.state.players[0].life as i32;
    let initial_p2_life = engine.state.players[1].life as i32;
    let mut last_turn = 0;

    // Capture initial state
    snapshots.push(capture_snapshot(&engine));

    let max_actions = 1000;
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < max_actions {
        let current_player = engine.current_player();
        let current_turn = engine.turn_number() as u32;

        // Capture state at start of new turn
        if current_turn != last_turn {
            snapshots.push(capture_snapshot(&engine));
            last_turn = current_turn;
        }

        // Track creature count before action
        let p1_creatures_before = engine.state.players[0].creatures.len();
        let p2_creatures_before = engine.state.players[1].creatures.len();

        // Select and apply action
        let action = if current_player == PlayerId::PLAYER_ONE {
            p1_actions += 1;
            bot1.select_action_with_engine(&engine)
        } else {
            p2_actions += 1;
            bot2.select_action_with_engine(&engine)
        };

        if engine.apply_action(action).is_err() {
            break;
        }

        // Check for first damage
        if first_damage_to_p1_turn.is_none() && (engine.state.players[0].life as i32) < initial_p1_life {
            first_damage_to_p1_turn = Some(current_turn);
        }
        if first_damage_to_p2_turn.is_none() && (engine.state.players[1].life as i32) < initial_p2_life {
            first_damage_to_p2_turn = Some(current_turn);
        }

        // Check for first creature death
        if first_creature_death_turn.is_none() {
            let p1_creatures_after = engine.state.players[0].creatures.len();
            let p2_creatures_after = engine.state.players[1].creatures.len();
            if p1_creatures_after < p1_creatures_before || p2_creatures_after < p2_creatures_before {
                first_creature_death_turn = Some(current_turn);
            }
        }

        action_count += 1;
    }

    // Final snapshot
    snapshots.push(capture_snapshot(&engine));

    GameDiagnostics {
        seed,
        winner: engine.winner(),
        total_turns: engine.turn_number() as u32,
        snapshots,
        first_damage_to_p1_turn,
        first_damage_to_p2_turn,
        first_creature_death_turn,
        p1_actions,
        p2_actions,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);

    println!("P1/P2 Asymmetry Diagnostic Tool");
    println!("================================");
    println!("Running {} games with GreedyBot vs GreedyBot...\n", games);

    let card_db = CardDatabase::load_from_directory("data/cards/core_set")
        .expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory("data/decks")
        .expect("Failed to load decks");

    // Use symbiote_aggro as our test deck (most common in validation)
    let deck = deck_registry.get("symbiote_aggro")
        .expect("symbiote_aggro deck not found");
    let deck_cards: Vec<CardId> = deck.cards.iter().map(|&id| CardId(id)).collect();

    let mut stats = AggregatedStats::default();

    for i in 0..games {
        if i % 50 == 0 {
            print!("\rProgress: {}/{}", i, games);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }

        let seed = 42 + i as u64;
        let diag = run_diagnostic_game(&card_db, deck_cards.clone(), deck_cards.clone(), seed);
        stats.record_game(&diag);
    }

    println!("\rProgress: {}/{}", games, games);

    stats.print_report();
}
