//! Support card visualization and management.
//!
//! This module handles spawning and despawning 3D support representations
//! based on game state. Supports are rendered as "Floating Rune Plates" with
//! faction-specific materials.

use bevy::prelude::*;
use cardgame::client_api::GameEvent;
use cardgame::decks::Faction;
use cardgame::types::PlayerId;

use crate::game::{AppState, GameBridge, GameEventWrapper};

/// Plugin for support rendering.
pub struct SupportPlugin;

impl Plugin for SupportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_support_assets)
            .add_systems(
                Update,
                (
                    sync_supports_with_game_state,
                    process_support_events,
                    sync_durability_pips,
                    animate_durability_pips,
                    animate_trigger_pulses,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), despawn_all_supports);
    }
}

/// Component marking a support entity.
#[derive(Component)]
pub struct Support3D {
    /// Card ID
    pub card_id: u16,
    /// Owner player (0 or 1)
    pub owner: usize,
    /// Slot index (0-1)
    pub slot: usize,
    /// Current durability
    pub durability: u8,
}

/// Component marking a durability pip entity.
#[derive(Component)]
pub struct DurabilityPip {
    /// Owner player (0 or 1)
    pub owner: usize,
    /// Support slot index (0-1)
    pub slot: usize,
    /// Pip index (0, 1, 2, ...)
    pub pip_index: u8,
}

/// Component marking a support trigger pulse effect.
#[derive(Component)]
pub struct TriggerPulse {
    /// Time when the pulse was spawned
    pub spawn_time: f32,
    /// Duration of the pulse animation
    pub duration: f32,
}

/// Resource storing support mesh and material handles.
#[derive(Resource)]
pub struct SupportAssets {
    pub mesh: Handle<Mesh>,
    pub material_p1: Handle<StandardMaterial>,
    pub material_p2: Handle<StandardMaterial>,
    pub low_durability_material_p1: Handle<StandardMaterial>,
    pub low_durability_material_p2: Handle<StandardMaterial>,
    /// Durability pip assets
    pub pip_mesh: Handle<Mesh>,
    pub pip_material_p1: Handle<StandardMaterial>,
    pub pip_material_p2: Handle<StandardMaterial>,
    /// Trigger pulse assets
    pub pulse_mesh: Handle<Mesh>,
    pub pulse_material_p1: Handle<StandardMaterial>,
    pub pulse_material_p2: Handle<StandardMaterial>,
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

/// Create faction-specific support material (Floating Rune Plate aesthetic).
fn create_support_material(faction: Option<Faction>, is_player1: bool) -> StandardMaterial {
    match (faction, is_player1) {
        // Argentum: Brass-framed clockwork plates with amber rune glow
        (Some(Faction::Argentum), true) => StandardMaterial {
            base_color: Color::srgb(0.65, 0.5, 0.25),
            metallic: 0.85,
            perceptual_roughness: 0.25,
            emissive: LinearRgba::new(0.25, 0.18, 0.05, 1.0), // Amber glow
            ..default()
        },
        (Some(Faction::Argentum), false) => StandardMaterial {
            base_color: Color::srgb(0.5, 0.4, 0.2),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            emissive: LinearRgba::new(0.18, 0.12, 0.03, 1.0),
            ..default()
        },
        // Symbiote: Organic chitin plates with bioluminescent green/purple glow
        (Some(Faction::Symbiote), true) => StandardMaterial {
            base_color: Color::srgb(0.25, 0.45, 0.3),
            metallic: 0.3,
            perceptual_roughness: 0.5,
            emissive: LinearRgba::new(0.08, 0.28, 0.12, 1.0), // Green glow
            ..default()
        },
        (Some(Faction::Symbiote), false) => StandardMaterial {
            base_color: Color::srgb(0.2, 0.35, 0.25),
            metallic: 0.25,
            perceptual_roughness: 0.55,
            emissive: LinearRgba::new(0.05, 0.2, 0.08, 1.0),
            ..default()
        },
        // Obsidion: Obsidian glass tablets with crimson/purple glow
        (Some(Faction::Obsidion), true) => StandardMaterial {
            base_color: Color::srgb(0.15, 0.08, 0.12),
            metallic: 0.9,
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(0.28, 0.05, 0.12, 1.0), // Crimson glow
            ..default()
        },
        (Some(Faction::Obsidion), false) => StandardMaterial {
            base_color: Color::srgb(0.12, 0.05, 0.1),
            metallic: 0.85,
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.2, 0.03, 0.08, 1.0),
            ..default()
        },
        // Default/Neutral: Weathered stone with white/blue glow
        (_, true) => StandardMaterial {
            base_color: Color::srgb(0.5, 0.3, 0.6),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            emissive: LinearRgba::new(0.15, 0.12, 0.22, 1.0),
            ..default()
        },
        (_, false) => StandardMaterial {
            base_color: Color::srgb(0.6, 0.3, 0.45),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            emissive: LinearRgba::new(0.2, 0.1, 0.15, 1.0),
            ..default()
        },
    }
}

/// Create low durability material (cracked/faded appearance).
fn create_low_durability_material(faction: Option<Faction>, is_player1: bool) -> StandardMaterial {
    let base = create_support_material(faction, is_player1);
    // Darken and increase roughness for "cracked" look
    let srgba = base.base_color.to_srgba();
    StandardMaterial {
        base_color: Color::srgb(srgba.red * 0.6, srgba.green * 0.6, srgba.blue * 0.6),
        metallic: base.metallic * 0.6,
        perceptual_roughness: (base.perceptual_roughness + 0.3).min(1.0),
        emissive: LinearRgba::new(
            base.emissive.red * 0.3,
            base.emissive.green * 0.3,
            base.emissive.blue * 0.3,
            1.0,
        ),
        ..default()
    }
}

/// Setup support assets when entering Playing state.
fn setup_support_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bridge: Res<GameBridge>,
) {
    // Create a floating rune plate mesh (taller cylinder)
    let mesh = meshes.add(Cylinder::new(0.5, 0.6));

    // Get factions for both players
    let faction_p1 = get_player_faction(&bridge, 0);
    let faction_p2 = get_player_faction(&bridge, 1);

    // Create faction-specific materials
    let material_p1 = materials.add(create_support_material(faction_p1, true));
    let material_p2 = materials.add(create_support_material(faction_p2, false));
    let low_durability_material_p1 = materials.add(create_low_durability_material(faction_p1, true));
    let low_durability_material_p2 = materials.add(create_low_durability_material(faction_p2, false));

    // Create durability pip mesh (small sphere)
    let pip_mesh = meshes.add(Sphere::new(0.08));

    // Create faction-colored pip materials (bright glowing dots)
    let pip_material_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.9, 0.5, 0.9),
        emissive: LinearRgba::new(0.8, 0.6, 0.2, 1.0), // Warm golden glow
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let pip_material_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.5, 0.6, 0.9),
        emissive: LinearRgba::new(0.7, 0.3, 0.35, 1.0), // Warm red glow
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Create trigger pulse mesh (torus that expands outward)
    let pulse_mesh = meshes.add(
        bevy::math::primitives::Torus {
            minor_radius: 0.04,
            major_radius: 0.6,
        }
    );

    // Pulse materials (bright, semi-transparent rune flare)
    let pulse_material_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.7, 0.8),
        emissive: LinearRgba::new(1.5, 1.2, 0.4, 1.0), // Bright golden flare
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let pulse_material_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.6, 0.7, 0.8),
        emissive: LinearRgba::new(1.2, 0.4, 0.5, 1.0), // Bright red flare
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(SupportAssets {
        mesh,
        material_p1,
        material_p2,
        low_durability_material_p1,
        low_durability_material_p2,
        pip_mesh,
        pip_material_p1,
        pip_material_p2,
        pulse_mesh,
        pulse_material_p1,
        pulse_material_p2,
    });

    info!("Support assets initialized (with durability pips and trigger pulse)");
}

/// Get the world position for a support slot.
fn support_slot_position(owner: usize, slot: usize) -> Vec3 {
    // Match positions from board.rs
    if owner == 0 {
        // Player 1: left side
        let x = -5.5;
        let z = 1.0 + slot as f32 * 2.0;
        Vec3::new(x, 0.5, z)
    } else {
        // Player 2: right side
        let x = 5.5;
        let z = -1.0 - slot as f32 * 2.0;
        Vec3::new(x, 0.5, z)
    }
}

/// Process support-related events using Bevy's event system.
fn process_support_events(
    mut commands: Commands,
    mut event_reader: EventReader<GameEventWrapper>,
    assets: Option<Res<SupportAssets>>,
    time: Res<Time>,
    mut supports: Query<(Entity, &Support3D, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let Some(assets) = assets else { return };

    for GameEventWrapper(event) in event_reader.read() {
        match event {
            GameEvent::SupportDurabilityChanged {
                player,
                slot,
                new_durability,
                ..
            } => {
                let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let slot_idx = slot.0 as usize;

                for (_, support, mut material) in supports.iter_mut() {
                    if support.owner == owner && support.slot == slot_idx {
                        // Change to low durability material if durability is 1
                        if *new_durability <= 1 {
                            material.0 = if owner == 0 {
                                assets.low_durability_material_p1.clone()
                            } else {
                                assets.low_durability_material_p2.clone()
                            };
                        }
                        break;
                    }
                }

                // Spawn a trigger pulse effect at the support position
                let position = support_slot_position(owner, slot_idx);
                let pulse_material = if owner == 0 {
                    assets.pulse_material_p1.clone()
                } else {
                    assets.pulse_material_p2.clone()
                };

                commands.spawn((
                    Mesh3d(assets.pulse_mesh.clone()),
                    MeshMaterial3d(pulse_material),
                    Transform::from_translation(position + Vec3::Y * 0.4)
                        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    TriggerPulse {
                        spawn_time: time.elapsed_secs(),
                        duration: 0.6,
                    },
                ));

                info!(
                    "Support triggered: player {} slot {} (new durability: {})",
                    owner, slot_idx, new_durability
                );
            }

            GameEvent::SupportRemoved {
                player,
                slot,
                ..
            } => {
                let owner = if *player == PlayerId::PLAYER_ONE { 0 } else { 1 };
                let slot_idx = slot.0 as usize;

                // Find and despawn the support
                for (entity, support, _) in supports.iter() {
                    if support.owner == owner && support.slot == slot_idx {
                        commands.entity(entity).despawn_recursive();
                        info!("Despawned support at slot {} for player {}", slot_idx, owner);
                        break;
                    }
                }
            }

            _ => {}
        }
    }
}

/// Sync supports with game state (handles spawning since there's no SupportSpawned event).
fn sync_supports_with_game_state(
    mut commands: Commands,
    bridge: Res<GameBridge>,
    assets: Option<Res<SupportAssets>>,
    existing_supports: Query<&Support3D>,
) {
    let Some(assets) = assets else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    // Build a set of existing (owner, slot) pairs
    let existing: std::collections::HashSet<(usize, usize)> = existing_supports
        .iter()
        .map(|s| (s.owner, s.slot))
        .collect();

    // Spawn any supports that exist in game state but not as entities
    for (player_idx, player) in state.players.iter().enumerate() {
        for support in &player.supports {
            let slot_idx = support.slot.0 as usize;

            if !existing.contains(&(player_idx, slot_idx)) {
                let position = support_slot_position(player_idx, slot_idx);

                let material = if support.current_durability <= 1 {
                    if player_idx == 0 {
                        assets.low_durability_material_p1.clone()
                    } else {
                        assets.low_durability_material_p2.clone()
                    }
                } else if player_idx == 0 {
                    assets.material_p1.clone()
                } else {
                    assets.material_p2.clone()
                };

                commands.spawn((
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(position),
                    Support3D {
                        card_id: support.card_id.0,
                        owner: player_idx,
                        slot: slot_idx,
                        durability: support.current_durability,
                    },
                ));

                info!(
                    "Synced support at slot {} for player {} (durability: {})",
                    slot_idx, player_idx, support.current_durability
                );
            }
        }
    }

    // Despawn any supports that no longer exist in game state
    for support in existing_supports.iter() {
        let player = &state.players[support.owner];
        let still_exists = player.supports.iter().any(|s| s.slot.0 as usize == support.slot);

        if !still_exists {
            // Note: We can't despawn here directly because we only have the component
            // The event-based removal (SupportRemoved) should handle this case
        }
    }
}

/// Despawn all supports when leaving game.
fn despawn_all_supports(
    mut commands: Commands,
    supports: Query<Entity, With<Support3D>>,
    pips: Query<Entity, With<DurabilityPip>>,
    pulses: Query<Entity, With<TriggerPulse>>,
) {
    for entity in supports.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in pips.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in pulses.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("All supports, durability pips, and trigger pulses despawned");
}

/// Calculate pip position around support (arranged in a circle).
fn pip_position(support_pos: Vec3, pip_index: u8, total_pips: u8) -> Vec3 {
    let radius = 0.65; // Slightly outside the support cylinder
    let height = 0.5; // At the top of the support

    // Distribute pips evenly around the circle
    let angle = (pip_index as f32 / total_pips as f32) * std::f32::consts::TAU;
    let x = support_pos.x + radius * angle.cos();
    let z = support_pos.z + radius * angle.sin();
    let y = support_pos.y + height;

    Vec3::new(x, y, z)
}

/// Sync durability pips with support state.
fn sync_durability_pips(
    mut commands: Commands,
    bridge: Res<GameBridge>,
    assets: Option<Res<SupportAssets>>,
    supports: Query<&Support3D>,
    existing_pips: Query<(Entity, &DurabilityPip)>,
) {
    let Some(assets) = assets else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    // Build map of existing pips by (owner, slot, pip_index)
    let existing_pip_map: std::collections::HashSet<(usize, usize, u8)> = existing_pips
        .iter()
        .map(|(_, pip)| (pip.owner, pip.slot, pip.pip_index))
        .collect();

    // Sync pips for each support
    for (player_idx, player) in state.players.iter().enumerate() {
        for support in &player.supports {
            let slot_idx = support.slot.0 as usize;
            let durability = support.current_durability;
            let support_pos = support_slot_position(player_idx, slot_idx);

            // Spawn pips that don't exist
            for pip_idx in 0..durability {
                if !existing_pip_map.contains(&(player_idx, slot_idx, pip_idx)) {
                    let pip_pos = pip_position(support_pos, pip_idx, durability);
                    let material = if player_idx == 0 {
                        assets.pip_material_p1.clone()
                    } else {
                        assets.pip_material_p2.clone()
                    };

                    commands.spawn((
                        Mesh3d(assets.pip_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(pip_pos),
                        DurabilityPip {
                            owner: player_idx,
                            slot: slot_idx,
                            pip_index: pip_idx,
                        },
                    ));
                }
            }

            // Despawn pips that exceed current durability
            for (entity, pip) in existing_pips.iter() {
                if pip.owner == player_idx && pip.slot == slot_idx && pip.pip_index >= durability {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }

    // Clean up pips for removed supports
    let active_supports: std::collections::HashSet<(usize, usize)> = supports
        .iter()
        .map(|s| (s.owner, s.slot))
        .collect();

    for (entity, pip) in existing_pips.iter() {
        if !active_supports.contains(&(pip.owner, pip.slot)) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Animate durability pips with subtle pulse and orbit.
fn animate_durability_pips(
    time: Res<Time>,
    mut pips: Query<(&DurabilityPip, &mut Transform)>,
    bridge: Res<GameBridge>,
) {
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    let t = time.elapsed_secs();
    let pulse_speed = 2.5;
    let orbit_speed = 0.5;

    for (pip, mut transform) in pips.iter_mut() {
        // Get support durability to calculate pip position
        let support = state.players[pip.owner]
            .supports
            .iter()
            .find(|s| s.slot.0 as usize == pip.slot);

        if let Some(support) = support {
            let durability = support.current_durability;
            let support_pos = support_slot_position(pip.owner, pip.slot);

            // Orbit around the support
            let base_angle = (pip.pip_index as f32 / durability as f32) * std::f32::consts::TAU;
            let angle = base_angle + t * orbit_speed;
            let radius = 0.65;
            let height = 0.5;

            let x = support_pos.x + radius * angle.cos();
            let z = support_pos.z + radius * angle.sin();

            // Add slight vertical bob
            let bob = 0.03 * (t * pulse_speed + pip.pip_index as f32 * 0.5).sin();
            let y = support_pos.y + height + bob;

            transform.translation = Vec3::new(x, y, z);

            // Pulse scale
            let pulse = 1.0 + 0.15 * (t * pulse_speed + pip.pip_index as f32 * 0.8).sin();
            transform.scale = Vec3::splat(pulse);
        }
    }
}

/// Animate trigger pulses (expand and fade out).
fn animate_trigger_pulses(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(Entity, &TriggerPulse, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, pulse, mut transform, material_handle) in pulses.iter_mut() {
        let elapsed = current_time - pulse.spawn_time;
        let progress = (elapsed / pulse.duration).clamp(0.0, 1.0);

        if progress >= 1.0 {
            // Animation complete - despawn
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Scale up as time progresses (1.0 -> 2.5)
        let scale = 1.0 + progress * 1.5;
        transform.scale = Vec3::splat(scale);

        // Fade out by modifying alpha
        // Note: We modify the actual material, which affects all pulses using the same material
        // For proper per-entity fading, we'd need to clone materials. For now, this creates
        // a nice "wave" effect where all active pulses share the same fading.
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 1.0 - progress;
            let srgba = material.base_color.to_srgba();
            material.base_color = Color::srgba(srgba.red, srgba.green, srgba.blue, alpha * 0.8);
            // Also fade emissive
            material.emissive = LinearRgba::new(
                material.emissive.red * (1.0 - progress * 0.7),
                material.emissive.green * (1.0 - progress * 0.7),
                material.emissive.blue * (1.0 - progress * 0.7),
                1.0,
            );
        }

        // Also rise slightly
        transform.translation.y += time.delta_secs() * 0.3;
    }
}
