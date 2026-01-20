//! Application state management for the Bevy client.

use bevy::prelude::*;

use super::turn_loop::TurnLoopPlugin;
use crate::CliArgs;
use crate::ui::GameModeConfig;

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
            .add_plugins(TurnLoopPlugin)
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

/// System to check if loading is complete and transition to menu or auto-start.
fn check_loading_complete(
    cli_args: Option<Res<CliArgs>>,
    mut bridge: Option<ResMut<super::GameBridge>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_mode_config: ResMut<GameModeConfig>,
) {
    let Some(bridge) = bridge.as_mut() else {
        return;
    };

    // Check if we should auto-start in headless mode
    if let Some(args) = cli_args {
        if args.headless {
            // Set game mode
            *game_mode_config = if args.human {
                GameModeConfig::human_vs_ai()
            } else {
                GameModeConfig::spectator()
            };

            // Start the game with CLI args
            match bridge.start_game(&args.deck1, &args.deck2, args.seed) {
                Ok(()) => {
                    let mode_name = if args.human { "Human vs AI" } else { "AI vs AI" };
                    info!("Headless mode: Game auto-started ({}) - {} vs {} seed={}",
                          mode_name, args.deck1, args.deck2, args.seed);
                    next_state.set(AppState::Playing);
                    return;
                }
                Err(e) => {
                    error!("Failed to auto-start game: {}", e);
                    // Fall through to menu
                }
            }
        }
    }

    // Normal flow - go to menu
    info!("Loading complete, transitioning to menu");
    next_state.set(AppState::Menu);
}
