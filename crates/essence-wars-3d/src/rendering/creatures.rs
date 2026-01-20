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
                    update_creature_state_visuals,
                    sync_creature_auras,
                    animate_idle_creatures,
                    animate_auras,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), (despawn_all_creatures, despawn_all_auras));
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

/// Component marking a buff aura entity (golden glow).
#[derive(Component)]
pub struct BuffAura {
    /// The creature instance this aura belongs to
    pub creature_instance_id: u32,
}

/// Component marking a debuff aura entity (dark shadow).
#[derive(Component)]
pub struct DebuffAura {
    /// The creature instance this aura belongs to
    pub creature_instance_id: u32,
}

/// Resource storing creature mesh and material handles.
#[derive(Resource)]
pub struct CreatureAssets {
    pub mesh: Handle<Mesh>,
    pub material_p1: Handle<StandardMaterial>,
    pub material_p2: Handle<StandardMaterial>,
    pub damaged_material_p1: Handle<StandardMaterial>,
    pub damaged_material_p2: Handle<StandardMaterial>,
    /// Glowing material for creatures that can attack
    pub ready_material_p1: Handle<StandardMaterial>,
    pub ready_material_p2: Handle<StandardMaterial>,
    /// Dimmed material for exhausted creatures
    pub exhausted_material_p1: Handle<StandardMaterial>,
    pub exhausted_material_p2: Handle<StandardMaterial>,
    /// Aura assets for buff/debuff visualization
    pub aura_mesh: Handle<Mesh>,
    pub buff_aura_material: Handle<StandardMaterial>,
    pub debuff_aura_material: Handle<StandardMaterial>,
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

/// Create ready-to-attack material (bright glowing).
fn create_ready_gem_material(faction: Option<Faction>, is_player1: bool) -> StandardMaterial {
    let mut mat = create_gem_material(faction, is_player1);
    // Boost emissive significantly for glowing effect
    mat.emissive = LinearRgba::new(
        mat.emissive.red * 3.0,
        mat.emissive.green * 3.0,
        mat.emissive.blue * 3.0,
        1.0,
    );
    // Slightly brighter base
    let srgba = mat.base_color.to_srgba();
    mat.base_color = Color::srgb(
        (srgba.red * 1.1).min(1.0),
        (srgba.green * 1.1).min(1.0),
        (srgba.blue * 1.1).min(1.0),
    );
    mat
}

/// Create exhausted material (dimmed, desaturated).
fn create_exhausted_gem_material(faction: Option<Faction>, is_player1: bool) -> StandardMaterial {
    let mut mat = create_gem_material(faction, is_player1);
    // Desaturate and darken
    let srgba = mat.base_color.to_srgba();
    let gray = (srgba.red + srgba.green + srgba.blue) / 3.0;
    // Blend toward gray (50% desaturation)
    mat.base_color = Color::srgb(
        (srgba.red * 0.5 + gray * 0.5) * 0.5,
        (srgba.green * 0.5 + gray * 0.5) * 0.5,
        (srgba.blue * 0.5 + gray * 0.5) * 0.5,
    );
    // Very dim emissive
    mat.emissive = LinearRgba::new(
        mat.emissive.red * 0.15,
        mat.emissive.green * 0.15,
        mat.emissive.blue * 0.15,
        1.0,
    );
    // Increased roughness
    mat.perceptual_roughness = (mat.perceptual_roughness + 0.2).min(1.0);
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
    let ready_material_p1 = materials.add(create_ready_gem_material(faction_p1, true));
    let ready_material_p2 = materials.add(create_ready_gem_material(faction_p2, false));
    let exhausted_material_p1 = materials.add(create_exhausted_gem_material(faction_p1, true));
    let exhausted_material_p2 = materials.add(create_exhausted_gem_material(faction_p2, false));

    // Create aura mesh (torus ring around creature)
    let aura_mesh = meshes.add(
        bevy::math::primitives::Torus {
            minor_radius: 0.05,
            major_radius: 0.55,
        }
    );

    // Golden buff aura - bright and warm
    let buff_aura_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.85, 0.3, 0.6),
        emissive: LinearRgba::new(1.0, 0.7, 0.2, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // Dark debuff aura - shadowy purple/red
    let debuff_aura_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.1, 0.3, 0.5),
        emissive: LinearRgba::new(0.3, 0.05, 0.15, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.insert_resource(CreatureAssets {
        mesh,
        material_p1,
        material_p2,
        damaged_material_p1,
        damaged_material_p2,
        ready_material_p1,
        ready_material_p2,
        exhausted_material_p1,
        exhausted_material_p2,
        aura_mesh,
        buff_aura_material,
        debuff_aura_material,
    });

    info!("Creature gem assets initialized (with state materials and auras)");
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

/// Update creature materials based on their game state (ready/exhausted/damaged).
fn update_creature_state_visuals(
    bridge: Res<GameBridge>,
    assets: Option<Res<CreatureAssets>>,
    mut creatures: Query<(&Creature3D, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let Some(assets) = assets else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    let current_turn = state.current_turn;

    for (creature_3d, mut material) in creatures.iter_mut() {
        // Find the corresponding game state creature
        let game_creature = state.players[creature_3d.owner]
            .creatures
            .iter()
            .find(|c| c.instance_id.0 == creature_3d.instance_id);

        let Some(game_creature) = game_creature else {
            continue;
        };

        // Determine the appropriate material based on state
        // Priority: Exhausted > Damaged > Ready > Normal
        let is_exhausted = game_creature.status.is_exhausted();
        let can_attack = game_creature.can_attack(current_turn);
        let is_damaged = game_creature.current_health <= 2 && game_creature.current_health < game_creature.base_health as i8;
        let is_player1 = creature_3d.owner == 0;

        let new_material = if is_exhausted {
            // Exhausted takes priority - creature has acted
            if is_player1 {
                assets.exhausted_material_p1.clone()
            } else {
                assets.exhausted_material_p2.clone()
            }
        } else if is_damaged {
            // Damaged but not exhausted
            if is_player1 {
                assets.damaged_material_p1.clone()
            } else {
                assets.damaged_material_p2.clone()
            }
        } else if can_attack {
            // Ready to attack - glow!
            if is_player1 {
                assets.ready_material_p1.clone()
            } else {
                assets.ready_material_p2.clone()
            }
        } else {
            // Normal state (summoning sick or no attack)
            if is_player1 {
                assets.material_p1.clone()
            } else {
                assets.material_p2.clone()
            }
        };

        material.0 = new_material;
    }
}

/// Sync buff/debuff auras with creature state.
/// Buff = attack > base_attack OR max_health > base_health
/// Debuff = attack < base_attack
fn sync_creature_auras(
    mut commands: Commands,
    bridge: Res<GameBridge>,
    assets: Option<Res<CreatureAssets>>,
    creatures: Query<(&Creature3D, &Transform)>,
    buff_auras: Query<(Entity, &BuffAura)>,
    debuff_auras: Query<(Entity, &DebuffAura)>,
) {
    let Some(assets) = assets else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    // Build sets of which creatures currently have auras
    let existing_buff_auras: std::collections::HashSet<u32> = buff_auras
        .iter()
        .map(|(_, aura)| aura.creature_instance_id)
        .collect();
    let existing_debuff_auras: std::collections::HashSet<u32> = debuff_auras
        .iter()
        .map(|(_, aura)| aura.creature_instance_id)
        .collect();

    for (creature_3d, transform) in creatures.iter() {
        // Find the corresponding game state creature
        let game_creature = state.players[creature_3d.owner]
            .creatures
            .iter()
            .find(|c| c.instance_id.0 == creature_3d.instance_id);

        let Some(game_creature) = game_creature else {
            continue;
        };

        // Check for buffs: attack > base_attack OR max_health > base_health
        let is_buffed = game_creature.attack > game_creature.base_attack as i8
            || game_creature.max_health > game_creature.base_health as i8;

        // Check for debuffs: attack < base_attack (negative buffs)
        let is_debuffed = game_creature.attack < game_creature.base_attack as i8;

        let instance_id = creature_3d.instance_id;

        // Spawn/despawn buff aura
        if is_buffed && !existing_buff_auras.contains(&instance_id) {
            commands.spawn((
                Mesh3d(assets.aura_mesh.clone()),
                MeshMaterial3d(assets.buff_aura_material.clone()),
                Transform::from_translation(transform.translation)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                BuffAura { creature_instance_id: instance_id },
            ));
        } else if !is_buffed && existing_buff_auras.contains(&instance_id) {
            for (entity, aura) in buff_auras.iter() {
                if aura.creature_instance_id == instance_id {
                    commands.entity(entity).despawn_recursive();
                    break;
                }
            }
        }

        // Spawn/despawn debuff aura
        if is_debuffed && !existing_debuff_auras.contains(&instance_id) {
            commands.spawn((
                Mesh3d(assets.aura_mesh.clone()),
                MeshMaterial3d(assets.debuff_aura_material.clone()),
                Transform::from_translation(transform.translation + Vec3::Y * 0.1)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                DebuffAura { creature_instance_id: instance_id },
            ));
        } else if !is_debuffed && existing_debuff_auras.contains(&instance_id) {
            for (entity, aura) in debuff_auras.iter() {
                if aura.creature_instance_id == instance_id {
                    commands.entity(entity).despawn_recursive();
                    break;
                }
            }
        }
    }

    // Clean up auras for dead creatures
    let alive_creatures: std::collections::HashSet<u32> = creatures
        .iter()
        .map(|(c, _)| c.instance_id)
        .collect();

    for (entity, aura) in buff_auras.iter() {
        if !alive_creatures.contains(&aura.creature_instance_id) {
            commands.entity(entity).despawn_recursive();
        }
    }
    for (entity, aura) in debuff_auras.iter() {
        if !alive_creatures.contains(&aura.creature_instance_id) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Animate auras: rotate and pulse.
fn animate_auras(
    time: Res<Time>,
    creatures: Query<(&Creature3D, &Transform)>,
    mut buff_auras: Query<(&BuffAura, &mut Transform), Without<Creature3D>>,
    mut debuff_auras: Query<(&DebuffAura, &mut Transform), (Without<Creature3D>, Without<BuffAura>)>,
) {
    let t = time.elapsed_secs();
    let rotation_speed = 1.5;
    let pulse_speed = 3.0;

    // Build a map of creature positions
    let creature_positions: std::collections::HashMap<u32, Vec3> = creatures
        .iter()
        .map(|(c, t)| (c.instance_id, t.translation))
        .collect();

    // Animate buff auras (rotate clockwise, pulse scale)
    for (aura, mut transform) in buff_auras.iter_mut() {
        if let Some(&creature_pos) = creature_positions.get(&aura.creature_instance_id) {
            // Follow creature position
            transform.translation.x = creature_pos.x;
            transform.translation.z = creature_pos.z;
            transform.translation.y = creature_pos.y - 0.3;

            // Rotate around Y axis
            let rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(t * rotation_speed);
            transform.rotation = rotation;

            // Pulse scale
            let pulse = 1.0 + 0.1 * (t * pulse_speed).sin();
            transform.scale = Vec3::splat(pulse);
        }
    }

    // Animate debuff auras (rotate counter-clockwise, slightly different timing)
    for (aura, mut transform) in debuff_auras.iter_mut() {
        if let Some(&creature_pos) = creature_positions.get(&aura.creature_instance_id) {
            // Follow creature position (slightly higher than buff aura)
            transform.translation.x = creature_pos.x;
            transform.translation.z = creature_pos.z;
            transform.translation.y = creature_pos.y - 0.2;

            // Rotate counter-clockwise
            let rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(-t * rotation_speed * 0.8);
            transform.rotation = rotation;

            // Pulse scale (different phase)
            let pulse = 1.0 + 0.08 * (t * pulse_speed + 1.5).sin();
            transform.scale = Vec3::splat(pulse);
        }
    }
}

/// Despawn all auras when leaving game.
fn despawn_all_auras(
    mut commands: Commands,
    buff_auras: Query<Entity, With<BuffAura>>,
    debuff_auras: Query<Entity, With<DebuffAura>>,
) {
    for entity in buff_auras.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in debuff_auras.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
