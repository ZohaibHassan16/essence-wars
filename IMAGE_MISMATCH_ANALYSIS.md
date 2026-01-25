# Image File Mismatch Analysis

## Problem Summary

There's a systemic mismatch between:
1. **Art Prompts YAML** (what images were generated from)
2. **Card Data YAML** (actual game cards)  
3. **Portrait Prompts MD** (portrait generation document)

## Argentum Commanders - The Mismatch

### Art Prompts (`data/art/prompts/core_set/argentum.yaml`)
These prompts generated the `.webp` image files:

| ID | Name | Description | Image File |
|----|------|-------------|------------|
| 1056 | The High Artificer | Female artificer with mechanical arms | `1056.webp` |
| 1057 | The Sanctum Healer | Female healing artificer | `1057.webp` |
| 1058 | Siege Marshal Vex | Male military commander | `1058.webp` |
| 1059 | The Grand Architect | Female architect | `1059.webp` |

### Card Data (`data/cards/core_set/argentum.yaml`)
These are the actual playable commanders:

| ID | Name | Description |
|----|------|-------------|
| 1057 | Iron Colossus Prime | Massive clockwork golem |
| 1058 | Siege Marshal Vex | Male military commander |
| 1059 | The Grand Architect | Female architect with Fortify |
| ~~1060~~ | ~~Cog Assembler~~ | **NOT A COMMANDER** - regular creature |

**Missing:** ID 1056 (The High Artificer) does not exist in card data!

### Portrait Prompts (`data/art/prompts/commander_portraits.md`)  
Uses **invented names** that don't match either source:

| ID | Wrong Name in File | Correct Name | Should Use Image |
|----|-------------------|--------------|------------------|
| 1057 | The Sanctum Healer ✓ | The Sanctum Healer | 1057.webp ✓ |
| 1058 | "Master Artificer Lyra" ❌ | Siege Marshal Vex | 1058.webp (not 1060.webp!) |
| 1059 | "General Ironforge" ❌ | The Grand Architect | 1059.webp (not 1058.webp!) |
| 1060 | "Architect Supreme Zara" ❌ | N/A - Not a commander! | 1059.webp used incorrectly |

## Root Cause

1. **Art prompts were created first** with 4 commanders (1056-1059)
2. **Card data was implemented** with only 3 commanders (1057-1059), dropping 1056
3. **Portrait prompts were written** using made-up names instead of canonical card names
4. **Image paths in portrait file are off-by-one** because they reference the wrong IDs

## Impact

### Files Affected
- ✅ `data/art/prompts/commander_portraits.md` - Wrong names, wrong image paths
- ❓ `crates/essence-wars-ui/` - May be rendering wrong images for commanders
- ❓ Any UI code that maps card ID → image file

### Correct Mapping Should Be

| Card ID | Card Name | Image File to Use |
|---------|-----------|-------------------|
| 1057 | The Sanctum Healer | `1057.webp` |
| 1058 | Siege Marshal Vex | `1058.webp` |
| 1059 | The Grand Architect | `1059.webp` |

**Note:** `1056.webp` (The High Artificer) exists but has no corresponding card!

## Similar Issues in Other Factions?

Need to check:
- Symbiote commanders (2055-2058)
- Obsidion commanders (3054-3057)

## Recommended Fix

1. **Update `commander_portraits.md`:**
   - Use canonical card names from `data/cards/core_set/*.yaml`
   - Fix image paths to match card IDs
   
2. **Check UI rendering code:**
   - Verify card ID 1058 renders `1058.webp` (not some other file)
   - Check if there's any off-by-one logic in image path resolution

3. **Decide on 1056.webp:**
   - Either: Create card 1056 "The High Artificer" as 4th Argentum commander
   - Or: Delete unused `1056.webp` file
   - Or: Repurpose image for a different card
