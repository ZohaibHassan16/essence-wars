//! Spectator hand display showing the current player's cards.
//!
//! This module displays a read-only view of the active player's hand during
//! AI vs AI (spectator) mode. When it's a human player's turn, the interactive
//! hand display from `player_input.rs` takes over instead.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use cardgame::types::PlayerId;

use crate::game::{AppState, GameBridge};
use super::player_input::is_human_turn;

/// Plugin for the spectator hand display.
pub struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        // Only show spectator hand when it's NOT a human's turn
        // (i.e., during AI vs AI spectator mode, or opponent's turn in Human vs AI)
        app.add_systems(
            Update,
            draw_spectator_hand
                .run_if(in_state(AppState::Playing))
                .run_if(not(is_human_turn)),
        );
    }
}

/// Draw the current player's hand (read-only spectator view).
fn draw_spectator_hand(
    mut contexts: EguiContexts,
    bridge: Option<Res<GameBridge>>,
) {
    let Some(bridge) = bridge else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };
    let card_db = &bridge.card_db;

    // Show the CURRENT (active) player's hand for spectating
    let current_player = state.active_player;
    let player_idx = if current_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let player = &state.players[player_idx];
    let player_name = if current_player == PlayerId::PLAYER_ONE { "Player 1" } else { "Player 2" };

    egui::TopBottomPanel::bottom("spectator_hand").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{}'s Hand:", player_name)).strong());
            ui.label(format!(
                "AP: {}/3 | Essence: {}/{}",
                player.action_points, player.current_essence, player.max_essence
            ));
            ui.separator();

            for card_instance in &player.hand {
                if let Some(card) = card_db.get(card_instance.card_id) {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&card.name).strong());
                            ui.label(format!("Cost: {}", card.cost));
                            match &card.card_type {
                                cardgame::cards::CardType::Creature { attack, health, keywords, .. } => {
                                    ui.label(format!("{}/{}", attack, health));
                                    if !keywords.is_empty() {
                                        ui.label(keywords.join(", "));
                                    }
                                }
                                cardgame::cards::CardType::Spell { .. } => {
                                    ui.label("Spell");
                                }
                                cardgame::cards::CardType::Support { durability, .. } => {
                                    ui.label(format!("Support ({})", durability));
                                }
                            }
                        });
                    });
                }
            }
        });
    });
}
