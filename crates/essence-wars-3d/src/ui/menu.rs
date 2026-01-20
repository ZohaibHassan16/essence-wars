//! Main menu and game over screens.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use cardgame::bots::BotType;
use crate::game::{AppState, GameBridge, HeadlessStats};
use crate::game::turn_loop::{BotConfig, TurnState};
use crate::CliArgs;
use super::player_input::GameModeConfig;

/// Plugin for menus.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_main_menu.run_if(in_state(AppState::Menu)))
            .add_systems(Update, draw_game_over.run_if(in_state(AppState::GameOver)))
            .add_systems(Update, handle_headless_game_over.run_if(in_state(AppState::GameOver)));
    }
}

/// Resource for menu state.
#[derive(Default)]
pub struct MenuState {
    pub deck1: String,
    pub deck2: String,
    pub seed: String,
    pub game_mode: usize,  // 0 = Human vs AI, 1 = AI vs AI
    pub ai_type_p1: usize, // 0 = Random, 1 = Greedy, 2 = MCTS
    pub ai_type_p2: usize, // 0 = Random, 1 = Greedy, 2 = MCTS
}

impl MenuState {
    /// Convert AI type index to BotType.
    fn bot_type_from_index(index: usize) -> BotType {
        match index {
            0 => BotType::Random,
            1 => BotType::Greedy,
            _ => BotType::Mcts,
        }
    }
}

/// Draw the main menu.
fn draw_main_menu(
    mut contexts: EguiContexts,
    mut menu_state: Local<MenuState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut bridge: Option<ResMut<GameBridge>>,
    mut game_mode_config: ResMut<GameModeConfig>,
    mut bot_config: ResMut<BotConfig>,
) {
    // Set defaults if empty (use MVP deck IDs from embedded_data)
    if menu_state.deck1.is_empty() {
        menu_state.deck1 = "colossus_wall".to_string(); // Iron Colossus Prime (Argentum)
        menu_state.ai_type_p1 = 2; // Default P1 AI to MCTS
    }
    if menu_state.deck2.is_empty() {
        menu_state.deck2 = "broodmother_swarm".to_string(); // The Broodmother (Symbiote)
        menu_state.ai_type_p2 = 2; // Default P2 AI to MCTS
    }
    if menu_state.seed.is_empty() {
        menu_state.seed = "42".to_string();
    }

    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(egui::RichText::new("Essence Wars").size(48.0));
            ui.label("Glassbox Mode - AI Visualization");
            ui.add_space(30.0);

            ui.group(|ui| {
                ui.label(egui::RichText::new("Game Mode").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.selectable_label(menu_state.game_mode == 0, "Human vs AI").clicked() {
                        menu_state.game_mode = 0;
                    }
                    if ui.selectable_label(menu_state.game_mode == 1, "AI vs AI (Spectator)").clicked() {
                        menu_state.game_mode = 1;
                    }
                });
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label(egui::RichText::new("Game Setup").strong());
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let p1_label = if menu_state.game_mode == 0 { "Your Deck:" } else { "Player 1 Deck:" };
                    ui.label(p1_label);
                    ui.text_edit_singleline(&mut menu_state.deck1);
                });

                ui.horizontal(|ui| {
                    let p2_label = if menu_state.game_mode == 0 { "AI Deck:" } else { "Player 2 Deck:" };
                    ui.label(p2_label);
                    ui.text_edit_singleline(&mut menu_state.deck2);
                });

                ui.horizontal(|ui| {
                    ui.label("Random Seed:");
                    ui.text_edit_singleline(&mut menu_state.seed);
                });
            });

            ui.add_space(10.0);

            // AI Type Selection
            ui.group(|ui| {
                ui.label(egui::RichText::new("AI Configuration").strong());
                ui.add_space(5.0);

                // In Human vs AI mode, only show P2 (AI opponent) selection
                // In AI vs AI mode, show both
                if menu_state.game_mode == 1 {
                    // AI vs AI - show P1 AI type
                    ui.horizontal(|ui| {
                        ui.label("Player 1 AI:");
                        if ui.selectable_label(menu_state.ai_type_p1 == 0, "Random").clicked() {
                            menu_state.ai_type_p1 = 0;
                        }
                        if ui.selectable_label(menu_state.ai_type_p1 == 1, "Greedy").clicked() {
                            menu_state.ai_type_p1 = 1;
                        }
                        if ui.selectable_label(menu_state.ai_type_p1 == 2, "MCTS").clicked() {
                            menu_state.ai_type_p1 = 2;
                        }
                    });
                }

                ui.horizontal(|ui| {
                    let ai_label = if menu_state.game_mode == 0 { "AI Opponent:" } else { "Player 2 AI:" };
                    ui.label(ai_label);
                    if ui.selectable_label(menu_state.ai_type_p2 == 0, "Random").clicked() {
                        menu_state.ai_type_p2 = 0;
                    }
                    if ui.selectable_label(menu_state.ai_type_p2 == 1, "Greedy").clicked() {
                        menu_state.ai_type_p2 = 1;
                    }
                    if ui.selectable_label(menu_state.ai_type_p2 == 2, "MCTS").clicked() {
                        menu_state.ai_type_p2 = 2;
                    }
                });

                // Show hint about AI types
                ui.add_space(5.0);
                ui.label(egui::RichText::new("Random: Fast, weak | Greedy: Fast, medium | MCTS: Slow, strong").small().weak());
            });

            ui.add_space(20.0);

            let button_text = if menu_state.game_mode == 0 {
                "Start Game"
            } else {
                "Watch Match"
            };

            if ui.button(egui::RichText::new(button_text).size(24.0)).clicked() {
                if let Some(ref mut bridge) = bridge {
                    let seed: u64 = menu_state.seed.parse().unwrap_or(42);

                    // Set game mode config
                    *game_mode_config = if menu_state.game_mode == 0 {
                        GameModeConfig::human_vs_ai()
                    } else {
                        GameModeConfig::spectator()
                    };

                    // Set bot config from menu selections
                    bot_config.player1_type = MenuState::bot_type_from_index(menu_state.ai_type_p1);
                    bot_config.player2_type = MenuState::bot_type_from_index(menu_state.ai_type_p2);
                    bot_config.bot_seed = seed;

                    match bridge.start_game(&menu_state.deck1, &menu_state.deck2, seed) {
                        Ok(()) => {
                            let mode_name = if menu_state.game_mode == 0 {
                                "Human vs AI"
                            } else {
                                "AI vs AI"
                            };
                            let p1_bot = bot_config.player1_type.name();
                            let p2_bot = bot_config.player2_type.name();
                            info!("Game started successfully ({}) - P1: {}, P2: {}", mode_name, p1_bot, p2_bot);
                            next_state.set(AppState::Playing);
                        }
                        Err(e) => {
                            error!("Failed to start game: {}", e);
                        }
                    }
                }
            }

            ui.add_space(20.0);

            // Show available decks
            if let Some(bridge) = &bridge {
                ui.collapsing("Available Decks", |ui| {
                    for deck_id in bridge.deck_registry.deck_ids() {
                        ui.label(deck_id);
                    }
                });
            }

            ui.add_space(20.0);

            // Instructions
            ui.group(|ui| {
                ui.label(egui::RichText::new("Controls").strong());
                ui.label("• Right-click + drag: Rotate camera");
                ui.label("• Scroll wheel: Zoom");
                ui.label("• G: Toggle Glassbox visualization");
                if menu_state.game_mode == 0 {
                    ui.label("• Click cards in hand to play them");
                }
            });
        });
    });
}

/// Draw the game over screen.
fn draw_game_over(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut bridge: Option<ResMut<GameBridge>>,
    game_mode: Res<GameModeConfig>,
    turn_state: Res<TurnState>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading(egui::RichText::new("Game Over").size(48.0));
            ui.add_space(20.0);

            if let Some(ref bridge) = bridge {
                if let Some(client) = &bridge.client {
                    if let Some(state) = client.get_state() {
                        if let Some(cardgame::state::GameResult::Win { winner, reason }) = &state.result {
                            let winner_name = if *winner == cardgame::types::PlayerId::PLAYER_ONE {
                                if game_mode.player1_human {
                                    "You Win!"
                                } else {
                                    "Player 1 Wins!"
                                }
                            } else {
                                if game_mode.player2_human {
                                    "You Win!"
                                } else if game_mode.player1_human {
                                    "AI Wins!"
                                } else {
                                    "Player 2 Wins!"
                                }
                            };
                            ui.label(egui::RichText::new(winner_name).size(32.0));

                            // Display win reason
                            let reason_text = match reason {
                                cardgame::state::WinReason::LifeReachedZero => "Opponent's life reached zero",
                                cardgame::state::WinReason::TurnLimitHigherLife => "Turn 30 reached - higher life wins",
                                cardgame::state::WinReason::VictoryPointsReached => "Victory points threshold reached",
                                cardgame::state::WinReason::Concession => "Opponent conceded",
                            };
                            ui.label(egui::RichText::new(reason_text).italics());
                        } else if let Some(cardgame::state::GameResult::Draw) = &state.result {
                            ui.label(egui::RichText::new("Draw!").size(32.0));
                            ui.label(egui::RichText::new("Both players reached zero life simultaneously").italics());
                        }

                        ui.add_space(15.0);

                        // Game statistics
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Game Statistics").strong());
                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.label(format!("Final Turn: {}", state.current_turn));
                                ui.separator();
                                ui.label(format!("Actions: {}", turn_state.actions_executed));
                                ui.separator();
                                let duration = turn_state.game_duration();
                                ui.label(format!("Duration: {:.1}s", duration.as_secs_f64()));
                            });

                            ui.horizontal(|ui| {
                                ui.label(format!("P1 Life: {}", state.players[0].life));
                                ui.separator();
                                ui.label(format!("P2 Life: {}", state.players[1].life));
                            });

                            // Actions per turn metric
                            if state.current_turn > 0 {
                                let actions_per_turn = turn_state.actions_executed as f64 / state.current_turn as f64;
                                ui.label(egui::RichText::new(
                                    format!("Avg {:.1} actions/turn", actions_per_turn)
                                ).small().weak());
                            }
                        });
                    }
                }
            }

            ui.add_space(30.0);

            ui.horizontal(|ui| {
                // Play Again button - restart with new seed
                if ui.button(egui::RichText::new("Play Again").size(20.0)).clicked() {
                    if let Some(ref mut bridge) = bridge {
                        let new_seed = bridge.current_seed.wrapping_add(1);
                        match bridge.restart_with_seed(new_seed) {
                            Ok(()) => {
                                info!("Restarting game with seed {}", new_seed);
                                next_state.set(AppState::Playing);
                            }
                            Err(e) => {
                                error!("Failed to restart game: {}", e);
                            }
                        }
                    }
                }

                ui.add_space(20.0);

                if ui.button(egui::RichText::new("Return to Menu").size(20.0)).clicked() {
                    next_state.set(AppState::Menu);
                }
            });
        });
    });
}

/// Handle game over in headless mode for multi-game benchmark runs.
fn handle_headless_game_over(
    cli_args: Option<Res<CliArgs>>,
    mut headless_stats: Option<ResMut<HeadlessStats>>,
    mut bridge: Option<ResMut<GameBridge>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    turn_state: Res<crate::game::turn_loop::TurnState>,
) {
    // Only run in headless mode with stats tracking
    let Some(args) = cli_args.as_ref() else {
        return;
    };
    if !args.headless {
        return;
    }

    let Some(ref mut stats) = headless_stats else {
        return;
    };
    let Some(ref mut bridge) = bridge else {
        return;
    };

    // Get game result
    let Some(client) = &bridge.client else {
        return;
    };

    let Some(state) = client.get_state() else {
        return;
    };

    // Determine winner
    let winner = match state.result {
        Some(cardgame::state::GameResult::Win { winner, .. }) => Some(winner),
        Some(cardgame::state::GameResult::Draw) => None,
        None => return, // Game not actually over
    };

    // Record this game's result
    let turns = state.current_turn as u32;
    let actions = turn_state.actions_executed;
    stats.record_game(winner, turns, actions);

    // Log progress for non-JSON mode
    if !args.json && args.games > 1 {
        let winner_str = match winner {
            Some(p) if p == cardgame::types::PlayerId::PLAYER_ONE => "P1",
            Some(p) if p == cardgame::types::PlayerId::PLAYER_TWO => "P2",
            Some(_) => "??",
            None => "Draw",
        };
        eprintln!(
            "[{}/{}] {} wins in {} turns ({} actions)",
            stats.games_played, stats.games_total, winner_str, turns, actions
        );
    }

    // Check if we need more games
    if !stats.is_complete() {
        // Start next game with incremented seed
        let new_seed = bridge.current_seed.wrapping_add(1);
        stats.start_game();

        match bridge.restart_with_seed(new_seed) {
            Ok(()) => {
                next_state.set(AppState::Playing);
            }
            Err(e) => {
                error!("Failed to restart game: {}", e);
                exit.send(AppExit::Error(1.try_into().unwrap()));
            }
        }
        return;
    }

    // All games complete - output results
    if args.json {
        let deck1 = bridge.current_deck1.as_deref().unwrap_or("unknown");
        let deck2 = bridge.current_deck2.as_deref().unwrap_or("unknown");
        let output = stats.to_json_output(deck1, deck2, args.seed);
        match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                error!("Failed to serialize JSON: {}", e);
                exit.send(AppExit::Error(1.try_into().unwrap()));
                return;
            }
        }
    } else {
        // Text summary
        let elapsed = stats.total_elapsed();
        eprintln!();
        eprintln!("=== Benchmark Complete ===");
        eprintln!("Games: {}", stats.games_played);
        eprintln!(
            "Results: P1 {} ({:.1}%) | P2 {} ({:.1}%) | Draw {}",
            stats.player1_wins,
            stats.player1_wins as f64 / stats.games_played.max(1) as f64 * 100.0,
            stats.player2_wins,
            stats.player2_wins as f64 / stats.games_played.max(1) as f64 * 100.0,
            stats.draws
        );
        eprintln!("Time: {:.2}s ({:.1} games/sec)", elapsed.as_secs_f64(), stats.games_per_second());
        eprintln!("Avg turns: {:.1}, Avg actions: {:.1}", stats.avg_turns(), stats.avg_actions());
    }

    // Exit successfully
    exit.send(AppExit::Success);
}
