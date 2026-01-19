//! Hand display showing player's cards.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};

/// Plugin for the hand display.
pub struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_hand.run_if(in_state(AppState::Playing)));
    }
}

/// Draw the player's hand using egui.
fn draw_hand(
    mut contexts: EguiContexts,
    bridge: Option<Res<GameBridge>>,
) {
    let Some(bridge) = bridge else { return };
    let Some(client) = &bridge.client else { return };

    let state = client.game_state();
    let card_db = &bridge.card_db;

    // Show Player 1's hand at the bottom
    egui::TopBottomPanel::bottom("player_hand").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Your Hand:").strong());
            ui.separator();

            for card_id in &state.players[0].hand {
                if let Some(card) = card_db.get(*card_id) {
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
