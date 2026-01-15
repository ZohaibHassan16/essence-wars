# The "Project 300" Roadmap

**Mission**: Design and implement 300 Cards for the initial `New Horizons` Edition of Essence Wars.

## Current State (v0.5.0 - Obsidion Expansion Complete)

### Card Pool Summary
| Category | Count | Target |
|----------|-------|--------|
| **Total Cards** | 105 | 300 |
| Argentum Combine | 25 | ~75 |
| Symbiote Circles | 35 | ~75 |
| Obsidion Syndicate | 30 | ~75 |
| Free-Walkers (Neutral) | 15 | ~75 |
| **Support Cards** | 12 | ~30 |
| **Legendary Cards** | 5 | ~12 |

### Balance Baseline (Round-Robin Validation, 2026-01-15)
| Metric | Value | Status |
|--------|-------|--------|
| P1 Win Rate | 54.7% | BALANCED |
| Argentum | 56.1% | Slightly High |
| Symbiote | 49.7% | Balanced |
| Obsidion | 45.1% | Slightly Low |
| Max Delta | 11.0% | **NEEDS TUNING** |

**Validation Method:** Round-robin across all 40 deck combinations (50 games/matchup/direction = 4,000 total games).

**Next Step:** Run comprehensive validation on Modal (1500 games/matchup = 120k total) for higher confidence metrics, then fine-tune.

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
│   ├── argentum.yaml   (IDs 1000-1024, 25 cards)
│   ├── symbiote.yaml   (IDs 2000-2034, 35 cards)
│   ├── obsidion.yaml   (IDs 3000-3029, 30 cards)
│   └── neutral.yaml    (IDs 4000-4014, 15 cards)
├── decks/
│   ├── argentum/
│   │   ├── control.toml
│   │   ├── midrange.toml
│   │   └── anti_swarm.toml
│   ├── symbiote/
│   │   ├── aggro.toml
│   │   ├── tempo.toml
│   │   ├── frenzy_aggro.toml
│   │   └── volatile_swarm.toml
│   └── obsidion/
│       ├── burst.toml
│       ├── control.toml
│       ├── assassin.toml
│       └── lifedrain.toml
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

### Phase 2C: Validation & Tuning [COMPLETED]

**Outcome:** Rush+Frenzy combo identified as problematic (60.2% P1 win rate).

**Nerf Applied:** Removed Rush from 3 cards:
- Frenzied Brood (2016): Rush+Frenzy → Frenzy only
- Savage Swarmling (2024): Rush+Frenzy → Frenzy only
- Frenzy Pack Leader (2022): Rush+Frenzy → Frenzy only

**Result:** P1 advantage fixed, but Symbiote now dominated Argentum (67.6% vs 32.4%).

---

## Milestone 3: Argentum Recovery Wave [COMPLETED]

**Goal:** Fix Argentum's 32.4% win rate vs Symbiote with anti-swarm tools.

### Phase 3A: Argentum Priority Wave (+10 cards) [DONE]

**Added 10 new Argentum cards (IDs 1015-1024):**

| Card | Cost | Stats | Effect | Purpose |
|------|------|-------|--------|---------|
| Forge Master | 4 | 2/4 | StartOfTurn: 1 damage to all enemies | Anti-swarm AoE |
| Steam Cannon | 5 | 2/7 Guard | StartOfTurn: 1 damage to all enemies | Durable AoE |
| Artillery Tower | 6 | 1/6 | StartOfTurn: 2 damage to all enemies | Heavy AoE |
| Reinforced Walls | 3 | Support | All allies +0/+2 | Board-wide buff |
| Shield Generator | 4 | Support | All allies gain Shield | Protection |
| Ironclad Bulwark | 4 | 1/8 Guard | - | Massive wall |
| Bronze Gatekeeper | 2 | 1/4 Guard | - | Cheap wall |
| Armored Sentinel | 3 | 2/5 Guard+Shield | - | Durable defender |
| Field Medic | 2 | 1/3 | OnPlay: Heal 3 | Sustain |
| Barricade | 3 | Spell | +0/+4 and Guard | Defensive trick |

**Additional Adjustments:**
- Nerfed Obsidion Blood Acolyte: 3/4 → 2/3 (was too efficient vs Argentum)

**New Decks Created:**
- `argentum/midrange.toml` - Balanced offense/defense with Piercing and AoE
- `argentum/anti_swarm.toml` - Pure counter-swarm with maximum AoE

### Phase 3B: Balance Validation [DONE]

**Validation System Improved:**
- Implemented round-robin matchup generation (tests ALL deck combinations)
- Removed need for `--all-decks` flag - now default behavior
- 26 deck pairs tested vs previous 3

**Final Results (100 games/matchup, round-robin):**
| Metric | Value | Status |
|--------|-------|--------|
| P1 Win Rate | 54.0% | BALANCED |
| Obsidion | 52.9% | Balanced |
| Argentum | 50.7% | Balanced |
| Symbiote | 47.4% | Balanced |
| Max Delta | 5.5% | BALANCED |

### Phase 3C: Obsidion Parity Wave (+15 cards) [DONE]

**Goal:** Bring Obsidion to card count parity with other factions.

**Added 15 new Obsidion cards (IDs 3015-3029):**

| Card | Cost | Stats | Keywords | Purpose |
|------|------|-------|----------|---------|
| Phantom Striker | 1 | 2/2 | Quick | Aggressive opener |
| Shade Lurker | 2 | 2/3 | Stealth | Cheap infiltrator |
| Blood Seeker | 3 | 3/3 | Lifesteal | Reliable sustain |
| Spectral Assassin | 3 | 4/3 | Ephemeral, Quick | Burst damage |
| Nightstalker | 4 | 4/4 | Stealth, Lifesteal | Mid-game threat |
| Soul Reaver | 5 | 5/4 | Quick, Lifesteal | Late finisher |
| Void Devourer | 5 | 4/4 | OnPlay: 4 damage | Burst removal |
| Twilight Executioner | 6 | 6/5 | Quick, Lethal | Legendary finisher |
| Blood Ritualist | 3 | 2/4 | OnPlay: Heal 4 | Sustain support |
| Shadowmeld | 2 | 3/2 | Stealth, Ephemeral | Aggressive burst |
| Life Tap | 1 | Spell | 2 dmg + 2 heal | Efficient removal |
| Dark Empowerment | 2 | Spell | +2/+1, grant Quick | Combat trick |
| Void Rift | 4 | Spell | 5 damage | Big removal |
| Shadow Sanctum | 3 | Support | Grant Stealth | Aggressive support |
| Blood Font | 4 | Support | Heal 2/turn | Sustain support |

**New Decks Created:**
- `obsidion/assassin.toml` - Stealth+Lethal hit-and-run strategy
- `obsidion/lifedrain.toml` - Lifesteal synergy for sustained attrition

**Balance Note:** Health buffs applied to several creatures to survive Argentum AoE better.

### Phase 3D: Future Expansion (Deferred)

**Available when ready:**
- New deck archetypes (Ramp, Token Swarm, Combo, etc.)
- Cost curve filling
- Additional supports

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
# Round-robin: 40 deck matchups × 2 directions × games = total games
modal run modal_tune.py::main --mode validate-only --validation-games 1500  # 120k total

# Or locally (slower)
cargo run --release --bin validate -- --games 1500 --output validation.json  # 120k total
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
| M1: Foundation Refactor | 0 | 60 | ✅ Done |
| M2: Symbiote Rising | +20 | 80 | ✅ Done |
| M3: Argentum Recovery + Obsidion Parity | +25 | **105** | ✅ Done |
| M4: Engine Enhancements | 0 | 105 | Planned |
| M5: Tactical Evolution | +45 | 150 | Planned |
| M6: Legends of Omyra | +150 | **300** | Planned |

---

## Current Deck Inventory

| Faction | Decks | Files |
|---------|-------|-------|
| Argentum | 3 | control.toml, midrange.toml, anti_swarm.toml |
| Symbiote | 4 | aggro.toml, tempo.toml, frenzy_aggro.toml, volatile_swarm.toml |
| Obsidion | 4 | burst.toml, control.toml, assassin.toml, lifedrain.toml |
| **Total** | **11** | - |

---

## Immediate Next Steps

**Option A: Large Validation (Recommended)**
1. Run 20k game validation via Modal cloud
2. Retune specialist weights for all factions
3. Higher confidence balance metrics
4. Fine-tune any outlier cards based on results

**Option B: Engine Work (Milestone 4)**
1. Implement creature filters in YAML
2. Add conditional triggers
3. New effect types (bounce, copy, transform)
4. Enables more interesting card designs

**Option C: Neutral Expansion**
1. Add 10-15 Neutral cards to reach parity
2. More utility options for all factions
3. Better deck variety

**Current Focus:** Run 20k Modal validation to get high-confidence balance metrics before more card additions.
