//! Combat visualization and effects.
//!
//! This module handles visual effects for combat, including:
//! - Attack animations (creature movement)
//! - Damage number displays
//! - Visual feedback for combat resolution
//! - Lane-grouped combat (processes combats sequentially by lane with delays)

use std::collections::VecDeque;

use bevy::prelude::*;
use cardgame::client_api::GameEvent;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameEventWrapper};
use super::creatures::Creature3D;

/// Plugin for combat visualization.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatState>()
            .add_systems(
                Update,
                (
                    queue_combat_events,
                    process_combat_queue,
                    process_other_combat_events,
                    animate_attacks,
                    animate_damage_numbers,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), cleanup_combat_effects);
    }
}

/// A queued combat event with lane information.
#[derive(Clone)]
pub struct QueuedCombat {
    /// Lane index (0-4) - used for grouping
    pub lane: u8,
    /// The original combat event data
    pub attacker_player: PlayerId,
    pub attacker_slot: u8,
    pub defender_player: PlayerId,
    pub defender_slot: u8,
}

/// Resource tracking current combat state.
#[derive(Resource, Default)]
pub struct CombatState {
    /// Active attack animation (if any)
    pub active_attack: Option<AttackAnimation>,
    /// Queue of pending combat events (processed sequentially)
    pub combat_queue: VecDeque<QueuedCombat>,
    /// Delay timer between lane combats
    pub lane_delay_timer: Option<Timer>,
    /// Last processed lane (for grouping visualization)
    pub last_lane: Option<u8>,
}

/// Data for an active attack animation.
pub struct AttackAnimation {
    /// Entity of the attacking creature
    pub attacker_entity: Entity,
    /// Original position of attacker
    pub attacker_origin: Vec3,
    /// Position of defender (attack target)
    pub defender_pos: Vec3,
    /// Animation progress (0.0 to 1.0)
    pub progress: f32,
    /// Attack phase (forward or return)
    pub phase: AttackPhase,
}

/// Phase of attack animation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttackPhase {
    /// Moving toward target
    Forward,
    /// Returning to origin
    Return,
}

/// Component for floating damage numbers.
#[derive(Component)]
pub struct DamageNumber {
    /// Damage amount to display
    pub amount: u8,
    /// Time alive
    pub lifetime: f32,
    /// Starting Y position
    pub start_y: f32,
}

/// Queue combat events for sequential processing (lane-by-lane visualization).
fn queue_combat_events(
    mut event_reader: EventReader<GameEventWrapper>,
    mut combat_state: ResMut<CombatState>,
) {
    for GameEventWrapper(event) in event_reader.read() {
        if let GameEvent::CombatStarted {
            attacker_player,
            attacker_slot,
            defender_player,
            defender_slot,
        } = event
        {
            // Queue the combat event for sequential processing
            // Lane is determined by the attacker's slot (combat happens in their lane)
            let queued = QueuedCombat {
                lane: attacker_slot.0,
                attacker_player: *attacker_player,
                attacker_slot: attacker_slot.0,
                defender_player: *defender_player,
                defender_slot: defender_slot.0,
            };
            combat_state.combat_queue.push_back(queued);
            debug!(
                "Queued combat: lane {} (slot {} attacks slot {})",
                attacker_slot.0, attacker_slot.0, defender_slot.0
            );
        }
    }
}

/// Process queued combats sequentially with delays between lanes.
fn process_combat_queue(
    time: Res<Time>,
    mut combat_state: ResMut<CombatState>,
    creatures: Query<(Entity, &Creature3D, &Transform)>,
) {
    // Don't process queue if animation is in progress
    if combat_state.active_attack.is_some() {
        return;
    }

    // Handle delay timer between lane combats
    if let Some(ref mut timer) = combat_state.lane_delay_timer {
        timer.tick(time.delta());
        if !timer.finished() {
            return;
        }
        combat_state.lane_delay_timer = None;
    }

    // Get next combat from queue
    let Some(queued) = combat_state.combat_queue.pop_front() else {
        return;
    };

    // Check if we're switching to a new lane (add delay for visual separation)
    if let Some(last_lane) = combat_state.last_lane {
        if queued.lane != last_lane {
            // Put the event back and start a delay
            combat_state.combat_queue.push_front(queued);
            combat_state.lane_delay_timer = Some(Timer::from_seconds(0.3, TimerMode::Once));
            info!("Lane change: {} -> {} (adding delay)", last_lane, combat_state.combat_queue.front().map(|q| q.lane).unwrap_or(0));
            return;
        }
    }

    // Process this combat
    let attacker_owner = if queued.attacker_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let defender_owner = if queued.defender_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
    let attacker_slot_idx = queued.attacker_slot as usize;
    let defender_slot_idx = queued.defender_slot as usize;

    let mut attacker_entity = None;
    let mut attacker_pos = Vec3::ZERO;
    let mut defender_pos = Vec3::ZERO;

    for (entity, creature, transform) in creatures.iter() {
        if creature.owner == attacker_owner && creature.slot == attacker_slot_idx {
            attacker_entity = Some(entity);
            attacker_pos = transform.translation;
        }
        if creature.owner == defender_owner && creature.slot == defender_slot_idx {
            defender_pos = transform.translation;
        }
    }

    // Start attack animation if we found both creatures
    if let Some(entity) = attacker_entity {
        combat_state.active_attack = Some(AttackAnimation {
            attacker_entity: entity,
            attacker_origin: attacker_pos,
            defender_pos,
            progress: 0.0,
            phase: AttackPhase::Forward,
        });
        combat_state.last_lane = Some(queued.lane);
        info!(
            "Combat started (lane {}): slot {} attacks slot {}",
            queued.lane, attacker_slot_idx, defender_slot_idx
        );
    }
}

/// Process non-combat events that still need immediate handling.
fn process_other_combat_events(
    mut commands: Commands,
    mut event_reader: EventReader<GameEventWrapper>,
    combat_state: Res<CombatState>,
    creatures: Query<(Entity, &Creature3D, &Transform)>,
) {
    for GameEventWrapper(event) in event_reader.read() {
        match event {
            GameEvent::CombatResolved {
                attacker_damage_dealt,
                defender_damage_dealt,
                ..
            } => {
                // Spawn damage numbers at both positions
                if let Some(ref attack) = combat_state.active_attack {
                    // Damage to defender (at defender position)
                    if *attacker_damage_dealt > 0 {
                        spawn_damage_number(
                            &mut commands,
                            attack.defender_pos,
                            *attacker_damage_dealt,
                        );
                    }
                    // Damage to attacker (at attacker origin)
                    if *defender_damage_dealt > 0 {
                        spawn_damage_number(
                            &mut commands,
                            attack.attacker_origin,
                            *defender_damage_dealt,
                        );
                    }
                }

                info!(
                    "Combat resolved: {} damage to defender, {} damage to attacker",
                    attacker_damage_dealt, defender_damage_dealt
                );
            }

            GameEvent::CreatureDamaged {
                player,
                slot,
                damage,
                ..
            } => {
                // Spawn damage number at creature position
                if *damage > 0 {
                    let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                    let slot_idx = slot.0 as usize;

                    // Find creature position
                    for (_, creature, transform) in creatures.iter() {
                        if creature.owner == owner && creature.slot == slot_idx {
                            spawn_damage_number(&mut commands, transform.translation, *damage);
                            break;
                        }
                    }
                }
            }

            GameEvent::LifeChanged {
                player,
                old_life,
                new_life,
                ..
            } => {
                // Show damage to player life
                let damage = old_life.saturating_sub(*new_life) as u8;
                if damage > 0 {
                    // Spawn damage number at player's side of board
                    let z = if *player == PlayerId::PLAYER_ONE { 4.0 } else { -4.0 };
                    let position = Vec3::new(0.0, 0.5, z);
                    spawn_damage_number(&mut commands, position, damage);
                    info!("{:?} takes {} damage (life: {} -> {})", player, damage, old_life, new_life);
                }
            }

            _ => {}
        }
    }
}

/// Spawn a floating damage number at a position.
fn spawn_damage_number(commands: &mut Commands, position: Vec3, amount: u8) {
    // Create 3D text for damage number
    // Using a simple colored cube as placeholder until text rendering is set up
    commands.spawn((
        Transform::from_translation(position + Vec3::new(0.0, 1.5, 0.0)),
        Visibility::Visible,
        DamageNumber {
            amount,
            lifetime: 0.0,
            start_y: position.y + 1.5,
        },
    ));
}

/// Animate active attack movements.
fn animate_attacks(
    time: Res<Time>,
    mut combat_state: ResMut<CombatState>,
    mut creatures: Query<&mut Transform, With<Creature3D>>,
) {
    let Some(ref mut attack) = combat_state.active_attack else {
        return;
    };

    let speed = 4.0; // Animation speed
    attack.progress += time.delta_secs() * speed;

    // Get creature transform
    let Ok(mut transform) = creatures.get_mut(attack.attacker_entity) else {
        // Entity no longer exists, clear animation
        combat_state.active_attack = None;
        return;
    };

    match attack.phase {
        AttackPhase::Forward => {
            // Move toward defender (stop at 70% of the way)
            let t = (attack.progress * 0.7).min(0.7);
            transform.translation = attack.attacker_origin.lerp(attack.defender_pos, t);

            // Switch to return phase at halfway through animation
            if attack.progress >= 1.0 {
                attack.progress = 0.0;
                attack.phase = AttackPhase::Return;
            }
        }
        AttackPhase::Return => {
            // Return to origin
            let t = attack.progress.min(1.0);
            let current_pos = attack.attacker_origin.lerp(attack.defender_pos, 0.7);
            transform.translation = current_pos.lerp(attack.attacker_origin, t);

            // Animation complete
            if attack.progress >= 1.0 {
                transform.translation = attack.attacker_origin;
                combat_state.active_attack = None;
            }
        }
    }
}

/// Animate floating damage numbers (float up and fade).
fn animate_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_numbers: Query<(Entity, &mut Transform, &mut DamageNumber)>,
) {
    for (entity, mut transform, mut damage) in damage_numbers.iter_mut() {
        damage.lifetime += time.delta_secs();

        // Float upward
        let float_speed = 1.5;
        transform.translation.y = damage.start_y + damage.lifetime * float_speed;

        // Remove after 1 second
        if damage.lifetime > 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Clean up combat effects when leaving game.
fn cleanup_combat_effects(
    mut commands: Commands,
    mut combat_state: ResMut<CombatState>,
    damage_numbers: Query<Entity, With<DamageNumber>>,
) {
    combat_state.active_attack = None;
    combat_state.combat_queue.clear();
    combat_state.lane_delay_timer = None;
    combat_state.last_lane = None;
    for entity in damage_numbers.iter() {
        commands.entity(entity).despawn();
    }
}
