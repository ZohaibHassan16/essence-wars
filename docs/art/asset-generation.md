# AI Asset Generation Guide

> **Images:** FLUX via stable-diffusion.cpp (local GPU)
> **Audio:** ElevenLabs Sound Effects API (cloud)

---

## Table of Contents

**Part 1: Image Generation (FLUX)**
- [Setup](#flux-setup)
- [Basic Usage](#flux-usage)
- [Art Direction](#art-direction)
- [LoRA Guide](#lora-guide)
- [Troubleshooting](#flux-troubleshooting)

**Part 2: Sound Effects (ElevenLabs)**
- [Setup](#elevenlabs-setup)
- [Generation Commands](#sfx-generation)
- [Sound Design](#sound-design)
- [Troubleshooting](#elevenlabs-troubleshooting)

---

# Part 1: Image Generation (FLUX)

## FLUX Setup

### Models Directory

```
~/.ai-assets/models/flux/
├── flux-dev-q8.gguf        # 12GB - High quality (20 steps)
├── flux-schnell-q4.gguf    # 6.4GB - Fast (4 steps)
├── ae.safetensors          # 320MB - VAE
├── clip_l.safetensors      # 235MB - CLIP encoder
└── t5-Q5_K_M.gguf          # 2GB - T5 encoder (recommended)

~/.ai-assets/loras/
├── classical-painting.safetensors
├── frazetta.safetensors
└── rutkowski.safetensors
```

### VRAM Requirements

| Configuration | VRAM |
|---------------|------|
| Schnell Q4 + Q5 T5 | ~8GB |
| Dev Q8 + Q5 T5 | ~14GB |

---

## FLUX Usage

### Base Command Template

```bash
cd ~/stable-diffusion.cpp

./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "YOUR_PROMPT <lora:classical-painting:0.7>" \
  --cfg-scale 1.0 \
  --sampling-method euler \
  --steps 20 \
  -H 896 -W 704 \
  -o ~/.ai-assets/output/image.png
```

### Key Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| `--cfg-scale` | 1.0 | Always 1.0 for FLUX |
| `--steps` | 20 | Dev: 15-30, Schnell: 4 |
| `-H -W` | 896x704 | Portrait card ratio |

### Quick Prototyping (Schnell)

Use `flux-schnell-q4.gguf` with `--steps 4` for ~30s generations.

---

## Art Direction

### Core Style

- **90s Magic the Gathering card art aesthetic**
- **Painted illustration** with dramatic lighting
- **Faction-specific aesthetics**

### Prompt Template

```
[CHARACTER] in [OUTFIT], [FEATURES], [POSE], [BACKGROUND],
90s Magic the Gathering card art <lora:NAME:STRENGTH>
```

### Faction Aesthetics

| Faction | Style | Colors | Keywords |
|---------|-------|--------|----------|
| **Argentum** | Art Deco Steampunk | White, Gold, Brass | clockwork, geometric, industrial |
| **Symbiote** | Primal Nature | Green, Brown, Bone | forest, predator, organic |
| **Obsidion** | Gothic Victorian | Crimson, Black, Blue | dark magic, ethereal, gothic |
| **Neutral** | Wasteland Mercenary | Earth tones, Rust | rugged, tactical, salvaged |

### Example Prompts

**Argentum Commander:**
```
A powerful female commander in gleaming white and gold art deco armor,
brass accents, confident pose, geometric steel patterns, marble background,
90s Magic the Gathering card art <lora:classical-painting:0.7>
```

**Symbiote Beast Master:**
```
A commanding female beast master with fierce predator features,
bone and leather armor, bioluminescent war paint, flanked by dire wolves,
primordial forest, 90s MTG art, Urza's Saga aesthetic <lora:frazetta:0.5>
```

**Obsidion Hemomancer:**
```
An alluring vampire sorceress in crimson velvet dress with gothic corset,
pale skin, glowing blue essence tubes, dark spire tower background,
90s Magic the Gathering card art <lora:classical-painting:0.5> <lora:frazetta:0.4>
```

---

## LoRA Guide

### Available LoRAs

| LoRA | Best For | Strength |
|------|----------|----------|
| `classical-painting` | Refined MTG look | 0.6-0.7 |
| `frazetta` | Dynamic poses, muscles | 0.5-0.6 |
| `rutkowski` | Detailed environments | 0.6-0.7 |

### Combining LoRAs

Keep total strength under 1.0:
```
<lora:classical-painting:0.5> <lora:frazetta:0.4>
```

---

## FLUX Troubleshooting

| Issue | Solution |
|-------|----------|
| "unknown format" | Check model files aren't 0 bytes |
| Out of VRAM | Use Q5 T5 encoder, reduce resolution |
| LoRA not loading | Use name without `.safetensors` extension |

---
---

# Part 2: Sound Effects (ElevenLabs)

## ElevenLabs Setup

### Why ElevenLabs?

- **High quality** - Professional 48kHz output
- **Fast** - ~2-3 seconds per sound
- **Reliable** - Cloud API, no GPU issues
- **Free tier** - 10k chars/month (~100 SFX)

### Install Dependencies

```bash
uv pip install -e ".[artgen]"
sudo apt install ffmpeg  # For OGG conversion
```

### API Key Setup

1. Sign up at https://elevenlabs.io/
2. Profile → API Keys → Create key
3. Set environment variable:

```bash
export ELEVENLABS_API_KEY='xi-xxxxxxxxxxxxxxxxxxxxxxxxxx'
```

### Verify Setup

```bash
python scripts/art/generate_sfx.py --dry-run
```

---

## SFX Generation

### Commands

```bash
# Generate all 66 battle sounds
python scripts/art/generate_sfx.py

# Filter by faction
python scripts/art/generate_sfx.py --faction argentum

# Filter by category
python scripts/art/generate_sfx.py --category attack

# Resume interrupted generation
python scripts/art/generate_sfx.py --skip-existing

# Custom duration (default: 2s)
python scripts/art/generate_sfx.py --duration 3.0
```

### Output Structure

```
crates/essence-wars-ui/static/sounds/battle/
├── attack_argentum_light_01.ogg
├── attack_argentum_medium_01.ogg
├── attack_argentum_heavy_01.ogg
├── summon_argentum_01.ogg
├── death_argentum_01.ogg
├── ...
└── debuff_generic_02.ogg
```

### Sound Count

| Faction | Attack | Summon | Death | Total |
|---------|--------|--------|-------|-------|
| Argentum | 7 | 3 | 3 | 13 |
| Symbiote | 7 | 3 | 3 | 13 |
| Obsidion | 7 | 3 | 3 | 13 |
| Neutral | 7 | 3 | 3 | 13 |
| Generic | - | - | - | 14 |
| **Total** | | | | **66** |

---

## Sound Design

### Faction Audio Palettes

**Argentum** - Mechanical / Clockwork
- Metallic impacts, brass clanks
- Gear grinding, steam hissing
- Industrial machinery

**Symbiote** - Primal / Organic
- Animal vocalizations
- Primal roars, growls
- Organic sounds

**Obsidion** - Dark Magic / Ethereal
- Arcane crackles
- Ghostly whispers
- Eldritch effects

**Neutral** - Physical Combat
- Sword slashes
- Shield blocks
- Armor clanks

**Generic** - Universal
- damage, heal, block, buff, debuff

### Prompts File

All prompts defined in `data/art/prompts/sfx_prompts.yml`:

```yaml
argentum:
  attack:
    - id: attack_argentum_light_01
      prompt: "Light metallic punch, brass knuckle impact..."
  summon:
    - id: summon_argentum_01
      prompt: "Clockwork automaton powering up..."
```

### Prompt Tips

1. **Be specific** - "brass gear grinding" > "metal sound"
2. **Keep short** - 10-20 words works best
3. **Avoid music** - May add unwanted melody
4. Script auto-adds: "game sound effect, high quality, clear"

---

## ElevenLabs Troubleshooting

| Issue | Solution |
|-------|----------|
| API Key Not Found | `export ELEVENLABS_API_KEY='...'` |
| Rate Limiting | Use `--delay 2.0`, `--skip-existing` |
| OGG Conversion Fails | Install ffmpeg |
| Poor Quality | Be more specific in prompts |

---

## Quick Reference

### Test FLUX Setup
```bash
cd ~/stable-diffusion.cpp && ./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  -p "A warrior, 90s MTG art" --cfg-scale 1.0 --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/test.png
```

### Test ElevenLabs Setup
```bash
python scripts/art/generate_sfx.py --faction argentum --category summon
```

---

*Updated: February 2026*
