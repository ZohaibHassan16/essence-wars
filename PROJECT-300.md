# The "Project 300" Roadmap

**Mission**: Design and implement 300 Cards for the initial `New Horizons` Edition of Essence Wars.

## Current State (v0.5.0 - Phase 2B Complete)

### Card Pool Summary
| Category | Count | Target |
|----------|-------|--------|
| **Total Cards** | 80 | 300 |
| Argentum Combine | 15 | ~75 |
| Symbiote Circles | 35 | ~75 |
| Obsidion Syndicate | 15 | ~75 |
| Free-Walkers (Neutral) | 15 | ~75 |
| **Support Cards** | 7 | ~30 |
| **Legendary Cards** | 3 | ~12 |

### Balance Baseline (20,000 games, validated 2026-01-15)
| Metric | Value | Status |
|--------|-------|--------|
| P1 Win Rate | 51.9% | BALANCED |
| Argentum | 58.8% | **Strongest** |
| Obsidion | 49.5% | Balanced |
| Symbiote | 41.7% | **Weakest** |
| Max Delta | 17.2% | Needs work |

**Matchup Matrix:**
| Matchup | Result |
|---------|--------|
| Argentum vs Symbiote | 60.2% / 39.8% |
| Argentum vs Obsidion | 57.5% / 42.5% |
| Symbiote vs Obsidion | 43.6% / 56.4% |

**Key Finding:** Symbiote is the weakest faction and needs targeted help.

### Keyword Slots
- **Used:** 14 of 16 (Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick, Ephemeral, Regenerate, Stealth, Charge, **Frenzy**, **Volatile**)
- **Available:** 2 slots reserved for future balance tuning

---

## Milestone 1: The Foundation Refactor [COMPLETED]

**Status:** Done (v0.4.0 - v0.5.0)

### Completed Work
- Data restructured into `data/cards/core_set/` with faction-specific YAML files
- Decks organized into `data/decks/{faction}/` subdirectories
- Card IDs migrated to faction ranges (Argentum 1000+, Symbiote 2000+, Obsidion 3000+, Neutral 4000+)
- P1/P2 balance fixed (P2 +2 bonus cards compensation)
- MCTS immediate win detection implemented
- All tests passing, CI/CD operational

### Current Structure
```
data/
├── cards/core_set/
│   ├── argentum.yaml   (IDs 1000-1014)
│   ├── symbiote.yaml   (IDs 2000-2034)
│   ├── obsidion.yaml   (IDs 3000-3014)
│   └── neutral.yaml    (IDs 4000-4014)
├── decks/
│   ├── argentum/
│   │   ├── control.toml
│   │   └── midrange.toml
│   ├── symbiote/
│   │   ├── aggro.toml
│   │   ├── tempo.toml
│   │   ├── frenzy_aggro.toml
│   │   └── volatile_swarm.toml
│   └── obsidion/
│       ├── burst.toml
│       └── control.toml
└── weights/
    ├── generalist.toml
    └── specialists/{faction}.toml
```

---

## Milestone 2: Symbiote Rising (Balance Fix) [COMPLETED]

**Goal:** Address Symbiote's 41.7% win rate through new keywords and targeted card additions.

### Phase 2A: New Keywords (Symbiote Focus) [DONE]

**Strategy:** Use 2 of 4 remaining keyword slots to give Symbiote aggressive tools.

| Keyword | Effect | Primary Faction | Design Rationale |
|---------|--------|-----------------|------------------|
| **Frenzy** | +1 attack after each attack this turn | Symbiote | Rewards aggressive multi-attack strategies |
| **Volatile** | Deal 2 damage to all enemy creatures on death | Symbiote | Punishes Argentum's board-centric control |

**Completed Implementation:**
- Added keywords to `src/core/keywords.rs` (bits 12-13)
- Frenzy: +1 attack per stack after each attack, stacks reset at end of turn
- Volatile: Deals 2 damage to all enemy creatures on death, can chain
- Created 6 initial cards using the keywords (IDs 2015-2020)
- Updated bot weights with keyword_frenzy and keyword_volatile parameters

**Reserved Keywords (Phase 2 later):**
- 2 slots kept for fine-tuning after initial balance pass
- Candidates: Fortify (Argentum defense), Siphon (Obsidion utility)

### Phase 2B: Symbiote Card Wave (+14 cards) [DONE]

**Focus:** "Sticky" minions and anti-control tools

| Card Type | Count | Theme |
|-----------|-------|-------|
| Frenzy creatures | 7 | Multi-attack aggro (IDs 2015-2017, 2021-2024) |
| Volatile creatures | 7 | Death synergy / board punish (IDs 2018-2020, 2025-2028) |
| Buff spells | 3 | Combat tricks (IDs 2029-2031) |
| Support cards | 3 | Passive swarm buffs (IDs 2032-2034) |

**Note:** Token generators deferred - requires engine support for Summon in YAML effects.

**New Decks Created:**
- `symbiote/frenzy_aggro.toml` - All-in attack deck with Frenzy synergy
- `symbiote/volatile_swarm.toml` - Death trigger synergy deck

### Phase 2C: Validation & Tuning [PENDING]

1. Run 20k game validation (Modal cloud)
2. Target: Symbiote 47-53% win rate
3. Retune specialist weights if needed
4. Iterate on card stats if balance is off

---

## Milestone 3: Support & Structure Wave

**Goal:** Address the severe support card shortage (3 → 20+) and fill cost curve gaps.

### Phase 3A: Support Card Expansion (+15-18 supports)

| Faction | Supports | Theme |
|---------|----------|-------|
| Argentum | 4-5 | Defensive auras, healing |
| Symbiote | 4-5 | Swarm buffs, token generation |
| Obsidion | 3-4 | Life manipulation, draw |
| Neutral | 4-5 | Utility (draw, cost reduction) |

### Phase 3B: Cost Curve Filling (+15 cards)

**Current Gaps:**
- 1-cost: Only 5 cards (need more aggro enablers)
- 6+ cost: Limited finishers

| Cost | Current | Target | Focus |
|------|---------|--------|-------|
| 1 | 5 | 12 | Cheap tempo plays |
| 6 | 7 | 15 | Faction finishers |
| 7+ | 3 | 8 | Legendary tier |

### Phase 3C: New Deck Archetypes

**Target:** 2-3 new archetypes per faction

| Faction | New Archetypes |
|---------|----------------|
| Argentum | Ramp (essence acceleration), Fatigue (outlast) |
| Symbiote | Token Swarm, Sacrifice (death triggers) |
| Obsidion | Combo (burst damage), Drain (lifesteal synergy) |
| Neutral | Toolbox splash options |

**Deck Composition Rule:** Maintain 70% faction / 30% neutral splash ratio.

---

## Milestone 4: Engine Enhancements

**Goal:** Expand card design space without using keyword slots.

### Phase 4A: Creature Filters in YAML

**Current Limitation:** Effects can't target creatures by criteria.

**Proposed Feature:**
```yaml
# Example: "Deal 2 damage to enemy creatures with 3 or less health"
effects:
  - type: damage
    amount: 2
    target: enemy_creatures
    filter:
      max_health: 3
```

**Implementation:**
1. Expose existing `CreatureFilter` struct to YAML parser
2. Add filter field to effect definitions
3. Update effect resolution to apply filters

**Unlocks:**
- Conditional removal ("Destroy creature with 2 or less attack")
- Targeted buffs ("Give +2/+2 to creatures with Rush")
- Board-aware effects

### Phase 4B: Conditional Triggers

**Current Limitation:** No if/then logic in ability chains.

**Proposed Feature:**
```yaml
# Example: "Deal 2 damage. If target dies, draw a card"
effects:
  - type: damage
    amount: 2
  - type: draw
    count: 1
    condition:
      target_died: true
```

**Implementation:**
1. Add `Condition` enum to effect system
2. Track effect results for condition checking
3. Chain effects with conditional gates

**Unlocks:**
- Execute effects ("If this kills, gain +1/+1")
- Threshold effects ("If you have 5+ creatures, draw 2")
- Combo enablers

### Phase 4C: Additional Effect Types

| Effect | Description | Use Case |
|--------|-------------|----------|
| `bounce` | Return creature to owner's hand | Tempo/control |
| `copy` | Create copy of creature | Value generation |
| `transform` | Change creature into another | Removal variant |
| `cost_modify` | Change card costs | Ramp/disruption |

---

## Milestone 5: Tactical Evolution (+60 cards)

**Goal:** Expand to ~150 cards with tech cards and neutral utility.

### Phase 5A: Tech Cards

**Situational answers for each faction:**

| Faction | Tech Focus | Example Cards |
|---------|------------|---------------|
| Argentum | Anti-spell | "Shield Wall: Give all allies Shield until end of turn" |
| Obsidion | Disruption | "Mind Rot: Enemy discards a random card" |
| Neutral | Silence | "Nullifier: Remove all keywords from target" |

### Phase 5B: Neutral Utility Wave (+20 cards)

**Focus:** Cards that help weaker factions more than stronger ones

- Card draw (helps aggro reload)
- Cheap removal (helps control stabilize)
- Flexible bodies (fill curve gaps)

### Phase 5C: Meta Validation

1. Run full 20k validation suite
2. Target: All factions within 45-55% win rate
3. No matchup worse than 40/60
4. Retune weights for all specialists

---

## Milestone 6: Legends of Omyra (Completion)

**Goal:** Reach 300 cards with Legendary commanders and final polish.

### Phase 6A: Faction Commanders (+12 Legendaries)

| Faction | Commander | Signature Ability |
|---------|-----------|-------------------|
| Argentum | The High Artificer | "All constructs gain +0/+2 and Guard" |
| Argentum | Iron Colossus Prime | "Cannot be destroyed by effects" |
| Argentum | The Grand Architect | "Start of turn: Summon a 1/1 Construct" |
| Symbiote | The Broodmother | "On ally death: Summon a 1/1 Spore" |
| Symbiote | Alpha of the Pack | "All allies with Rush gain +2/+0" |
| Symbiote | The Hivemind | "Your creatures share keywords" |
| Obsidion | The Eternal One | "Lifesteal. On kill: Gain +2/+2" |
| Obsidion | Shadow Emperor | "Stealth. On attack: Deal 2 to all enemies" |
| Obsidion | The Soul Collector | "On any death: Draw a card" |
| Neutral | The Wanderer | "Start of turn: Gain a random keyword" |
| Neutral | Mercenary King | "Your neutral cards cost 1 less" |
| Neutral | The Arbiter | "On play: Silence all creatures" |

### Phase 6B: Final Card Wave (+50 cards)

Fill remaining gaps to reach 300:
- Common/Uncommon filler for draft variety
- Rare tech options
- Legendary finishers

### Phase 6C: Golden Master Balance Pass

1. Final 20k validation on all matchups
2. Individual card stat adjustments
3. Weight retuning for all agents
4. Documentation update

---

## Validation Protocol

**Standard validation for each milestone:**
```bash
# Run via Modal cloud (recommended)
modal run modal_tune.py::main --mode validate-only

# Or locally (slower)
cargo run --release --bin validate -- --games 20000 --output validation.json
```

**Balance Targets:**
| Metric | Target | Acceptable |
|--------|--------|------------|
| P1 Win Rate | 50-52% | 48-55% |
| Faction Win Rate | 48-52% | 45-55% |
| Max Faction Delta | <5% | <10% |
| Worst Matchup | >45% | >40% |

---

## Summary: Card Targets by Milestone

| Milestone | Cards Added | Running Total | Status |
|-----------|-------------|---------------|--------|
| M1 (Complete) | 0 | 60 | Done |
| M2: Symbiote Rising | +20 | **80** | **Phase 2A/2B Done, 2C Pending** |
| M3: Support Wave | +30 | 110 | Planned |
| M4: Engine Work | 0 | 110 | Planned |
| M5: Tactical Evolution | +60 | 170 | Planned |
| M6: Legends | +130 | **300** | Planned |

---

## Immediate Next Steps

1. **Run Phase 2C validation** - 20k games via Modal cloud
2. **Measure Symbiote win rate** - Target 47-53%
3. **Retune specialist weights** if needed
4. **Begin Milestone 3** - Support Card Expansion

Let's validate the Symbiote changes!
