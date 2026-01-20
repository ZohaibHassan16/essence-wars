//! Human player input handling.
//!
//! This module handles card selection and slot targeting for human players.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use cardgame::actions::Action;
use cardgame::effects::TargetingRule;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameBridge};

/// Plugin for human player input.
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputState>()
            .init_resource::<GameModeConfig>()
            .add_systems(
                Update,
                (draw_player_hand, draw_slot_selection, handle_end_turn_button)
                    .chain()
                    .run_if(in_state(AppState::Playing))
                    .run_if(is_human_turn),
            );
    }
}

/// Resource tracking player's current input state.
#[derive(Resource, Default)]
pub struct PlayerInputState {
    /// Currently selected card index in hand (if any)
    pub selected_card: Option<usize>,
    /// Pending action to apply (set by UI, consumed by turn loop)
    pub pending_action: Option<Action>,
}

/// Configuration for game mode (who is human, who is AI).
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct GameModeConfig {
    /// Is Player 1 controlled by human?
    pub player1_human: bool,
    /// Is Player 2 controlled by human?
    pub player2_human: bool,
}

impl Default for GameModeConfig {
    fn default() -> Self {
        Self {
            player1_human: true,  // Player 1 is human by default
            player2_human: false, // Player 2 is AI by default
        }
    }
}

impl GameModeConfig {
    /// AI vs AI mode (spectator)
    pub fn spectator() -> Self {
        Self {
            player1_human: false,
            player2_human: false,
        }
    }

    /// Human vs AI mode
    pub fn human_vs_ai() -> Self {
        Self {
            player1_human: true,
            player2_human: false,
        }
    }

    /// Check if the given player is human-controlled.
    pub fn is_human(&self, player: PlayerId) -> bool {
        if player == PlayerId::PLAYER_ONE {
            self.player1_human
        } else {
            self.player2_human
        }
    }
}

/// Run condition: returns true if it's currently a human player's turn.
/// Exported for use by other UI modules (e.g., spectator hand display).
pub fn is_human_turn(
    bridge: Res<GameBridge>,
    game_mode: Res<GameModeConfig>,
) -> bool {
    let Some(client) = &bridge.client else {
        return false;
    };

    if client.is_game_over() {
        return false;
    }

    let Some(current_player) = client.current_player() else {
        return false;
    };

    game_mode.is_human(current_player)
}

/// Draw the player's hand with clickable cards.
fn draw_player_hand(
    mut contexts: EguiContexts,
    bridge: Res<GameBridge>,
    mut input_state: ResMut<PlayerInputState>,
) {
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };
    let card_db = &bridge.card_db;

    // Get current player's hand
    let current_player = state.active_player;
    let player_idx = if current_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let player = &state.players[player_idx];

    egui::TopBottomPanel::bottom("human_hand")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Your Hand:").strong());
                ui.label(format!(
                    "AP: {}/{} | Essence: {}/{}",
                    player.action_points, 3,
                    player.current_essence, player.max_essence
                ));
                ui.separator();

                for (idx, card_instance) in player.hand.iter().enumerate() {
                    if let Some(card) = card_db.get(card_instance.card_id) {
                        let is_selected = input_state.selected_card == Some(idx);
                        let can_afford = card.cost as u8 <= player.current_essence;
                        let has_ap = player.action_points > 0;
                        let playable = can_afford && has_ap;

                        // Create button style based on state
                        let button_color = if is_selected {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else if playable {
                            egui::Color32::from_rgb(60, 60, 80)
                        } else {
                            egui::Color32::from_rgb(40, 40, 50)
                        };

                        // Set visual style for this card
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = button_color;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill =
                            egui::Color32::from_rgb(80, 80, 100);

                        let group_response = ui.group(|ui| {
                            ui.set_min_width(100.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&card.name).strong());
                                ui.label(format!("Cost: {}", card.cost));
                                match &card.card_type {
                                    cardgame::cards::CardType::Creature { attack, health, keywords, .. } => {
                                        ui.label(format!("{}/{}", attack, health));
                                        if !keywords.is_empty() {
                                            ui.label(
                                                egui::RichText::new(keywords.join(", "))
                                                    .small()
                                            );
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

                        // Handle click on the group
                        if group_response.response.interact(egui::Sense::click()).clicked() && playable {
                            if is_selected {
                                // Deselect
                                input_state.selected_card = None;
                            } else {
                                // Select
                                input_state.selected_card = Some(idx);
                            }
                        }
                    }
                }
            });
        });
}

/// Draw slot selection overlay when a card is selected.
fn draw_slot_selection(
    mut contexts: EguiContexts,
    bridge: Res<GameBridge>,
    mut input_state: ResMut<PlayerInputState>,
) {
    let Some(selected_idx) = input_state.selected_card else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };
    let card_db = &bridge.card_db;

    let current_player = state.active_player;
    let player_idx = if current_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let player = &state.players[player_idx];

    // Get the selected card
    let Some(card_instance) = player.hand.get(selected_idx) else {
        input_state.selected_card = None;
        return;
    };

    let Some(card) = card_db.get(card_instance.card_id) else { return };

    // Check what kind of targeting this card needs
    let is_creature = matches!(card.card_type, cardgame::cards::CardType::Creature { .. });
    let is_support = matches!(card.card_type, cardgame::cards::CardType::Support { .. });
    let is_spell = matches!(card.card_type, cardgame::cards::CardType::Spell { .. });

    // Show slot selection panel
    egui::Window::new("Select Target")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -50.0])
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Playing: {}", card.name));
            ui.separator();

            if is_creature {
                ui.label("Select a slot for your creature:");
                ui.horizontal(|ui| {
                    for slot in 0..5 {
                        // Check if slot is empty
                        let is_empty = player.creatures.iter().all(|c| c.slot.0 != slot as u8);

                        let button_text = if is_empty {
                            format!("Slot {}", slot + 1)
                        } else {
                            format!("[Occupied]")
                        };

                        if ui
                            .add_enabled(is_empty, egui::Button::new(button_text))
                            .clicked()
                        {
                            // Create PlayCard action
                            let action = Action::PlayCard {
                                hand_index: selected_idx as u8,
                                slot: cardgame::types::Slot(slot as u8),
                            };
                            input_state.pending_action = Some(action);
                            input_state.selected_card = None;
                        }
                    }
                });
            } else if is_support {
                ui.label("Select a support slot:");
                ui.horizontal(|ui| {
                    for slot in 0..2 {
                        let is_empty = player.supports.iter().all(|s| s.slot.0 != slot as u8);
                        let button_text = if is_empty {
                            format!("Support {}", slot + 1)
                        } else {
                            "[Occupied]".to_string()
                        };

                        if ui
                            .add_enabled(is_empty, egui::Button::new(button_text))
                            .clicked()
                        {
                            let action = Action::PlayCard {
                                hand_index: selected_idx as u8,
                                slot: cardgame::types::Slot(slot as u8),
                            };
                            input_state.pending_action = Some(action);
                            input_state.selected_card = None;
                        }
                    }
                });
            } else if is_spell {
                // Get targeting rule for this spell
                let targeting = card.spell_targeting().cloned().unwrap_or_default();
                let opponent_idx = if current_player == PlayerId::PLAYER_ONE { 1 } else { 0 };
                let opponent = &state.players[opponent_idx];

                match &targeting {
                    TargetingRule::NoTarget => {
                        // No target needed - just cast
                        ui.label("Cast this spell?");
                        if ui.button("Cast").clicked() {
                            let action = Action::PlayCard {
                                hand_index: selected_idx as u8,
                                slot: cardgame::types::Slot(0),
                            };
                            input_state.pending_action = Some(action);
                            input_state.selected_card = None;
                        }
                    }

                    TargetingRule::TargetEnemyCreature => {
                        ui.label("Select an enemy creature:");
                        ui.horizontal(|ui| {
                            for slot in 0..5u8 {
                                // Check if enemy has creature in this slot
                                let creature = opponent.creatures.iter().find(|c| c.slot.0 == slot);
                                if let Some(c) = creature {
                                    let name = card_db.get(c.card_id)
                                        .map(|cd| cd.name.as_str())
                                        .unwrap_or("???");
                                    if ui.button(format!("{}\n({}/{})", name, c.attack, c.current_health)).clicked() {
                                        let action = Action::PlayCard {
                                            hand_index: selected_idx as u8,
                                            slot: cardgame::types::Slot(slot), // 0-4 = enemy
                                        };
                                        input_state.pending_action = Some(action);
                                        input_state.selected_card = None;
                                    }
                                }
                            }
                        });
                        if opponent.creatures.is_empty() {
                            ui.label(egui::RichText::new("No enemy creatures to target").weak());
                        }
                    }

                    TargetingRule::TargetAllyCreature => {
                        ui.label("Select one of your creatures:");
                        ui.horizontal(|ui| {
                            for slot in 0..5u8 {
                                let creature = player.creatures.iter().find(|c| c.slot.0 == slot);
                                if let Some(c) = creature {
                                    let name = card_db.get(c.card_id)
                                        .map(|cd| cd.name.as_str())
                                        .unwrap_or("???");
                                    if ui.button(format!("{}\n({}/{})", name, c.attack, c.current_health)).clicked() {
                                        let action = Action::PlayCard {
                                            hand_index: selected_idx as u8,
                                            slot: cardgame::types::Slot(slot), // 0-4 = ally
                                        };
                                        input_state.pending_action = Some(action);
                                        input_state.selected_card = None;
                                    }
                                }
                            }
                        });
                        if player.creatures.is_empty() {
                            ui.label(egui::RichText::new("No friendly creatures to target").weak());
                        }
                    }

                    TargetingRule::TargetCreature(_) => {
                        // Can target any creature
                        ui.label("Select a creature (enemy or friendly):");

                        if !opponent.creatures.is_empty() {
                            ui.label(egui::RichText::new("Enemy:").small());
                            ui.horizontal(|ui| {
                                for slot in 0..5u8 {
                                    let creature = opponent.creatures.iter().find(|c| c.slot.0 == slot);
                                    if let Some(c) = creature {
                                        let name = card_db.get(c.card_id)
                                            .map(|cd| cd.name.as_str())
                                            .unwrap_or("???");
                                        if ui.button(format!("{}\n({}/{})", name, c.attack, c.current_health)).clicked() {
                                            let action = Action::PlayCard {
                                                hand_index: selected_idx as u8,
                                                slot: cardgame::types::Slot(slot), // 0-4 = enemy
                                            };
                                            input_state.pending_action = Some(action);
                                            input_state.selected_card = None;
                                        }
                                    }
                                }
                            });
                        }

                        if !player.creatures.is_empty() {
                            ui.label(egui::RichText::new("Friendly:").small());
                            ui.horizontal(|ui| {
                                for slot in 0..5u8 {
                                    let creature = player.creatures.iter().find(|c| c.slot.0 == slot);
                                    if let Some(c) = creature {
                                        let name = card_db.get(c.card_id)
                                            .map(|cd| cd.name.as_str())
                                            .unwrap_or("???");
                                        if ui.button(format!("{}\n({}/{})", name, c.attack, c.current_health)).clicked() {
                                            let action = Action::PlayCard {
                                                hand_index: selected_idx as u8,
                                                slot: cardgame::types::Slot(slot + 5), // 5-9 = ally
                                            };
                                            input_state.pending_action = Some(action);
                                            input_state.selected_card = None;
                                        }
                                    }
                                }
                            });
                        }

                        if opponent.creatures.is_empty() && player.creatures.is_empty() {
                            ui.label(egui::RichText::new("No creatures to target").weak());
                        }
                    }

                    TargetingRule::TargetEnemyPlayer => {
                        // Target enemy player (slot doesn't matter)
                        ui.label("Target the enemy player?");
                        if ui.button(format!("Target Enemy (Life: {})", opponent.life)).clicked() {
                            let action = Action::PlayCard {
                                hand_index: selected_idx as u8,
                                slot: cardgame::types::Slot(0),
                            };
                            input_state.pending_action = Some(action);
                            input_state.selected_card = None;
                        }
                    }

                    TargetingRule::TargetPlayer => {
                        ui.label("Select a player:");
                        ui.horizontal(|ui| {
                            if ui.button(format!("Enemy (Life: {})", opponent.life)).clicked() {
                                let action = Action::PlayCard {
                                    hand_index: selected_idx as u8,
                                    slot: cardgame::types::Slot(0), // 0 = enemy
                                };
                                input_state.pending_action = Some(action);
                                input_state.selected_card = None;
                            }
                            if ui.button(format!("Self (Life: {})", player.life)).clicked() {
                                let action = Action::PlayCard {
                                    hand_index: selected_idx as u8,
                                    slot: cardgame::types::Slot(1), // 1 = self
                                };
                                input_state.pending_action = Some(action);
                                input_state.selected_card = None;
                            }
                        });
                    }

                    TargetingRule::TargetAny => {
                        // Can target any creature or player
                        ui.label("Select any target:");

                        // Enemy creatures (slots 0-4)
                        if !opponent.creatures.is_empty() {
                            ui.label(egui::RichText::new("Enemy Creatures:").small());
                            ui.horizontal(|ui| {
                                for slot in 0..5u8 {
                                    let creature = opponent.creatures.iter().find(|c| c.slot.0 == slot);
                                    if let Some(c) = creature {
                                        let name = card_db.get(c.card_id)
                                            .map(|cd| cd.name.as_str())
                                            .unwrap_or("???");
                                        if ui.button(format!("{}\n({}/{})", name, c.attack, c.current_health)).clicked() {
                                            let action = Action::PlayCard {
                                                hand_index: selected_idx as u8,
                                                slot: cardgame::types::Slot(slot),
                                            };
                                            input_state.pending_action = Some(action);
                                            input_state.selected_card = None;
                                        }
                                    }
                                }
                            });
                        }

                        // Ally creatures (slots 5-9)
                        if !player.creatures.is_empty() {
                            ui.label(egui::RichText::new("Friendly Creatures:").small());
                            ui.horizontal(|ui| {
                                for slot in 0..5u8 {
                                    let creature = player.creatures.iter().find(|c| c.slot.0 == slot);
                                    if let Some(c) = creature {
                                        let name = card_db.get(c.card_id)
                                            .map(|cd| cd.name.as_str())
                                            .unwrap_or("???");
                                        if ui.button(format!("{}\n({}/{})", name, c.attack, c.current_health)).clicked() {
                                            let action = Action::PlayCard {
                                                hand_index: selected_idx as u8,
                                                slot: cardgame::types::Slot(slot + 5),
                                            };
                                            input_state.pending_action = Some(action);
                                            input_state.selected_card = None;
                                        }
                                    }
                                }
                            });
                        }

                        // Players (slots 10-11)
                        ui.label(egui::RichText::new("Players:").small());
                        ui.horizontal(|ui| {
                            if ui.button(format!("Enemy (Life: {})", opponent.life)).clicked() {
                                let action = Action::PlayCard {
                                    hand_index: selected_idx as u8,
                                    slot: cardgame::types::Slot(10), // enemy player
                                };
                                input_state.pending_action = Some(action);
                                input_state.selected_card = None;
                            }
                            if ui.button(format!("Self (Life: {})", player.life)).clicked() {
                                let action = Action::PlayCard {
                                    hand_index: selected_idx as u8,
                                    slot: cardgame::types::Slot(11), // self
                                };
                                input_state.pending_action = Some(action);
                                input_state.selected_card = None;
                            }
                        });
                    }

                    TargetingRule::TargetSlot => {
                        // Target an empty friendly slot (for summon effects)
                        ui.label("Select an empty slot:");
                        ui.horizontal(|ui| {
                            for slot in 0..5u8 {
                                let is_empty = player.creatures.iter().all(|c| c.slot.0 != slot);
                                if is_empty {
                                    if ui.button(format!("Slot {}", slot + 1)).clicked() {
                                        let action = Action::PlayCard {
                                            hand_index: selected_idx as u8,
                                            slot: cardgame::types::Slot(slot),
                                        };
                                        input_state.pending_action = Some(action);
                                        input_state.selected_card = None;
                                    }
                                }
                            }
                        });
                        let empty_count = (0..5u8).filter(|&s| player.creatures.iter().all(|c| c.slot.0 != s)).count();
                        if empty_count == 0 {
                            ui.label(egui::RichText::new("No empty slots available").weak());
                        }
                    }
                }
            }

            ui.separator();
            if ui.button("Cancel").clicked() {
                input_state.selected_card = None;
            }
        });
}

/// Draw end turn button.
fn handle_end_turn_button(
    mut contexts: EguiContexts,
    mut input_state: ResMut<PlayerInputState>,
    bridge: Res<GameBridge>,
) {
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    let current_player = state.active_player;
    let player_idx = if current_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let player = &state.players[player_idx];

    egui::TopBottomPanel::top("turn_controls").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Turn {} - Your Turn",
                state.current_turn
            ));
            ui.label(format!(
                "Life: {} | AP: {}",
                player.life, player.action_points
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("End Turn").size(16.0)).clicked() {
                    input_state.pending_action = Some(Action::EndTurn);
                }
            });
        });
    });
}
