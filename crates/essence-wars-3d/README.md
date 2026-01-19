# Essence Wars 3D - Bevy Client

A 3D visualization client for the Essence Wars card game, built with Bevy game engine and featuring Glassbox AI transparency tools.

## Overview

This client provides a 3D rendering of the Essence Wars card game with real-time visualization of game state and AI decision-making processes. The Glassbox feature allows players to see inside the AI's thinking, including MCTS tree exploration, action probabilities, and position evaluation.

## Building

```bash
# From workspace root
cargo build --release -p essence-wars-3d

# With faster dev compilation (no optimizations)
cargo build -p essence-wars-3d
```

## Running

```bash
cargo run --release -p essence-wars-3d
```

## Features

- **3D Board Rendering**: Full 3D game board with creature slots for both players
- **Real-time Game State**: Live visualization of creatures, supports, and player stats
- **egui-based UI**: HUD panels, hand display, game menus, and settings
- **Glassbox AI Visualization**:
  - MCTS tree explorer showing search depth and node visits
  - Action probability distribution for AI decisions
  - Position value gauge indicating AI's evaluation
  - Press **'G'** to toggle glassbox panels on/off

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| bevy | 0.15 | Game engine and rendering |
| bevy_egui | 0.31 | Immediate mode UI integration |
| cardgame | workspace | Core game engine |

## Architecture

The client is organized into the following modules:

- **game/**: Game state management and integration with the cardgame engine
- **rendering/**: 3D scene setup, board rendering, creature visualization
- **ui/**: egui panels for HUD, hand display, menus, and settings
- **glassbox/**: AI transparency visualization (MCTS tree, probabilities, value gauge)

## Current Limitations

- **Single-player vs AI only**: No human vs human mode
- **No network multiplayer**: Local play only
- **Audio disabled**: To avoid alsa-sys dependency issues on Linux

## Requirements

### Linux/WSL
- X11 or Wayland display server
- Graphics drivers with OpenGL or Vulkan support

### General
- GPU with modern graphics API support (OpenGL 3.3+ or Vulkan 1.0+)
