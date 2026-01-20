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
