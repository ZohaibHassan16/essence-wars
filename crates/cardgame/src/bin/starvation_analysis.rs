//! Hand starvation analysis for Commander's Insight evaluation.
//!
//! Runs games and tracks how often players enter "top-deck mode"
//! (0-1 cards in hand after turn 10).

use std::collections::HashMap;

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use cardgame::engine::GameEngine;
use cardgame::execution::GameData;
use cardgame::types::PlayerId;

fn main() {
    let games = 500;
    let bot_type = BotType::AlphaBeta;
    let depth = 6;

    println!("=== Hand Starvation Analysis for Commander's Insight ===\n");
    println!("Running {} games with {:?} (depth {})...\n", games, bot_type, depth);

    // Load game data
    let data = GameData::load_default(true).expect("Failed to load game data");

    // Get all decks
    let decks: Vec<_> = data.deck_registry.decks().cloned().collect();

    // Tracking variables
    let mut _total_turns = 0u64;
    let mut turns_after_10 = 0u64;
    let mut starvation_turns_0_cards = 0u64;  // 0 cards in hand
    let mut starvation_turns_1_card = 0u64;   // 1 card in hand
    let mut games_reaching_turn_10 = 0u32;
    let mut games_reaching_turn_15 = 0u32;
    let mut games_reaching_turn_20 = 0u32;
    let mut game_lengths = Vec::new();
    let mut hand_sizes_by_turn: HashMap<u32, Vec<usize>> = HashMap::new();
    let mut total_games = 0u32;

    // Run games with various deck combinations
    let mcts_config = MctsConfig::default();
    let ab_config = AlphaBetaConfig::with_depth(depth);

    for (i, deck1) in decks.iter().enumerate() {
        for deck2 in decks.iter().skip(i) {
            if total_games >= games as u32 {
                break;
            }

            // Run a game
            let seed = (total_games as u64) * 12345 + 42;

            let mut bot1 = create_bot(&data.card_db, &bot_type, None, &mcts_config, &ab_config, seed);
            let mut bot2 = create_bot(&data.card_db, &bot_type, None, &mcts_config, &ab_config, seed + 1);

            let mut engine = GameEngine::new(&data.card_db);
            engine.start_game(deck1, deck2, seed);

            let mut action_count = 0;
            let max_actions = 1000;

            while !engine.is_game_over() && action_count < max_actions {
                let turn = engine.turn_number();

                // Record hand sizes at start of each turn
                let p1_hand = engine.state.players[0].hand.len();
                let p2_hand = engine.state.players[1].hand.len();

                hand_sizes_by_turn.entry(turn as u32).or_default().push(p1_hand);
                hand_sizes_by_turn.entry(turn as u32).or_default().push(p2_hand);

                _total_turns += 1;

                if turn > 10 {
                    turns_after_10 += 1;
                    if p1_hand == 0 {
                        starvation_turns_0_cards += 1;
                    } else if p1_hand == 1 {
                        starvation_turns_1_card += 1;
                    }
                    if p2_hand == 0 {
                        starvation_turns_0_cards += 1;
                    } else if p2_hand == 1 {
                        starvation_turns_1_card += 1;
                    }
                }

                let action = if engine.current_player() == PlayerId::PLAYER_ONE {
                    bot1.select_action_with_engine(&engine)
                } else {
                    bot2.select_action_with_engine(&engine)
                };

                if engine.apply_action(action).is_err() {
                    break;
                }
                action_count += 1;
            }

            let final_turn = engine.turn_number();
            game_lengths.push(final_turn);

            if final_turn > 10 {
                games_reaching_turn_10 += 1;
            }
            if final_turn > 15 {
                games_reaching_turn_15 += 1;
            }
            if final_turn > 20 {
                games_reaching_turn_20 += 1;
            }

            total_games += 1;

            if total_games.is_multiple_of(50) {
                println!("Progress: {}/{}", total_games, games);
            }
        }
        if total_games >= games as u32 {
            break;
        }
    }

    // Calculate statistics
    game_lengths.sort();
    let median_length = game_lengths[game_lengths.len() / 2];
    let avg_length: f64 = game_lengths.iter().map(|&x| x as f64).sum::<f64>() / game_lengths.len() as f64;
    let p10 = game_lengths[game_lengths.len() / 10];
    let p90 = game_lengths[game_lengths.len() * 9 / 10];

    println!("\n=== Results ===\n");

    println!("Game Length Distribution:");
    println!("  Total games: {}", total_games);
    println!("  P10 (10th percentile): {} turns", p10);
    println!("  Median: {} turns", median_length);
    println!("  Average: {:.1} turns", avg_length);
    println!("  P90 (90th percentile): {} turns", p90);
    println!();

    println!("Games Reaching Late Game:");
    println!("  Turn 10+: {} ({:.1}%)", games_reaching_turn_10, 100.0 * games_reaching_turn_10 as f64 / total_games as f64);
    println!("  Turn 15+: {} ({:.1}%)", games_reaching_turn_15, 100.0 * games_reaching_turn_15 as f64 / total_games as f64);
    println!("  Turn 20+: {} ({:.1}%)", games_reaching_turn_20, 100.0 * games_reaching_turn_20 as f64 / total_games as f64);
    println!();

    println!("Hand Starvation (after turn 10):");
    println!("  Total player-turns after turn 10: {}", turns_after_10);
    println!("  Turns with 0 cards: {} ({:.1}%)", starvation_turns_0_cards, 100.0 * starvation_turns_0_cards as f64 / turns_after_10.max(1) as f64);
    println!("  Turns with 1 card: {} ({:.1}%)", starvation_turns_1_card, 100.0 * starvation_turns_1_card as f64 / turns_after_10.max(1) as f64);
    println!("  Turns with 0-1 cards: {} ({:.1}%)", starvation_turns_0_cards + starvation_turns_1_card, 100.0 * (starvation_turns_0_cards + starvation_turns_1_card) as f64 / turns_after_10.max(1) as f64);
    println!();

    println!("Average Hand Size by Turn:");
    let mut turns: Vec<_> = hand_sizes_by_turn.keys().copied().collect();
    turns.sort();
    for turn in turns.iter().take(25) {
        let sizes = hand_sizes_by_turn.get(turn).unwrap();
        let avg: f64 = sizes.iter().map(|&x| x as f64).sum::<f64>() / sizes.len() as f64;
        let zeros = sizes.iter().filter(|&&x| x == 0).count();
        let ones = sizes.iter().filter(|&&x| x == 1).count();
        let pct_0_1 = 100.0 * (zeros + ones) as f64 / sizes.len() as f64;
        println!("  Turn {:>2}: avg {:.2} cards, {:.1}% with 0-1 cards (n={})", turn, avg, pct_0_1, sizes.len());
    }

    println!("\n=== Commander's Insight Relevance ===\n");

    let starvation_pct = 100.0 * (starvation_turns_0_cards + starvation_turns_1_card) as f64 / turns_after_10.max(1) as f64;

    if starvation_pct < 10.0 {
        println!("LOW STARVATION ({:.1}%): Hand starvation is rare after turn 10.", starvation_pct);
        println!("Commander's Insight may not be necessary - players usually have cards to play.");
    } else if starvation_pct < 30.0 {
        println!("MODERATE STARVATION ({:.1}%): Hand starvation occurs occasionally.", starvation_pct);
        println!("Commander's Insight could provide meaningful strategic depth.");
    } else {
        println!("HIGH STARVATION ({:.1}%): Hand starvation is common after turn 10.", starvation_pct);
        println!("Commander's Insight would significantly impact late-game play patterns.");
    }

    let late_game_pct = 100.0 * games_reaching_turn_10 as f64 / total_games as f64;
    println!("\n{:.0}% of games reach turn 10+, so the mechanic would affect {:.0}% of games.", late_game_pct, late_game_pct);
}
