# FLUX Model Guide for Essence Wars Card Art
## Setup and Usage with stable-diffusion.cpp

---

## 1. Model Overview

**FLUX** models are state-of-the-art image generation models (2024-2026) that produce significantly better quality than SDXL models. They offer modern image quality comparable to DALL-E 3 and Gemini.

### Available Models

- **FLUX.1-dev** (High Quality): Best quality, slower generation (~20 steps, ~2-3 minutes)
- **FLUX.1-schnell** (Fast): Good quality, rapid prototyping (~4 steps, ~30 seconds)

---

## 2. File Structure

Your models should be organized in `~/.ai-assets/models/flux/`:

```
~/.ai-assets/models/flux/
├── flux-dev-q8.gguf              # 12GB - Main model (high quality)
├── flux-schnell-q4.gguf          # 6.4GB - Fast model
├── ae.safetensors                # 320MB - VAE (shared)
├── clip_l.safetensors            # 235MB - CLIP encoder (shared)
├── t5-Q5_K_M.gguf               # ~2GB - T5 text encoder (faster, recommended)
└── t5xxl_fp8_e4m3fn.safetensors # 4.6GB - T5 text encoder (alternative)
```

LoRAs should be in `~/.ai-assets/loras/`:
```
~/.ai-assets/loras/
├── classical-painting.safetensors
├── frazetta.safetensors
└── rutkowski.safetensors
```

---

## 3. Command Structure

### Base Command (FLUX Dev - High Quality)

```bash
cd ~/stable-diffusion.cpp

./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "YOUR_PROMPT_HERE" \
  --cfg-scale 1.0 \
  --sampling-method euler \
  --steps 20 \
  -H 896 -W 704 \
  -o ~/.ai-assets/output/OUTPUT_NAME.png
```

**Key Parameters:**
- `--steps 20`: More steps = better quality (15-30 recommended for dev)
- `--cfg-scale 1.0`: Always use 1.0 for FLUX models
- `-H 896 -W 704`: Portrait card aspect ratio
- `-H 704 -W 896`: Landscape orientation

---

## 4. Essence Wars Art Direction

### Core Art Style
- **90s Magic the Gathering card art aesthetic**
- **Empowered female characters** using beauty and sexuality as assets of power and influence
- **Faction-specific aesthetics** (Art Deco, Primal Nature, Gothic Victorian, Wasteland)
- **Painted illustration style** with dramatic lighting

### Prompt Template Structure

```
[CHARACTER DESCRIPTION] in [OUTFIT/ARMOR], [PHYSICAL FEATURES], 
[POSE/ACTION], [BACKGROUND], [AESTHETIC/STYLE], 
90s Magic the Gathering card art, [painting style] <lora:LORA_NAME:STRENGTH>
```

---

## 5. Faction-Specific Prompts

### 🏛️ ARGENTUM COMBINE (Order & Industry)

**Female Commander:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "A powerful female military commander in gleaming white and gold art deco armor, polished brass accents, confident pose with hand on sword hilt, geometric steel patterns, white marble background, revealing armor design showing strength and authority, 90s Magic the Gathering card art style, painted illustration <lora:classical-painting:0.7>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/argentum_commander.png
```

**Female Engineer:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "An attractive female engineer with brass goggles, white corset with gold filigree, working on a massive clockwork automaton, art deco steampunk workshop, confident and intelligent expression, elegant pose, 90s Magic the Gathering card art, fantasy illustration <lora:classical-painting:0.6>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/argentum_engineer.png
```

---

### 🌿 SYMBIOTE CIRCLES (Primal Nature & The Pack)

**Female Grove Keeper:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "A fierce female druid with living wood staff and bone ornaments, fur-trimmed leather armor, deep forest green robes, powerful stance in ancient primordial forest with massive trees, 90s Magic the Gathering art style, Urza's Saga aesthetic, painted fantasy illustration <lora:classical-painting:0.6>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/symbiote_grovekeeper.png
```

**Female Beast Master:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "A commanding female beast master with fierce predator features, bone and leather armor revealing athletic form, bioluminescent war paint, flanked by massive dire wolves and great cats, confident commanding pose, primordial forest background with ancient trees, 90s MTG card art, Urza's Saga aesthetic, fantasy painting <lora:frazetta:0.5>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/symbiote_beastmaster.png
```

---

### 🔮 OBSIDION SYNDICATE (Knowledge & Ambition)

**Female Hemomancer:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "An alluring vampire sorceress in elegant crimson velvet dress with gothic corset, pale skin, glowing blue essence tubes connected to ornate gauntlets, neon blue magical energy crackling around her hands, confident seductive expression, dark spire tower background, 90s Magic the Gathering card art, gothic fantasy painting <lora:classical-painting:0.5> <lora:frazetta:0.4>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/obsidion_hemomancer.png
```

**Female Spellblade:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "A powerful female assassin mage in black leather armor with crimson accents, wielding a glowing neon blue energy blade, revealing outfit showing deadly grace, cyberpunk monocle with glowing data displays, dramatic shadows, gothic victorian architecture, 90s MTG art style, dark fantasy illustration <lora:rutkowski:0.7>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/obsidion_spellblade.png
```

---

### ⚖️ FREE-WALKERS (Mercenaries & Guilds)

**Female Archer:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "A rugged female mercenary archer in tactical wasteland gear, mix of brass goggles and leather straps, wielding an ornate composite bow, confident combat-ready pose, desert scavenger aesthetic, athletic form, wasteland ruins background, 90s Magic the Gathering card art, painted illustration <lora:frazetta:0.6>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/freewalker_archer.png
```

**Female Warlord:**
```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "A formidable female warlord in patchwork armor made from all three factions, powerful muscular build, wielding a massive salvaged weapon, commanding presence, rugged mercenary aesthetic with mix of technologies, confident dominant pose, neutral zone battlefield background, 90s MTG art style, fantasy painting <lora:frazetta:0.7>" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/freewalker_warlord.png
```

---

## 6. LoRA Usage Guide

### Single LoRA
```bash
<lora:classical-painting:0.7>  # Good for refined, classical MTG look
<lora:frazetta:0.6>            # Good for muscular, dynamic poses
<lora:rutkowski:0.7>           # Good for detailed fantasy environments
```

### Mixed LoRAs (Combine styles)
```bash
<lora:classical-painting:0.5> <lora:frazetta:0.4>  # Classical with dynamic energy
<lora:rutkowski:0.5> <lora:classical-painting:0.3> # Detailed fantasy with refinement
```

**Strength Recommendations:**
- `0.4-0.6`: Subtle influence
- `0.7-0.8`: Strong influence
- `0.9-1.0`: Very strong (may overpower base prompt)

---

## 7. Rapid Prototyping with FLUX Schnell

For quick iterations and testing compositions, use FLUX Schnell (4 steps):

```bash
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-schnell-q4.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  --lora-model-dir ~/.ai-assets/loras/ \
  -p "YOUR_PROMPT_HERE <lora:LORA_NAME:STRENGTH>" \
  --cfg-scale 1.0 --sampling-method euler --steps 4 \
  -H 896 -W 704 -o ~/.ai-assets/output/quick_test.png
```

**Workflow:**
1. Use **Schnell** to test 3-5 prompt variations (~2 minutes total)
2. Pick the best composition
3. Refine with **Dev** model for final quality (~2-3 minutes)

**Speed Comparison:**
- Schnell: ~30 seconds per image
- Dev: ~2-3 minutes per image

---

## 8. Troubleshooting

### Issue: "unknown format" errors
**Solution:** Ensure all model files are properly downloaded (not 0 bytes)
```bash
ls -lh ~/.ai-assets/models/flux/
```

### Issue: Out of VRAM
**Solutions:**
- Use the Q5 T5 encoder instead of fp8 (saves ~2GB)
- Use flux-schnell-q4 instead of flux-dev-q8 (saves ~6GB)
- Reduce image resolution: `-H 768 -W 640`

### Issue: LoRA not loading
**Solution:** Ensure LoRA filename matches exactly (without `.safetensors` extension)
```bash
ls ~/.ai-assets/loras/
# Use: <lora:classical-painting:0.7>
# Not: <lora:classical-painting.safetensors:0.7>
```

---

## 9. Performance Tips

### Optimize Generation Speed:
1. **Use Q5 T5 encoder** (faster, less VRAM): `t5-Q5_K_M.gguf`
2. **Lower steps for testing**: `--steps 15` (still good quality)
3. **Batch similar prompts**: Generate all characters for one faction together
4. **Use Schnell for iterations**: Only use Dev for final art

### Memory Usage:
- **Schnell Q4 + Q5 T5**: ~8GB VRAM (fastest)
- **Dev Q8 + Q5 T5**: ~14GB VRAM (best quality)
- **Dev Q8 + FP8 T5**: ~16GB VRAM (alternative)

---

## 10. Quick Reference Commands

### Test Your Setup:
```bash
cd ~/stable-diffusion.cpp
./build/bin/sd-cli \
  --diffusion-model ~/.ai-assets/models/flux/flux-dev-q8.gguf \
  --vae ~/.ai-assets/models/flux/ae.safetensors \
  --clip_l ~/.ai-assets/models/flux/clip_l.safetensors \
  --t5xxl ~/.ai-assets/models/flux/t5-Q5_K_M.gguf \
  -p "A beautiful female warrior, 90s Magic the Gathering card art" \
  --cfg-scale 1.0 --sampling-method euler --steps 20 \
  -H 896 -W 704 -o ~/.ai-assets/output/test.png
```

### Check Model Sizes:
```bash
ls -lh ~/.ai-assets/models/flux/
ls -lh ~/.ai-assets/loras/
```

### View Recent Output:
```bash
ls -lht ~/.ai-assets/output/ | head -10
```

---

## Notes

- Always use `--cfg-scale 1.0` for FLUX models
- Card art aspect ratio: `-H 896 -W 704` (portrait)
- Dev model quality is worth the extra time for final card art
- Mix LoRAs for unique styles (keep combined strength under 1.0)
- Navigate to `~/stable-diffusion.cpp` before running commands

---

**Created:** January 2026  
**Game:** Essence Wars Card Game  
**Art Direction:** 90s MTG aesthetic with empowered female characters
