# Essence Wars: New Horizons Edition - Card Database

> **Version:** 1.0
> **Last Updated:** January 2026
> **Card Count:** 300 cards

This document provides a complete reference for all cards in the New Horizons Edition.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Card Distribution](#2-card-distribution)
3. [Faction Reference](#3-faction-reference)
4. [Legendary Commanders](#4-legendary-commanders)
5. [Keyword Distribution](#5-keyword-distribution)
6. [Card Cycles](#6-card-cycles)
7. [Commander Decks](#7-commander-decks)

---

## 1. Overview

The New Horizons Edition is the first complete expansion for Essence Wars, featuring:

- **300 cards** organized across 4 factions
- **12 Legendary Commanders** (4 per faction)
- **12 pre-built Commander Decks**
- **16 keywords** including 4 new faction-specific mechanics

### Card Files (Source of Truth)

All cards are defined in YAML format:

```
data/cards/core_set/
├── argentum.yaml    # 75 cards (IDs 1000-1074)
├── symbiote.yaml    # 75 cards (IDs 2000-2074)
├── obsidion.yaml    # 75 cards (IDs 3000-3074)
└── neutral.yaml     # 75 cards (IDs 4000-4074)
```

---

## 2. Card Distribution

### By Faction

| Faction | Total | Creatures | Spells | Supports | ID Range |
|---------|-------|-----------|--------|----------|----------|
| Argentum Combine | 75 | 55 | 11 | 9 | 1000-1074 |
| Symbiote Circles | 75 | 52 | 11 | 12 | 2000-2074 |
| Obsidion Syndicate | 75 | 53 | 13 | 9 | 3000-3074 |
| Free-Walkers (Neutral) | 75 | 51 | 14 | 10 | 4000-4074 |
| **Total** | **300** | **211** | **49** | **40** | |

### By Card Type

| Type | Count | Percentage |
|------|-------|------------|
| Creatures | 211 | 70.3% |
| Spells | 49 | 16.3% |
| Supports | 40 | 13.4% |

### By Rarity (Approximate)

| Rarity | Per Faction | Total |
|--------|-------------|-------|
| Common | ~30 | ~120 |
| Uncommon | ~25 | ~100 |
| Rare | ~15 | ~60 |
| Legendary | ~5 | ~20 |

---

## 3. Faction Reference

### Argentum Combine ("The Wall")

**Identity:** Defensive constructs and mechanical soldiers. Focus on durability, damage reduction, and outlasting opponents.

**Primary Keywords:** Guard, Piercing, Shield, Fortify

**ID Range:** 1000-1074

| ID Range | Card Types |
|----------|------------|
| 1000-1019 | Common creatures and spells |
| 1020-1039 | Uncommon creatures and spells |
| 1040-1055 | Rare creatures, spells, supports |
| 1056-1059 | Legendary Commanders |
| 1060-1074 | Commander support cards |

**Signature Mechanics:**
- **Fortify** — Take 1 less damage from all sources (new in New Horizons)
- **Token Generation** — Construct tokens from commanders and supports
- **High Health** — Creatures prioritize health over attack

---

### Symbiote Circles ("The Swarm")

**Identity:** Aggressive biological creatures. Focus on speed, board flooding, and death triggers.

**Primary Keywords:** Rush, Lethal, Regenerate, Frenzy, Volatile

**ID Range:** 2000-2074

| ID Range | Card Types |
|----------|------------|
| 2000-2019 | Common creatures and spells |
| 2020-2039 | Uncommon creatures and spells |
| 2040-2059 | Rare creatures, spells, supports |
| 2060-2063 | Legendary Commanders |
| 2064-2074 | Commander support cards |

**Signature Mechanics:**
- **Frenzy** — +1 attack after each attack this turn (new in New Horizons)
- **Volatile** — Deal 2 damage to all enemies when this dies (new in New Horizons)
- **Rush Swarm** — Flood board with cheap Rush creatures
- **Regenerate** — Outlast through healing

---

### Obsidion Syndicate ("The Shadow")

**Identity:** Shadow assassins and blood mages. Focus on burst damage, life manipulation, and precision removal.

**Primary Keywords:** Lifesteal, Stealth, Quick, Ephemeral, Ward

**ID Range:** 3000-3074

| ID Range | Card Types |
|----------|------------|
| 3000-3019 | Common creatures and spells |
| 3020-3039 | Uncommon creatures and spells |
| 3040-3054 | Rare creatures, spells, supports |
| 3055-3058 | Legendary Commanders |
| 3059-3074 | Commander support cards |

**Signature Mechanics:**
- **Ward** — Block first targeted spell/ability (new in New Horizons)
- **Lifesteal** — Heal when dealing damage
- **Stealth** — Cannot be targeted until attacking
- **Ephemeral** — Dies at end of turn (bonus stats)

---

### Free-Walkers ("The Toolbox")

**Identity:** Neutral mercenaries and wanderers. Utility cards that support any faction.

**Primary Keywords:** Ranged, Charge

**ID Range:** 4000-4074

| ID Range | Card Types |
|----------|------------|
| 4000-4019 | Common creatures and spells |
| 4020-4039 | Uncommon creatures and spells |
| 4040-4059 | Rare creatures, spells, supports |
| 4060-4074 | Legendary creatures (no commanders) |

**Signature Mechanics:**
- **Ranged** — Attack any enemy creature (bypass lane restrictions)
- **Charge** — +2 attack damage when attacking
- **Card Draw** — Many neutral cards provide card advantage
- **Flexibility** — Fill gaps in faction decks

---

## 4. Legendary Commanders

Commanders are the centerpieces of deck building. Each has a unique ability that defines an archetype.

### Argentum Combine Commanders

| ID | Name | Cost | Stats | Keywords | Ability |
|----|------|------|-------|----------|---------|
| 1056 | **The High Artificer** | 6 | 3/5 | — | OnPlay: Summon two 2/2 Construct tokens |
| 1057 | **Iron Colossus Prime** | 7 | 2/10 | Guard | Passive: All friendly creatures have +0/+2 |
| 1058 | **Siege Marshal Vex** | 6 | 5/4 | Piercing | OnAttack: Deal 2 damage to enemy player |
| 1059 | **The Grand Architect** | 6 | 3/6 | Fortify | Passive: All friendly creatures have Fortify |

### Symbiote Circles Commanders

| ID | Name | Cost | Stats | Keywords | Ability |
|----|------|------|-------|----------|---------|
| 2060 | **The Broodmother** | 6 | 3/5 | Rush | OnAttack: Summon a 2/2 Rush Broodling |
| 2061 | **Plague Sovereign** | 6 | 4/4 | Volatile | OnAllyDeath: Deal 1 damage to enemy player |
| 2062 | **Alpha of the Hunt** | 5 | 4/3 | Frenzy | Passive: All friendly creatures have +1 Attack |
| 2063 | **The Eternal Grove** | 6 | 2/8 | Regenerate | Passive: All friendly creatures have Regenerate |

### Obsidion Syndicate Commanders

| ID | Name | Cost | Stats | Keywords | Ability |
|----|------|------|-------|----------|---------|
| 3055 | **The Blood Sovereign** | 6 | 4/5 | Lifesteal | Passive: All friendly creatures have Lifesteal |
| 3056 | **Shadow Emperor Kael** | 6 | 5/4 | Stealth, Quick | OnKill: Return this creature to hand |
| 3057 | **The Shadow Weaver** | 6 | 3/4 | Stealth | OnPlay: Summon two 2/2 Ephemeral Stealth Shadow Clones |
| 3058 | **Void Archon** | 5 | 4/4 | Quick | Passive: All friendly creatures have Quick |

---

## 5. Keyword Distribution

### Keywords by Faction

| Keyword | Argentum | Symbiote | Obsidion | Neutral |
|---------|:--------:|:--------:|:--------:|:-------:|
| Rush | — | ★★★ | ★ | ★ |
| Ranged | ★ | ★ | — | ★★★ |
| Piercing | ★★ | — | — | ★★ |
| Guard | ★★★ | — | — | ★ |
| Lifesteal | — | — | ★★★ | — |
| Lethal | — | ★★★ | ★ | — |
| Shield | ★★ | — | — | ★ |
| Quick | — | — | ★★★ | ★ |
| Ephemeral | — | — | ★★★ | — |
| Regenerate | ★ | ★★★ | — | — |
| Stealth | — | — | ★★★ | — |
| Charge | — | — | — | ★★★ |
| Frenzy | — | ★★★ | — | — |
| Volatile | — | ★★★ | — | — |
| Fortify | ★★★ | — | — | — |
| Ward | — | — | ★★ | ★ |

**Legend:** ★★★ Primary | ★★ Secondary | ★ Rare | — Avoided

### New Horizons Keywords

These keywords were added in the New Horizons Edition:

| Keyword | Effect | Primary Faction |
|---------|--------|-----------------|
| **Frenzy** | +1 attack after each attack this turn | Symbiote |
| **Volatile** | Deal 2 damage to all enemy creatures when this dies | Symbiote |
| **Fortify** | Take 1 less damage from all sources (min 1) | Argentum |
| **Ward** | First targeted spell/ability has no effect; Ward removed | Obsidion |

---

## 6. Card Cycles

### Faction Staples

Each faction has common card patterns at similar costs:

**1-Cost Creatures:**
- Argentum: 1/3 vanilla, defensive
- Symbiote: 2/2 Rush or 1/2 Lethal
- Obsidion: 2/1 Ephemeral or 1/2 Lifesteal
- Neutral: 2/2 vanilla or 1/2 with utility

**3-Cost Removal:**
- Argentum: Deal 3 damage (Execute variants with filters)
- Symbiote: Buff spells (+2/+1 type effects)
- Obsidion: Soul Drain (damage + heal)
- Neutral: Card draw or bounce effects

**6-Cost Finishers:**
- All Commanders are 5-7 cost with powerful abilities

### Cross-Faction Patterns

**Token Generators:**
- 1056 (Argentum) — Construct tokens
- 2060 (Symbiote) — Broodling tokens
- 3057 (Obsidion) — Shadow Clone tokens

**Passive Aura Commanders:**
- 1057 — +0/+2 to all allies
- 1059 — Fortify to all allies
- 2062 — +1 Attack to all allies
- 2063 — Regenerate to all allies
- 3055 — Lifesteal to all allies
- 3058 — Quick to all allies

---

## 7. Commander Decks

Each faction has 4 pre-built Commander Decks organized around their Legendary Commanders:

### Argentum Combine

| Deck ID | Deck Name | Commander | Archetype |
|---------|-----------|-----------|-----------|
| artificer_tokens | The High Artificer | 1056 | Token/Construct |
| colossus_wall | Iron Colossus Prime | 1057 | Guard/Wall |
| vex_piercing | Siege Marshal Vex | 1058 | Piercing/Aggro |
| architect_fortify | The Grand Architect | 1059 | Fortify/Control |

### Symbiote Circles

| Deck ID | Deck Name | Commander | Archetype |
|---------|-----------|-----------|-----------|
| broodmother_swarm | The Broodmother | 2060 | Rush/Swarm |
| plague_volatile | Plague Sovereign | 2061 | Volatile/Death |
| alpha_frenzy | Alpha of the Hunt | 2062 | Frenzy/Aggro |
| grove_regenerate | The Eternal Grove | 2063 | Regenerate/Midrange |

### Obsidion Syndicate

| Deck ID | Deck Name | Commander | Archetype |
|---------|-----------|-----------|-----------|
| sovereign_lifesteal | The Blood Sovereign | 3055 | Lifesteal/Sustain |
| kael_assassin | Shadow Emperor Kael | 3056 | Stealth/Assassin |
| shadow_weaver | The Shadow Weaver | 3057 | Shadow Clone |
| archon_burst | Void Archon | 3058 | Quick/Burst |

### Deck Files

All deck definitions are in TOML format:

```
data/decks/
├── argentum/
│   ├── artificer_tokens.toml
│   ├── colossus_wall.toml
│   ├── vex_piercing.toml
│   └── architect_fortify.toml
├── symbiote/
│   ├── broodmother_swarm.toml
│   ├── plague_volatile.toml
│   ├── alpha_frenzy.toml
│   └── grove_regenerate.toml
└── obsidion/
    ├── sovereign_lifesteal.toml
    ├── kael_assassin.toml
    ├── shadow_weaver.toml
    └── archon_burst.toml
```

### Deck Composition

Standard deck format:
- **30 cards total**
- **~21 faction cards** (70%)
- **~9 neutral cards** (30%)
- **1 Legendary Commander**
- **Max 2 copies** of non-Legendary cards

---

## Appendix: Quick Card Lookup

### By Effect Type

**Damage Spells:**
- 1010-1014 (Argentum)
- 2010-2016 (Symbiote)
- 3008-3016 (Obsidion)
- 4010-4016 (Neutral)

**Buff Spells:**
- 1015-1019 (Argentum)
- 2017-2024 (Symbiote)
- 3017-3024 (Obsidion)
- 4017-4024 (Neutral)

**Removal:**
- Execute variants: 1025 (Argentum), 3025 (Obsidion)
- Destroy effects: Various across factions
- Bounce effects: 1029 (Argentum), 2040 (Symbiote)

**Card Draw:**
- 4003 (Hedge Wizard), 4010 (Supply Cache)
- Various support cards with StartOfTurn draw

### Conditional Effect Cards (Phase 4)

Cards using the conditional trigger system:

| ID | Name | Condition | Bonus Effect |
|----|------|-----------|--------------|
| 1033 | Judgment Strike | target_died | Heal 3 |
| 2035 | Soul Reaper | target_died | Draw 1 |
| 3032 | Vampiric Execution | target_died | Heal 5, Draw 1 |

### Filtered Effect Cards (Phase 4)

Cards using creature filters:

| ID | Name | Filter | Effect |
|----|------|--------|--------|
| 1025 | Execute | max_health: 3 | Destroy |
| 1026 | Purge the Weak | max_health: 3 | Damage 2 to all |
| 1028 | Rally the Guards | has_keyword: Guard | +1/+1 |
| 1030 | Mass Recall | max_health: 3 | Bounce all |

---

*For the complete card definitions, see the YAML files in `data/cards/core_set/`.*

*End of Card Database Reference*
