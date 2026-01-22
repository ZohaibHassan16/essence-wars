//! HTTP server for screenshot capture and state synchronization.
//!
//! Runs alongside Tauri, exposing endpoints for MCP tools:
//! - `/screenshot` - Capture window screenshot as PNG
//! - `/health` - Health check
//! - `/sync_state` - Receive game state pushed from MCP server
//! - `/synced_state` - Get the current synced state (for frontend polling)
//!
//! This enables Claude Code to visually inspect the UI during autonomous development
//! and synchronize game state between MCP and Tauri UI.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;
use xcap::Window;

use crate::state::{GameEventDto, GameStateDto};

/// Default port for the screenshot server.
pub const DEFAULT_PORT: u16 = 9999;

/// Request to sync game state from MCP to Tauri.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateSyncRequest {
    /// The game state to sync.
    pub state: GameStateDto,
    /// Events that occurred (for animations).
    #[serde(default)]
    pub events: Vec<GameEventDto>,
    /// Timestamp in milliseconds since epoch.
    #[serde(default)]
    pub timestamp: u64,
}

/// Response from sync state endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    pub status: String,
    pub synced_at: u64,
}

/// Cached game state from MCP sync.
#[derive(Debug, Clone)]
pub struct CachedGameState {
    pub state: GameStateDto,
    pub events: Vec<GameEventDto>,
    pub synced_at: Instant,
    pub timestamp: u64,
}

/// State shared with Tauri to track the window title and synced game state.
pub struct ScreenshotState {
    /// The window title to capture (set by Tauri on startup).
    window_title: RwLock<Option<String>>,
    /// Cached game state from MCP (synced via HTTP POST).
    synced_game_state: RwLock<Option<CachedGameState>>,
}

impl ScreenshotState {
    /// Create new screenshot state.
    pub fn new() -> Self {
        Self {
            window_title: RwLock::new(None),
            synced_game_state: RwLock::new(None),
        }
    }

    /// Set the window title to capture.
    pub fn set_window_title(&self, title: String) {
        *self.window_title.write() = Some(title);
    }

    /// Get the current window title.
    pub fn get_window_title(&self) -> Option<String> {
        self.window_title.read().clone()
    }

    /// Store synced game state from MCP.
    pub fn set_synced_state(&self, state: GameStateDto, events: Vec<GameEventDto>, timestamp: u64) {
        *self.synced_game_state.write() = Some(CachedGameState {
            state,
            events,
            synced_at: Instant::now(),
            timestamp,
        });
    }

    /// Get the current synced game state.
    pub fn get_synced_state(&self) -> Option<CachedGameState> {
        self.synced_game_state.read().clone()
    }

    /// Clear the synced game state.
    pub fn clear_synced_state(&self) {
        *self.synced_game_state.write() = None;
    }
}

impl Default for ScreenshotState {
    fn default() -> Self {
        Self::new()
    }
}

/// Capture screenshot and return as PNG bytes.
async fn capture_screenshot(State(state): State<Arc<ScreenshotState>>) -> impl IntoResponse {
    let title = match state.get_window_title() {
        Some(t) => t,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Tauri window not registered",
            )
                .into_response()
        }
    };

    // Find window by title (run blocking operation in spawn_blocking)
    let result = tokio::task::spawn_blocking(move || capture_window_by_title(&title)).await;

    match result {
        Ok(Ok(png_bytes)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            png_bytes,
        )
            .into_response(),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task join error: {}", e),
        )
            .into_response(),
    }
}

/// Find and capture a window by title substring.
fn capture_window_by_title(title: &str) -> Result<Vec<u8>, String> {
    // Try xcap first (works on normal displays)
    match capture_with_xcap(title) {
        Ok(bytes) => return Ok(bytes),
        Err(xcap_err) => {
            eprintln!("xcap failed: {}, trying ImageMagick fallback...", xcap_err);
        }
    }

    // Fallback: use ImageMagick import (works in Xvfb headless mode)
    capture_with_imagemagick()
}

/// Capture window using xcap library.
fn capture_with_xcap(title: &str) -> Result<Vec<u8>, String> {
    // Enumerate all windows
    let windows = Window::all().map_err(|e| format!("Failed to enumerate windows: {}", e))?;

    // Find window containing the title
    let window = windows
        .into_iter()
        .find(|w| w.title().contains(title))
        .ok_or_else(|| format!("Window '{}' not found", title))?;

    // Capture the window
    let image = window
        .capture_image()
        .map_err(|e| format!("Screenshot failed: {}", e))?;

    // Convert to PNG bytes
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);

    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding failed: {}", e))?;

    Ok(buf)
}

/// Capture screen using ImageMagick import command.
/// This works in Xvfb headless mode where xcap can't find windows.
fn capture_with_imagemagick() -> Result<Vec<u8>, String> {
    use std::process::Command;

    // Create temp file for screenshot
    let temp_path = "/tmp/essence-wars-screenshot.png";

    // Run ImageMagick import command to capture root window
    let output = Command::new("import")
        .args(["-window", "root", temp_path])
        .output()
        .map_err(|e| format!("Failed to run ImageMagick import: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ImageMagick import failed: {}", stderr));
    }

    // Read the PNG file
    let bytes = std::fs::read(temp_path)
        .map_err(|e| format!("Failed to read screenshot file: {}", e))?;

    // Clean up temp file (ignore errors)
    let _ = std::fs::remove_file(temp_path);

    Ok(bytes)
}

/// Health check endpoint.
async fn health() -> &'static str {
    "ok"
}

/// Receive game state pushed from MCP server.
///
/// This allows MCP to sync its game state to Tauri so screenshots reflect
/// the current MCP game state.
async fn sync_state(
    State(state): State<Arc<ScreenshotState>>,
    Json(request): Json<GameStateSyncRequest>,
) -> impl IntoResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    state.set_synced_state(request.state, request.events, request.timestamp);

    eprintln!(
        "State synced from MCP (game: {}, turn: {})",
        state
            .get_synced_state()
            .map(|s| s.state.id.clone())
            .unwrap_or_default(),
        state
            .get_synced_state()
            .map(|s| s.state.turn)
            .unwrap_or(0)
    );

    Json(SyncResponse {
        status: "synced".to_string(),
        synced_at: now,
    })
}

/// Get the current synced game state.
///
/// Returns the most recent game state pushed from MCP, if any.
async fn get_synced_state(
    State(state): State<Arc<ScreenshotState>>,
) -> impl IntoResponse {
    match state.get_synced_state() {
        Some(cached) => {
            let age_ms = cached.synced_at.elapsed().as_millis() as u64;

            Json(serde_json::json!({
                "state": cached.state,
                "events": cached.events,
                "timestamp": cached.timestamp,
                "ageMs": age_ms
            }))
            .into_response()
        }
        None => (StatusCode::NOT_FOUND, "No synced state available").into_response(),
    }
}

/// Clear the synced game state.
async fn clear_synced_state(
    State(state): State<Arc<ScreenshotState>>,
) -> impl IntoResponse {
    state.clear_synced_state();
    Json(serde_json::json!({ "status": "cleared" }))
}

/// UI element description for click targeting.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElement {
    /// Unique element identifier
    pub id: String,
    /// Element type (hand_card, creature_slot, support_slot, button)
    pub element_type: String,
    /// Index within its category (e.g., hand index, slot number)
    pub index: Option<usize>,
    /// Side (player or opponent)
    pub side: Option<String>,
    /// Human-readable description
    pub description: String,
    /// Approximate screen region (for reference, not pixel-perfect)
    pub region: String,
}

/// Get list of UI elements for reference.
///
/// These are the logical UI elements in the game board, useful for
/// understanding what's visible in screenshots.
async fn list_elements() -> Json<Vec<UIElement>> {
    let mut elements = Vec::new();

    // Hand cards (player only - opponent hand is hidden)
    for i in 0..10 {
        elements.push(UIElement {
            id: format!("hand_{}", i),
            element_type: "hand_card".to_string(),
            index: Some(i),
            side: Some("player".to_string()),
            description: format!("Hand card {}", i),
            region: "bottom center".to_string(),
        });
    }

    // Player creature slots
    for i in 0..5 {
        elements.push(UIElement {
            id: format!("creature_player_{}", i),
            element_type: "creature_slot".to_string(),
            index: Some(i),
            side: Some("player".to_string()),
            description: format!("Player creature slot {}", i),
            region: "center-bottom".to_string(),
        });
    }

    // Opponent creature slots
    for i in 0..5 {
        elements.push(UIElement {
            id: format!("creature_opponent_{}", i),
            element_type: "creature_slot".to_string(),
            index: Some(i),
            side: Some("opponent".to_string()),
            description: format!("Opponent creature slot {}", i),
            region: "center-top".to_string(),
        });
    }

    // Player support slots
    for i in 0..2 {
        elements.push(UIElement {
            id: format!("support_player_{}", i),
            element_type: "support_slot".to_string(),
            index: Some(i),
            side: Some("player".to_string()),
            description: format!("Player support slot {}", i),
            region: "left center-bottom".to_string(),
        });
    }

    // Opponent support slots
    for i in 0..2 {
        elements.push(UIElement {
            id: format!("support_opponent_{}", i),
            element_type: "support_slot".to_string(),
            index: Some(i),
            side: Some("opponent".to_string()),
            description: format!("Opponent support slot {}", i),
            region: "left center-top".to_string(),
        });
    }

    // Action buttons
    elements.push(UIElement {
        id: "btn_end_turn".to_string(),
        element_type: "button".to_string(),
        index: None,
        side: None,
        description: "End Turn button".to_string(),
        region: "bottom right".to_string(),
    });

    elements.push(UIElement {
        id: "btn_undo".to_string(),
        element_type: "button".to_string(),
        index: None,
        side: None,
        description: "Undo button".to_string(),
        region: "bottom right".to_string(),
    });

    Json(elements)
}

/// Start the screenshot HTTP server.
///
/// Binds to localhost only for security.
pub async fn start_server(state: Arc<ScreenshotState>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/screenshot", get(capture_screenshot))
        .route("/health", get(health))
        // State sync endpoints
        .route("/sync_state", post(sync_state))
        .route("/synced_state", get(get_synced_state))
        .route("/synced_state", axum::routing::delete(clear_synced_state))
        // UI element discovery
        .route("/elements", get(list_elements))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("Screenshot server listening on http://127.0.0.1:{}", port);
    eprintln!("  - GET  /screenshot    - Capture window screenshot");
    eprintln!("  - GET  /health        - Health check");
    eprintln!("  - POST /sync_state    - Sync game state from MCP");
    eprintln!("  - GET  /synced_state  - Get current synced state");
    eprintln!("  - GET  /elements      - List UI elements");

    axum::serve(listener, app).await?;

    Ok(())
}
