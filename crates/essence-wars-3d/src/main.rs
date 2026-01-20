//! Essence Wars 3D - Bevy client with Glassbox AI visualization.
//!
//! This client provides a 3D view of the card game with MCTS decision
//! tree visualization for AI transparency.
//!
//! # CLI Arguments (native only)
//!
//! ```bash
//! # Interactive mode (default)
//! cargo run --release -p essence-wars-3d
//!
//! # Headless mode - auto-start AI vs AI game
//! cargo run --release -p essence-wars-3d -- --headless
//!
//! # With custom decks and seed
//! cargo run --release -p essence-wars-3d -- --headless --deck1 colossus_wall --deck2 broodmother_swarm --seed 42
//!
//! # Human vs AI mode
//! cargo run --release -p essence-wars-3d -- --headless --human
//! ```

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod game;
mod rendering;
mod ui;
mod glassbox;

// Native-only CLI
#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

/// CLI arguments for native builds.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser, Debug, Clone, Resource)]
#[command(name = "essence-wars-3d")]
#[command(about = "3D card game with AI visualization")]
pub struct CliArgs {
    /// Run in headless mode (auto-start game, skip menu)
    #[arg(long)]
    pub headless: bool,

    /// Player 1 deck ID
    #[arg(long, default_value = "colossus_wall")]
    pub deck1: String,

    /// Player 2 deck ID
    #[arg(long, default_value = "broodmother_swarm")]
    pub deck2: String,

    /// Random seed
    #[arg(long, default_value = "42")]
    pub seed: u64,

    /// Human player mode (player 1 is human)
    #[arg(long)]
    pub human: bool,

    /// Fast mode - skip visual delays for quick testing
    #[arg(long)]
    pub fast: bool,

    /// Number of games to play (benchmark mode)
    #[arg(long, default_value = "1")]
    pub games: usize,

    /// Output results as JSON (for benchmarking)
    #[arg(long)]
    pub json: bool,

    /// Debug logging - print each action like arena
    #[arg(long)]
    pub debug: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for CliArgs {
    fn default() -> Self {
        Self {
            headless: false,
            deck1: "colossus_wall".to_string(),
            deck2: "broodmother_swarm".to_string(),
            seed: 42,
            human: false,
            fast: false,
            games: 1,
            json: false,
            debug: false,
        }
    }
}

// WASM doesn't have CLI args
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Resource, Default)]
pub struct CliArgs {
    pub headless: bool,
    pub deck1: String,
    pub deck2: String,
    pub seed: u64,
    pub human: bool,
    pub fast: bool,
    pub games: usize,
    pub json: bool,
    pub debug: bool,
}

// WASM-specific imports
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Call JavaScript to hide the loading screen (WASM only)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn hideLoadingScreen();

    #[wasm_bindgen(js_namespace = window)]
    fn updateLoadingProgress(text: &str);

    // Direct console.log access
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// WASM entry point
#[cfg(target_arch = "wasm32")]
fn setup_wasm() {
    // Set up better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Log to browser console
    log("[Essence Wars] WASM module starting...");

    // Update loading progress
    updateLoadingProgress("Initializing game engine...");
}

/// System to hide loading screen after first frame renders (WASM only)
#[cfg(target_arch = "wasm32")]
fn hide_loading_after_startup(mut ran: Local<bool>) {
    if !*ran {
        *ran = true;
        hideLoadingScreen();
    }
}

fn main() {
    // Parse CLI args (native only)
    #[cfg(not(target_arch = "wasm32"))]
    let cli_args = CliArgs::parse();

    #[cfg(target_arch = "wasm32")]
    let cli_args = CliArgs::default();

    // Initialize headless stats for multi-game runs (native only)
    #[cfg(not(target_arch = "wasm32"))]
    let headless_stats = if cli_args.headless {
        Some(game::HeadlessStats::new(cli_args.games))
    } else {
        None
    };

    // Very first thing - log that main() was called (try multiple methods)
    #[cfg(target_arch = "wasm32")]
    {
        // Method 1: Our extern log
        log("[Essence Wars] main() called! (extern)");

        // Method 2: js_sys::eval as fallback
        let _ = js_sys::eval("console.log('[Essence Wars] main() called! (js_sys::eval)')");

        // Method 3: web_sys
        web_sys::console::log_1(&"[Essence Wars] main() called! (web_sys)".into());
    }

    // WASM-specific initialization
    #[cfg(target_arch = "wasm32")]
    setup_wasm();

    #[cfg(target_arch = "wasm32")]
    log("[Essence Wars] Creating Bevy App...");

    let mut app = App::new();

    // Insert CLI args as a resource
    app.insert_resource(cli_args);

    // Configure window differently for native vs web
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Essence Wars - Glassbox Mode".into(),
                resolution: (1600., 900.).into(),
                ..default()
            }),
            ..default()
        }));
    }

    #[cfg(target_arch = "wasm32")]
    {
        log("[Essence Wars] Configuring Bevy for WASM...");
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Essence Wars - Glassbox Mode".into(),
                // Let the canvas fill the browser window
                fit_canvas_to_parent: true,
                // Prevent default browser behavior on right-click etc.
                prevent_default_event_handling: true,
                // Let Bevy create the canvas (it will append to body)
                ..default()
            }),
            ..default()
        }));
        log("[Essence Wars] Bevy plugins configured");
    }

    // Insert headless stats if in headless mode (native only)
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(stats) = headless_stats {
        app.insert_resource(stats);
    }

    app.add_plugins(EguiPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(glassbox::GlassboxPlugin);

    // Add system to hide loading screen after startup (WASM only)
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, hide_loading_after_startup);

    #[cfg(target_arch = "wasm32")]
    log("[Essence Wars] Starting Bevy app.run()...");

    app.run();

    // This won't be reached in WASM since app.run() takes over
    #[cfg(target_arch = "wasm32")]
    log("[Essence Wars] app.run() returned (unexpected in WASM)");
}
