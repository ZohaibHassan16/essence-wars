//! Scene lighting configuration.
//!
//! Creates the "Farsight Table in dark void" aesthetic:
//! - Dark void background (near-black clear color)
//! - Dim ambient light with cool blue tint
//! - Main directional light for shadows
//! - Rim light for dramatic edge highlights

use bevy::prelude::*;

/// Plugin for scene lighting.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting);
    }
}

/// Set up scene lighting for the Farsight Table aesthetic.
fn setup_lighting(mut commands: Commands) {
    // Dark void background - near-black with subtle blue tint
    commands.insert_resource(ClearColor(Color::srgba(0.01, 0.01, 0.03, 1.0)));

    // Main directional light - provides shadows and primary illumination
    // Positioned front-upper-right to light the table surface
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.95), // Warm white
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 6.0,
            0.0,
        )),
    ));

    // Rim light - creates dramatic edge highlights on creatures and table
    // Positioned back-upper-left, cool purple tint
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: false,
            color: Color::srgb(0.5, 0.45, 0.65), // Cool purple tint
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            std::f32::consts::PI / 3.0,       // Pitch down from behind
            -std::f32::consts::PI * 0.75,     // Behind-left
            0.0,
        )),
    ));

    // Dim ambient light - dark void with subtle blue fill
    // Table's emissive materials become the primary light source
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.15, 0.15, 0.25), // Dark blue tint
        brightness: 20.0,                      // Very dim (down from 200)
    });

    info!("Farsight Table lighting configured (dark void aesthetic)");
}
