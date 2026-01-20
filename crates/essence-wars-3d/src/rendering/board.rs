//! 3D game board rendering.
//!
//! The board uses a "Farsight Table" aesthetic with faction-specific materials:
//! - **Argentum**: Polished dark wood with brass/metal inlays, amber glow
//! - **Symbiote**: Living wood with bioluminescent veins, green/purple glow
//! - **Obsidion**: Obsidian glass with crimson veins, purple/red glow

use bevy::prelude::*;
use cardgame::decks::Faction;
use crate::game::{AppState, DragState, DragType, GameBridge};

/// Plugin for rendering the game board.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), (spawn_board, setup_slot_materials))
            .add_systems(
                Update,
                highlight_slots_on_drag.run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), despawn_board);
    }
}

/// Component marking the game board entity.
#[derive(Component)]
pub struct GameBoard;

/// Component marking a creature slot on the board.
#[allow(dead_code)]
#[derive(Component)]
pub struct CreatureSlot {
    /// Player who owns this slot (0 or 1)
    pub player: usize,
    /// Slot index (0-4)
    pub index: usize,
}

/// Component marking a support slot on the board.
#[allow(dead_code)]
#[derive(Component)]
pub struct SupportSlot {
    /// Player who owns this slot (0 or 1)
    pub player: usize,
    /// Slot index (0-1)
    pub index: usize,
}

/// Component marking the lane divider.
#[derive(Component)]
pub struct LaneDivider;

/// Resource holding material handles for slot highlighting.
#[derive(Resource)]
pub struct SlotMaterials {
    /// Normal materials for creature slots (player 1, player 2)
    pub creature_normal: [Handle<StandardMaterial>; 2],
    /// Highlighted materials for creature slots (player 1, player 2)
    pub creature_highlight: [Handle<StandardMaterial>; 2],
    /// Occupied (red) materials for creature slots
    pub creature_occupied: Handle<StandardMaterial>,
    /// Normal materials for support slots (player 1, player 2)
    pub support_normal: [Handle<StandardMaterial>; 2],
    /// Highlighted materials for support slots (player 1, player 2)
    pub support_highlight: [Handle<StandardMaterial>; 2],
    /// Occupied (red) materials for support slots
    pub support_occupied: Handle<StandardMaterial>,
}

/// Get Player 1's faction from the current game.
fn get_player1_faction(bridge: &GameBridge) -> Option<Faction> {
    let deck_id = bridge.current_deck1.as_ref()?;
    let deck = bridge.deck_registry.get(deck_id)?;
    deck.faction()
}

/// Create faction-specific table material.
/// Emissive values boosted to serve as primary light source in dark void.
fn create_table_material(faction: Option<Faction>) -> StandardMaterial {
    match faction {
        Some(Faction::Argentum) => StandardMaterial {
            // Polished dark wood with brass undertones
            base_color: Color::srgb(0.15, 0.1, 0.05),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            emissive: LinearRgba::new(0.12, 0.08, 0.02, 1.0), // Warm amber glow (boosted)
            ..default()
        },
        Some(Faction::Symbiote) => StandardMaterial {
            // Living wood with bioluminescent undertones
            base_color: Color::srgb(0.08, 0.12, 0.06),
            metallic: 0.1,
            perceptual_roughness: 0.6,
            emissive: LinearRgba::new(0.04, 0.18, 0.08, 1.0), // Green bioluminescence (boosted)
            ..default()
        },
        Some(Faction::Obsidion) => StandardMaterial {
            // Obsidian glass with crimson veins
            base_color: Color::srgb(0.05, 0.02, 0.05),
            metallic: 0.7,
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.14, 0.02, 0.06, 1.0), // Deep crimson glow (boosted)
            ..default()
        },
        _ => StandardMaterial {
            // Default neutral table (warm brown)
            base_color: Color::srgb(0.2, 0.15, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.7,
            emissive: LinearRgba::new(0.06, 0.04, 0.02, 1.0), // Subtle warm glow
            ..default()
        },
    }
}

/// Create faction-specific edge trim material.
/// Edge trim has stronger emissive for prominent glow effect in dark void.
fn create_edge_material(faction: Option<Faction>) -> StandardMaterial {
    match faction {
        Some(Faction::Argentum) => StandardMaterial {
            // Brass/gold metallic trim
            base_color: Color::srgb(0.7, 0.5, 0.2),
            metallic: 0.9,
            perceptual_roughness: 0.3,
            emissive: LinearRgba::new(0.25, 0.18, 0.04, 1.0), // Boosted
            ..default()
        },
        Some(Faction::Symbiote) => StandardMaterial {
            // Bioluminescent vein accent
            base_color: Color::srgb(0.2, 0.5, 0.3),
            metallic: 0.2,
            perceptual_roughness: 0.5,
            emissive: LinearRgba::new(0.08, 0.35, 0.15, 1.0), // Boosted
            ..default()
        },
        Some(Faction::Obsidion) => StandardMaterial {
            // Crimson crystal accent
            base_color: Color::srgb(0.4, 0.05, 0.1),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            emissive: LinearRgba::new(0.35, 0.04, 0.10, 1.0), // Boosted
            ..default()
        },
        _ => StandardMaterial {
            // Default neutral edge
            base_color: Color::srgb(0.3, 0.25, 0.2),
            metallic: 0.4,
            perceptual_roughness: 0.5,
            emissive: LinearRgba::new(0.08, 0.06, 0.04, 1.0), // Subtle warm glow
            ..default()
        },
    }
}

/// Spawn the game board.
fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bridge: Res<GameBridge>,
) {
    // Determine faction from Player 1's deck
    let faction = get_player1_faction(&bridge);
    let faction_name = faction.map(|f| f.display_name()).unwrap_or("Neutral");
    info!("Spawning Farsight Table with {} theme", faction_name);

    // Board base (main table surface)
    let table_material = materials.add(create_table_material(faction));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(12.0, 0.2, 8.0))),
        MeshMaterial3d(table_material),
        Transform::from_xyz(0.0, -0.1, 0.0),
        GameBoard,
    ));

    // Edge trim (decorative border around the table)
    let edge_material = materials.add(create_edge_material(faction));
    let edge_mesh = meshes.add(Cuboid::new(12.4, 0.25, 0.2));
    let side_edge_mesh = meshes.add(Cuboid::new(0.2, 0.25, 8.0));

    // Front and back edges
    commands.spawn((
        Mesh3d(edge_mesh.clone()),
        MeshMaterial3d(edge_material.clone()),
        Transform::from_xyz(0.0, -0.075, 4.1),
        GameBoard,
    ));
    commands.spawn((
        Mesh3d(edge_mesh),
        MeshMaterial3d(edge_material.clone()),
        Transform::from_xyz(0.0, -0.075, -4.1),
        GameBoard,
    ));
    // Left and right edges
    commands.spawn((
        Mesh3d(side_edge_mesh.clone()),
        MeshMaterial3d(edge_material.clone()),
        Transform::from_xyz(-6.1, -0.075, 0.0),
        GameBoard,
    ));
    commands.spawn((
        Mesh3d(side_edge_mesh),
        MeshMaterial3d(edge_material),
        Transform::from_xyz(6.1, -0.075, 0.0),
        GameBoard,
    ));

    // Lane divider (glowing line between player rows)
    let divider_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.7, 0.7, 0.8, 0.4),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.15, 0.15, 0.2, 1.0), // Faint glow
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 0.05, 0.1))),
        MeshMaterial3d(divider_material.clone()),
        Transform::from_xyz(0.0, 0.05, 0.0),
        LaneDivider,
        GameBoard,
    ));

    // Vertical lane separators (between lanes 1-2, 2-3, 3-4, 4-5)
    // Creates faint grid overlay for spatial clarity
    let vertical_line_mesh = meshes.add(Cuboid::new(0.04, 0.03, 4.2));
    for i in 0..4 {
        let x = -3.0 + i as f32 * 2.0; // x = -3, -1, 1, 3
        commands.spawn((
            Mesh3d(vertical_line_mesh.clone()),
            MeshMaterial3d(divider_material.clone()),
            Transform::from_xyz(x, 0.03, 0.0),
            GameBoard,
        ));
    }

    // Creature slots for Player 1 (front row)
    let slot_material_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.4, 0.8, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Creature slots for Player 2 (back row)
    let slot_material_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.2, 0.2, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let slot_mesh = meshes.add(Cuboid::new(1.5, 0.1, 1.5));

    // Player 1 slots (5 slots)
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 2.0;
        commands.spawn((
            Mesh3d(slot_mesh.clone()),
            MeshMaterial3d(slot_material_p1.clone()),
            Transform::from_xyz(x, 0.05, 2.0),
            CreatureSlot { player: 0, index: i },
        ));
    }

    // Player 2 slots (5 slots)
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 2.0;
        commands.spawn((
            Mesh3d(slot_mesh.clone()),
            MeshMaterial3d(slot_material_p2.clone()),
            Transform::from_xyz(x, 0.05, -2.0),
            CreatureSlot { player: 1, index: i },
        ));
    }

    // Support slots (2 per player, positioned at the edges)
    // Design: "Floating Rune Plates" - using a different shape to distinguish from creature slots
    let support_slot_mesh = meshes.add(Cylinder::new(0.6, 0.15));

    // Support slot material (purple tones to distinguish from creature slots)
    let support_material_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.2, 0.6, 0.6),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.1, 0.05, 0.15, 1.0),
        ..default()
    });
    let support_material_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.2, 0.4, 0.6),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.15, 0.05, 0.1, 1.0),
        ..default()
    });

    // Player 1 support slots (left side of board)
    for i in 0..2 {
        let x = -5.5; // Left edge
        let z = 1.0 + i as f32 * 2.0; // Stacked vertically
        commands.spawn((
            Mesh3d(support_slot_mesh.clone()),
            MeshMaterial3d(support_material_p1.clone()),
            Transform::from_xyz(x, 0.1, z),
            SupportSlot { player: 0, index: i },
        ));
    }

    // Player 2 support slots (right side of board)
    for i in 0..2 {
        let x = 5.5; // Right edge
        let z = -1.0 - i as f32 * 2.0; // Stacked vertically (mirrored)
        commands.spawn((
            Mesh3d(support_slot_mesh.clone()),
            MeshMaterial3d(support_material_p2.clone()),
            Transform::from_xyz(x, 0.1, z),
            SupportSlot { player: 1, index: i },
        ));
    }

    info!("Game board spawned (5 creature slots + 2 support slots per player)");
}

/// Despawn the game board when leaving playing state.
fn despawn_board(
    mut commands: Commands,
    board_query: Query<Entity, With<GameBoard>>,
    creature_slot_query: Query<Entity, With<CreatureSlot>>,
    support_slot_query: Query<Entity, With<SupportSlot>>,
) {
    for entity in board_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in creature_slot_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in support_slot_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("Game board despawned");
}

/// Set up slot materials for highlighting.
fn setup_slot_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player 1 creature slot - normal (semi-transparent blue)
    let creature_normal_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.4, 0.8, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Player 2 creature slot - normal (semi-transparent red)
    let creature_normal_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.2, 0.2, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Player 1 creature slot - highlighted (brighter blue with glow)
    let creature_highlight_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.6, 1.0, 0.6),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.2, 0.4, 0.8, 1.0),
        ..default()
    });

    // Player 2 creature slot - highlighted (brighter red with glow)
    let creature_highlight_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.3, 0.3, 0.6),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.8, 0.2, 0.2, 1.0),
        ..default()
    });

    // Occupied slot (red warning)
    let creature_occupied = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.1, 0.1, 0.5),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.5, 0.0, 0.0, 1.0),
        ..default()
    });

    // Player 1 support slot - normal (semi-transparent purple)
    let support_normal_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.2, 0.6, 0.4),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.05, 0.02, 0.08, 1.0),
        ..default()
    });

    // Player 2 support slot - normal (semi-transparent magenta)
    let support_normal_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.2, 0.4, 0.4),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.08, 0.02, 0.05, 1.0),
        ..default()
    });

    // Player 1 support slot - highlighted (brighter purple with glow)
    let support_highlight_p1 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.3, 0.9, 0.7),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.3, 0.15, 0.5, 1.0),
        ..default()
    });

    // Player 2 support slot - highlighted
    let support_highlight_p2 = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.3, 0.6, 0.7),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.5, 0.15, 0.3, 1.0),
        ..default()
    });

    // Occupied support slot
    let support_occupied = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.1, 0.1, 0.5),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.5, 0.0, 0.0, 1.0),
        ..default()
    });

    commands.insert_resource(SlotMaterials {
        creature_normal: [creature_normal_p1, creature_normal_p2],
        creature_highlight: [creature_highlight_p1, creature_highlight_p2],
        creature_occupied,
        support_normal: [support_normal_p1, support_normal_p2],
        support_highlight: [support_highlight_p1, support_highlight_p2],
        support_occupied,
    });

    info!("Slot materials initialized for highlighting");
}

/// Highlight slots based on current drag state.
fn highlight_slots_on_drag(
    drag_state: Res<DragState>,
    slot_materials: Option<Res<SlotMaterials>>,
    mut creature_slots: Query<(&CreatureSlot, &mut MeshMaterial3d<StandardMaterial>)>,
    mut support_slots: Query<
        (&SupportSlot, &mut MeshMaterial3d<StandardMaterial>),
        Without<CreatureSlot>,
    >,
) {
    let Some(materials) = slot_materials else {
        return;
    };

    // Update creature slot materials
    for (slot, mut material) in creature_slots.iter_mut() {
        // Only highlight player 1's slots (the human player)
        if slot.player != 0 {
            // Reset player 2 slots to normal
            material.0 = materials.creature_normal[1].clone();
            continue;
        }

        let new_material = match drag_state.drag_type {
            DragType::Creature => {
                // Check if this slot is occupied
                if drag_state.occupied_slots[slot.index] {
                    materials.creature_occupied.clone()
                } else {
                    materials.creature_highlight[0].clone()
                }
            }
            _ => materials.creature_normal[0].clone(),
        };
        material.0 = new_material;
    }

    // Update support slot materials
    for (slot, mut material) in support_slots.iter_mut() {
        // Only highlight player 1's slots
        if slot.player != 0 {
            material.0 = materials.support_normal[1].clone();
            continue;
        }

        let new_material = match drag_state.drag_type {
            DragType::Support => {
                // Check if this slot is occupied
                if drag_state.occupied_supports[slot.index] {
                    materials.support_occupied.clone()
                } else {
                    materials.support_highlight[0].clone()
                }
            }
            _ => materials.support_normal[0].clone(),
        };
        material.0 = new_material;
    }
}
