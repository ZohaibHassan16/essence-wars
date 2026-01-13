//! Color theme using Catppuccin Mocha palette
//!
//! A professional dark theme that's accessible and widely loved.
//! See: https://github.com/catppuccin/catppuccin

use ratatui::style::Color;

/// Catppuccin Mocha color theme
pub struct Theme {
    // Base colors
    pub bg: Color,
    pub bg_alt: Color,
    pub fg: Color,
    pub fg_dim: Color,

    // Accent colors
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Border colors
    pub border: Color,
    pub border_focused: Color,

    // Chart colors
    pub chart_line1: Color,
    pub chart_line2: Color,
    pub chart_line3: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::mocha()
    }
}

impl Theme {
    /// Catppuccin Mocha palette
    pub fn mocha() -> Self {
        Self {
            // Base colors
            bg: Color::Rgb(30, 30, 46),        // #1e1e2e (base)
            bg_alt: Color::Rgb(49, 50, 68),    // #313244 (surface0)
            fg: Color::Rgb(205, 214, 244),     // #cdd6f4 (text)
            fg_dim: Color::Rgb(108, 112, 134), // #6c7086 (overlay0)

            // Accent colors
            primary: Color::Rgb(137, 180, 250), // #89b4fa (blue)
            success: Color::Rgb(166, 227, 161), // #a6e3a1 (green)
            warning: Color::Rgb(249, 226, 175), // #f9e2af (yellow)
            error: Color::Rgb(243, 139, 168),   // #f38ba8 (red)
            info: Color::Rgb(148, 226, 213),    // #94e2d5 (teal)

            // Border colors
            border: Color::Rgb(69, 71, 90),     // #45475a (surface1)
            border_focused: Color::Rgb(137, 180, 250), // #89b4fa (blue)

            // Chart colors
            chart_line1: Color::Rgb(243, 139, 168), // #f38ba8 (red)
            chart_line2: Color::Rgb(137, 180, 250), // #89b4fa (blue)
            chart_line3: Color::Rgb(166, 227, 161), // #a6e3a1 (green)
        }
    }

    /// Additional Catppuccin Mocha colors for special use cases
    pub fn mauve() -> Color {
        Color::Rgb(203, 166, 247) // #cba6f7
    }

    pub fn pink() -> Color {
        Color::Rgb(245, 194, 231) // #f5c2e7
    }

    pub fn peach() -> Color {
        Color::Rgb(250, 179, 135) // #fab387
    }

    pub fn lavender() -> Color {
        Color::Rgb(180, 190, 254) // #b4befe
    }

    pub fn sky() -> Color {
        Color::Rgb(137, 220, 235) // #89dceb
    }
}
