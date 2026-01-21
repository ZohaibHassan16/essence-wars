//! 3D rendering module for the game board, creatures, and supports.

mod board;
pub mod camera;
mod combat;
mod creatures;
mod crystal_nodes;
mod lighting;
pub mod meshes;
pub mod parallax_material;
mod supports;

pub use board::BoardPlugin;
pub use camera::{CameraPlugin, GameCamera};
pub use combat::CombatPlugin;
pub use creatures::{Creature3D, CreaturePlugin};
pub use crystal_nodes::CrystalNodePlugin;
pub use lighting::LightingPlugin;
pub use parallax_material::{CardTextureCache, ParallaxCardMaterial, ParallaxMaterialPlugin};
pub use supports::{Support3D, SupportPlugin};

use bevy::prelude::*;

/// Combined rendering plugin that includes all rendering subsystems.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoardPlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(CombatPlugin)
            .add_plugins(CreaturePlugin)
            .add_plugins(CrystalNodePlugin)
            .add_plugins(LightingPlugin)
            .add_plugins(ParallaxMaterialPlugin)
            .add_plugins(SupportPlugin);
    }
}
