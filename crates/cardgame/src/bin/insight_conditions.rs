//! Analysis of Commander's Insight catch-up conditions.
//!
//! Checks how often players are "behind" in late game to inform
//! the choice between AND, OR, and XOR conditions.

use cardgame::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use cardgame::engine::GameEngine;
use cardgame::execution::GameData;
use cardgame::types::PlayerId;

fn main() {
    let games = 300;
    let bot_type = BotType::AlphaBeta;
    let depth = 6;

    println!("=== Commander's Insight Condition Analysis ===\n");
    println!("Analyzing {} games to determine optimal catch-up condition...\n", games);

    let data = GameData::load_default(true).expect("Failed to load game data");
    let decks: Vec<_> = data.deck_registry.decks().cloned().collect();

    // Track conditions for turns 10+
    let mut total_player_turns = 0u64;
    let mut starving_turns = 0u64;  // 0-1 cards in hand

    // When starving, what's the "behind" status?
    let mut starving_behind_creatures_only = 0u64;
    let mut starving_behind_life_only = 0u64;
    let mut starving_behind_both = 0u64;
    let mut starving_behind_neither = 0u64;  // Starving but winning/tied
    let mut starving_tied_creatures = 0u64;
    let mut starving_tied_life = 0u64;
    let mut starving_tied_both = 0u64;

    let mcts_config = MctsConfig::default();
    let ab_config = AlphaBetaConfig::with_depth(depth);

    let mut total_games = 0u32;

    for (i, deck1) in decks.iter().enumerate() {
        for deck2 in decks.iter().skip(i) {
            if total_games >= games as u32 {
                break;
            }

            let seed = (total_games as u64) * 12345 + 42;

            let mut bot1 = create_bot(&data.card_db, &bot_type, None, &mcts_config, &ab_config, seed);
            let mut bot2 = create_bot(&data.card_db, &bot_type, None, &mcts_config, &ab_config, seed + 1);

            let mut engine = GameEngine::new(&data.card_db);
            engine.start_game(deck1, deck2, seed);

            let mut action_count = 0;
            let max_actions = 1000;

            while !engine.is_game_over() && action_count < max_actions {
                let turn = engine.turn_number();

                if turn >= 10 {
                    // Analyze P1's state
                    let p1_hand = engine.state.players[0].hand.len();
                    let p2_hand = engine.state.players[1].hand.len();
                    let p1_creatures = engine.state.players[0].creatures.len();
                    let p2_creatures = engine.state.players[1].creatures.len();
                    let p1_life = engine.state.players[0].life;
                    let p2_life = engine.state.players[1].life;

                    // Check P1
                    total_player_turns += 1;
                    if p1_hand <= 1 {
                        starving_turns += 1;

                        let behind_creatures = p1_creatures < p2_creatures;
                        let behind_life = p1_life < p2_life;
                        let tied_creatures = p1_creatures == p2_creatures;
                        let tied_life = p1_life == p2_life;

                        if tied_creatures && tied_life {
                            starving_tied_both += 1;
                        } else if tied_creatures {
                            starving_tied_creatures += 1;
                        } else if tied_life {
                            starving_tied_life += 1;
                        }

                        if behind_creatures && behind_life {
                            starving_behind_both += 1;
                        } else if behind_creatures && !behind_life {
                            starving_behind_creatures_only += 1;
                        } else if !behind_creatures && behind_life {
                            starving_behind_life_only += 1;
                        } else {
                            starving_behind_neither += 1;
                        }
                    }

                    // Check P2
                    total_player_turns += 1;
                    if p2_hand <= 1 {
                        starving_turns += 1;

                        let behind_creatures = p2_creatures < p1_creatures;
                        let behind_life = p2_life < p1_life;
                        let tied_creatures = p2_creatures == p1_creatures;
                        let tied_life = p2_life == p1_life;

                        if tied_creatures && tied_life {
                            starving_tied_both += 1;
                        } else if tied_creatures {
                            starving_tied_creatures += 1;
                        } else if tied_life {
                            starving_tied_life += 1;
                        }

                        if behind_creatures && behind_life {
                            starving_behind_both += 1;
                        } else if behind_creatures && !behind_life {
                            starving_behind_creatures_only += 1;
                        } else if !behind_creatures && behind_life {
                            starving_behind_life_only += 1;
                        } else {
                            starving_behind_neither += 1;
                        }
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

            total_games += 1;
            if total_games.is_multiple_of(50) {
                println!("Progress: {}/{}", total_games, games);
            }
        }
        if total_games >= games as u32 {
            break;
        }
    }

    println!("\n=== Results (Turn 10+ only) ===\n");

    println!("Total player-turns analyzed: {}", total_player_turns);
    println!("Starving turns (0-1 cards): {} ({:.1}%)", starving_turns,
             100.0 * starving_turns as f64 / total_player_turns as f64);
    println!();

    println!("=== When Starving, What's the Board State? ===\n");

    let pct = |n: u64| 100.0 * n as f64 / starving_turns.max(1) as f64;

    println!("Behind on BOTH (creatures AND life):     {:>5} ({:>5.1}%) ← AND condition",
             starving_behind_both, pct(starving_behind_both));
    println!("Behind on creatures ONLY:                {:>5} ({:>5.1}%)",
             starving_behind_creatures_only, pct(starving_behind_creatures_only));
    println!("Behind on life ONLY:                     {:>5} ({:>5.1}%)",
             starving_behind_life_only, pct(starving_behind_life_only));
    println!("Behind on NEITHER (winning or tied):     {:>5} ({:>5.1}%) ← blocked correctly",
             starving_behind_neither, pct(starving_behind_neither));
    println!();

    let or_eligible = starving_behind_both + starving_behind_creatures_only + starving_behind_life_only;
    let xor_eligible = starving_behind_creatures_only + starving_behind_life_only;
    let and_eligible = starving_behind_both;

    println!("=== Eligibility by Condition Type ===\n");
    println!("OR condition (behind on either):   {:>5} ({:>5.1}%) of starving turns",
             or_eligible, pct(or_eligible));
    println!("XOR condition (behind on one):     {:>5} ({:>5.1}%) of starving turns",
             xor_eligible, pct(xor_eligible));
    println!("AND condition (behind on both):    {:>5} ({:>5.1}%) of starving turns",
             and_eligible, pct(and_eligible));
    println!();

    println!("=== Tie Scenarios (when starving) ===\n");
    println!("Tied on BOTH (creatures = life =):       {:>5} ({:>5.1}%)",
             starving_tied_both, pct(starving_tied_both));
    println!("Tied on creatures only:                  {:>5} ({:>5.1}%)",
             starving_tied_creatures, pct(starving_tied_creatures));
    println!("Tied on life only:                       {:>5} ({:>5.1}%)",
             starving_tied_life, pct(starving_tied_life));
    println!();

    println!("=== Recommendation ===\n");

    let or_pct = pct(or_eligible);
    let and_pct = pct(and_eligible);

    if and_pct < 30.0 {
        println!("AND condition is too restrictive ({:.1}% of starving turns).", and_pct);
        println!("Many players who are struggling wouldn't get help.");
    }

    if or_pct > 70.0 {
        println!("OR condition covers most struggling players ({:.1}%).", or_pct);
        println!("This seems like the right balance - helps when behind on either metric.");
    }

    let blocked_pct = pct(starving_behind_neither);
    println!("\n{:.1}% of starving turns are correctly blocked (player is winning/tied).", blocked_pct);
}
