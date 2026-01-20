//! Creature visualization and management.
//!
//! This module handles spawning and despawning 3D creature representations
//! based on game events.

use bevy::prelude::*;
use cardgame::client_api::GameEvent;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameBridge, GameEventWrapper};

/// Plugin for creature rendering.
pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_creature_assets)
            .add_systems(
                Update,
                (sync_creatures_with_game_state, process_creature_events)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), despawn_all_creatures);
    }
}

/// Component marking a creature entity.
#[derive(Component)]
pub struct Creature3D {
    /// Instance ID from the game engine
    pub instance_id: u32,
    /// Card ID
    pub card_id: u16,
    /// Owner player (0 or 1)
    pub owner: usize,
    /// Slot index (0-4)
    pub slot: usize,
    /// Current attack value
    pub attack: i8,
    /// Current health value
    pub health: i8,
}

/// Resource storing creature mesh and material handles.
#[derive(Resource)]
pub struct CreatureAssets {
    pub mesh: Handle<Mesh>,
    pub material_p1: Handle<StandardMaterial>,
    pub material_p2: Handle<StandardMaterial>,
    pub damaged_material_p1: Handle<StandardMaterial>,
    pub damaged_material_p2: Handle<StandardMaterial>,
}

/// Setup creature assets when entering Playing state.
fn setup_creature_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a simple capsule mesh for creatures (placeholder)
    let mesh = meshes.add(Capsule3d::new(0.4, 0.8));

    // Player 1 materials (blue tones)
    let material_p1 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.5, 0.9),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    let damaged_material_p1 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.3, 0.7),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    // Player 2 materials (red tones)
    let material_p2 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.3, 0.2),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    let damaged_material_p2 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.2, 0.4),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    commands.insert_resource(CreatureAssets {
        mesh,
        material_p1,
        material_p2,
        damaged_material_p1,
        damaged_material_p2,
    });

    info!("Creature assets initialized");
}

/// Process creature-related events using Bevy's event system.
fn process_creature_events(
    mut commands: Commands,
    mut event_reader: EventReader<GameEventWrapper>,
    assets: Option<Res<CreatureAssets>>,
    mut creatures: Query<(Entity, &Creature3D, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let Some(assets) = assets else { return };

    for GameEventWrapper(event) in event_reader.read() {
        match event {
            GameEvent::CreatureSpawned {
                player,
                slot,
                card_id,
                instance_id,
                attack,
                health,
                ..
            } => {
                let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let slot_idx = slot.0 as usize;

                // Calculate position
                let x = (slot_idx as f32 - 2.0) * 2.0;
                let z = if owner == 0 { 2.0 } else { -2.0 };

                let material = if owner == 0 {
                    assets.material_p1.clone()
                } else {
                    assets.material_p2.clone()
                };

                commands.spawn((
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(x, 0.75, z),
                    Creature3D {
                        instance_id: instance_id.0,
                        card_id: card_id.0,
                        owner,
                        slot: slot_idx,
                        attack: *attack,
                        health: *health,
                    },
                ));

                info!(
                    "Spawned creature {} at slot {} for player {}",
                    instance_id.0, slot_idx, owner
                );
            }

            GameEvent::CreatureDied {
                instance_id, ..
            } => {
                // Find and despawn the creature
                for (entity, creature, _) in creatures.iter() {
                    if creature.instance_id == instance_id.0 {
                        commands.entity(entity).despawn_recursive();
                        info!("Despawned creature {}", instance_id.0);
                        break;
                    }
                }
            }

            GameEvent::CreatureDamaged {
                slot,
                new_health,
                player,
                ..
            } => {
                // Update creature health and potentially material
                let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let slot_idx = slot.0 as usize;
                for (_, creature, mut material) in creatures.iter_mut() {
                    if creature.owner == owner && creature.slot == slot_idx {
                        // Change to damaged material if health is low
                        if *new_health <= 2 {
                            material.0 = if owner == 0 {
                                assets.damaged_material_p1.clone()
                            } else {
                                assets.damaged_material_p2.clone()
                            };
                        }
                        break;
                    }
                }
            }

            GameEvent::CreatureHealed {
                slot,
                new_health,
                player,
                ..
            } => {
                // Update creature visual
                let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let slot_idx = slot.0 as usize;
                for (_, creature, mut material) in creatures.iter_mut() {
                    if creature.owner == owner && creature.slot == slot_idx {
                        // Restore normal material if healed above threshold
                        if *new_health > 2 {
                            material.0 = if owner == 0 {
                                assets.material_p1.clone()
                            } else {
                                assets.material_p2.clone()
                            };
                        }
                        break;
                    }
                }
            }

            GameEvent::CreatureStatsChanged {
                slot,
                player,
                ..
            } => {
                // Stats changed - could add visual indicator here
                let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let slot_idx = slot.0 as usize;
                debug!(
                    "Creature stats changed: player {} slot {}",
                    owner, slot_idx
                );
            }

            _ => {}
        }
    }
}

/// Sync creatures with game state on first frame (in case we missed events).
fn sync_creatures_with_game_state(
    mut commands: Commands,
    bridge: Res<GameBridge>,
    assets: Option<Res<CreatureAssets>>,
    existing_creatures: Query<&Creature3D>,
    mut has_synced: Local<bool>,
) {
    // Only sync once per game
    if *has_synced {
        return;
    }

    let Some(assets) = assets else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    // Collect existing creature instance IDs
    let existing_ids: std::collections::HashSet<u32> = existing_creatures
        .iter()
        .map(|c| c.instance_id)
        .collect();

    // Spawn any creatures that exist in game state but not as entities
    // creatures is an ArrayVec<Creature, 5> where each creature has a slot field
    for (player_idx, player) in state.players.iter().enumerate() {
        for creature in &player.creatures {
            if !existing_ids.contains(&creature.instance_id.0) {
                let slot_idx = creature.slot.0 as usize;
                let x = (slot_idx as f32 - 2.0) * 2.0;
                let z = if player_idx == 0 { 2.0 } else { -2.0 };

                let material = if player_idx == 0 {
                    assets.material_p1.clone()
                } else {
                    assets.material_p2.clone()
                };

                commands.spawn((
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(x, 0.75, z),
                    Creature3D {
                        instance_id: creature.instance_id.0,
                        card_id: creature.card_id.0,
                        owner: player_idx,
                        slot: slot_idx,
                        attack: creature.attack,
                        health: creature.current_health,
                    },
                ));

                info!(
                    "Synced creature {} at slot {} for player {}",
                    creature.instance_id.0, slot_idx, player_idx
                );
            }
        }
    }

    *has_synced = true;
}

/// Despawn all creatures when leaving game.
fn despawn_all_creatures(
    mut commands: Commands,
    creatures: Query<Entity, With<Creature3D>>,
) {
    for entity in creatures.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("All creatures despawned");
}
