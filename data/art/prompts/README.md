# Art Prompts - Essence Wars

This directory contains YAML-formatted prompts for generating art assets using FLUX via stable-diffusion.cpp.

## Structure

- `commanders.yaml` - Commander portrait prompts (IDs 5000-5011)
- `core_set/` - Card art prompts organized by faction
  - `argentum.yaml` - Argentum Combine cards (IDs 1000-1074)
  - `symbiote.yaml` - Symbiote Circles cards (IDs 2000-2074)
  - `obsidion.yaml` - Obsidion Syndicate cards (IDs 3000-3074)
  - `neutral.yaml` - Neutral cards (IDs 4000-4074)

## YAML Format

All prompt files follow a consistent structure:

```yaml
faction: argentum  # or symbiote, obsidion, neutral (cards only)
set: core_set      # cards only

cards:  # or commanders:
  - id: 1050
    name: "Card Name"
    card_type: creature  # cards only
    tags: [Soldier]      # cards only
    faction: argentum    # commanders only
    archetype: "..."     # commanders only
    prompt: >
      Detailed generation prompt...
    lora: "model:weight, model:weight"
    negative: "unwanted elements..."
```

## Generation Scripts

### Card Art
```bash
# Generate all cards for a faction
python3 scripts/generate-card-art.py --faction argentum

# Generate specific card range
python3 scripts/generate-card-art.py --faction symbiote --start 2000 --end 2010

# Generate single card
python3 scripts/generate-card-art.py --faction argentum --card 1050

# Generate all factions
python3 scripts/generate-card-art.py --faction all
```

### Commander Portraits
```bash
# Generate all commanders for a faction
python3 scripts/generate-commander-art.py --faction argentum

# Generate single commander
python3 scripts/generate-commander-art.py --commander 5000

# Generate all commanders
python3 scripts/generate-commander-art.py --faction all

# Dry run (preview without generating)
python3 scripts/generate-commander-art.py --faction argentum --dry-run
```

## Output Locations

- Card art: `crates/essence-wars-ui/static/cards/core_set/{id}.webp`
- Commander portraits: `crates/essence-wars-ui/static/portrait/{name}.webp`

## Deprecated Files

The following files have been replaced by the YAML-based system:

- `commander_portraits.md.deprecated` - Old markdown format (replaced by `commanders.yaml`)
- `scripts/generate-commander-portraits.sh.deprecated` - Old bash script (replaced by `generate-commander-art.py`)

## Style Guide

See `data/art/style_guide.yaml` for:
- Faction aesthetics
- Color palettes
- Art direction guidelines
- Technical specifications
