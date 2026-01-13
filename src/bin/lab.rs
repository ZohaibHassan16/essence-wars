//! Essence Wars Research Lab - TUI Application
//!
//! A terminal user interface for AI research workflows.
//!
//! Usage:
//!   cargo run --release --bin lab

use std::process;

fn main() {
    if let Err(e) = cardgame::tui::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
