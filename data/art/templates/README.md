# Art Templates for ControlNet

This directory contains reference templates for consistent ControlNet-guided generation.

## Commander Pose Template

**File:** `commander-pose-edges.png`

**Purpose:** Ensure all 12 commanders have consistent portrait composition and pose.

**How to Create:**

### Option 1: Use a Reference Photo
```bash
# Find a good portrait pose (heroic, confident stance)
# Save as: commander-pose-reference.png

# Extract edges
python3 scripts/extract-edges.py commander-pose-reference.png commander-pose-edges.png
```

### Option 2: Use 3D Mannequin (Blender/DAZ)
```bash
# Create pose in 3D software
# Render simple white figure on black background (896x704)
# Extract edges

python3 scripts/extract-edges.py mannequin-render.png commander-pose-edges.png
```

### Option 3: Simple Sketch
```bash
# Draw a simple stick figure pose in any image editor
# Save as PNG (896x704)
# Extract edges

python3 scripts/extract-edges.py sketch.png commander-pose-edges.png
```

## UI Frame Templates

**Files:** `frame-{shape}-edges.png`

**Purpose:** Generate faction-specific frames and borders with consistent shapes.

**Examples:**
- `frame-card-edges.png` - Card frame outline
- `frame-button-edges.png` - UI button borders
- `frame-banner-edges.png` - Banner decorations

## Playmat Template

**File:** `playmat-layout-edges.png`

**Purpose:** Maintain consistent zones for card placement while varying background art.

**Layout Zones:**
- Top: Opponent creature lanes
- Center: Battlefield/combat zone
- Bottom: Player creature lanes
- Sides: Safe zones for effects/resources

## Usage with Scripts

Once templates exist, run:

```bash
# Generate all commanders with consistent pose
./scripts/generate-commanders.sh quality

# Or fast iteration
./scripts/generate-commanders.sh fast
```

## Tips

- **Control Strength:**
  - 0.9-1.0: Very strict (UI frames, exact shapes)
  - 0.7-0.8: Balanced (commander poses)
  - 0.5-0.6: Loose guidance (backgrounds, playmats)

- **Edge Quality:**
  - Clean, high-contrast edges work best
  - Adjust thresholds in extract-edges.py if needed
  - Test with FLUX Schnell before full Dev generation
