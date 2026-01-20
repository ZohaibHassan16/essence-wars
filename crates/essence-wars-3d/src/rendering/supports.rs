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
                (sync_supports_with_game_state, process_support_events)
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

/// Resource storing support mesh and material handles.
#[derive(Resource)]
pub struct SupportAssets {
    pub mesh: Handle<Mesh>,
    pub material_p1: Handle<StandardMaterial>,
    pub material_p2: Handle<StandardMaterial>,
    pub low_durability_material_p1: Handle<StandardMaterial>,
    pub low_durability_material_p2: Handle<StandardMaterial>,
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

    commands.insert_resource(SupportAssets {
        mesh,
        material_p1,
        material_p2,
        low_durability_material_p1,
        low_durability_material_p2,
    });

    info!("Support assets initialized");
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
) {
    for entity in supports.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("All supports despawned");
}
