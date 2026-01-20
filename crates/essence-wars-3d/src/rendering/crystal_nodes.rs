//! Crystal node visualization.
//!
//! Crystal nodes are small glowing gems at each creature slot position.
//! They serve as visual anchors for the "holographic projection" aesthetic
//! and pulse gently to indicate spawn points.

use bevy::prelude::*;
use cardgame::decks::Faction;

use crate::game::{AppState, GameBridge};

/// Plugin for crystal node rendering.
pub struct CrystalNodePlugin;

impl Plugin for CrystalNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_crystal_nodes)
            .add_systems(
                Update,
                (animate_crystal_nodes, animate_slot_lights).run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), despawn_crystal_nodes);
    }
}

/// Component marking a crystal node entity.
#[derive(Component)]
pub struct CrystalNode {
    /// Owner player (0 or 1)
    pub player: usize,
    /// Slot index (0-4)
    pub slot: usize,
    /// Base emissive intensity for pulsing
    pub base_emissive: LinearRgba,
}

/// Component marking a slot point light entity.
#[derive(Component)]
pub struct SlotLight {
    /// Owner player (0 or 1)
    pub player: usize,
    /// Slot index (0-4)
    pub slot: usize,
    /// Base light color for pulsing
    pub base_color: Color,
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

/// Create faction-colored crystal node material.
fn create_node_material(faction: Option<Faction>, is_player1: bool) -> (StandardMaterial, LinearRgba) {
    let (base_color, emissive) = match (faction, is_player1) {
        // Argentum: Amber crystal
        (Some(Faction::Argentum), true) => (
            Color::srgba(0.8, 0.6, 0.2, 0.8),
            LinearRgba::new(0.4, 0.3, 0.1, 1.0),
        ),
        (Some(Faction::Argentum), false) => (
            Color::srgba(0.7, 0.5, 0.15, 0.7),
            LinearRgba::new(0.3, 0.22, 0.08, 1.0),
        ),
        // Symbiote: Green bioluminescent
        (Some(Faction::Symbiote), true) => (
            Color::srgba(0.2, 0.7, 0.3, 0.8),
            LinearRgba::new(0.1, 0.4, 0.15, 1.0),
        ),
        (Some(Faction::Symbiote), false) => (
            Color::srgba(0.15, 0.6, 0.25, 0.7),
            LinearRgba::new(0.08, 0.3, 0.1, 1.0),
        ),
        // Obsidion: Crimson crystal
        (Some(Faction::Obsidion), true) => (
            Color::srgba(0.6, 0.1, 0.2, 0.8),
            LinearRgba::new(0.4, 0.08, 0.15, 1.0),
        ),
        (Some(Faction::Obsidion), false) => (
            Color::srgba(0.5, 0.08, 0.15, 0.7),
            LinearRgba::new(0.3, 0.05, 0.1, 1.0),
        ),
        // Default: Blue/purple crystal
        (_, true) => (
            Color::srgba(0.3, 0.4, 0.8, 0.8),
            LinearRgba::new(0.15, 0.2, 0.4, 1.0),
        ),
        (_, false) => (
            Color::srgba(0.6, 0.3, 0.5, 0.7),
            LinearRgba::new(0.3, 0.15, 0.25, 1.0),
        ),
    };

    let material = StandardMaterial {
        base_color,
        metallic: 0.7,
        perceptual_roughness: 0.2,
        emissive,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };

    (material, emissive)
}

/// Get light color for a faction.
fn get_light_color(faction: Option<Faction>, is_player1: bool) -> Color {
    match (faction, is_player1) {
        (Some(Faction::Argentum), _) => Color::srgb(1.0, 0.85, 0.5),   // Warm amber
        (Some(Faction::Symbiote), _) => Color::srgb(0.4, 1.0, 0.6),    // Bioluminescent green
        (Some(Faction::Obsidion), _) => Color::srgb(1.0, 0.3, 0.4),    // Crimson
        (_, true) => Color::srgb(0.5, 0.7, 1.0),                        // Blue for P1
        (_, false) => Color::srgb(1.0, 0.5, 0.5),                       // Red for P2
    }
}

/// Spawn crystal nodes at each creature slot position.
fn spawn_crystal_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bridge: Res<GameBridge>,
) {
    // Create a small octahedron mesh for the crystal nodes
    let node_mesh = meshes.add(
        Mesh::from(
            bevy::math::primitives::Tetrahedron::default()
        )
        .scaled_by(Vec3::new(0.15, 0.25, 0.15))
    );

    // Get factions
    let faction_p1 = get_player_faction(&bridge, 0);
    let faction_p2 = get_player_faction(&bridge, 1);

    // Create materials
    let (mat_p1, emissive_p1) = create_node_material(faction_p1, true);
    let (mat_p2, emissive_p2) = create_node_material(faction_p2, false);
    let material_p1 = materials.add(mat_p1);
    let material_p2 = materials.add(mat_p2);

    // Get light colors
    let light_color_p1 = get_light_color(faction_p1, true);
    let light_color_p2 = get_light_color(faction_p2, false);

    // Spawn nodes and lights for Player 1 (front row, z = 2.0)
    for slot in 0..5 {
        let x = (slot as f32 - 2.0) * 2.0;

        // Crystal node
        commands.spawn((
            Mesh3d(node_mesh.clone()),
            MeshMaterial3d(material_p1.clone()),
            Transform::from_xyz(x, 0.2, 2.0),
            CrystalNode {
                player: 0,
                slot,
                base_emissive: emissive_p1,
            },
        ));

        // Point light at slot position (slightly above node)
        commands.spawn((
            PointLight {
                color: light_color_p1,
                intensity: 800.0, // Dim but noticeable
                radius: 2.0,
                range: 3.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(x, 0.5, 2.0),
            SlotLight {
                player: 0,
                slot,
                base_color: light_color_p1,
            },
        ));
    }

    // Spawn nodes and lights for Player 2 (back row, z = -2.0)
    for slot in 0..5 {
        let x = (slot as f32 - 2.0) * 2.0;

        // Crystal node
        commands.spawn((
            Mesh3d(node_mesh.clone()),
            MeshMaterial3d(material_p2.clone()),
            Transform::from_xyz(x, 0.2, -2.0),
            CrystalNode {
                player: 1,
                slot,
                base_emissive: emissive_p2,
            },
        ));

        // Point light at slot position
        commands.spawn((
            PointLight {
                color: light_color_p2,
                intensity: 800.0,
                radius: 2.0,
                range: 3.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(x, 0.5, -2.0),
            SlotLight {
                player: 1,
                slot,
                base_color: light_color_p2,
            },
        ));
    }

    info!("Crystal nodes and slot lights spawned (10 nodes, 10 lights)");
}

/// Animate crystal nodes with pulsing glow effect.
fn animate_crystal_nodes(
    time: Res<Time>,
    nodes: Query<(&CrystalNode, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    let pulse_speed = 2.0;

    for (node, material_handle) in nodes.iter() {
        // Each node has unique phase based on slot
        let phase = node.slot as f32 * 0.8 + node.player as f32 * 1.5;
        let pulse = 0.7 + 0.3 * (t * pulse_speed + phase).sin();

        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.emissive = LinearRgba::new(
                node.base_emissive.red * pulse,
                node.base_emissive.green * pulse,
                node.base_emissive.blue * pulse,
                1.0,
            );
        }
    }
}

/// Animate slot lights with pulsing intensity in sync with crystal nodes.
fn animate_slot_lights(
    time: Res<Time>,
    mut lights: Query<(&SlotLight, &mut PointLight)>,
) {
    let t = time.elapsed_secs();
    let pulse_speed = 2.0;
    let base_intensity = 800.0;

    for (slot_light, mut point_light) in lights.iter_mut() {
        // Each light pulses with unique phase based on slot
        let phase = slot_light.slot as f32 * 0.8 + slot_light.player as f32 * 1.5;
        let pulse = 0.6 + 0.4 * (t * pulse_speed + phase).sin();

        point_light.intensity = base_intensity * pulse;
    }
}

/// Despawn all crystal nodes and slot lights when leaving game.
fn despawn_crystal_nodes(
    mut commands: Commands,
    nodes: Query<Entity, With<CrystalNode>>,
    lights: Query<Entity, With<SlotLight>>,
) {
    for entity in nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in lights.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("Crystal nodes and slot lights despawned");
}
