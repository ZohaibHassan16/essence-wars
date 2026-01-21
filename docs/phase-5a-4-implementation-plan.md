# Phase 5A-4: Art Pipeline Implementation Plan

> **Status**: Planning
> **Created**: 2026-01-21
> **Approach**: Local FLUX inference + Meshy.ai for 3D

---

## Overview

Generate card art for 70 unique cards across 3 MVP decks using local FLUX inference, with depth maps for parallax shaders and 3D models for commanders.

**Pipeline Summary:**
```
Card YAML + Lore → Task Agent → prompts.json → FLUX Batch → PNG → Depth Maps → Bevy Assets
                                                    ↓
                                            Commanders → Meshy.ai → GLB Models
```

---

## 1. Prompt Generation (Task Agent)

### 1.1 Input Files

| File | Purpose |
|------|---------|
| `data/cards/core_set/argentum.yaml` | Argentum card definitions |
| `data/cards/core_set/symbiote.yaml` | Symbiote card definitions |
| `data/cards/core_set/obsidion.yaml` | Obsidion card definitions |
| `data/cards/core_set/neutral.yaml` | Neutral card definitions |
| `docs/lore.md` | Faction art direction |
| `data/decks/argentum/colossus_wall.toml` | MVP deck 1 |
| `data/decks/symbiote/broodmother_swarm.toml` | MVP deck 2 |
| `data/decks/obsidion/sovereign_lifesteal.toml` | MVP deck 3 |

### 1.2 Output Format: `prompts.json`

```json
{
  "version": "1.0",
  "generated": "2026-01-21T12:00:00Z",
  "art_direction": {
    "base_style": "90s Magic the Gathering card art, painted fantasy illustration",
    "aspect_ratio": "704x896 (portrait)",
    "loras": {
      "classical": "classical-painting",
      "dynamic": "frazetta",
      "detailed": "rutkowski"
    }
  },
  "cards": [
    {
      "id": 1057,
      "name": "Iron Colossus Prime",
      "faction": "argentum",
      "card_type": "creature",
      "tier": 1,
      "is_commander": true,
      "prompt": "Massive armored construct commander, Art Deco industrial aesthetic...",
      "lora_config": "<lora:classical-painting:0.7>",
      "negative": "organic, soft, natural, blurry, low quality, anime, cartoon",
      "notes": "Commander - generate high quality for Meshy.ai input"
    },
    {
      "id": 1001,
      "name": "Brass Sentinel",
      "faction": "argentum",
      "card_type": "creature",
      "tier": 2,
      "is_commander": false,
      "prompt": "Mechanical soldier construct with brass armor plating...",
      "lora_config": "<lora:classical-painting:0.6>",
      "negative": "organic, natural, blurry, low quality"
    }
  ]
}
```

### 1.3 Prompt Template Structure

**Creatures:**
```
[Character description based on name/tags] in [faction aesthetic],
[keywords as visual elements], [pose/action],
[faction-specific background], 90s Magic the Gathering card art,
painted fantasy illustration <lora:LORA:STRENGTH>
```

**Spells:**
```
[Magical effect visualization], [faction energy colors],
[dynamic composition showing spell impact],
[faction-specific magical aesthetic], 90s MTG card art,
painted fantasy illustration <lora:LORA:STRENGTH>
```

**Supports:**
```
[Object/structure description], [faction materials and design],
[ambient glow effect], [faction architectural style],
90s Magic the Gathering card art, painted illustration <lora:LORA:STRENGTH>
```

### 1.4 Faction-Specific Prompt Elements

| Faction | Colors | Materials | Aesthetic | LoRA Preference |
|---------|--------|-----------|-----------|-----------------|
| **Argentum** | Steel, white, gold, bronze | Brass, polished metal, marble | Art Deco Steampunk | classical-painting:0.7 |
| **Symbiote** | Deep green, bioluminescent purple, bone | Living chitin, organic matter | Biopunk Fantasy | classical-painting:0.6 |
| **Obsidion** | Black, crimson, neon blue | Dark velvet, glass tubes, gothic stone | Victorian Gothic Cyber | classical-painting:0.5 + frazetta:0.4 |
| **Neutral** | Earth tones, rust, weathered gray | Salvaged metal, patchwork | Wasteland Scavenger | frazetta:0.6 |

### 1.5 Keyword Visual Mapping

| Keyword | Visual Element |
|---------|----------------|
| Guard | Shield, defensive stance, protective posture |
| Rush | Motion blur, charging pose, aggressive stance |
| Lifesteal | Red energy drain, vampiric aura, blood effects |
| Lethal | Poison dripping, death imagery, skull motifs |
| Ranged | Projectile weapon, aiming pose, distance |
| Piercing | Sharp weapons, penetrating energy |
| Shield | Glowing barrier, protective aura |
| Stealth | Shadows, partial invisibility, dark cloak |
| Regenerate | Healing glow, growing tissue, green energy |

---

## 2. Batch Generation Script

### 2.1 Script: `python/scripts/generate_card_art.py`

```python
"""
Batch card art generation using local FLUX inference.

Usage:
    # Generate drafts with Schnell (fast iteration)
    uv run python python/scripts/generate_card_art.py \
        --prompts prompts.json \
        --model schnell \
        --output ~/.ai-assets/output/essence-wars/drafts/

    # Generate finals with Dev (high quality)
    uv run python python/scripts/generate_card_art.py \
        --prompts prompts.json \
        --model dev \
        --cards 1057,1001,1002 \
        --output ~/.ai-assets/output/essence-wars/finals/

    # Resume from specific card
    uv run python python/scripts/generate_card_art.py \
        --prompts prompts.json \
        --model schnell \
        --start-from 2015 \
        --output ~/.ai-assets/output/essence-wars/drafts/
"""
```

### 2.2 Generation Parameters

| Mode | Model | Steps | Time/Image | Use Case |
|------|-------|-------|------------|----------|
| **Draft** | flux-schnell-q4 | 4 | ~30s | First pass, composition check |
| **Final** | flux-dev-q8 | 20 | ~2-3min | Approved cards only |

### 2.3 Output Structure

```
~/.ai-assets/output/essence-wars/
├── drafts/
│   ├── argentum/
│   │   ├── 1057_iron_colossus_prime_v1.png
│   │   ├── 1057_iron_colossus_prime_v2.png
│   │   ├── 1001_brass_sentinel_v1.png
│   │   └── ...
│   ├── symbiote/
│   ├── obsidion/
│   └── neutral/
├── finals/
│   ├── argentum/
│   │   ├── 1057_iron_colossus_prime.png
│   │   └── ...
│   └── ...
├── depth_maps/
│   ├── 1057_iron_colossus_prime_depth.png
│   └── ...
└── commanders/
    ├── 1057_iron_colossus_prime/
    │   ├── source.png
    │   └── model.glb
    └── ...
```

---

## 3. Depth Map Generation

### 3.1 Tool: Marigold Depth Estimation

Use Marigold for monocular depth estimation on approved card art.

**Options:**
- **HuggingFace Spaces**: Upload images, download depth maps
- **Local inference**: `diffusers` library with Marigold model

### 3.2 Script: `python/scripts/generate_depth_maps.py`

```python
"""
Generate depth maps for parallax shader effect.

Usage:
    uv run python python/scripts/generate_depth_maps.py \
        --input ~/.ai-assets/output/essence-wars/finals/ \
        --output ~/.ai-assets/output/essence-wars/depth_maps/
"""
```

### 3.3 Depth Map Specifications

- **Format**: Grayscale PNG (8-bit)
- **Resolution**: Same as source image (704x896)
- **Convention**: White = close, Black = far (standard depth convention)

---

## 4. Commander 3D Pipeline

### 4.1 Meshy.ai Workflow

For the 3 MVP commanders (Tier 1):

1. **Prepare Source Image**
   - Use final FLUX Dev output (highest quality)
   - Ensure clean background (may need manual editing)
   - Character should be centered and fully visible

2. **Meshy.ai Image-to-3D**
   - Upload source image
   - Select "High Quality" preset
   - Enable auto-rigging for idle animation
   - Export as GLB with embedded animation

3. **Post-Processing**
   - Verify poly count (<50k triangles)
   - Check idle animation loop (2-4 seconds)
   - Apply Draco compression if needed
   - Test in Bevy

### 4.2 Commander List

| ID | Name | Faction | Notes |
|----|------|---------|-------|
| 1057 | Iron Colossus Prime | Argentum | Massive armored construct |
| 2060 | The Broodmother | Symbiote | Matriarch insectoid entity |
| 3055 | The Blood Sovereign | Obsidion | Vampire-like noble |

---

## 5. Bevy Integration

### 5.1 Parallax Shader Implementation

Create custom WGSL shader for parallax depth effect on gem tokens.

**File**: `crates/essence-wars-3d/assets/shaders/parallax_gem.wgsl`

**Features:**
- Parallax occlusion mapping using depth texture
- Faction-specific rim lighting
- State-based visual modifiers (damaged, exhausted, ready)
- "Window into another dimension" effect

### 5.2 Asset Loading System

**New Resources:**
```rust
#[derive(Resource)]
pub struct CardArtAssets {
    pub textures: HashMap<CardId, Handle<Image>>,
    pub depth_maps: HashMap<CardId, Handle<Image>>,
}

#[derive(Resource)]
pub struct CommanderModelAssets {
    pub models: HashMap<CardId, Handle<Scene>>,
}
```

**Loading Strategy:**
- Lazy load on deck selection
- Preload MVP deck assets
- Fallback to procedural gems if asset missing

### 5.3 Asset Directory Structure

```
crates/essence-wars-3d/assets/
├── textures/
│   └── cards/
│       ├── argentum/
│       │   ├── 1057.png
│       │   ├── 1057_depth.png
│       │   └── ...
│       ├── symbiote/
│       ├── obsidion/
│       └── neutral/
├── models/
│   └── commanders/
│       ├── iron_colossus_prime.glb
│       ├── the_broodmother.glb
│       └── the_blood_sovereign.glb
└── shaders/
    └── parallax_gem.wgsl
```

---

## 6. Implementation Phases

### Phase 4.1: Prompt Generation (~2 hours)
- [x] Task Agent generates prompts.json for all 70 cards
- [x] Review and refine prompts
- [x] Create prompt template documentation

### Phase 4.2: Batch Generation Script (~4 hours)
- [x] Create `generate_card_art.py` script
- [x] Integrate with stable-diffusion.cpp CLI
- [x] Add progress tracking and resume capability
- [x] Test with 3-5 sample cards

### Phase 4.3: Draft Generation (~4 hours GPU time)
- [x] Run Schnell on all 70 cards
- [x] Review drafts, note issues
- [x] Iterate on problematic prompts
- [x] Select best variants for finals

### Phase 4.4: Final Generation (~3 hours GPU time)
- [x] Run Dev on approved drafts
- [x] Final quality review
- [x] Manual touch-ups if needed

### Phase 4.5: Depth Map Generation (~1 hour)
- [x] Create `generate_depth_maps.py` script
- [x] Run Marigold on final images
- [x] Quality check depth maps

### Phase 4.6: Commander 3D Models (~2 hours)
- [x] Upload 3 commander images to Meshy.ai
- [x] Configure and generate 3D models
- [x] Export GLB files
- [ ] Test idle animations

### Phase 4.7: Bevy Parallax Shader (~8 hours)
- [ ] Research Bevy 0.15 custom shader workflow
- [ ] Implement parallax occlusion mapping in WGSL
- [ ] Create shader material wrapper
- [ ] Add state-based visual modifiers

### Phase 4.8: Asset Integration (~4 hours)
- [ ] Implement CardArtAssets resource
- [ ] Update creature rendering to use textures
- [ ] Add fallback for missing assets
- [ ] Test full pipeline

### Phase 4.9: Polish & Testing (~2 hours)
- [ ] Cross-browser WASM testing
- [ ] Performance optimization
- [ ] Fix visual bugs

---

## 7. Estimated Timeline

| Phase | Task | Effort | GPU Time |
|-------|------|--------|----------|
| 4.1 | Prompt Generation | 2h | - |
| 4.2 | Batch Script | 4h | - |
| 4.3 | Draft Generation | 2h | 4h |
| 4.4 | Final Generation | 1h | 3h |
| 4.5 | Depth Maps | 1h | 0.5h |
| 4.6 | Commander 3D | 2h | - (cloud) |
| 4.7 | Parallax Shader | 8h | - |
| 4.8 | Asset Integration | 4h | - |
| 4.9 | Polish | 2h | - |
| **Total** | | **~26h dev** | **~7.5h GPU** |

---

## 8. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| FLUX quality inconsistent | Use Schnell for rapid iteration, only commit Dev time to approved compositions |
| Parallax shader complexity | Start with simple depth offset, iterate to full occlusion mapping |
| Meshy.ai model quality | Have fallback plan: use 2.5D parallax for commanders if 3D fails |
| WASM shader compatibility | Test WebGL2 early, have non-parallax fallback material |
| Art style inconsistency | Lock in LoRA config per faction, use consistent negative prompts |

---

## 9. Files to Create

| File | Purpose |
|------|---------|
| `prompts.json` | Generated prompts for all 70 cards |
| `python/scripts/generate_card_art.py` | Batch FLUX generation script |
| `python/scripts/generate_depth_maps.py` | Marigold depth map script |
| `crates/essence-wars-3d/assets/shaders/parallax_gem.wgsl` | Custom parallax shader |
| `crates/essence-wars-3d/src/rendering/card_art.rs` | Asset loading and rendering |

---

## 10. Next Steps

1. **Immediate**: Run Task Agent to generate `prompts.json`
2. **Then**: Create and test `generate_card_art.py` with 3-5 sample cards
3. **Then**: Full draft generation with Schnell
4. **Parallel**: Start parallax shader research/implementation

---

*Document Version: 1.0*
*Created: 2026-01-21*
