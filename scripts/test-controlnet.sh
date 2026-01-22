#!/bin/bash
# Test FLUX ControlNet setup with a simple example
# Usage: ./test-controlnet.sh [fast|quality]

set -e

MODE="${1:-fast}"  # Default to fast mode for quick testing

# Paths
TEST_DIR="$HOME/.ai-assets/test-controlnet"
CONTROLNET="$HOME/.ai-assets/models/flux/controlnet/controlnet.safetensors"
SD_CLI="$HOME/stable-diffusion.cpp/build/bin/sd-cli"

# Model selection
if [ "$MODE" == "fast" ]; then
    echo "=== FAST TEST: Using FLUX Schnell ==="
    MODEL="$HOME/.ai-assets/models/flux/flux-schnell-q4.gguf"
    STEPS=4
else
    echo "=== QUALITY TEST: Using FLUX Dev ==="
    MODEL="$HOME/.ai-assets/models/flux/flux-dev-q8.gguf"
    STEPS=20
fi

VAE="$HOME/.ai-assets/models/flux/ae.safetensors"
CLIP_L="$HOME/.ai-assets/models/flux/clip_l.safetensors"
T5XXL="$HOME/.ai-assets/models/flux/t5-Q5_K_M.gguf"

# Create test directory
mkdir -p "$TEST_DIR"

echo "Creating test reference image..."
# Create a simple stick figure pose
convert -size 704x896 xc:white \
    -fill black \
    -draw "ellipse 352,300 150,200 0,360" \
    -draw "rectangle 250,500 454,700" \
    -draw "line 250,600 150,650" \
    -draw "line 454,600 554,650" \
    "$TEST_DIR/reference.png"

echo "Extracting Canny edges..."
# Extract edges as grayscale, then convert to RGB (ControlNet needs 3 channels)
convert "$TEST_DIR/reference.png" -canny 0x1+10%+30% "$TEST_DIR/edges_gray.png"
convert "$TEST_DIR/edges_gray.png" -separate -combine "PNG24:$TEST_DIR/edges.png"
rm "$TEST_DIR/edges_gray.png"

echo "Testing ControlNet generation..."
$SD_CLI \
    --diffusion-model "$MODEL" \
    --vae "$VAE" \
    --clip_l "$CLIP_L" \
    --t5xxl "$T5XXL" \
    --control-net "$CONTROLNET" \
    --control-image "$TEST_DIR/edges.png" \
    --control-strength 0.75 \
    -p "A powerful female warrior in ornate armor, heroic pose, 90s Magic the Gathering card art" \
    --cfg-scale 1.0 \
    --sampling-method euler \
    --steps $STEPS \
    -H 896 -W 704 \
    -o "$TEST_DIR/controlnet_test_output.png"

echo ""
echo "==================================="
echo "✓ ControlNet test complete!"
echo "Reference: $TEST_DIR/reference.png"
echo "Edges: $TEST_DIR/edges.png"
echo "Output: $TEST_DIR/controlnet_test_output.png"
echo "==================================="
echo ""
echo "View results:"
echo "  ls -lh $TEST_DIR"
