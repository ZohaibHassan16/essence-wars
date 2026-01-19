//! Creature visualization and management.

use bevy::prelude::*;
use crate::game::AppState;

/// Plugin for creature rendering.
pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::Playing), despawn_all_creatures);
    }
}

/// Component marking a creature entity.
#[allow(dead_code)]
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
#[allow(dead_code)]
#[derive(Resource)]
pub struct CreatureAssets {
    pub mesh: Handle<Mesh>,
    pub material_p1: Handle<StandardMaterial>,
    pub material_p2: Handle<StandardMaterial>,
}

/// Spawn a creature at the given slot.
#[allow(dead_code)]
pub fn spawn_creature(
    commands: &mut Commands,
    assets: &CreatureAssets,
    creature: Creature3D,
) -> Entity {
    let x = (creature.slot as f32 - 2.0) * 2.0;
    let z = if creature.owner == 0 { 2.0 } else { -2.0 };

    let material = if creature.owner == 0 {
        assets.material_p1.clone()
    } else {
        assets.material_p2.clone()
    };

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(x, 0.5, z),
        creature,
    )).id()
}

/// Despawn all creatures when leaving game.
fn despawn_all_creatures(
    mut commands: Commands,
    creatures: Query<Entity, With<Creature3D>>,
) {
    for entity in creatures.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Update creature position based on slot assignment.
#[allow(dead_code)]
pub fn update_creature_position(
    transform: &mut Transform,
    creature: &Creature3D,
) {
    let x = (creature.slot as f32 - 2.0) * 2.0;
    let z = if creature.owner == 0 { 2.0 } else { -2.0 };
    transform.translation = Vec3::new(x, 0.5, z);
}
