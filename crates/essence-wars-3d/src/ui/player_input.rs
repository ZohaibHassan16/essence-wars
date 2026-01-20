//! Human player input handling.
//!
//! This module handles card selection and slot targeting for human players.
//! Supports both click-to-select and drag-and-drop for playing cards.
//! Drop zones are rendered at 3D world positions for intuitive board interaction.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use cardgame::actions::Action;
use cardgame::cards::CardType;
use cardgame::effects::TargetingRule;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameBridge};
use crate::rendering::GameCamera;

/// Plugin for human player input.
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputState>()
            .init_resource::<GameModeConfig>()
            .add_systems(
                Update,
                (
                    draw_player_hand,
                    draw_drop_zones,
                    draw_slot_selection,
                    handle_end_turn_button,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing))
                    .run_if(is_human_turn),
            );
    }
}

/// Payload for drag-and-drop card playing.
#[derive(Clone, Copy, Debug)]
struct CardDragPayload {
    /// Index in hand
    hand_index: usize,
    /// What type of slot this card needs
    slot_type: CardSlotType,
}

/// Type of slot a card can be played to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardSlotType {
    Creature,
    Support,
    Spell, // Spells use click-to-select, not drag-and-drop
}

/// Resource tracking player's current input state.
#[derive(Resource, Default)]
pub struct PlayerInputState {
    /// Currently selected card index in hand (for spell targeting modal)
    pub selected_card: Option<usize>,
    /// Pending action to apply (set by UI, consumed by turn loop)
    pub pending_action: Option<Action>,
    /// Whether a drag is currently in progress
    pub dragging: bool,
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

/// Draw the player's hand with draggable cards.
/// - Creatures and Supports: Drag to slot to play
/// - Spells: Click to open targeting modal
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
                ui.label(egui::RichText::new("(Drag cards to slots)").small().weak());
                ui.separator();

                for (idx, card_instance) in player.hand.iter().enumerate() {
                    if let Some(card) = card_db.get(card_instance.card_id) {
                        let is_selected = input_state.selected_card == Some(idx);
                        let can_afford = card.cost as u8 <= player.current_essence;
                        let has_ap = player.action_points > 0;
                        let playable = can_afford && has_ap;

                        // Determine card slot type
                        let slot_type = match &card.card_type {
                            CardType::Creature { .. } => CardSlotType::Creature,
                            CardType::Support { .. } => CardSlotType::Support,
                            CardType::Spell { .. } => CardSlotType::Spell,
                        };

                        // Create button style based on state
                        let button_color = if is_selected {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else if playable {
                            egui::Color32::from_rgb(60, 60, 80)
                        } else {
                            egui::Color32::from_rgb(40, 40, 50)
                        };

                        // Render card content
                        let card_ui = |ui: &mut egui::Ui| {
                            ui.set_min_width(100.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&card.name).strong());
                                ui.label(format!("Cost: {}", card.cost));
                                match &card.card_type {
                                    CardType::Creature { attack, health, keywords, .. } => {
                                        ui.label(format!("{}/{}", attack, health));
                                        if !keywords.is_empty() {
                                            ui.label(
                                                egui::RichText::new(keywords.join(", "))
                                                    .small()
                                            );
                                        }
                                    }
                                    CardType::Spell { .. } => {
                                        ui.label("Spell");
                                    }
                                    CardType::Support { durability, .. } => {
                                        ui.label(format!("Support ({})", durability));
                                    }
                                }
                            });
                        };

                        // Set visual style
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = button_color;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill =
                            egui::Color32::from_rgb(80, 80, 100);

                        // For creatures and supports: make draggable
                        // For spells: click to select for targeting modal
                        if playable && slot_type != CardSlotType::Spell {
                            // Drag source for creatures and supports
                            let item_id = egui::Id::new(("hand_card", idx));
                            let payload = CardDragPayload { hand_index: idx, slot_type };

                            ui.dnd_drag_source(item_id, payload, |ui| {
                                ui.group(card_ui);
                            });
                        } else {
                            // Regular clickable group for spells (and unplayable cards)
                            let group_response = ui.group(card_ui);

                            // Handle click for spells
                            if slot_type == CardSlotType::Spell && playable {
                                if group_response.response.interact(egui::Sense::click()).clicked() {
                                    if is_selected {
                                        input_state.selected_card = None;
                                    } else {
                                        input_state.selected_card = Some(idx);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        });
}

/// Draw drop zones at the 3D world positions of creature and support slots.
/// This allows intuitive drag-and-drop directly onto the board.
fn draw_drop_zones(
    mut contexts: EguiContexts,
    bridge: Res<GameBridge>,
    mut input_state: ResMut<PlayerInputState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };

    let current_player = state.active_player;
    let player_idx = if current_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let player = &state.players[player_idx];

    // Check if we're dragging something
    let ctx = contexts.ctx_mut();
    let drag_payload = egui::DragAndDrop::payload::<CardDragPayload>(ctx);
    let is_dragging_creature = drag_payload
        .as_ref()
        .map(|p| p.slot_type == CardSlotType::Creature)
        .unwrap_or(false);
    let is_dragging_support = drag_payload
        .as_ref()
        .map(|p| p.slot_type == CardSlotType::Support)
        .unwrap_or(false);

    input_state.dragging = is_dragging_creature || is_dragging_support;

    // Only show drop zones when dragging
    if !input_state.dragging {
        return;
    }

    // Draw creature drop zones at 3D slot positions
    if is_dragging_creature {
        // Player 1's row is at Z=2.0, Player 2's at Z=-2.0
        let z = if current_player == PlayerId::PLAYER_ONE { 2.0 } else { -2.0 };

        for slot in 0..5u8 {
            let is_empty = player.creatures.iter().all(|c| c.slot.0 != slot);
            let x = (slot as f32 - 2.0) * 2.0;
            let world_pos = Vec3::new(x, 0.5, z);

            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                draw_world_drop_zone(
                    ctx,
                    &mut input_state,
                    screen_pos,
                    slot,
                    is_empty,
                    false, // is_support
                );
            }
        }
    }

    // Draw support drop zones at 3D slot positions
    if is_dragging_support {
        // Player 1's supports at X=-5.5, Player 2's at X=5.5
        let (x, z_base) = if current_player == PlayerId::PLAYER_ONE {
            (-5.5, 1.0)
        } else {
            (5.5, -1.0)
        };

        for slot in 0..2u8 {
            let is_empty = player.supports.iter().all(|s| s.slot.0 != slot);
            let z = if current_player == PlayerId::PLAYER_ONE {
                z_base + slot as f32 * 2.0
            } else {
                z_base - slot as f32 * 2.0
            };
            let world_pos = Vec3::new(x, 0.5, z);

            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                draw_world_drop_zone(
                    ctx,
                    &mut input_state,
                    screen_pos,
                    slot,
                    is_empty,
                    true, // is_support
                );
            }
        }
    }

    // Show instruction text at top of screen
    egui::Area::new(egui::Id::new("drop_instruction"))
        .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                .inner_margin(10.0)
                .rounding(8.0)
                .show(ui, |ui| {
                    let msg = if is_dragging_creature {
                        "Drop on a creature slot (green = empty)"
                    } else {
                        "Drop on a support slot (green = empty)"
                    };
                    ui.label(egui::RichText::new(msg).size(16.0).color(egui::Color32::WHITE));
                });
        });
}

/// Draw a single drop zone at a screen position (projected from 3D world).
fn draw_world_drop_zone(
    ctx: &egui::Context,
    input_state: &mut ResMut<PlayerInputState>,
    screen_pos: Vec2,
    slot: u8,
    is_empty: bool,
    is_support: bool,
) {
    let zone_size = if is_support { 70.0 } else { 90.0 };
    let half_size = zone_size / 2.0;

    let (bg_color, border_color) = if is_empty {
        (
            egui::Color32::from_rgba_unmultiplied(40, 120, 40, 180),
            egui::Color32::from_rgb(100, 255, 100),
        )
    } else {
        (
            egui::Color32::from_rgba_unmultiplied(80, 40, 40, 150),
            egui::Color32::from_rgb(150, 80, 80),
        )
    };

    let id_prefix = if is_support { "support_drop" } else { "creature_drop" };
    let zone_id = egui::Id::new((id_prefix, slot));

    egui::Area::new(zone_id)
        .fixed_pos(egui::pos2(screen_pos.x - half_size, screen_pos.y - half_size))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let frame = egui::Frame::none()
                .fill(bg_color)
                .stroke(egui::Stroke::new(3.0, border_color))
                .rounding(8.0)
                .inner_margin(8.0);

            let (_, dropped) = ui.dnd_drop_zone::<CardDragPayload, ()>(frame, |ui| {
                ui.set_min_size(egui::vec2(zone_size - 16.0, zone_size - 16.0));
                ui.centered_and_justified(|ui| {
                    let label = if is_support {
                        format!("S{}", slot + 1)
                    } else {
                        format!("{}", slot + 1)
                    };
                    let text = if is_empty {
                        egui::RichText::new(label).size(20.0).strong().color(egui::Color32::WHITE)
                    } else {
                        egui::RichText::new("X").size(18.0).color(egui::Color32::from_rgb(180, 80, 80))
                    };
                    ui.label(text);
                });
            });

            // Handle drop
            if let Some(payload) = dropped {
                if is_empty {
                    let action = Action::PlayCard {
                        hand_index: payload.hand_index as u8,
                        slot: cardgame::types::Slot(slot),
                    };
                    input_state.pending_action = Some(action);
                }
            }
        });
}

/// Draw spell targeting overlay when a spell is selected.
/// Note: Creatures and supports now use drag-and-drop, only spells use this modal.
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

    // Only show modal for spells - creatures and supports use drag-and-drop
    let is_spell = matches!(card.card_type, CardType::Spell { .. });
    if !is_spell {
        // Deselect non-spell cards (they should use drag-and-drop)
        input_state.selected_card = None;
        return;
    }

    // Show spell targeting panel
    egui::Window::new("Select Target")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -50.0])
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Casting: {}", card.name));
            ui.separator();

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
