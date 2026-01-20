//! 3D rendering module for the game board, creatures, and supports.

mod board;
mod camera;
mod combat;
mod creatures;
mod lighting;
mod supports;

pub use board::BoardPlugin;
pub use camera::CameraPlugin;
pub use combat::CombatPlugin;
pub use creatures::CreaturePlugin;
pub use lighting::LightingPlugin;
pub use supports::SupportPlugin;

use bevy::prelude::*;

/// Combined rendering plugin that includes all rendering subsystems.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoardPlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(CombatPlugin)
            .add_plugins(CreaturePlugin)
            .add_plugins(LightingPlugin)
            .add_plugins(SupportPlugin);
    }
}
