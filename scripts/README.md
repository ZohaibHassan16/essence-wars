# Scripts Directory

Automation scripts for Essence Wars development, testing, and art generation.

## Game Development Scripts

### Core Testing & Validation

**`run-tests.sh`**
- Runs the full test suite using cargo nextest
- Shows only failures for cleaner output
- Usage: `./scripts/run-tests.sh`

**`run-clippy.sh`**
- Lints production code (excludes tests)
- Checks for common issues and non-idiomatic code
- Usage: `./scripts/run-clippy.sh`

**`run-benchmarks.sh`**
- Runs Criterion benchmark suite
- Generates performance reports
- Usage: `./scripts/run-benchmarks.sh`

**`validate-decks.sh`**
- Validates all TOML deck files against card database
- Prevents runtime errors from missing card references
- Usage: `./scripts/validate-decks.sh`

### Performance & Analysis

**`perf-stats.sh`**
- Performance profiling and statistics
- Memory usage analysis
- Usage: `./scripts/perf-stats.sh`

**`generate_batch.py`**
- Batch game data generation for ML training
- Creates datasets for neural network training
- Usage: `python3 scripts/generate_batch.py --help`

## Art Generation Scripts (NEW)

### ControlNet Setup & Testing

**`test-controlnet.sh`** ⭐ Start here!
- Test FLUX ControlNet setup with a simple example
- Creates test reference, extracts edges, generates sample image
- **Usage:**
  ```bash
  # Fast test with FLUX Schnell (~30 seconds)
  ./scripts/test-controlnet.sh fast
  
  # Quality test with FLUX Dev (~2-3 minutes)
  ./scripts/test-controlnet.sh quality
  ```
- **Output:** `~/.ai-assets/test-controlnet/controlnet_test_output.png`

### Edge Extraction

**`extract-edges.py`** 🎨
- Extract Canny edges from reference images for ControlNet guidance
- CLI-friendly for AI agent automation
- **Usage:**
  ```bash
  # Basic usage (OpenCV method)
  python3 scripts/extract-edges.py reference.png edges.png
  
  # Adjust sensitivity
  python3 scripts/extract-edges.py reference.png edges.png --low 50 --high 150
  
  # Use ImageMagick instead
  python3 scripts/extract-edges.py reference.png edges.png --method imagemagick
  
  # Help
  python3 scripts/extract-edges.py --help
  ```

### Batch Commander Generation

**`generate-commanders.sh`** 👑
- Generate all 12 commander portraits with consistent pose using ControlNet
- Requires edge template: `data/art/templates/commander-pose-edges.png`
- **Usage:**
  ```bash
  # Fast iteration (FLUX Schnell, 4 steps each)
  ./scripts/generate-commanders.sh fast
  
  # Final quality (FLUX Dev, 20 steps each)
  ./scripts/generate-commanders.sh quality
  ```
- **Output:** `data/art/commanders/{faction}-{name}.png`
- **Commanders Generated:**
  - 4 Argentum Combine commanders
  - 4 Symbiote Circles commanders
  - 4 Obsidion Syndicate commanders

## UI Launch Scripts

**`launch-ui.sh`**
- Builds and launches the Tauri desktop UI
- Compiles Rust backend and Svelte frontend
- Usage: `./scripts/launch-ui.sh`

## Art Generation Workflow

### 1. Create Reference Pose Template

```bash
# Option A: Use a reference photo
python3 scripts/extract-edges.py hero-pose-photo.png data/art/templates/commander-pose-edges.png

# Option B: Draw a simple sketch (896x704 portrait)
# Then extract edges:
python3 scripts/extract-edges.py sketch.png data/art/templates/commander-pose-edges.png
```

### 2. Test ControlNet Setup

```bash
# Quick test to verify everything works
./scripts/test-controlnet.sh fast
```

### 3. Generate All Commanders

```bash
# Fast iteration to preview all 12
./scripts/generate-commanders.sh fast

# Review outputs in: data/art/commanders/

# Final high-quality generation
./scripts/generate-commanders.sh quality
```

### 4. Custom UI Elements

```bash
# Extract edges from frame sketch
python3 scripts/extract-edges.py frame-sketch.png frame-edges.png

# Generate with faction-specific prompt
cd ~/stable-diffusion.cpp
./build/bin/sd-cli \
  --control-net ~/.ai-assets/models/flux/controlnet/controlnet.safetensors \
  --control-image frame-edges.png \
  --control-strength 0.9 \
  -p "ornate art deco gold filigree frame, white marble" \
  -H 512 -W 512 -o argentum-frame.png
```

## Environment Variables

Art generation scripts support these overrides:

```bash
# Custom edge template location
export EDGE_TEMPLATE=/path/to/custom-edges.png

# Custom output directory
export OUTPUT_DIR=/path/to/output

# Then run:
./scripts/generate-commanders.sh quality
```

## Requirements

### Game Development
- Rust toolchain (stable)
- cargo-nextest (for testing)
- cargo-criterion (for benchmarks)

### Art Generation
- **ImageMagick**: `sudo apt-get install imagemagick`
- **Python 3** with opencv-python (optional): `pip install opencv-python`
- **stable-diffusion.cpp**: Built and available at `~/stable-diffusion.cpp`
- **FLUX models**: Located at `~/.ai-assets/models/flux/`
- **FLUX ControlNet**: `~/.ai-assets/models/flux/controlnet/controlnet.safetensors`

## Troubleshooting

### ControlNet Issues

**Problem:** `test-controlnet.sh` fails with "model not found"
```bash
# Check if ControlNet exists
ls -lh ~/.ai-assets/models/flux/controlnet/controlnet.safetensors

# Should be ~1.4GB
```

**Problem:** Edge extraction fails
```bash
# Try ImageMagick method
python3 scripts/extract-edges.py ref.png edges.png --method imagemagick

# Or install OpenCV
pip install opencv-python
```

**Problem:** Commander generation fails on template
```bash
# Template doesn't exist yet - create one first:
python3 scripts/extract-edges.py your-pose-reference.png data/art/templates/commander-pose-edges.png
```

## AI Agent Integration

All scripts are designed to be:
- **CLI-only** (no GUI required)
- **Deterministic** (same inputs = same outputs)
- **Parseable output** (clear SUCCESS/ERROR messages)
- **Automatable** (exit codes, standard error handling)

Perfect for AI developer and AI artist agents!

---

**See also:**
- [FLUX Guide](../docs/flux-guide.md) - Complete FLUX + ControlNet documentation
- [Art Templates README](../data/art/templates/README.md) - Template creation guide
