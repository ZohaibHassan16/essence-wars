//! UI module for game interface (HUD, hand display, menus).

mod hud;
mod hand;
mod menu;
mod player_input;

pub use hud::HudPlugin;
pub use hand::HandPlugin;
pub use menu::MenuPlugin;
pub use player_input::{GameModeConfig, PlayerInputPlugin, PlayerInputState};

use bevy::prelude::*;

/// Combined UI plugin.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HudPlugin)
            .add_plugins(HandPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(PlayerInputPlugin);
    }
}
