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

    info!("Game board spawned");
}

/// Despawn the game board when leaving playing state.
fn despawn_board(
    mut commands: Commands,
    board_query: Query<Entity, With<GameBoard>>,
    slot_query: Query<Entity, With<CreatureSlot>>,
) {
    for entity in board_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in slot_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("Game board despawned");
}
