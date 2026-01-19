//! Main menu and game over screens.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};

/// Plugin for menus.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_main_menu.run_if(in_state(AppState::Menu)))
            .add_systems(Update, draw_game_over.run_if(in_state(AppState::GameOver)));
    }
}

/// Resource for menu state.
#[derive(Resource, Default)]
pub struct MenuState {
    pub deck1: String,
    pub deck2: String,
    pub seed: String,
}

/// Draw the main menu.
fn draw_main_menu(
    mut contexts: EguiContexts,
    mut menu_state: Local<MenuState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut bridge: Option<ResMut<GameBridge>>,
) {
    // Set defaults if empty
    if menu_state.deck1.is_empty() {
        menu_state.deck1 = "argentum_control".to_string();
    }
    if menu_state.deck2.is_empty() {
        menu_state.deck2 = "symbiote_aggro".to_string();
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
                ui.label("Game Setup");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Player 1 Deck:");
                    ui.text_edit_singleline(&mut menu_state.deck1);
                });

                ui.horizontal(|ui| {
                    ui.label("Player 2 Deck:");
                    ui.text_edit_singleline(&mut menu_state.deck2);
                });

                ui.horizontal(|ui| {
                    ui.label("Random Seed:");
                    ui.text_edit_singleline(&mut menu_state.seed);
                });
            });

            ui.add_space(20.0);

            if ui.button(egui::RichText::new("Start Game").size(24.0)).clicked() {
                if let Some(ref mut bridge) = bridge {
                    let seed: u64 = menu_state.seed.parse().unwrap_or(42);
                    match bridge.start_game(&menu_state.deck1, &menu_state.deck2, seed) {
                        Ok(()) => {
                            info!("Game started successfully");
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
                    for deck_id in bridge.deck_registry.list() {
                        ui.label(deck_id);
                    }
                });
            }
        });
    });
}

/// Draw the game over screen.
fn draw_game_over(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    bridge: Option<Res<GameBridge>>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading(egui::RichText::new("Game Over").size(48.0));
            ui.add_space(20.0);

            if let Some(bridge) = bridge {
                if let Some(client) = &bridge.client {
                    let state = client.game_state();
                    if let Some(winner) = state.winner {
                        let winner_name = if winner == cardgame::types::PlayerId::Player1 {
                            "Player 1"
                        } else {
                            "Player 2"
                        };
                        ui.label(egui::RichText::new(format!("{} Wins!", winner_name)).size(32.0));
                    }
                    ui.label(format!("Final Turn: {}", state.current_turn));
                    ui.label(format!("P1 Life: {} | P2 Life: {}",
                        state.players[0].life,
                        state.players[1].life
                    ));
                }
            }

            ui.add_space(30.0);

            if ui.button(egui::RichText::new("Return to Menu").size(20.0)).clicked() {
                next_state.set(AppState::Menu);
            }
        });
    });
}
