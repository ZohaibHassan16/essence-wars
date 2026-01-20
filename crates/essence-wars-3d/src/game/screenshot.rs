//! Screenshot capture functionality for headless mode.
//!
//! This module provides screenshot capture capability for visual debugging
//! and inspection of game state. Screenshots can be triggered by:
//! - Specific turn numbers
//! - Time intervals
//! - Game events (turn_start, combat, game_over)

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use super::turn_loop::GameEventWrapper;
use super::{AppState, GameBridge, HeadlessStats};
use crate::CliArgs;

/// Plugin for screenshot capture functionality.
pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenshotConfig>()
            .init_resource::<ScreenshotState>()
            .add_systems(Startup, setup_screenshot_config)
            .add_systems(OnEnter(AppState::Playing), reset_screenshot_state)
            // Run screenshot systems after game events are dispatched
            // Using after() to ensure events are available when we check for them
            .add_systems(
                Update,
                (
                    screenshot_on_turn_start,
                    screenshot_on_combat,
                    screenshot_on_interval,
                    screenshot_on_game_end,
                )
                    .run_if(in_state(AppState::Playing))
                    .after(super::turn_loop::dispatch_game_events),
            );
    }
}

/// Screenshot event types that can trigger captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenshotEvent {
    TurnStart,
    Combat,
    GameOver,
}

impl ScreenshotEvent {
    /// Parse from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "turn_start" | "turnstart" => Some(Self::TurnStart),
            "combat" => Some(Self::Combat),
            "game_over" | "gameover" => Some(Self::GameOver),
            _ => None,
        }
    }
}

/// Configuration for screenshot capture, parsed from CLI args.
#[derive(Resource, Default)]
pub struct ScreenshotConfig {
    /// Whether screenshot capture is enabled.
    pub enabled: bool,
    /// Output directory for screenshots.
    pub output_dir: PathBuf,
    /// Specific turns to capture.
    pub capture_turns: HashSet<u16>,
    /// Capture interval in seconds (None = disabled).
    pub capture_interval: Option<f32>,
    /// Events that trigger capture.
    pub capture_events: HashSet<ScreenshotEvent>,
}


/// Runtime state for screenshot capture.
#[derive(Resource, Default)]
pub struct ScreenshotState {
    /// Current game number (1-indexed).
    pub game_num: usize,
    /// Turns that have been captured in current game.
    pub captured_turns: HashSet<u16>,
    /// Time of last interval capture.
    pub last_interval_capture: Option<Instant>,
    /// Game start time (for elapsed time in filenames).
    pub game_start: Option<Instant>,
    /// All screenshot paths captured (for JSON output).
    pub screenshot_paths: Vec<String>,
    /// Current turn number (updated by events).
    pub current_turn: u16,
    /// Time when the last screenshot was requested (for exit delay).
    pub last_screenshot_time: Option<Instant>,
}

impl ScreenshotState {
    /// Reset state for a new game.
    pub fn reset_for_new_game(&mut self) {
        self.game_num += 1;
        self.captured_turns.clear();
        self.last_interval_capture = None;
        self.game_start = Some(Instant::now());
        self.current_turn = 0;
    }

    /// Get elapsed seconds since game start.
    pub fn elapsed_secs(&self) -> u64 {
        self.game_start
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    /// Generate filename for a screenshot.
    pub fn generate_filename(&self, event: &str, output_dir: &Path) -> PathBuf {
        let filename = format!(
            "game_{}_turn_{}_{}_{}s.png",
            self.game_num,
            self.current_turn,
            event,
            self.elapsed_secs()
        );
        output_dir.join(filename)
    }

    /// Mark that a screenshot was just requested.
    pub fn mark_screenshot_requested(&mut self) {
        self.last_screenshot_time = Some(Instant::now());
    }

    /// Check if we need to wait for screenshots to complete.
    /// Returns true if a screenshot was recently requested and we should delay exit.
    pub fn needs_screenshot_delay(&self) -> bool {
        const SCREENSHOT_DELAY_MS: u128 = 500; // Wait 500ms for screenshot to complete
        self.last_screenshot_time
            .map(|t| t.elapsed().as_millis() < SCREENSHOT_DELAY_MS)
            .unwrap_or(false)
    }
}

/// System to initialize screenshot config from CLI args.
fn setup_screenshot_config(
    cli_args: Option<Res<CliArgs>>,
    mut config: ResMut<ScreenshotConfig>,
) {
    let Some(args) = cli_args else {
        return;
    };

    // Check if any screenshot options are configured
    let has_turns = args.screenshot_turns.as_ref().is_some_and(|v| !v.is_empty());
    let has_interval = args.screenshot_interval.is_some();
    let has_events = args.screenshot_events.as_ref().is_some_and(|v| !v.is_empty());

    if !has_turns && !has_interval && !has_events {
        config.enabled = false;
        return;
    }

    config.enabled = true;
    config.output_dir = PathBuf::from(&args.screenshot_dir);

    // Parse capture turns
    if let Some(turns) = &args.screenshot_turns {
        config.capture_turns = turns.iter().copied().collect();
    }

    // Parse capture interval
    config.capture_interval = args.screenshot_interval;

    // Parse capture events
    if let Some(events) = &args.screenshot_events {
        for event_str in events {
            if let Some(event) = ScreenshotEvent::from_str(event_str) {
                config.capture_events.insert(event);
            } else {
                warn!("Unknown screenshot event: {}", event_str);
            }
        }
    }

    // Create output directory if needed
    if config.enabled && !config.output_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
            error!("Failed to create screenshot directory {:?}: {}", config.output_dir, e);
            config.enabled = false;
        } else {
            info!("Created screenshot directory: {:?}", config.output_dir);
        }
    }

    if config.enabled {
        info!(
            "Screenshot capture enabled: turns={:?}, interval={:?}, events={:?}",
            config.capture_turns, config.capture_interval, config.capture_events
        );
    }
}

/// System to reset screenshot state when starting a new game.
fn reset_screenshot_state(
    config: Res<ScreenshotConfig>,
    mut state: ResMut<ScreenshotState>,
    headless_stats: Option<Res<HeadlessStats>>,
) {
    if !config.enabled {
        return;
    }

    // Sync game number with headless stats if available
    if let Some(stats) = headless_stats {
        state.game_num = stats.games_played;
    }

    state.reset_for_new_game();
    info!("Screenshot state reset for game {}", state.game_num);
}

/// System to capture screenshots on turn start events.
fn screenshot_on_turn_start(
    mut commands: Commands,
    config: Res<ScreenshotConfig>,
    mut state: ResMut<ScreenshotState>,
    mut event_reader: EventReader<GameEventWrapper>,
) {
    if !config.enabled {
        return;
    }

    for GameEventWrapper(event) in event_reader.read() {
        if let cardgame::client_api::GameEvent::TurnStarted { turn_number, .. } = event {
            // Update current turn
            state.current_turn = *turn_number;

            // Check if we should capture on turn_start event
            let should_capture_event = config.capture_events.contains(&ScreenshotEvent::TurnStart);

            // Check if this specific turn should be captured
            let should_capture_turn = config.capture_turns.contains(turn_number)
                && !state.captured_turns.contains(turn_number);

            if should_capture_event || should_capture_turn {
                let path = state.generate_filename("turn_start", &config.output_dir);
                info!("Capturing turn start screenshot: {:?}", path);

                // Mark turn as captured
                state.captured_turns.insert(*turn_number);

                // Store path for JSON output
                if let Some(path_str) = path.to_str() {
                    state.screenshot_paths.push(path_str.to_string());
                }

                // Mark screenshot time for exit delay
                state.mark_screenshot_requested();

                // Spawn screenshot capture
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));
            }
        }
    }
}

/// System to capture screenshots on combat events.
fn screenshot_on_combat(
    mut commands: Commands,
    config: Res<ScreenshotConfig>,
    mut state: ResMut<ScreenshotState>,
    mut event_reader: EventReader<GameEventWrapper>,
) {
    if !config.enabled || !config.capture_events.contains(&ScreenshotEvent::Combat) {
        return;
    }

    for GameEventWrapper(event) in event_reader.read() {
        // Combat events include combat start, creature damage, and death
        match event {
            cardgame::client_api::GameEvent::CombatStarted { .. }
            | cardgame::client_api::GameEvent::CreatureDamaged { .. }
            | cardgame::client_api::GameEvent::CreatureDied { .. } => {
                let path = state.generate_filename("combat", &config.output_dir);
                info!("Capturing combat screenshot: {:?}", path);

                // Store path for JSON output
                if let Some(path_str) = path.to_str() {
                    state.screenshot_paths.push(path_str.to_string());
                }

                // Mark screenshot time for exit delay
                state.mark_screenshot_requested();

                // Spawn screenshot capture
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));

                // Only capture one combat screenshot per frame to avoid spam
                break;
            }
            _ => {}
        }
    }
}

/// System to capture screenshots at regular intervals.
fn screenshot_on_interval(
    mut commands: Commands,
    config: Res<ScreenshotConfig>,
    mut state: ResMut<ScreenshotState>,
    bridge: Res<GameBridge>,
) {
    if !config.enabled {
        return;
    }

    let Some(interval) = config.capture_interval else {
        return;
    };

    // Get current time
    let now = Instant::now();

    // Check if enough time has passed since last capture
    let should_capture = match state.last_interval_capture {
        None => {
            // First capture - wait a moment for the scene to be ready
            state.game_start.is_some_and(|start| start.elapsed().as_secs_f32() >= 0.5)
        }
        Some(last) => last.elapsed().as_secs_f32() >= interval,
    };

    if !should_capture {
        return;
    }

    // Update current turn from game state
    if let Some(client) = &bridge.client {
        if let Some(game_state) = client.get_state() {
            state.current_turn = game_state.current_turn;
        }
    }

    let path = state.generate_filename("interval", &config.output_dir);
    info!("Capturing interval screenshot: {:?}", path);

    state.last_interval_capture = Some(now);

    // Store path for JSON output
    if let Some(path_str) = path.to_str() {
        state.screenshot_paths.push(path_str.to_string());
    }

    // Mark screenshot time for exit delay
    state.mark_screenshot_requested();

    // Spawn screenshot capture
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

/// System to capture screenshot when game ends (via GameEnded event).
fn screenshot_on_game_end(
    mut commands: Commands,
    config: Res<ScreenshotConfig>,
    mut state: ResMut<ScreenshotState>,
    mut event_reader: EventReader<GameEventWrapper>,
) {
    if !config.enabled || !config.capture_events.contains(&ScreenshotEvent::GameOver) {
        return;
    }

    for GameEventWrapper(event) in event_reader.read() {
        if let cardgame::client_api::GameEvent::GameEnded { final_turn, .. } = event {
            state.current_turn = *final_turn;

            let path = state.generate_filename("game_over", &config.output_dir);
            info!("Capturing game over screenshot: {:?}", path);

            // Store path for JSON output
            if let Some(path_str) = path.to_str() {
                state.screenshot_paths.push(path_str.to_string());
            }

            // Mark screenshot time for exit delay
            state.mark_screenshot_requested();

            // Spawn screenshot capture
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
    }
}
