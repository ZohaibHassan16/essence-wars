//! Essence Wars 3D - Bevy client with Glassbox AI visualization.
//!
//! This client provides a 3D view of the card game with MCTS decision
//! tree visualization for AI transparency.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod game;
mod rendering;
mod ui;
mod glassbox;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Essence Wars - Glassbox Mode".into(),
                resolution: (1600., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(glassbox::GlassboxPlugin)
        .run();
}
