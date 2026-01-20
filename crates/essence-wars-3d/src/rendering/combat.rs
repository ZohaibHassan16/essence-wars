//! Combat visualization and effects.
//!
//! This module handles visual effects for combat, including:
//! - Attack animations (creature movement)
//! - Damage number displays
//! - Visual feedback for combat resolution

use bevy::prelude::*;
use cardgame::client_api::GameEvent;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameEventQueue};
use super::creatures::Creature3D;

/// Plugin for combat visualization.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatState>()
            .add_systems(
                Update,
                (
                    process_combat_events,
                    animate_attacks,
                    animate_damage_numbers,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), cleanup_combat_effects);
    }
}

/// Resource tracking current combat state.
#[derive(Resource, Default)]
pub struct CombatState {
    /// Active attack animation (if any)
    pub active_attack: Option<AttackAnimation>,
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

/// Process combat events from the queue.
fn process_combat_events(
    mut commands: Commands,
    mut event_queue: ResMut<GameEventQueue>,
    mut combat_state: ResMut<CombatState>,
    creatures: Query<(Entity, &Creature3D, &Transform)>,
) {
    // Collect combat events
    let mut events_to_requeue = Vec::new();
    let mut combat_events = Vec::new();

    while let Some(event) = event_queue.events.pop_front() {
        match &event {
            GameEvent::CombatStarted { .. }
            | GameEvent::CombatResolved { .. }
            | GameEvent::CreatureDamaged { .. }
            | GameEvent::LifeChanged { .. } => {
                combat_events.push(event);
            }
            _ => {
                events_to_requeue.push(event);
            }
        }
    }

    // Put non-combat events back
    for event in events_to_requeue {
        event_queue.events.push_back(event);
    }

    // Process combat events
    for event in combat_events {
        match event {
            GameEvent::CombatStarted {
                attacker_player,
                attacker_slot,
                defender_player,
                defender_slot,
            } => {
                // Find attacker and defender entities
                let attacker_owner = if attacker_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let defender_owner = if defender_player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let attacker_slot_idx = attacker_slot.0 as usize;
                let defender_slot_idx = defender_slot.0 as usize;

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
                    info!(
                        "Combat started: slot {} attacks slot {}",
                        attacker_slot_idx, defender_slot_idx
                    );
                }
            }

            GameEvent::CombatResolved {
                attacker_damage_dealt,
                defender_damage_dealt,
                ..
            } => {
                // Spawn damage numbers at both positions
                if let Some(ref attack) = combat_state.active_attack {
                    // Damage to defender (at defender position)
                    if attacker_damage_dealt > 0 {
                        spawn_damage_number(
                            &mut commands,
                            attack.defender_pos,
                            attacker_damage_dealt,
                        );
                    }
                    // Damage to attacker (at attacker origin)
                    if defender_damage_dealt > 0 {
                        spawn_damage_number(
                            &mut commands,
                            attack.attacker_origin,
                            defender_damage_dealt,
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
                if damage > 0 {
                    let owner = if player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                    let slot_idx = slot.0 as usize;

                    // Find creature position
                    for (_, creature, transform) in creatures.iter() {
                        if creature.owner == owner && creature.slot == slot_idx {
                            spawn_damage_number(&mut commands, transform.translation, damage);
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
                let damage = old_life.saturating_sub(new_life) as u8;
                if damage > 0 {
                    // Spawn damage number at player's side of board
                    let z = if player == PlayerId::PLAYER_ONE { 4.0 } else { -4.0 };
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
    for entity in damage_numbers.iter() {
        commands.entity(entity).despawn();
    }
}
