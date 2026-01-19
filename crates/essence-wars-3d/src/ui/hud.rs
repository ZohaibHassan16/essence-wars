//! HUD (Heads-Up Display) for game information.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};

/// Plugin for the game HUD.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_hud.run_if(in_state(AppState::Playing)));
    }
}

/// Draw the game HUD using egui.
fn draw_hud(
    mut contexts: EguiContexts,
    bridge: Option<Res<GameBridge>>,
) {
    let Some(bridge) = bridge else { return };
    let Some(client) = &bridge.client else { return };

    let state = client.game_state();

    egui::TopBottomPanel::top("game_hud").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            // Player 1 info
            ui.group(|ui| {
                ui.label(egui::RichText::new("Player 1").strong());
                ui.label(format!("Life: {}", state.players[0].life));
                ui.label(format!("Essence: {}/{}",
                    state.players[0].current_essence,
                    state.players[0].max_essence
                ));
                ui.label(format!("Hand: {} cards", state.players[0].hand.len()));
            });

            ui.separator();

            // Turn info
            ui.group(|ui| {
                ui.label(format!("Turn {}", state.current_turn));
                let current_player = if state.active_player == cardgame::types::PlayerId::Player1 {
                    "Player 1"
                } else {
                    "Player 2"
                };
                ui.label(format!("Current: {}", current_player));
            });

            ui.separator();

            // Player 2 info
            ui.group(|ui| {
                ui.label(egui::RichText::new("Player 2").strong());
                ui.label(format!("Life: {}", state.players[1].life));
                ui.label(format!("Essence: {}/{}",
                    state.players[1].current_essence,
                    state.players[1].max_essence
                ));
                ui.label(format!("Hand: {} cards", state.players[1].hand.len()));
            });
        });
    });
}
