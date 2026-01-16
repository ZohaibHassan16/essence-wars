# The "Project 300" Roadmap

**Mission**: Design and implement 300 Cards for the initial `New Horizons` Edition of Essence Wars.

## Current State (v0.5.2 - Phase 8B Complete)

### Card Pool Summary
| Category | Count | Target | Status |
|----------|-------|--------|--------|
| **Total Cards** | **300** | 300 | ✅ COMPLETE |
| Argentum Combine | 75 | 75 | ✅ |
| Symbiote Circles | 75 | 75 | ✅ |
| Obsidion Syndicate | 75 | 75 | ✅ |
| Free-Walkers (Neutral) | 75 | 75 | ✅ |
| **Legendary Commanders** | 12 | 12 | ✅ |

### Balance Baseline (Post-Phase 7, 2026-01-16)
| Metric | Value | Status |
|--------|-------|--------|
| P1 Win Rate | 49.0% | WARNING |
| Argentum | 49.6% | Balanced |
| Symbiote | 46.9% | Balanced |
| Obsidion | 53.8% | Balanced |
| **Max Delta** | **6.9%** | **BALANCED** ✅ |

**Phase 7 Balance Summary:**
- Phase 7A: Argentum +15 cards → 49.6% (up from 44.9%)
- Phase 7B: Symbiote +15 conservative cards → 46.9% (down from 54.0%)
- Phase 7C: Obsidion +15 cards → 53.8% (up from 46.0%)
- All factions now within 47-54% range - best balance achieved!

**Balance Patch v0.5.1 Summary:**
- 3 rounds of targeted stat adjustments after Phase 6 cards caused imbalance
- Started at 24.1% faction delta → reduced to 10.4%
- Argentum buffs: Steam Knight, Iron Colossus, Ironclad Bulwark, Bronze Gatekeeper, Hardened Vanguard, Armored Titan, Fortress Wall
- Symbiote nerfs: Bloodrage Berserker, Frenzy Pack Leader, Ravager Alpha, Deathburst Lurker, Regenerating Ooze
- Created 4th Argentum deck (Piercing) to match other factions

**Validation Method:** Round-robin across all 48 deck combinations (12 decks × 4 factions, 100 games/matchup).

**Status:** Phase 8B complete. **300 cards achieved!** Ready for Phase 8C (Commander Decks) and 8D (Modal Validation).

### Keyword Slots
- **Used:** 16 of 16 (Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick, Ephemeral, Regenerate, Stealth, Charge, Frenzy, Volatile, **Fortify**, **Ward**)
- **Available:** None (all slots utilized)

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
│   ├── argentum.yaml   (IDs 1000-1040, 41 cards)
│   ├── symbiote.yaml   (IDs 2000-2044, 45 cards)
│   ├── obsidion.yaml   (IDs 3000-3039, 40 cards)
│   └── neutral.yaml    (IDs 4000-4028, 29 cards)
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

## Milestone 4: Engine Enhancements [COMPLETED]

**Goal:** Expand card design space without using keyword slots.

**Status:** Done (v0.5.0)

### Phase 4A: Creature Filters in YAML [DONE]

**Implementation:**
- Added `filter` field to `EffectDefinition` variants (Damage, Heal, BuffStats, Destroy, GrantKeyword, RemoveKeyword, Silence, Bounce)
- Implemented `CreatureFilter::matches()` method for runtime filtering
- Applied filters in effect resolution (`effect_queue.rs`) and legal action generation (`legal.rs`)

**YAML Syntax:**
```yaml
# Execute-style removal: Destroy creature with ≤3 health
- type: destroy
  filter:
    max_health: 3

# Keyword synergy: Buff all Rush creatures
- type: buff_stats
  attack: 2
  health: 1
  filter:
    has_keyword: 8  # Rush keyword bit
```

**Filter Fields:**
- `max_health`: Target must have health ≤ value
- `min_health`: Target must have health ≥ value
- `has_keyword`: Target must have keyword (uses keyword bit values)
- `lacks_keyword`: Target must NOT have keyword

### Phase 4B: Conditional Triggers [DONE]

**Implementation:**
- Added `Condition` enum with `TargetDied` variant
- Added `EffectResult` struct to track effect outcomes
- Added `conditional_effects` field to `AbilityDefinition` and `CardType::Spell`
- Modified effect resolution to track results and check conditions

**YAML Syntax:**
```yaml
# Kill reward: Deal 3 damage, if target dies draw a card
effects:
  - type: damage
    amount: 3
conditional_effects:
  - condition: target_died
    effects:
      - type: draw
        count: 1
```

### Phase 4C: Bounce Effect [DONE]

**Implementation:**
- Added `Effect::Bounce` variant with optional filter
- Implemented `apply_bounce` handler in effect queue
- Creature returns to owner's hand (removed from board, card added to hand)

**YAML Syntax:**
```yaml
# Return target enemy creature to hand
effects:
  - type: bounce

# Mass bounce: Return all creatures with ≤3 health to hand
effects:
  - type: bounce
    filter:
      max_health: 3
```

### Phase 4D: New Cards Using Phase 4 Features (+35 cards) [DONE]

**Argentum (IDs 1025-1034, 10 cards):**
| Card | Cost | Type | Effect |
|------|------|------|--------|
| Execute | 2 | Spell | Destroy creature with ≤3 health |
| Purge the Weak | 4 | Spell | 2 damage to creatures with ≤3 health |
| Shield Wall Captain | 3 | 1/5 Guard | OnPlay: Grant Shield to Guard creatures |
| Rally the Guards | 3 | Spell | +1/+1 to all Guard creatures |
| Temporal Displacement | 3 | Spell | Bounce enemy creature |
| Mass Recall | 5 | Spell | Bounce all creatures with ≤3 health |
| Stalwart Defender | 4 | 2/7 Guard | - |
| Siege Breaker | 5 | 4/4 Piercing | OnPlay: 2 damage to creature with ≤4 health |
| Judgment Strike | 3 | Spell | 3 damage, if dies heal 3 |
| Culling Blade | 4 | Spell | Destroy creature ≤4 health, if dies draw 1 |

**Symbiote (IDs 2035-2044, 10 cards):**
| Card | Cost | Type | Effect |
|------|------|------|--------|
| Soul Reaper | 3 | Spell | 3 damage, if dies draw 1 |
| Feast | 2 | Spell | 2 damage, if dies +2/+2 to ally |
| Predatory Strike | 4 | Spell | 4 damage, if dies heal 4 |
| Pounce Hunter | 3 | 3/2 Rush | OnKill: Draw 1 |
| Evolution Burst | 3 | Spell | +2/+1 to all Rush creatures |
| Pack Recall | 2 | Spell | Bounce ally creature, draw 1 |
| Spawn Caller | 4 | 2/4 | OnPlay: 2 damage to creature with ≤2 health |
| Feral Striker | 2 | 3/2 Rush | - |
| Swarm Ambusher | 4 | 4/3 Rush Lethal | - |
| Consume the Fallen | 3 | Spell | Destroy creature ≤3 health, if dies +1/+1 |

**Obsidion (IDs 3030-3039, 10 cards):**
| Card | Cost | Type | Effect |
|------|------|------|--------|
| Soul Harvest | 3 | Spell | 3 damage, if dies heal 3 |
| Consume Essence | 4 | Spell | Destroy creature ≤4 health, if dies draw 2 |
| Vampiric Execution | 5 | Spell | 5 damage, if dies heal 5 + draw 1 |
| Shadow Return | 2 | Spell | Bounce + 1 damage |
| Mass Dispersion | 5 | Spell | Bounce all creatures with ≤3 health |
| Vampiric Surge | 3 | Spell | +2/+0 to all Lifesteal creatures |
| Blood Pact | 2 | Spell | +1/+2, grant Lifesteal |
| Reaper's Due | 4 | 4/3 Lifesteal Quick | OnKill: Heal 4 |
| Soul Collector | 3 | 3/3 Lifesteal | OnKill: Draw 1 |
| Void Executioner | 5 | 4/4 Quick | OnPlay: Destroy creature ≤3 health |

**Neutral (IDs 4015-4019, 5 cards):**
| Card | Cost | Type | Effect |
|------|------|------|--------|
| Temporal Shift | 3 | Spell | Bounce enemy creature |
| Triage | 2 | Spell | Heal 4 to creature with ≤3 health |
| Opportunist | 3 | 3/3 | OnPlay: 2 damage to creature with ≤3 health |
| Silencing Shot | 2 | Spell | Silence creature with ≤4 health |
| Scavenger | 2 | 2/2 | OnAllyDeath: Draw 1 |

### Phase 4E: Future Effect Types (Deferred)

| Effect | Description | Use Case |
|--------|-------------|----------|
| `copy` | Create copy of creature | Value generation |
| `transform` | Change creature into another | Removal variant |
| `cost_modify` | Change card costs | Ramp/disruption |
| `summon` | Create token creature | Board presence |

---

## Milestone 5: Engine Expansion (+15 cards) [COMPLETED]

**Goal:** Add final 2 keywords and 3 new effect types to maximize design space.

**Status:** Done (v0.5.0)

### Phase 5A: Final Keywords [DONE]

| Keyword | Bit | Effect | Primary Faction | Implementation |
|---------|-----|--------|-----------------|----------------|
| **Fortify** | 14 | Takes 1 less damage from all sources (minimum 1) | Argentum | `keywords.rs`, `combat.rs`, `effect_queue.rs` |
| **Ward** | 15 | First spell/ability targeting this has no effect, then Ward is consumed | Neutral | `keywords.rs`, `effect_queue.rs` |

**Keyword Distribution (Final):**
| Faction | Keywords | Count |
|---------|----------|-------|
| Argentum | Guard, Shield, Piercing, **Fortify** | 4 |
| Symbiote | Rush, Frenzy, Volatile, Regenerate, Lethal | 5 |
| Obsidion | Lifesteal, Quick, Stealth, Ephemeral | 4 |
| Neutral | Charge, Ranged, **Ward** | 3 |

### Phase 5B: New Effect Types [DONE]

| Effect | Description | YAML Syntax |
|--------|-------------|-------------|
| **SummonToken** | Create a token creature in an empty slot | `type: summon_token` with `token: {name, attack, health, keywords}` |
| **Transform** | Replace target creature with a token | `type: transform` with `into: {name, attack, health, keywords}` |
| **Copy** | Create a copy of target creature | `type: copy` |

**Implementation Details:**
- Added `TokenDefinition` struct for inline token definitions
- Tokens use `CardId(0)` as sentinel (not from card database)
- Transform/Copy don't trigger OnPlay (creature not played from hand)
- Ward only blocks single-target effects, not AoE or combat damage

**YAML Examples:**
```yaml
# Summon Token
effects:
  - type: summon_token
    token:
      name: "Militia"
      attack: 1
      health: 1
      keywords: []

# Transform
effects:
  - type: transform
    into:
      name: "Sheep"
      attack: 1
      health: 1
      keywords: []

# Copy (targets resolved from spell targeting)
effects:
  - type: copy
```

### Phase 5C: Showcase Cards (+15 cards) [DONE]

**Argentum (IDs 1035-1040, 6 cards):**
| Card | Cost | Stats | Keywords/Effects |
|------|------|-------|------------------|
| Hardened Vanguard | 3 | 2/4 | Guard, Fortify |
| Armored Titan | 5 | 3/7 | Fortify |
| Bulwark Commander | 4 | 2/5 | Guard, Fortify. OnPlay: Grant Fortify to Guards |
| Fortress Wall | 6 | 1/10 | Guard, Fortify, Shield |
| Transmutation Ray | 4 | Spell | Transform enemy into 1/1 Brass Cog |
| Assembly Protocol | 5 | Spell | Copy ally creature |

**Neutral (IDs 4020-4028, 9 cards):**
| Card | Cost | Stats | Keywords/Effects |
|------|------|-------|------------------|
| Warded Sentinel | 3 | 2/3 | Ward |
| Mystic Guardian | 4 | 3/4 | Ward, Guard |
| Shield Mage | 3 | 2/3 | OnPlay: Grant Ward to ally |
| Arcane Protector | 5 | 3/6 | Ward, Shield |
| Raise Militia | 2 | Spell | Summon 1/1 Militia |
| Conjure Guardian | 4 | Spell | Summon 2/3 Arcane Guardian with Guard |
| Token Master | 4 | 2/4 | OnPlay: Summon 2/2 Warrior with Rush |
| Polymorph | 5 | Spell | Transform enemy into 1/1 Sheep |
| Mirror Image | 4 | Spell | Copy ally creature |

### Phase 5D: Validation [DONE]

- All 576 tests passing
- Clippy checks passing
- Cards loading correctly
- Balance validated with arena tests

---

## Milestone 6: Neutral Foundation (+25 cards) [COMPLETED]

**Goal:** Expand neutral pool from 29 to ~54 cards.

**Status:** Done (v0.5.0 → v0.5.1)

### Phase 6A: Neutral Creatures (+15 cards) [DONE]

**Added 15 new neutral creatures (IDs 4029-4043):**

| Card | Cost | Stats | Keywords/Effects |
|------|------|-------|------------------|
| Pathfinder Scout | 1 | 1/2 | Charge |
| Street Urchin | 1 | 2/1 | - |
| Traveling Merchant | 2 | 2/2 | OnPlay: Draw 1 |
| Frontier Guard | 2 | 1/3 | Guard |
| Sellsword | 3 | 3/3 | - |
| Caravan Guard | 3 | 2/4 | Guard |
| Wandering Sage | 3 | 2/3 | OnPlay: Heal 3 |
| Border Sentinel | 4 | 3/5 | Guard |
| Duelist | 4 | 4/3 | Quick |
| Mercenary Captain | 4 | 3/4 | OnPlay: +1/+1 to ally |
| Wandering Champion | 5 | 4/5 | Charge |
| Hired Blade | 5 | 5/4 | Rush |
| Siege Giant | 6 | 6/5 | Piercing |
| War Elephant | 6 | 5/7 | Guard |
| The Warbringer | 7 | 7/7 | Charge, Piercing (Legendary) |

### Phase 6B: Neutral Spells (+10 cards) [DONE]

**Added 10 new neutral spells (IDs 4044-4053):**

| Card | Cost | Type | Effect |
|------|------|------|--------|
| Quick Strike | 1 | Spell | Deal 2 damage |
| Minor Heal | 1 | Spell | Heal 3 |
| Precision Shot | 2 | Spell | Deal 3 damage |
| Scout Ahead | 2 | Spell | Draw 2 cards |
| Disarm | 3 | Spell | -2/-0 debuff |
| Battle Cry | 3 | Spell | +2/+0 to all allies |
| Lightning Bolt | 4 | Spell | Deal 4 damage |
| Mass Healing | 4 | Spell | Heal 3 to all allies |
| Devastate | 5 | Spell | Deal 5 damage |
| Cataclysm | 6 | Spell | Deal 3 damage to all creatures |

### Phase 6C: Validation & Balance Patch [DONE]

**Initial Results (after adding Phase 6 cards):**
- Faction Delta: 24.1% (Symbiote dominating at 61.9%)

**Balance Patch v0.5.1 (3 rounds):**
| Round | Changes | Result |
|-------|---------|--------|
| 1 | Buff Fortify creatures (+1 atk), Nerf Frenzy health | 17.4% delta |
| 2 | Buff more Argentum Guards (+1 atk), Nerf more Symbiote | 13.0% delta |
| 3 | Buff Steam Knight, Nerf Regenerating Ooze | **10.4% delta** |

**Created 4th Argentum deck:** `argentum/piercing.toml` - Aggressive Piercing creatures with Fortify sustain.

---

## Milestone 7: Faction Deepening (+45 cards) [COMPLETED]

**Goal:** Bring each faction to ~60 cards with archetype support.

**Status:** Complete - All phases done, balance achieved (6.9% max delta)

### Phase 7A: Argentum Wave (+15 cards) [DONE]

**Added 15 new Argentum cards (IDs 1041-1055):**

| Category | Cards | Examples |
|----------|-------|----------|
| Fortify Synergies | 5 | Fortified Sentinel (2/3), Steel Templar (3/5 Guard+Fortify), Bastion Lord (3/8, grant Fortify all) |
| Construct Tokens | 4 | Assembly Overseer, Factory Heart, Forge of Creation (support), Mass Production (2 tokens) |
| Defensive Tech | 4 | Interceptor (anti-Rush), Spotlight Tower (anti-Stealth), Lockdown Protocol, Expose Weakness |
| Support Cards | 2 | Repair Station (heal 2/turn), Dampening Field (grant Fortify all) |

**All 4 Argentum decks updated:**
- `control.toml` - Heavy Fortify synergy focus
- `midrange.toml` - Token generation focus
- `anti_swarm.toml` - Anti-Rush/Stealth tech focus
- `piercing.toml` - Fortify sustain focus

### Phase 7B: Symbiote Wave (+15 cards) [DONE]

**Added 15 conservative Symbiote cards (IDs 2045-2059):**

Note: Designed conservatively since Symbiote was at 54.0% win rate.

| Category | Cards | Examples |
|----------|-------|----------|
| Basic Swarm Bodies | 3 | Hiveling (1/1), Brood Tender (1/3 +0/+1 ally), Spore Colony (2/4 vanilla) |
| Volatile Options | 2 | Volatile Drone (1/2 Volatile), Volatile Stalker (2/2 Volatile+Rush) |
| Regenerate/Defense | 4 | Regenerating Spawn (2/2 Regen), Carapace Scout (2/3 Ranged), Brood Protector (2/5 Guard), Brood Caller (2/3 summon token) |
| Buff Spells | 3 | Pack Tactics (+1/+1 Rush), Adaptive Form (+0/+2 Regen), Swarm Fury (all +1/+0) |
| Supports | 2 | Symbiotic Growth (+0/+1 aura), Hive Network (draw 1/turn) |
| Finisher | 1 | Brood Overlord (5/6 grant Rush all) |

**All 4 Symbiote decks updated:**
- `aggro.toml` - Pack Tactics, Brood Caller, Brood Overlord
- `tempo.toml` - Regenerating Spawn, Adaptive Form, Symbiotic Growth, defensive options
- `frenzy_aggro.toml` - Hiveling, Pack Tactics, Swarm Fury
- `volatile_swarm.toml` - Volatile Drone, Volatile Stalker

### Phase 7C: Obsidion Wave (+15 cards) [DONE]

**Added 15 new Obsidion cards (IDs 3040-3054):**

| Category | Cards | Examples |
|----------|-------|----------|
| Copy/Clone | 4 | Shadow Clone, Doppelganger, Mirror Assassin (OnKill copy), Echo of the Void |
| Transform | 3 | Corruption (enemy to 2/2), Dark Metamorphosis (ally to 4/4 Quick), Void Corruptor |
| Lifesteal Synergy | 5 | Blood Cultist, Sanguine Lord (grant Lifesteal all), Life Drain Aura (support), Bloodthirst |
| Stealth Support | 3 | Shadow Stalker (3/3 Stealth+Quick), Cloak of Shadows, Shadow Network (support) |

**All 4 Obsidion decks updated:**
- `burst.toml` - Copy effects to duplicate threats
- `control.toml` - Transform removal + Lifesteal sustain
- `assassin.toml` - Shadow Network grants Stealth to all
- `lifedrain.toml` - Life Drain Aura grants Lifesteal to all

### Phase 7D: Validation [DONE]

**Final Results:** All factions within 47-54% win rate, Max Delta 6.9% ✅

Target achieved: All factions 45-55% win rate, max delta under 10%.

---

## Milestone 8: Commander Foundation Set (+75 cards) [REVISED]

**Goal:** Complete the 300-card Foundation Set with 12 Legendary Commanders and supporting cards.

**Design Philosophy:**
- Free-Walkers are mercenaries ("No Flag. Just Gold") - they don't have faction leaders
- Each main faction gets 4 Legendary Commanders (one per archetype)
- Each Commander defines a deck archetype
- Final product: 12 Commander Decks ready for marketing/competitive play

### Phase 8A: Design 12 Legendary Commanders (+12 cards) [COMPLETE]

**Argentum Combine** - "Precision. Protocol. Peace."

| Commander | Archetype | Cost | Stats | Signature Ability |
|-----------|-----------|------|-------|-------------------|
| **The High Artificer** | Construct/Token | 6 | 3/5 | "Start of turn: Summon a 1/1 Brass Cog. Constructs you control have +0/+1." |
| **Iron Colossus Prime** | Guard/Wall | 7 | 4/8 | "Guard, Fortify. Adjacent creatures have Guard." |
| **Siege Marshal Vex** | Piercing/Aggro | 5 | 4/4 | "Piercing. Your Piercing creatures have +1/+0." |
| **The Grand Architect** | Fortify/Control | 6 | 2/6 | "Fortify. On play: Give all ally creatures Fortify." |

**Symbiote Circles** - "Adapt or Perish."

| Commander | Archetype | Cost | Stats | Signature Ability |
|-----------|-----------|------|-------|-------------------|
| **The Broodmother** | Rush/Swarm | 6 | 3/5 | "Rush. On attack: Summon a 1/1 Broodling with Rush." |
| **Plague Sovereign** | Volatile/Death | 6 | 4/4 | "Volatile. On ally death: Deal 1 damage to all enemies." |
| **Alpha of the Hunt** | Frenzy/Aggro | 5 | 4/3 | "Frenzy. Your Frenzy creatures have +1/+0." |
| **The Eternal Grove** | Regenerate/Midrange | 7 | 3/7 | "Regenerate. Your creatures have Regenerate." |

**Obsidion Syndicate** - "Ambition Unbound."

| Commander | Archetype | Cost | Stats | Signature Ability |
|-----------|-----------|------|-------|-------------------|
| **The Blood Sovereign** | Lifesteal/Sustain | 6 | 4/5 | "Lifesteal. Your creatures have Lifesteal." |
| **Shadow Emperor Kael** | Stealth/Assassin | 6 | 5/4 | "Stealth, Quick. On kill: Return to hand." |
| **The Doppelganger King** | Copy/Clone | 7 | 4/4 | "On play: Become a copy of target creature with +2/+2." |
| **Void Archon** | Quick/Burst | 5 | 4/3 | "Quick. On play: Your creatures gain Quick this turn." |

### Phase 8B: Fill Remaining Cards (+63 cards) [COMPLETE]

All factions now at 75 cards:

| Faction | Before | Commanders | Phase 8B | Final |
|---------|--------|------------|----------|-------|
| Argentum | 56 | +4 | +15 | **75** ✅ |
| Symbiote | 60 | +4 | +11 | **75** ✅ |
| Obsidion | 55 | +4 | +16 | **75** ✅ |
| Free-Walkers | 54 | +0 | +21 | **75** ✅ |
| **Total** | 225 | +12 | +63 | **300** ✅ |

Cards added support commander archetypes: token generators, keyword synergy, curve fillers.

### Phase 8C: Build 12 Commander Decks [COMPLETE]

All 12 Commander Decks built with 30 cards each, full 300-card coverage:

| Faction | Deck ID | Commander | Cards |
|---------|---------|-----------|-------|
| Argentum | `artificer_tokens` | The High Artificer | 30 |
| Argentum | `colossus_wall` | Iron Colossus Prime | 30 |
| Argentum | `vex_piercing` | Siege Marshal Vex | 30 |
| Argentum | `architect_fortify` | The Grand Architect | 30 |
| Symbiote | `broodmother_swarm` | The Broodmother | 30 |
| Symbiote | `plague_volatile` | Plague Sovereign | 30 |
| Symbiote | `alpha_frenzy` | Alpha of the Hunt | 30 |
| Symbiote | `grove_regenerate` | The Eternal Grove | 30 |
| Obsidion | `sovereign_lifesteal` | The Blood Sovereign | 30 |
| Obsidion | `kael_assassin` | Shadow Emperor Kael | 30 |
| Obsidion | `doppelganger_copy` | The Doppelganger King | 30 |
| Obsidion | `archon_burst` | Void Archon | 30 |

**Deck Composition:** 1 Commander + 22 faction cards + 7 neutral cards = 30

### Phase 8D: Modal Validation & Tuning

```bash
# Full pipeline: validate all 12 commander decks
modal run modal_tune.py::main --mode validate-only

# Retrain specialists if needed
modal run modal_tune.py::main
```

Target: All factions 45-55%, max delta <10%, no commander >55% win rate.

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
| M3: Argentum Recovery + Obsidion Parity | +25 | 105 | ✅ Done |
| M4: Engine Enhancements (Phase 4) | +35 | 140 | ✅ Done |
| M5: Engine Expansion (Phase 5) | +15 | 155 | ✅ Done |
| M6: Neutral Foundation | +25 | 180 | ✅ Done |
| M7: Faction Deepening | +45 | 225 | ✅ Done |
| **M8: Commander Foundation Set** | **+75** | **300** | 🔄 In Progress |

**Milestone 8 Breakdown:**
- Phase 8A: 12 Legendary Commanders (+12 cards)
- Phase 8B: Fill to 75/faction (+63 cards)
- Phase 8C: Build 12 Commander Decks
- Phase 8D: Modal Validation & Tuning

---

## Current Deck Inventory

| Faction | Decks | Files |
|---------|-------|-------|
| Argentum | 4 | control.toml, midrange.toml, anti_swarm.toml, **piercing.toml** |
| Symbiote | 4 | aggro.toml, tempo.toml, frenzy_aggro.toml, volatile_swarm.toml |
| Obsidion | 4 | burst.toml, control.toml, assassin.toml, lifedrain.toml |
| **Total** | **12** | - |

---

## Immediate Next Steps

**Current Focus: Milestone 8 - Commander Foundation Set**

### ✅ Milestone 7 Complete!
- Phase 7A: Argentum Wave (+15 cards) ✓
- Phase 7B: Symbiote Wave (+15 cards, conservative) ✓
- Phase 7C: Obsidion Wave (+15 cards) ✓
- Phase 7D: Validation ✓ → **6.9% max delta** (best balance achieved!)

### 🎯 Milestone 8: Commander Foundation Set (+75 cards)

**Phase 8A: Design 12 Legendary Commanders** ← CURRENT
- 4 Argentum: Artificer, Iron Colossus, Siege Marshal, Grand Architect
- 4 Symbiote: Broodmother, Plague Sovereign, Alpha of the Hunt, Eternal Grove
- 4 Obsidion: Blood Sovereign, Shadow Emperor, Doppelganger King, Void Archon

**Phase 8B: Fill Remaining Cards (+63)**
- Argentum: +15 cards → 75 total
- Symbiote: +11 cards → 75 total
- Obsidion: +16 cards → 75 total
- Free-Walkers: +21 cards → 75 total

**Phase 8C: Build 12 Commander Decks**
- Replace current decks with commander-focused builds
- Each deck showcases its commander's archetype

**Phase 8D: Modal Validation & Tuning**
```bash
modal run modal_tune.py::main  # Full pipeline on Modal cloud
```

**Notes:**
- All 16 keyword slots are used
- Engine supports: filters, conditionals, bounce, summon, transform, copy
- 225 cards complete, 75 remaining to reach 300 target
- Current balance: 6.9% faction delta (within target!)
- **Deliverable:** 300-card Foundation Set with 12 Commander Decks
