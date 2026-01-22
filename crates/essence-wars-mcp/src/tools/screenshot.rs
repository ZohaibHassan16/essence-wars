//! Screenshot capture tool via Tauri HTTP endpoint.
//!
//! This tool connects to the screenshot HTTP server running in the Tauri app
//! and returns a base64-encoded PNG image for vision analysis.

use base64::Engine;

/// Default URL for the screenshot server.
const SCREENSHOT_URL: &str = "http://127.0.0.1:9999/screenshot";

/// Take a screenshot of the Essence Wars UI.
///
/// Returns markdown-formatted output with the base64-encoded PNG data
/// suitable for vision analysis.
pub fn take_screenshot() -> String {
    match capture_screenshot() {
        Ok((bytes, b64)) => {
            format!(
                "# Screenshot Captured\n\n\
                 **Image size:** {} bytes ({:.1} KB)\n\n\
                 The screenshot has been captured successfully. \
                 Use your vision capabilities to analyze the base64-encoded PNG below.\n\n\
                 ```base64\n{}\n```\n",
                bytes.len(),
                bytes.len() as f64 / 1024.0,
                b64
            )
        }
        Err(e) => e,
    }
}

/// Internal function to capture and encode the screenshot.
fn capture_screenshot() -> Result<(Vec<u8>, String), String> {
    // Make HTTP request to screenshot server
    let mut response = ureq::get(SCREENSHOT_URL).call().map_err(|e| {
        format!(
            "# Screenshot Error\n\n\
             Could not connect to Essence Wars UI.\n\n\
             **Troubleshooting:**\n\
             1. Is the Tauri desktop app running?\n\
             2. Check if http://127.0.0.1:9999/health responds\n\
             3. The app window must be visible (not minimized)\n\n\
             **Error:** {}",
            e
        )
    })?;

    // Check status code
    if response.status() != 200 {
        return Err(format!(
            "# Screenshot Error\n\n\
             HTTP {}: Failed to capture screenshot.\n\n\
             The screenshot server returned an error. \
             Make sure the Essence Wars window is visible and not minimized.",
            response.status()
        ));
    }

    // Read PNG bytes (allow up to 50MB for large screenshots)
    let bytes = response
        .body_mut()
        .with_config()
        .limit(50 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| {
            format!(
                "# Screenshot Error\n\n\
                 Failed to read response body: {}",
                e
            )
        })?;

    // Encode as base64
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok((bytes, b64))
}
