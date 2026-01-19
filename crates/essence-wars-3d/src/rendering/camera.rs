//! Camera controller for the 3D view.

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

/// Plugin for camera management.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_orbit, camera_zoom));
    }
}

/// Component marking the main game camera.
#[derive(Component)]
pub struct GameCamera {
    /// Distance from the focus point
    pub distance: f32,
    /// Horizontal rotation angle (radians)
    pub yaw: f32,
    /// Vertical rotation angle (radians)
    pub pitch: f32,
    /// Focus point the camera orbits around
    pub focus: Vec3,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            distance: 15.0,
            yaw: 0.0,
            pitch: std::f32::consts::PI / 4.0, // 45 degrees
            focus: Vec3::ZERO,
        }
    }
}

/// Spawn the main camera.
fn spawn_camera(mut commands: Commands) {
    let camera = GameCamera::default();
    let position = calculate_camera_position(&camera);

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(position).looking_at(camera.focus, Vec3::Y),
        camera,
    ));

    info!("Game camera spawned");
}

/// Calculate camera position from orbit parameters.
fn calculate_camera_position(camera: &GameCamera) -> Vec3 {
    let x = camera.distance * camera.yaw.sin() * camera.pitch.cos();
    let y = camera.distance * camera.pitch.sin();
    let z = camera.distance * camera.yaw.cos() * camera.pitch.cos();
    camera.focus + Vec3::new(x, y, z)
}

/// System to handle camera orbit with right mouse button.
fn camera_orbit(
    mut camera_query: Query<(&mut Transform, &mut GameCamera)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    if !mouse_button.pressed(MouseButton::Right) {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    for (mut transform, mut camera) in camera_query.iter_mut() {
        camera.yaw -= delta.x * 0.005;
        camera.pitch = (camera.pitch - delta.y * 0.005)
            .clamp(0.1, std::f32::consts::PI / 2.0 - 0.1);

        let position = calculate_camera_position(&camera);
        *transform = Transform::from_translation(position).looking_at(camera.focus, Vec3::Y);
    }
}

/// System to handle camera zoom with scroll wheel.
fn camera_zoom(
    mut camera_query: Query<(&mut Transform, &mut GameCamera)>,
    mut scroll: EventReader<MouseWheel>,
) {
    let mut scroll_amount = 0.0;
    for event in scroll.read() {
        scroll_amount += event.y;
    }

    if scroll_amount == 0.0 {
        return;
    }

    for (mut transform, mut camera) in camera_query.iter_mut() {
        camera.distance = (camera.distance - scroll_amount * 0.5).clamp(5.0, 30.0);
        let position = calculate_camera_position(&camera);
        *transform = Transform::from_translation(position).looking_at(camera.focus, Vec3::Y);
    }
}
