# New Horizons Card Expansion Plan

**Phase 2 of the Essence Wars: Card Expansion Initiative**

---

## Overview

**Goal**: Expand from 47 cards to ~200+ cards using faction-based design and AI-driven balance testing.

**Current Version**: 0.3.0 (Phase 1.5 complete)
**Target Version**: 0.4.0

---

## Part 1: Design Philosophy

### 1.1 Stat Budget Formula

Based on starter set analysis:

| Cost | Base Stats | With 1 Keyword | With 2 Keywords |
|------|------------|----------------|-----------------|
| 1 | 3-4 | 2-3 | 1-2 |
| 2 | 5 | 4-5 | 3-4 |
| 3 | 6-7 | 5-6 | 4-5 |
| 4 | 8-9 | 7-8 | 6-7 |
| 5 | 9-10 | 8-9 | 7-8 |
| 6 | 10-11 | 9-10 | 8-9 |
| 7+ | 12+ | 11+ | 10+ |

**Keyword Value Costs**:
- Rush, Guard, Shield, Ranged: ~1 stat
- Piercing, Lifesteal: ~1.5 stats
- Quick, Lethal: ~2 stats
- Ephemeral: **-1.5 stats (BONUS!)**
- Regenerate, Stealth, Charge: ~1 stat

### 1.2 Rarity Guidelines

| Rarity | % of Set | Characteristics |
|--------|----------|-----------------|
| **Common** | 50% | Vanilla or single keyword |
| **Uncommon** | 35% | Single keyword + ability, or 2 keywords |
| **Rare** | 12% | Multi-keyword, strong abilities |
| **Legendary** | 3% | Game-defining power level |

### 1.3 Target Card Distribution (200 cards)

- **Creatures**: 150 (75%)
- **Spells**: 35 (17.5%)
- **Supports**: 15 (7.5%)

---

## Part 2: Faction Design Framework

### 🏛️ ARGENTUM COMBINE — "The Wall"

**Identity**: Order, Industry, Defense
**Lore**: Art Deco Steampunk, "Structure is Safety"

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Guard, Piercing, Shield |
| **Archetypes** | Soldiers, Constructs, Engineers |
| **Strengths** | High HP, defensive formations |
| **Weaknesses** | Low burst, slow tempo |
| **Avoid** | Rush, Lethal, Ephemeral, Stealth |

**Design Space**:
- Formation bonuses (adjacent creature buffs)
- Construct synergies
- Damage reduction effects
- "Wall-building" supports

---

### 🌿 SYMBIOTE CIRCLES — "The Swarm"

**Identity**: Growth, Adaptation, Evolution
**Lore**: Biopunk Fantasy, "Adapt or Perish"

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Rush, Lethal, Regenerate |
| **Archetypes** | Beasts, Healers, Parasites |
| **Strengths** | Tempo, trading, sustain |
| **Weaknesses** | Low board control |
| **Avoid** | Guard, Shield |

**Design Space**:
- Token generation (spawn creatures)
- Evolution/mutation effects
- Poison persistence
- Self-healing synergies

---

### 🔮 OBSIDION SYNDICATE — "The Glass Cannon"

**Identity**: Knowledge, Ambition, Power
**Lore**: Gothic Cyber-Magic, "Power is Personal"

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Lifesteal, Quick, Stealth, Ephemeral |
| **Archetypes** | Mages, Cultists, Assassins, Undead |
| **Strengths** | Direct damage, resource manipulation |
| **Weaknesses** | Low creature stats |
| **Avoid** | Guard, Regenerate |

**Design Space**:
- Life manipulation (drain, sacrifice)
- Temporary/ephemeral creatures
- "Draw at a cost" effects
- Assassination combos

---

### ⚖️ FREE-WALKERS — "The Toolbox"

**Identity**: Mercenaries, Flexibility, Profit
**Lore**: Rugged & Practical, "No Flag. Just Gold."

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Ranged, Charge |
| **Archetypes** | Giants, Hunters, Mercenaries |
| **Strengths** | Flexibility, precision |
| **Weaknesses** | No strong faction identity |
| **Can Use** | Any keywords (neutral) |

**Design Space**:
- Ranged precision damage
- Charge aggression
- Support infrastructure
- "For hire" utility effects

---

## Part 3: The "Grinder" Balance Process

### 3.1 Batch Workflow

```
DESIGN → Create 15 cards in YAML
   ↓
TEST → Run Arena (MCTS vs MCTS, 100+ games)
   ↓
ANALYZE → Check win rates (target: 45-55%)
   ↓
TUNE → Adjust stats if outside range
   ↓
COMMIT → Lock balanced cards
```

### 3.2 Balance Commands

```bash
# Test new batch vs existing decks
cargo run --release --bin arena -- \
  --bot1 mcts --bot2 mcts \
  --deck1 batch_test --deck2 defensive_control \
  --games 100 --progress

# Check specific matchup
cargo run --release --bin arena -- \
  --deck1 argentum_test --deck2 symbiote_test \
  --games 50
```

### 3.3 Win Rate Targets

| Range | Verdict | Action |
|-------|---------|--------|
| 45-55% | ✅ Balanced | Commit |
| 55-60% | ⚠️ Slightly OP | Minor stat reduction |
| 60%+ | 🔴 OP | Major nerf |
| 40-45% | ⚠️ Slightly UP | Minor stat increase |
| <40% | 🔴 UP | Major buff |

---

## Part 4: Implementation Batches

### Batch A: Argentum Combine (15 cards)

**File**: `data/cards/sets/new-horizons.yaml` (IDs 48-62)

| ID | Name | Cost | Stats | Keywords | Type | Rarity |
|----|------|------|-------|----------|------|--------|
| 48 | Brass Sentinel | 2 | 1/5 | Guard | Creature | Common |
| 49 | Steam Knight | 3 | 3/3 | Piercing | Creature | Common |
| 50 | Cogwork Defender | 3 | 2/6 | Guard | Creature | Common |
| 51 | Factory Foreman | 4 | 2/4 | - | Creature | Uncommon |
| 52 | Iron Colossus | 5 | 3/8 | Guard | Creature | Uncommon |
| 53 | Pressure Trooper | 3 | 4/2 | Piercing | Creature | Common |
| 54 | Shield Bearer | 2 | 2/3 | Shield | Creature | Common |
| 55 | Clockwork Medic | 3 | 1/4 | - | Creature | Uncommon |
| 56 | Siege Engine | 6 | 4/7 | Guard, Piercing | Creature | Rare |
| 57 | Grand Constructor | 7 | 5/8 | - | Creature | Legendary |
| 58 | Reinforce | 2 | - | - | Spell | Common |
| 59 | Emergency Repairs | 3 | - | - | Spell | Common |
| 60 | Construct Protocol | 4 | - | - | Spell | Uncommon |
| 61 | Assembly Line | 4 | dur:3 | - | Support | Uncommon |
| 62 | Fortification | 3 | dur:4 | - | Support | Uncommon |

---

### Batch B: Symbiote Circles (15 cards)

**File**: `data/cards/sets/new-horizons.yaml` (IDs 63-77)

| ID | Name | Cost | Stats | Keywords | Type | Rarity |
|----|------|------|-------|----------|------|--------|
| 63 | Spore Crawler | 1 | 1/2 | - | Creature | Common |
| 64 | Venom Fang | 2 | 2/2 | Lethal | Creature | Uncommon |
| 65 | Regenerating Ooze | 3 | 2/5 | Regenerate | Creature | Uncommon |
| 66 | Broodling | 1 | 2/1 | Rush | Creature | Common |
| 67 | Hive Guardian | 4 | 3/5 | Guard, Regen | Creature | Rare |
| 68 | Pack Hunter | 2 | 3/1 | Rush | Creature | Common |
| 69 | Parasitic Larva | 1 | 1/1 | Lethal | Creature | Uncommon |
| 70 | Evolution Chamber | 3 | 2/4 | - | Creature | Uncommon |
| 71 | Alpha Predator | 5 | 5/4 | Rush, Lethal | Creature | Rare |
| 72 | Swarm Mother | 6 | 4/6 | - | Creature | Rare |
| 73 | Acid Spitter | 3 | 3/2 | Ranged | Creature | Common |
| 74 | Carapace Warrior | 4 | 2/6 | Regenerate | Creature | Common |
| 75 | Rapid Mutation | 2 | - | - | Spell | Uncommon |
| 76 | Consume | 3 | - | - | Spell | Uncommon |
| 77 | Spawning Pool | 4 | dur:3 | - | Support | Rare |

---

### Batch C: Obsidion Syndicate (15 cards)

**File**: `data/cards/sets/new-horizons.yaml` (IDs 78-92)

| ID | Name | Cost | Stats | Keywords | Type | Rarity |
|----|------|------|-------|----------|------|--------|
| 78 | Soul Fragment | 1 | 2/2 | Ephemeral | Creature | Common |
| 79 | Shadow Blade | 2 | 3/1 | Stealth | Creature | Uncommon |
| 80 | Blood Acolyte | 2 | 2/3 | Lifesteal | Creature | Common |
| 81 | Void Walker | 3 | 3/3 | Ephemeral, Rush | Creature | Uncommon |
| 82 | Hemomancer | 4 | 3/4 | Lifesteal | Creature | Uncommon |
| 83 | Silent Assassin | 4 | 4/2 | Stealth, Quick | Creature | Rare |
| 84 | Ritual Master | 5 | 4/4 | - | Creature | Rare |
| 85 | The Eternal One | 8 | 6/6 | Lifesteal, Regen | Creature | Legendary |
| 86 | Soul Drain | 2 | - | - | Spell | Common |
| 87 | Dark Insight | 3 | - | - | Spell | Uncommon |
| 88 | Shadow Strike | 1 | - | - | Spell | Common |
| 89 | Obliteration | 4 | - | - | Spell | Uncommon |
| 90 | Mind Shatter | 3 | - | - | Spell | Uncommon |
| 91 | Cataclysm | 6 | - | - | Spell | Rare |
| 92 | Dark Altar | 5 | dur:3 | - | Support | Rare |

---

### Batch D: Free-Walkers (15 cards)

**File**: `data/cards/sets/new-horizons.yaml` (IDs 93-107)

| ID | Name | Cost | Stats | Keywords | Type | Rarity |
|----|------|------|-------|----------|------|--------|
| 93 | Bounty Hunter | 2 | 2/2 | Ranged | Creature | Common |
| 94 | Berserker | 3 | 3/2 | Charge | Creature | Uncommon |
| 95 | Wasteland Scout | 2 | 2/3 | Ranged | Creature | Common |
| 96 | Hired Blade | 3 | 3/3 | - | Creature | Common |
| 97 | War Veteran | 4 | 4/4 | Charge | Creature | Uncommon |
| 98 | Siege Giant | 6 | 6/5 | Piercing | Creature | Uncommon |
| 99 | Sharpshooter | 4 | 3/3 | Ranged | Creature | Common |
| 100 | Reckless Charger | 2 | 4/1 | Charge, Rush | Creature | Uncommon |
| 101 | Scrap Golem | 5 | 4/6 | - | Creature | Common |
| 102 | The Warbringer | 7 | 7/7 | Charge, Piercing | Creature | Legendary |
| 103 | Hired Help | 3 | - | - | Spell | Common |
| 104 | Precision Shot | 2 | - | - | Spell | Common |
| 105 | Supply Depot | 3 | dur:4 | - | Support | Uncommon |
| 106 | Mercenary Camp | 4 | dur:3 | - | Support | Uncommon |
| 107 | Artillery Platform | 5 | dur:2 | - | Support | Rare |

---

## Part 5: Files to Create

```
data/cards/sets/new-horizons.yaml     # 60 new cards (Batches A-D)
data/decks/argentum_fortress.toml     # Argentum faction deck
data/decks/symbiote_swarm.toml        # Symbiote faction deck
data/decks/obsidion_shadow.toml       # Obsidion faction deck
data/decks/freewalker_mercenary.toml  # Free-Walker deck
```

---

## Part 6: Verification

1. **Card Validation**: All cards load correctly
   ```bash
   cargo run --bin arena -- --list-decks
   ```

2. **Balance Testing**: Run cross-faction matchups
   ```bash
   cargo run --release --bin arena -- \
     --deck1 argentum_fortress --deck2 symbiote_swarm \
     --games 100 --progress
   ```

3. **Regression**: Existing tests still pass
   ```bash
   cargo nextest run --status-level=fail
   ```

---

## Questions for User

1. **Batch Scope**: Start with all 4 batches (60 cards), or one batch at a time?

2. **Legendary Count**: 1 legendary per faction (4 total) or more?

3. **Deck Composition**: Pure faction or faction + neutrals?

4. **Ability Effects**: Should new cards have triggered abilities, or keep it simple with keywords only for now?
