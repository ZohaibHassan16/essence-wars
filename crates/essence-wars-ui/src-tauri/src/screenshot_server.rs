//! HTTP server for screenshot capture.
//!
//! Runs alongside Tauri, exposing a screenshot endpoint for MCP tools.
//! This enables Claude Code to visually inspect the UI during autonomous development.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use parking_lot::RwLock;
use std::io::Cursor;
use std::sync::Arc;
use xcap::Window;

/// Default port for the screenshot server.
pub const DEFAULT_PORT: u16 = 9999;

/// State shared with Tauri to track the window title.
pub struct ScreenshotState {
    /// The window title to capture (set by Tauri on startup).
    window_title: RwLock<Option<String>>,
}

impl ScreenshotState {
    /// Create new screenshot state.
    pub fn new() -> Self {
        Self {
            window_title: RwLock::new(None),
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

/// Health check endpoint.
async fn health() -> &'static str {
    "ok"
}

/// Start the screenshot HTTP server.
///
/// Binds to localhost only for security.
pub async fn start_server(state: Arc<ScreenshotState>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/screenshot", get(capture_screenshot))
        .route("/health", get(health))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("Screenshot server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
