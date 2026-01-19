# Workspace Migration Guide

## Overview

The Essence Wars project migrated from a single crate to a Cargo workspace in January 2026. This guide documents the new structure and how to work with it.

## Why Workspace?

- **Separation of concerns**: Core game engine vs 3D client are now independent crates
- **Independent compilation and testing**: Each crate can be built and tested separately
- **Shared dependencies via `[workspace.dependencies]`**: Common dependencies are defined once at the workspace level
- **Enables future crates**: Easy to add Python bindings (PyO3), WASM builds, or other clients

## Structure

```
ai-cardgame/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── cardgame/           # Core game engine (library + binaries)
│   └── essence-wars-3d/    # Bevy 3D client
├── data/                   # Shared game data (cards, decks, weights)
├── scripts/                # Build and analysis scripts
└── docs/                   # Documentation
```

## Building

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build --release -p cardgame
cargo build --release -p essence-wars-3d
```

## Testing

```bash
# Test all crates
cargo nextest run

# Test specific crate
cargo nextest run -p cardgame
```

## Key Changes from Pre-Workspace

| Before | After |
|--------|-------|
| `src/` | `crates/cardgame/src/` |
| `tests/` | `crates/cardgame/tests/` |
| `benches/` | `crates/cardgame/benches/` |

Additional changes:

- Version managed in root `Cargo.toml` under `[workspace.package]`
- Data paths resolved via `cardgame::data_dir()` helper function

## For Developers

- All cargo commands work from the workspace root
- Scripts in `scripts/` work unchanged
- IDE should recognize the workspace structure automatically
- When adding new dependencies, prefer adding them to `[workspace.dependencies]` in the root `Cargo.toml` and referencing them with `{ workspace = true }` in crate-level `Cargo.toml` files
