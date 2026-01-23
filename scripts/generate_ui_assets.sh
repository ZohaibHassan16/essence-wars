#!/bin/bash
# Generate UI assets (backgrounds, frames, card backs) using FLUX Schnell
# Run from ai-cardgame directory

set -e

SD_CLI="$HOME/stable-diffusion.cpp/build/bin/sd-cli"
MODELS="$HOME/.ai-assets/models/flux"
LORAS="$HOME/.ai-assets/loras"
OUTPUT="$HOME/.ai-assets/output/essence-wars"

# Common FLUX Schnell parameters
FLUX_ARGS=(
    --diffusion-model "$MODELS/flux-schnell-q4.gguf"
    --vae "$MODELS/ae.safetensors"
    --clip_l "$MODELS/clip_l.safetensors"
    --t5xxl "$MODELS/t5-Q5_K_M.gguf"
    --lora-model-dir "$LORAS"
    --cfg-scale 1.0
    --sampling-method euler
    --steps 4
)

mkdir -p "$OUTPUT/backgrounds"
mkdir -p "$OUTPUT/frames"
mkdir -p "$OUTPUT/card_backs"

echo "=== Generating Board Backgrounds (1920x1080) ==="

# Argentum - White marble factory
echo "Generating: Argentum background..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Art deco white marble factory floor, polished brass gears and pipes, subtle steam, geometric patterns, warm golden lighting from above, clean minimalist industrial aesthetic, top-down perspective, board game background <lora:classical-painting:0.5>" \
    -H 1080 -W 1920 \
    -o "$OUTPUT/backgrounds/argentum.png"

# Symbiote - Bioluminescent jungle
echo "Generating: Symbiote background..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Bioluminescent jungle clearing at night, glowing purple and green fungi, floating spores, organic vine patterns, deep forest atmosphere, mystical nature scene, top-down perspective, board game background <lora:rutkowski:0.5>" \
    -H 1080 -W 1920 \
    -o "$OUTPUT/backgrounds/symbiote.png"

# Obsidion - Gothic cathedral
echo "Generating: Obsidion background..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Dark gothic cathedral interior, crimson velvet floor with black obsidian tiles, neon blue essence veins in the stone, flickering candles, ornate wrought iron patterns, ominous atmosphere, top-down perspective, board game background <lora:rutkowski:0.5>" \
    -H 1080 -W 1920 \
    -o "$OUTPUT/backgrounds/obsidion.png"

# Neutral - Desert trading post
echo "Generating: Neutral background..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Desert trading post plaza, worn sandstone tiles, leather and copper merchant stalls, dusty atmosphere, warm sunset lighting, weathered utilitarian aesthetic, top-down perspective, board game background <lora:frazetta:0.4>" \
    -H 1080 -W 1920 \
    -o "$OUTPUT/backgrounds/neutral.png"

# Mixed - Contested borderlands
echo "Generating: Mixed background..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Contested battlefield borderlands, mixing industrial marble with organic vines and gothic stone, neutral ground where three aesthetics meet, dramatic lighting, war-torn but elegant, top-down perspective, board game background <lora:rutkowski:0.4>" \
    -H 1080 -W 1920 \
    -o "$OUTPUT/backgrounds/mixed.png"

echo ""
echo "=== Generating Card Backs (704x896) ==="

# Universal card back
echo "Generating: Universal card back..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Ornate card back design, mystical swirling energy pattern, gold and silver filigree border, central glowing orb symbol, elegant fantasy playing card back, dark blue background, symmetrical decorative pattern <lora:classical-painting:0.6>" \
    -H 896 -W 704 \
    -o "$OUTPUT/card_backs/universal.png"

# Argentum card back
echo "Generating: Argentum card back..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Art deco card back design, white marble texture with gold gear patterns, brass sunburst in center, geometric borders, steampunk elegance, symmetrical ornate pattern, fantasy card back <lora:classical-painting:0.6>" \
    -H 896 -W 704 \
    -o "$OUTPUT/card_backs/argentum.png"

# Symbiote card back
echo "Generating: Symbiote card back..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Organic bio-mechanical card back design, deep green with purple bioluminescent veins, central eye or spore cluster motif, chitinous border pattern, living tissue texture, symmetrical nature pattern, fantasy card back <lora:rutkowski:0.5>" \
    -H 896 -W 704 \
    -o "$OUTPUT/card_backs/symbiote.png"

# Obsidion card back
echo "Generating: Obsidion card back..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Gothic Victorian card back design, crimson velvet with black obsidian, neon blue essence runes, wrought iron filigree border, central crystal or skull motif, dark elegant symmetrical pattern, fantasy card back <lora:rutkowski:0.5>" \
    -H 896 -W 704 \
    -o "$OUTPUT/card_backs/obsidion.png"

# Neutral card back
echo "Generating: Neutral card back..."
"$SD_CLI" "${FLUX_ARGS[@]}" \
    -p "Weathered mercenary card back design, brown leather texture with copper rivets, central guild emblem, worn practical border pattern, road dust and adventure aesthetic, symmetrical utilitarian design, fantasy card back <lora:frazetta:0.4>" \
    -H 896 -W 704 \
    -o "$OUTPUT/card_backs/neutral.png"

echo ""
echo "=== Generation Complete ==="
echo ""
echo "Generated assets in: $OUTPUT"
echo ""
echo "Next step: Run convert_assets.py to convert to WebP"
echo "  ./scripts/convert_assets.py --other"
