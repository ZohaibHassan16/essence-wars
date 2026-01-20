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
}

/// WASM entry point
#[cfg(target_arch = "wasm32")]
fn setup_wasm() {
    // Set up better panic messages in the browser console
    console_error_panic_hook::set_once();

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
    // WASM-specific initialization
    #[cfg(target_arch = "wasm32")]
    setup_wasm();

    let mut app = App::new();

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
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Essence Wars - Glassbox Mode".into(),
                // Let the canvas fill the browser window
                fit_canvas_to_parent: true,
                // Prevent default browser behavior on right-click etc.
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }));
    }

    app.add_plugins(EguiPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(glassbox::GlassboxPlugin);

    // Add system to hide loading screen after startup (WASM only)
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, hide_loading_after_startup);

    app.run();
}
