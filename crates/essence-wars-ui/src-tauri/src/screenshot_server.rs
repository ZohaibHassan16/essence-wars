//! HTTP server for MCP state synchronization.
//!
//! Runs alongside Tauri, exposing endpoints for MCP tools:
//! - `/health` - Health check
//! - `/sync_state` - Receive game state pushed from MCP server
//! - `/synced_state` - Get the current synced state (for frontend polling)
//!
//! This enables synchronizing game state between MCP and Tauri UI,
//! allowing the UI to display games played via MCP.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

use crate::state::{GameEventDto, GameStateDto};

/// Default port for the sync server.
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

/// State shared with Tauri to track synced game state.
pub struct ScreenshotState {
    /// Cached game state from MCP (synced via HTTP POST).
    synced_game_state: RwLock<Option<CachedGameState>>,
}

impl ScreenshotState {
    /// Create new state.
    pub fn new() -> Self {
        Self {
            synced_game_state: RwLock::new(None),
        }
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

/// Health check endpoint.
async fn health() -> &'static str {
    "ok"
}

/// Receive game state pushed from MCP server.
///
/// This allows MCP to sync its game state to Tauri so the UI can display it.
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

/// Start the sync HTTP server.
///
/// Binds to localhost only for security.
pub async fn start_server(state: Arc<ScreenshotState>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/sync_state", post(sync_state))
        .route("/synced_state", get(get_synced_state))
        .route("/synced_state", axum::routing::delete(clear_synced_state))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("MCP sync server listening on http://127.0.0.1:{}", port);
    eprintln!("  - GET  /health        - Health check");
    eprintln!("  - POST /sync_state    - Sync game state from MCP");
    eprintln!("  - GET  /synced_state  - Get current synced state");

    axum::serve(listener, app).await?;

    Ok(())
}
