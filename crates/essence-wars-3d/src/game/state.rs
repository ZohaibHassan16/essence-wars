//! Application state management for the Bevy client.

use bevy::prelude::*;

/// Main application states for the game.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Initial loading and setup
    #[default]
    Loading,
    /// Main menu
    Menu,
    /// Game in progress
    Playing,
    /// Game has ended
    GameOver,
}

/// Plugin that manages game state and the cardgame engine integration.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(OnEnter(AppState::Loading), setup_game_resources)
            .add_systems(Update, check_loading_complete.run_if(in_state(AppState::Loading)));
    }
}

/// System to set up game resources during loading.
fn setup_game_resources(mut commands: Commands) {
    // Initialize game bridge
    commands.insert_resource(super::GameBridge::new());
    info!("Game resources initialized");
}

/// System to check if loading is complete and transition to menu.
fn check_loading_complete(
    bridge: Option<Res<super::GameBridge>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if bridge.is_some() {
        info!("Loading complete, transitioning to menu");
        next_state.set(AppState::Menu);
    }
}
