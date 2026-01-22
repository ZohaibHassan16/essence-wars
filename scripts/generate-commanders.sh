#!/bin/bash
# Generate all 12 Essence Wars commanders with consistent pose using ControlNet
# Usage: ./generate-commanders.sh [fast|quality]

set -e

MODE="${1:-quality}"  # Default to quality mode

# Paths
EDGE_TEMPLATE="${EDGE_TEMPLATE:-data/art/templates/commander-pose-edges.png}"
OUTPUT_DIR="${OUTPUT_DIR:-data/art/commanders}"
CONTROLNET="$HOME/.ai-assets/models/flux/controlnet/controlnet.safetensors"
SD_CLI="$HOME/stable-diffusion.cpp/build/bin/sd-cli"
LORA_DIR="$HOME/.ai-assets/loras"

# Model selection based on mode
if [ "$MODE" == "fast" ]; then
    echo "=== FAST MODE: Using FLUX Schnell (4 steps) ==="
    MODEL="$HOME/.ai-assets/models/flux/flux-schnell-q4.gguf"
    STEPS=4
else
    echo "=== QUALITY MODE: Using FLUX Dev (20 steps) ==="
    MODEL="$HOME/.ai-assets/models/flux/flux-dev-q8.gguf"
    STEPS=20
fi

VAE="$HOME/.ai-assets/models/flux/ae.safetensors"
CLIP_L="$HOME/.ai-assets/models/flux/clip_l.safetensors"
T5XXL="$HOME/.ai-assets/models/flux/t5-Q5_K_M.gguf"

# Check if edge template exists
if [ ! -f "$EDGE_TEMPLATE" ]; then
    echo "ERROR: Edge template not found: $EDGE_TEMPLATE"
    echo "Please create a commander pose reference and run:"
    echo "  python3 scripts/extract-edges.py <reference.png> $EDGE_TEMPLATE"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Commander definitions (Name => Prompt)
declare -A ARGENTUM_COMMANDERS=(
    ["supreme-commander"]="A powerful female Supreme Commander in gleaming white and gold art deco armor, polished brass accents, confident authoritative pose with hand on sword hilt, white marble background"
    ["field-marshal"]="A commanding female Field Marshal in polished white and brass military armor, tactical stance, steel geometric patterns, industrial workshop background"
    ["master-artificer"]="An elegant female Master Artificer in white corset with golden filigree circuits, brass goggles on head, confident intelligent expression, clockwork workshop"
    ["grand-inquisitor"]="A stern female Grand Inquisitor in white robes with gold trim, holding a glowing justice scale, authoritative pose, marble courthouse background"
)

declare -A SYMBIOTE_COMMANDERS=(
    ["alpha-bioweaver"]="A fierce female Alpha Bio-Weaver with living bio-armor made of bioluminescent purple vines and bone plates, powerful stance, jungle background with glowing spore trees"
    ["beast-master"]="A commanding female Beast Master with genetically enhanced features, chitinous armor, bioluminescent tattoos, confident pose, living jungle city background"
    ["spore-matriarch"]="A regal female Spore Matriarch covered in organic fungal growths, purple bioluminescent patterns, wise expression, massive mushroom throne background"
    ["apex-predator"]="A dangerous female Apex Predator with bio-engineered hunter features, sleek adaptive armor, predatory stance, deep jungle hunting grounds"
)

declare -A OBSIDION_COMMANDERS=(
    ["hemomancer-queen"]="An alluring vampire sorceress Hemomancer Queen in elegant crimson velvet dress with gothic corset, pale skin, glowing blue essence tubes, confident seductive expression, dark spire tower"
    ["spellblade-assassin"]="A deadly female Spellblade Assassin in black leather armor with crimson accents, wielding glowing neon blue energy blade, graceful combat pose, gothic victorian rooftops"
    ["lich-scholar"]="An ancient female Lich Scholar with preserved pale beauty, dark robes with blue glowing runes, holding arcane tome, mysterious pose, library of forbidden knowledge"
    ["blood-oracle"]="A mysterious female Blood Oracle in flowing crimson and black robes, glowing blue eyes, blood magic swirling around hands, prophetic stance, ritual chamber"
)

# Function to generate a commander
generate_commander() {
    local faction=$1
    local name=$2
    local prompt=$3
    local lora_style=$4
    
    local full_prompt="${prompt}, 90s Magic the Gathering card art, painted fantasy illustration <lora:${lora_style}>"
    local output_path="${OUTPUT_DIR}/${faction}-${name}.png"
    
    echo "Generating: ${faction} - ${name}"
    
    $SD_CLI \
        --diffusion-model "$MODEL" \
        --vae "$VAE" \
        --clip_l "$CLIP_L" \
        --t5xxl "$T5XXL" \
        --control-net "$CONTROLNET" \
        --control-image "$EDGE_TEMPLATE" \
        --control-strength 0.75 \
        --lora-model-dir "$LORA_DIR" \
        -p "$full_prompt" \
        --cfg-scale 1.0 \
        --sampling-method euler \
        --steps $STEPS \
        -H 896 -W 704 \
        -o "$output_path"
    
    echo "✓ Saved: $output_path"
    echo ""
}

# Generate all Argentum commanders
echo "=== Generating ARGENTUM COMBINE Commanders ==="
for name in "${!ARGENTUM_COMMANDERS[@]}"; do
    generate_commander "argentum" "$name" "${ARGENTUM_COMMANDERS[$name]}" "classical-painting:0.7"
done

# Generate all Symbiote commanders
echo "=== Generating SYMBIOTE CIRCLES Commanders ==="
for name in "${!SYMBIOTE_COMMANDERS[@]}"; do
    generate_commander "symbiote" "$name" "${SYMBIOTE_COMMANDERS[$name]}" "classical-painting:0.6"
done

# Generate all Obsidion commanders
echo "=== Generating OBSIDION SYNDICATE Commanders ==="
for name in "${!OBSIDION_COMMANDERS[@]}"; do
    generate_commander "obsidion" "$name" "${OBSIDION_COMMANDERS[$name]}" "classical-painting:0.5,frazetta:0.4"
done

echo "==================================="
echo "✓ All commanders generated!"
echo "Output directory: $OUTPUT_DIR"
echo "==================================="
