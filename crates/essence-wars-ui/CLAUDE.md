# essence-wars-ui - Tauri Desktop App CLAUDE.md

<!-- Last verified: 2026-02-05 -->

## Overview

Desktop UI built with **Tauri 2** (Rust backend) + **Svelte 5** (frontend) + **Tailwind CSS v4**.

## Development

```bash
pnpm install           # First time
pnpm tauri:dev         # Dev mode with hot reload
pnpm run check         # Svelte/TS type checking
pnpm run lint          # ESLint
```

## Building

```bash
# Linux
pnpm tauri:build       # Creates .deb, .rpm, .AppImage

# Windows (from WSL2)
./scripts/build-windows.sh           # Unsigned
./scripts/build-windows.sh --sign    # Signed
```

## Svelte 5 Runes

This project uses **Svelte 5 runes** (not Svelte 4 stores):

| Rune | Purpose |
|------|---------|
| `$state` | Reactive state |
| `$derived` | Computed values |
| `$effect` | Side effects |
| `$props` | Component props |
| `$bindable` | Two-way binding props |

**Important:** `SvelteSet` and `SvelteMap` are already reactive - don't wrap in `$state()`.

```svelte
<!-- Correct -->
<script>
  import { SvelteSet } from 'svelte/reactivity';
  const items = new SvelteSet(['a', 'b']);
</script>

<!-- Wrong - unnecessary wrapper -->
<script>
  let items = $state(new SvelteSet(['a', 'b']));
</script>
```

## Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `DeckSelectionWizard` | `src/lib/components/menu/` | 3-step setup wizard |
| `GameBoard` | `src/lib/components/board/` | Main gameplay board |
| `CommanderCardLarge` | `src/lib/components/board/` | Commander display |
| `SetupScreen` | `src/lib/components/` | Human vs AI setup |
| `SpectatorSetup` | `src/lib/components/` | AI vs AI setup |

## Game Modes

| Mode | Description |
|------|-------------|
| **Human vs AI** | Play against AI opponent |
| **AI vs AI (Spectator)** | Watch two AIs battle |

Both use a 3-step wizard: Choose Commander 1 → Choose Commander 2 → Game Options.

## Faction Theming

Colors defined in `src/app.css`:

| Faction | Primary | Accent |
|---------|---------|--------|
| Argentum | `#F5F5F5` | `#D4AF37` (gold) |
| Symbiote | `#1A472A` | `#7FFF00` (lime) |
| Obsidion | `#8B0000` | `#00FFFF` (cyan) |
| Neutral | `#8B4513` | `#B87333` (copper) |

## Audio System

Sound effects and music in `src/lib/audio/`:

| Module | Purpose |
|--------|---------|
| `manager.ts` | Sound effects (UI, cards, combat) |
| `music.ts` | Background music, victory/defeat stings |

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `Tab` | Navigate elements |
| `Enter` / `Space` | Select/activate |
| `Escape` | Go back |

## Directory Structure

```
src/
├── lib/
│   ├── api/types.ts       # TypeScript types for game state
│   ├── audio/             # Sound effects and music
│   ├── components/
│   │   ├── board/         # Gameplay components
│   │   └── menu/          # Setup/wizard components
│   └── utils/arrays.ts    # Array padding utilities
├── routes/                # SvelteKit routes
└── app.css                # Global styles, faction colors
src-tauri/
├── src/
│   ├── ai/                # AI introspection, decision insights
│   ├── state/             # Game state management
│   └── lib.rs             # Tauri commands
└── Cargo.toml
```
