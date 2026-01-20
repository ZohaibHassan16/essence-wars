//! Creature visualization and management.
//!
//! This module handles spawning and despawning 3D creature representations
//! based on game events. Creatures are rendered as faceted gem tokens with
//! faction-specific materials.

use bevy::prelude::*;
use cardgame::client_api::GameEvent;
use cardgame::decks::Faction;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameBridge, GameEventWrapper};
use super::meshes::create_detailed_gem_mesh;

/// Plugin for creature rendering.
pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_creature_assets)
            .add_systems(
                Update,
                (
                    sync_creatures_with_game_state,
                    process_creature_events,
                    animate_idle_creatures,
                )
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

/// Get faction for a player's deck.
fn get_player_faction(bridge: &GameBridge, player: usize) -> Option<Faction> {
    let deck_id = if player == 0 {
        bridge.current_deck1.as_ref()?
    } else {
        bridge.current_deck2.as_ref()?
    };
    let deck = bridge.deck_registry.get(deck_id)?;
    deck.faction()
}

/// Create faction-specific gem material for a player.
fn create_gem_material(faction: Option<Faction>, is_player1: bool) -> StandardMaterial {
    match (faction, is_player1) {
        // Argentum - Brass/gold crystalline gems
        (Some(Faction::Argentum), true) => StandardMaterial {
            base_color: Color::srgb(0.8, 0.65, 0.3),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.2, 0.15, 0.05, 1.0),
            ..default()
        },
        (Some(Faction::Argentum), false) => StandardMaterial {
            base_color: Color::srgb(0.6, 0.5, 0.25),
            metallic: 0.7,
            perceptual_roughness: 0.25,
            emissive: LinearRgba::new(0.15, 0.1, 0.03, 1.0),
            ..default()
        },
        // Symbiote - Organic green/purple gems
        (Some(Faction::Symbiote), true) => StandardMaterial {
            base_color: Color::srgb(0.2, 0.7, 0.4),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            emissive: LinearRgba::new(0.05, 0.2, 0.1, 1.0),
            ..default()
        },
        (Some(Faction::Symbiote), false) => StandardMaterial {
            base_color: Color::srgb(0.5, 0.3, 0.6),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            emissive: LinearRgba::new(0.1, 0.05, 0.15, 1.0),
            ..default()
        },
        // Obsidion - Dark crystalline with red/purple glow
        (Some(Faction::Obsidion), true) => StandardMaterial {
            base_color: Color::srgb(0.15, 0.05, 0.1),
            metallic: 0.9,
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(0.25, 0.05, 0.1, 1.0),
            ..default()
        },
        (Some(Faction::Obsidion), false) => StandardMaterial {
            base_color: Color::srgb(0.2, 0.05, 0.15),
            metallic: 0.85,
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.2, 0.02, 0.08, 1.0),
            ..default()
        },
        // Default/Neutral - Blue for P1, Red for P2
        (_, true) => StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.9),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            emissive: LinearRgba::new(0.05, 0.1, 0.2, 1.0),
            ..default()
        },
        (_, false) => StandardMaterial {
            base_color: Color::srgb(0.9, 0.3, 0.3),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            emissive: LinearRgba::new(0.2, 0.05, 0.05, 1.0),
            ..default()
        },
    }
}

/// Create damaged gem material (darker, cracked appearance).
fn create_damaged_gem_material(faction: Option<Faction>, is_player1: bool) -> StandardMaterial {
    let mut mat = create_gem_material(faction, is_player1);
    // Darken the base color
    let srgba = mat.base_color.to_srgba();
    mat.base_color = Color::srgb(
        srgba.red * 0.6,
        srgba.green * 0.6,
        srgba.blue * 0.6,
    );
    // Reduce emissive
    mat.emissive = LinearRgba::new(
        mat.emissive.red * 0.3,
        mat.emissive.green * 0.3,
        mat.emissive.blue * 0.3,
        1.0,
    );
    // Make it rougher (cracked)
    mat.perceptual_roughness = (mat.perceptual_roughness + 0.3).min(1.0);
    mat
}

/// Setup creature assets when entering Playing state.
fn setup_creature_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bridge: Res<GameBridge>,
) {
    // Create a faceted gem mesh for creatures
    // Width: 0.8, Height: 1.2, Bevel: 0.3 for a nice crystal shape
    let mesh = meshes.add(create_detailed_gem_mesh(0.8, 1.2, 0.3));

    // Get factions for both players
    let faction_p1 = get_player_faction(&bridge, 0);
    let faction_p2 = get_player_faction(&bridge, 1);

    info!(
        "Creating gem materials - P1: {:?}, P2: {:?}",
        faction_p1.map(|f| f.display_name()),
        faction_p2.map(|f| f.display_name())
    );

    // Create faction-specific materials
    let material_p1 = materials.add(create_gem_material(faction_p1, true));
    let material_p2 = materials.add(create_gem_material(faction_p2, false));
    let damaged_material_p1 = materials.add(create_damaged_gem_material(faction_p1, true));
    let damaged_material_p2 = materials.add(create_damaged_gem_material(faction_p2, false));

    commands.insert_resource(CreatureAssets {
        mesh,
        material_p1,
        material_p2,
        damaged_material_p1,
        damaged_material_p2,
    });

    info!("Creature gem assets initialized");
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

/// Animate creatures with subtle idle bob/sway motion.
/// Each creature bobs gently up and down with a unique phase based on slot.
fn animate_idle_creatures(
    time: Res<Time>,
    mut creatures: Query<(&mut Transform, &Creature3D)>,
) {
    let base_height = 0.6; // Base Y position for creature gems
    let bob_amplitude = 0.04; // How far up/down to bob
    let bob_speed = 1.8; // Cycles per second

    for (mut transform, creature) in creatures.iter_mut() {
        // Each creature has a unique phase based on slot and owner
        // This prevents all creatures bobbing in sync
        let phase = (creature.slot as f32 * 0.7) + (creature.owner as f32 * 2.5);
        let t = time.elapsed_secs();
        let bob = (t * bob_speed + phase).sin() * bob_amplitude;

        // Update only Y position, preserve X and Z
        transform.translation.y = base_height + bob;
    }
}
