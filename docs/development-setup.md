# Development Setup Guide

## Prerequisites

- Rust 1.75+ (install via rustup)
- cargo-nextest (recommended): `cargo install cargo-nextest`
- For 3D client on Linux: X11/Wayland dev libraries
  ```bash
  # Ubuntu/Debian
  sudo apt-get install pkg-config libx11-dev libwayland-dev libxkbcommon-dev
  ```

## Clone and Build

```bash
git clone https://github.com/your-repo/ai-cardgame
cd ai-cardgame

# Build all crates
cargo build --release

# Or build specific crate
cargo build --release -p cardgame        # Core engine
cargo build --release -p essence-wars-3d # 3D client
```

## Running Tests

```bash
# Fast test runner (recommended)
cargo nextest run --status-level=fail

# Standard cargo test
cargo test

# Run specific test tiers
./scripts/run-tests.sh quick   # ~2 min
./scripts/run-tests.sh medium  # ~10 min
```

## Running the 3D Client

```bash
cargo run --release -p essence-wars-3d
```

Press 'G' to toggle Glassbox AI visualization.

## Running Arena Matches

```bash
# Quick match
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100

# MCTS vs Greedy with progress
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 10 --progress
```

## IDE Setup

### VS Code
Install rust-analyzer extension. The workspace structure is auto-detected.

### IntelliJ/CLion
Open the root directory. Cargo workspace is auto-detected.

## Common Issues

### Linux: Missing display server
The 3D client requires X11 or Wayland. On WSL2, ensure WSLg is enabled or use an X server.

### Slow builds
Use `cargo build` (debug) for faster iteration. Release builds (`--release`) are optimized but slower to compile.

## Project Structure

See CLAUDE.md for full project structure documentation.
