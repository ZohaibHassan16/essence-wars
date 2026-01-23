# Essence Wars Asset Pipeline

## Overview

This document describes the complete pipeline for generating, converting, and integrating art assets into the Essence Wars desktop client.

## Design Decisions

| Asset Type | Approach | Rationale |
|------------|----------|-----------|
| Card Art | FLUX Schnell → WebP | AI generation works well for card portraits |
| Board Backgrounds | CSS Gradients | AI backgrounds look generic/"asset flip-y" |
| Card Backs | CSS Design | Crisp, small, no additional files needed |
| Card Frames | CSS Borders | Flexible, faction-colored via Tailwind |
| Keyword Icons | Lucide (SVG) | Clean, scalable, professional icon library |

## Directory Structure

```
~/.ai-assets/output/essence-wars/     # Generated PNGs (not in repo)
└── drafts/
    ├── argentum/                     # Card art drafts
    ├── symbiote/
    ├── obsidion/
    └── neutral/

crates/essence-wars-ui/static/        # Final WebP assets (bundled with app)
└── cards/
    └── core_set/                     # 300 card images (1000-4074.webp)
```

**Note:** Backgrounds, card backs, and frames are handled via CSS - no image files needed.

## Pipeline Steps

### 1. Generate Card Art

Uses FLUX Schnell for fast draft generation (~30 sec/image).

```bash
# Generate all cards for a faction
./scripts/generate_batch.py --faction argentum --mode draft
./scripts/generate_batch.py --faction symbiote --mode draft
./scripts/generate_batch.py --faction obsidion --mode draft
./scripts/generate_batch.py --faction neutral --mode draft

# Or regenerate specific cards
./scripts/generate_batch.py --faction argentum --mode draft --ids 1000,1001,1002
```

Output: `~/.ai-assets/output/essence-wars/drafts/{faction}/{id}_{name}.png`

### 2. Convert to WebP

Convert PNGs to WebP for smaller distribution size (~30% reduction).

```bash
# Convert all card art to WebP
./scripts/convert_assets.py --cards

# Check current asset status
./scripts/convert_assets.py --stats
```

Output goes to `crates/essence-wars-ui/static/cards/core_set/`

### 3. Build & Test

```bash
cd crates/essence-wars-ui
pnpm dev          # Development with hot reload
pnpm tauri build  # Production build
```

## Asset Specifications

### Card Art
- **Resolution:** 704x896 (portrait, 0.78 aspect ratio)
- **Format:** WebP, quality 85
- **Naming:** `{card_id}.webp` (e.g., `1000.webp`)
- **Count:** 300 cards total

### Board Backgrounds (CSS)
Implemented via CSS gradients in `app.css`:
- `.board-bg-argentum` - Gold/white subtle gradient
- `.board-bg-symbiote` - Green/purple subtle gradient
- `.board-bg-obsidion` - Red/cyan subtle gradient
- `.board-bg-neutral` - Brown/tan subtle gradient
- `.board-bg-default` - Neutral dark gradient

### Card Backs (CSS)
Implemented via CSS in `HandCard.svelte` - elegant dark design with decorative borders and central emblem.

### Keyword Icons (Lucide)
Using `lucide-svelte` library - 14 keywords mapped to appropriate icons:
- Rush → Zap, Ranged → Target, Piercing → Swords, Guard → Shield
- Lifesteal → Droplet, Lethal → Skull, Shield → ShieldCheck, Quick → ChevronsRight
- Ephemeral → Ghost, Regenerate → HeartPulse, Stealth → EyeOff
- Charge → BatteryCharging, Frenzy → Flame, Volatile → Bomb

## Distribution Size

| Asset Type | Size | Notes |
|------------|------|-------|
| Card Art (300 WebP) | ~29 MB | ~93% smaller than PNG |
| CSS (backgrounds, backs, frames) | ~5 KB | Included in app bundle |
| Lucide Icons | ~50 KB | Tree-shaken, only used icons |
| **Total Assets** | ~30 MB | Very lightweight! |

## Backend Integration

The Rust backend generates art paths in `serialization.rs`:

```rust
// Card art (for cards, creatures, supports)
let art_path = Some(format!("cards/core_set/{}.webp", card.id));

// Hidden cards (opponent's hand) - use CSS card back
art_path: None,
```

Frontend components load card art via:
```svelte
{#if card.artPath}
  <img src="/{card.artPath}" />
{/if}
```

Hidden cards are detected via `card.cardId === 0` and display the CSS card back design.

## Regenerating Specific Assets

### Replace a bad card image:
```bash
# 1. Regenerate the PNG
./scripts/generate_batch.py --faction argentum --ids 1042 --mode draft

# 2. Convert to WebP (will overwrite)
rm crates/essence-wars-ui/static/cards/core_set/1042.webp
./scripts/convert_assets.py --cards --faction argentum
```

### Upgrade to final quality:
```bash
# Use mode=final for 20-step generation (~3 min/image)
./scripts/generate_batch.py --faction argentum --ids 1000,1001 --mode final
```

## Notes

### Art Prompts
Card art prompts are in `data/art/prompts/core_set/{faction}.yaml`. Edit these to adjust card art style.

### VRAM Requirements
- FLUX Schnell (4 steps): ~8GB VRAM for 704x896 images
- FLUX Dev (20 steps): ~12GB VRAM for 704x896 images
- Larger resolutions (1920x1080) may produce noise with 12GB VRAM

### Why CSS Over AI Generation?
- **Backgrounds:** AI-generated board backgrounds often look generic and clash with card art
- **Card backs:** CSS is crisp, small, and easy to customize
- **Frames:** CSS borders are more flexible than fixed images and adapt to card content
