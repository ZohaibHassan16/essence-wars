//! Main menu and game over screens.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};
use super::player_input::GameModeConfig;

/// Plugin for menus.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_main_menu.run_if(in_state(AppState::Menu)))
            .add_systems(Update, draw_game_over.run_if(in_state(AppState::GameOver)));
    }
}

/// Resource for menu state.
#[derive(Default)]
pub struct MenuState {
    pub deck1: String,
    pub deck2: String,
    pub seed: String,
    pub game_mode: usize,  // 0 = Human vs AI, 1 = AI vs AI
}

/// Draw the main menu.
fn draw_main_menu(
    mut contexts: EguiContexts,
    mut menu_state: Local<MenuState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut bridge: Option<ResMut<GameBridge>>,
    mut game_mode_config: ResMut<GameModeConfig>,
) {
    // Set defaults if empty (use MVP deck IDs from embedded_data)
    if menu_state.deck1.is_empty() {
        menu_state.deck1 = "colossus_wall".to_string(); // Iron Colossus Prime (Argentum)
    }
    if menu_state.deck2.is_empty() {
        menu_state.deck2 = "broodmother_swarm".to_string(); // The Broodmother (Symbiote)
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

                    match bridge.start_game(&menu_state.deck1, &menu_state.deck2, seed) {
                        Ok(()) => {
                            let mode_name = if menu_state.game_mode == 0 {
                                "Human vs AI"
                            } else {
                                "AI vs AI"
                            };
                            info!("Game started successfully ({})", mode_name);
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
    bridge: Option<Res<GameBridge>>,
    game_mode: Res<GameModeConfig>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading(egui::RichText::new("Game Over").size(48.0));
            ui.add_space(20.0);

            if let Some(bridge) = bridge {
                if let Some(client) = &bridge.client {
                    if let Some(state) = client.get_state() {
                        if let Some(cardgame::state::GameResult::Win { winner, .. }) = state.result {
                            let winner_name = if winner == cardgame::types::PlayerId::PLAYER_ONE {
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
                        } else if let Some(cardgame::state::GameResult::Draw) = state.result {
                            ui.label(egui::RichText::new("Draw!").size(32.0));
                        }
                        ui.label(format!("Final Turn: {}", state.current_turn));
                        ui.label(format!("P1 Life: {} | P2 Life: {}",
                            state.players[0].life,
                            state.players[1].life
                        ));
                    }
                }
            }

            ui.add_space(30.0);

            if ui.button(egui::RichText::new("Return to Menu").size(20.0)).clicked() {
                next_state.set(AppState::Menu);
            }
        });
    });
}
