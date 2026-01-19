//! Glassbox AI visualization module.
//!
//! Shows MCTS decision trees and action probabilities for AI transparency.

mod mcts_panel;
mod value_gauge;
mod action_probs;

pub use mcts_panel::MctsPanelPlugin;
pub use value_gauge::ValueGaugePlugin;
pub use action_probs::ActionProbsPlugin;

use bevy::prelude::*;

/// Combined glassbox visualization plugin.
pub struct GlassboxPlugin;

impl Plugin for GlassboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlassboxState>()
            .add_plugins(MctsPanelPlugin)
            .add_plugins(ValueGaugePlugin)
            .add_plugins(ActionProbsPlugin)
            .add_systems(Update, toggle_glassbox);
    }
}

/// Resource tracking glassbox display state.
#[derive(Resource, Default)]
pub struct GlassboxState {
    /// Whether glassbox visualizations are visible
    pub visible: bool,
}

/// System to toggle glassbox visibility with 'G' key.
fn toggle_glassbox(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GlassboxState>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        state.visible = !state.visible;
        info!("Glassbox visibility: {}", state.visible);
    }
}
