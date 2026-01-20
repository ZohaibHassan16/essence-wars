//! 3D game board rendering.

use bevy::prelude::*;
use crate::game::AppState;

/// Plugin for rendering the game board.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_board)
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

/// Spawn the game board.
fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Board base
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(12.0, 0.2, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.15, 0.1),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
        GameBoard,
    ));

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
