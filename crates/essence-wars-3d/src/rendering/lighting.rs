//! Scene lighting configuration.

use bevy::prelude::*;

/// Plugin for scene lighting.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting);
    }
}

/// Set up scene lighting.
fn setup_lighting(mut commands: Commands) {
    // Main directional light (sun-like)
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 6.0,
            0.0,
        )),
    ));

    // Ambient light for fill
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.9, 1.0),
        brightness: 200.0,
    });

    info!("Scene lighting configured");
}
