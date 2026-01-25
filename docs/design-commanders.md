# Commander System Design Document

> **Version:** 2.0 (Commander Rework)
> **Last Updated:** 2026-01-25
> **Status:** Design Phase - Awaiting Implementation

---

## Table of Contents

1. [Overview](#1-overview)
2. [Design Principles](#2-design-principles)
3. [Core Rules](#3-core-rules)
4. [Command Zone](#4-command-zone)
5. [Ability System](#5-ability-system)
6. [Commander Card Schema](#6-commander-card-schema)
7. [Redesigned Commanders](#7-redesigned-commanders)
8. [Game Flow Changes](#8-game-flow-changes)
9. [UI Requirements](#9-ui-requirements)
10. [Migration Plan](#10-migration-plan)

---

## 1. Overview

### 1.1 The Problem

In the current system, commanders are Legendary creatures that can be played from hand and killed like any other creature. This creates several issues:

- **Commanders die quickly** - Even powerful commanders can be removed on turn 2-3 by removal spells or efficient trades
- **Deck identity is fragile** - When your commander dies, your deck's unique identity disappears
- **Thematic inconsistency** - The lore describes commanders as directing battles from Farsight Tables in high-orbit airships, not fighting on the front lines
- **Wasted design space** - Unique commander abilities often never activate because the creature is removed immediately

### 1.2 The Solution

**Commanders become the player's persona.** Instead of being creatures that can die, commanders:

- ARE the player's 30 life (Commander Life = Player Life)
- Observe from the **Command Zone** (a special board position, not a creature slot)
- Provide **persistent abilities** throughout the entire game
- Cannot be targeted by attacks or spells directly
- When their life reaches 0, they **retreat** (the player loses)

### 1.3 Lore Alignment

This design aligns with the established lore:

> *"The players (You and the Opponent) are **Commanders** sitting in high-orbit airships or using 'Farsight Tables.' In this era of high science, sensors are perfect. You can see exactly what the enemy has deployed. The strategy isn't about hiding; it's about calculating the outcome better than they can."*

The commander was never meant to be on the battlefield—they direct operations from above.

---

## 2. Design Principles

| Principle | Description |
|-----------|-------------|
| **Identity** | The commander defines WHO you are in this battle |
| **Persistence** | Commander abilities matter for the entire game, not just until removed |
| **Strategic Anchor** | Your commander shapes HOW you play, not just what cards you include |
| **Win Condition** | The commander's fate = the battle's outcome |
| **Thematic Coherence** | Commanders direct from safety, not fight on the front lines |
| **Simplicity** | One ability per commander (passive OR triggered), no activated abilities |

---

## 3. Core Rules

### 3.1 Commander = Player Identity

| Aspect | Rule |
|--------|------|
| **Commander Life** | Commander IS your 30 life. Damage to "player" = damage to commander. |
| **Commander Location** | Command Zone - visible on board but NOT a creature slot |
| **Deck Composition** | 30 cards minimum + 1 Commander (commander separate from deck) |
| **Starting State** | Both commanders visible in Command Zones from turn 1 |
| **Win Condition** | Reduce enemy commander to 0 life → they retreat → you win |

### 3.2 Damage to Commander

**Face attacks only.** The existing face attack rules apply unchanged:

- Creatures can attack the enemy commander when their **direct lane is empty**
- Adjacent lanes do NOT block face attacks
- Guard creatures in range MUST be attacked first
- **Spells and abilities CANNOT target commanders directly**

This preserves the importance of creature combat and board control.

### 3.3 Commander Targeting Rules

| Source | Can Target Commander? |
|--------|----------------------|
| Creature face attacks | YES (when direct lane empty) |
| Damage spells | NO |
| Targeted abilities | NO |
| AoE damage effects | NO |
| Lifesteal healing | YES (heals your commander) |
| Direct healing spells | YES (if targeting "player") |

### 3.4 Deck Composition

**Before (v1.x):**
- 30 cards including 1 commander creature

**After (v2.0):**
- 30 cards (commander NOT included in deck)
- 1 Commander (separate, always in Command Zone)
- Commander cannot be drawn, discarded, bounced, or destroyed

---

## 4. Command Zone

### 4.1 Board Layout

```
┌─────────────────────────────────────────────────────────────────┐
│                        OPPONENT                                  │
│  ┌────────────────┐                                             │
│  │  COMMAND ZONE  │  Commander Life: 30    Deck: 22   Hand: 5  │
│  │  [Commander]   │  Ability: "Your creatures have +1 Attack"  │
│  └────────────────┘                                             │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐    ┌────┐ ┌────┐          │
│  │ C1 │ │ C2 │ │ C3 │ │ C4 │ │ C5 │    │ S1 │ │ S2 │          │
│  └────┘ └────┘ └────┘ └────┘ └────┘    └────┘ └────┘          │
│         5 Creature Slots                2 Support Slots         │
├─────────────────────────────────────────────────────────────────┤
│         5 Creature Slots                2 Support Slots         │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐    ┌────┐ ┌────┐          │
│  │ C1 │ │ C2 │ │ C3 │ │ C4 │ │ C5 │    │ S1 │ │ S2 │          │
│  └────┘ └────┘ └────┘ └────┘ └────┘    └────┘ └────┘          │
│  ┌────────────────┐                                             │
│  │  COMMAND ZONE  │  Commander Life: 30    Deck: 22   Hand: 5  │
│  │  [Commander]   │  Ability: "Your creatures with Guard..."   │
│  └────────────────┘                                             │
│                          YOU                                     │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Command Zone Properties

| Property | Description |
|----------|-------------|
| **Visibility** | Always visible to both players |
| **Contents** | Displays commander card art, name, and ability text |
| **Interaction** | Cannot be targeted, attacked directly, or affected by card effects |
| **State Display** | Shows commander life (= player life) prominently |

---

## 5. Ability System

### 5.1 Design Constraints

- **One ability per commander** (either passive OR triggered)
- **No activated abilities** (no "Pay 2 AP: Do X")
- **No combat-based triggers** (commanders don't fight)
- **Abilities persist entire game** (no way to silence/remove them)

### 5.2 Passive Abilities

Passive abilities provide continuous effects while the commander is in play (always).

**Examples:**
- "Your creatures have +1 Attack"
- "Your creatures with Guard have +1 Health"
- "Your creatures have Regenerate"
- "Your creatures have Lifesteal"

**Implementation:** Applied during state evaluation, similar to support passive effects.

### 5.3 Triggered Abilities

Triggered abilities fire when specific game events occur.

**Available Triggers:**

| Trigger | Description |
|---------|-------------|
| `StartOfTurn` | At the start of your turn |
| `EndOfTurn` | At the end of your turn |
| `OnCreaturePlayed` | When you play any creature |
| `OnAllyDeath` | When one of your creatures dies |
| `OnEnemyDeath` | When an enemy creature dies |
| `OnCombatStart` | When combat begins (before damage) |
| `OnFaceAttack` | When you deal damage to enemy commander |
| `OnCardDrawn` | When you draw a card |

**NOT Available (since commanders don't fight):**
- `OnAttack`, `OnDefend`, `OnDealDamage`, `OnTakeDamage`, `OnKill`, `OnDeath`

### 5.4 Effect Types for Commander Abilities

| Effect Type | Example |
|-------------|---------|
| **Stat Buff** | Give creature +X/+Y |
| **Grant Keyword** | Give creature(s) a keyword |
| **Summon Token** | Create a token creature |
| **Damage** | Deal X damage to target(s) |
| **Heal** | Restore X health to player |
| **Draw** | Draw X cards |
| **Essence** | Gain X temporary essence |

---

## 6. Commander Card Schema

### 6.1 Old Schema (Creature-Based)

```yaml
# DEPRECATED - v1.x commander format
- id: 1059
  name: "The Grand Architect"
  cost: 6
  card_type: creature
  attack: 2
  health: 6
  keywords: [Fortify]
  rarity: Legendary
  tags: [Commander, Construct]
  abilities:
    - trigger: OnPlay
      targeting: NoTarget
      effects:
        - type: grant_keyword
          keyword: Fortify
```

### 6.2 New Schema (Commander Type)

```yaml
# NEW - v2.0 commander format
- id: 1059
  name: "The Grand Architect"
  card_type: commander
  faction: Argentum
  rarity: Legendary
  # No cost, attack, health, or keywords - commanders don't fight

  # EITHER a passive_ability OR a triggered_ability (not both)
  passive_ability:
    description: "Your creatures have Fortify"
    effect:
      type: grant_keyword
      keyword: Fortify

  flavor: "Every bolt placed with purpose. Every gear turning in harmony."
```

### 6.3 Schema Definition

```yaml
commander_schema:
  required:
    - id: integer           # Unique card ID
    - name: string          # Display name
    - card_type: commander  # Must be "commander"
    - faction: string       # Argentum, Symbiote, Obsidion, Neutral
    - rarity: Legendary     # Always Legendary

  one_of:  # Exactly one ability type
    - passive_ability:
        description: string  # Human-readable ability text
        effect: Effect       # Effect to apply continuously

    - triggered_ability:
        trigger: TriggerType     # When ability fires
        description: string      # Human-readable ability text
        condition: Condition?    # Optional condition
        effects: Effect[]        # Effects to apply

  optional:
    - flavor: string        # Flavor text
    - tags: string[]        # Additional tags (optional)
```

---

## 7. Redesigned Commanders

### 7.1 Argentum Combine

#### The High Artificer (ID: 1056)

| Property | Value |
|----------|-------|
| **Faction** | Argentum Combine |
| **Archetype** | Token/Construct |
| **Ability Type** | Triggered |
| **Ability** | **StartOfTurn:** Summon a 1/1 Brass Cog token |
| **Flavor** | *"The Combine doesn't build soldiers. We manufacture victory."* |

```yaml
- id: 1056
  name: "The High Artificer"
  card_type: commander
  faction: Argentum
  rarity: Legendary
  triggered_ability:
    trigger: StartOfTurn
    description: "At the start of your turn, summon a 1/1 Brass Cog"
    effects:
      - type: summon_token
        token:
          name: "Brass Cog"
          attack: 1
          health: 1
          keywords: []
  flavor: "The Combine doesn't build soldiers. We manufacture victory."
```

---

#### The Sanctum Healer (ID: 1057)

| Property | Value |
|----------|-------|
| **Faction** | Argentum Combine |
| **Archetype** | Regenerate/Healing |
| **Ability Type** | Passive |
| **Ability** | Your creatures have Regenerate |
| **Flavor** | *"Where gears mend flesh and essence restores steel."* |

```yaml
- id: 1057
  name: "The Sanctum Healer"
  card_type: commander
  faction: Argentum
  rarity: Legendary
  passive_ability:
    description: "Your creatures have Regenerate"
    effect:
      type: grant_keyword
      keyword: Regenerate
  flavor: "Where gears mend flesh and essence restores steel."
```

---

#### Siege Marshal Vex (ID: 1058)

| Property | Value |
|----------|-------|
| **Faction** | Argentum Combine |
| **Archetype** | Piercing/Aggro |
| **Ability Type** | Passive |
| **Ability** | Your creatures have +1 Attack |
| **Flavor** | *"A wall is just a door that hasn't been opened hard enough."* |

```yaml
- id: 1058
  name: "Siege Marshal Vex"
  card_type: commander
  faction: Argentum
  rarity: Legendary
  passive_ability:
    description: "Your creatures have +1 Attack"
    effect:
      type: buff_stats
      attack: 1
      health: 0
  flavor: "A wall is just a door that hasn't been opened hard enough."
```

---

#### The Grand Architect (ID: 1059)

| Property | Value |
|----------|-------|
| **Faction** | Argentum Combine |
| **Archetype** | Fortify/Control |
| **Ability Type** | Passive |
| **Ability** | Your creatures have Fortify |
| **Flavor** | *"Every bolt placed with purpose. Every gear turning in harmony."* |

```yaml
- id: 1059
  name: "The Grand Architect"
  card_type: commander
  faction: Argentum
  rarity: Legendary
  passive_ability:
    description: "Your creatures have Fortify"
    effect:
      type: grant_keyword
      keyword: Fortify
  flavor: "Every bolt placed with purpose. Every gear turning in harmony."
```

---

### 7.2 Symbiote Circles

#### The Broodmother (ID: 2060)

| Property | Value |
|----------|-------|
| **Faction** | Symbiote Circles |
| **Archetype** | Rush/Swarm |
| **Ability Type** | Triggered |
| **Ability** | **OnCreaturePlayed:** If it has Rush, summon a 1/1 Broodling with Rush |
| **Flavor** | *"Where she commands, the swarm follows."* |

```yaml
- id: 2060
  name: "The Broodmother"
  card_type: commander
  faction: Symbiote
  rarity: Legendary
  triggered_ability:
    trigger: OnCreaturePlayed
    description: "When you play a creature with Rush, summon a 1/1 Broodling with Rush"
    condition:
      has_keyword: Rush
    effects:
      - type: summon_token
        token:
          name: "Broodling"
          attack: 1
          health: 1
          keywords: [Rush]
  flavor: "Where she commands, the swarm follows."
```

---

#### Plague Sovereign (ID: 2061)

| Property | Value |
|----------|-------|
| **Faction** | Symbiote Circles |
| **Archetype** | Volatile/Death |
| **Ability Type** | Triggered |
| **Ability** | **OnAllyDeath:** Deal 1 damage to the enemy commander |
| **Flavor** | *"Every death feeds the plague."* |

```yaml
- id: 2061
  name: "Plague Sovereign"
  card_type: commander
  faction: Symbiote
  rarity: Legendary
  triggered_ability:
    trigger: OnAllyDeath
    description: "When one of your creatures dies, deal 1 damage to the enemy commander"
    effects:
      - type: damage
        amount: 1
        target: EnemyCommander
  flavor: "Every death feeds the plague."
```

---

#### Alpha of the Hunt (ID: 2062)

| Property | Value |
|----------|-------|
| **Faction** | Symbiote Circles |
| **Archetype** | Frenzy/Aggro |
| **Ability Type** | Passive |
| **Ability** | Your creatures have +1 Attack |
| **Flavor** | *"The pack hunts as one. The Alpha strikes first."* |

```yaml
- id: 2062
  name: "Alpha of the Hunt"
  card_type: commander
  faction: Symbiote
  rarity: Legendary
  passive_ability:
    description: "Your creatures have +1 Attack"
    effect:
      type: buff_stats
      attack: 1
      health: 0
  flavor: "The pack hunts as one. The Alpha strikes first."
```

---

#### The Eternal Grove (ID: 2063)

| Property | Value |
|----------|-------|
| **Faction** | Symbiote Circles |
| **Archetype** | Regenerate/Midrange |
| **Ability Type** | Passive |
| **Ability** | Your creatures have Regenerate |
| **Flavor** | *"The forest remembers. The forest endures."* |

```yaml
- id: 2063
  name: "The Eternal Grove"
  card_type: commander
  faction: Symbiote
  rarity: Legendary
  passive_ability:
    description: "Your creatures have Regenerate"
    effect:
      type: grant_keyword
      keyword: Regenerate
  flavor: "The forest remembers. The forest endures."
```

---

### 7.3 Obsidion Syndicate

#### The Blood Sovereign (ID: 3055)

| Property | Value |
|----------|-------|
| **Faction** | Obsidion Syndicate |
| **Archetype** | Lifesteal/Sustain |
| **Ability Type** | Passive |
| **Ability** | Your creatures have Lifesteal |
| **Flavor** | *"Blood is such an inefficient fuel. Let me show you how to refine it."* |

```yaml
- id: 3055
  name: "The Blood Sovereign"
  card_type: commander
  faction: Obsidion
  rarity: Legendary
  passive_ability:
    description: "Your creatures have Lifesteal"
    effect:
      type: grant_keyword
      keyword: Lifesteal
  flavor: "Blood is such an inefficient fuel. Let me show you how to refine it."
```

---

#### Shadow Emperor Kael (ID: 3056)

| Property | Value |
|----------|-------|
| **Faction** | Obsidion Syndicate |
| **Archetype** | Stealth/Assassin |
| **Ability Type** | Triggered |
| **Ability** | **OnEnemyDeath:** Draw a card |
| **Flavor** | *"You never see him twice. Once is enough."* |

```yaml
- id: 3056
  name: "Shadow Emperor Kael"
  card_type: commander
  faction: Obsidion
  rarity: Legendary
  triggered_ability:
    trigger: OnEnemyDeath
    description: "When an enemy creature dies, draw a card"
    effects:
      - type: draw
        count: 1
  flavor: "You never see him twice. Once is enough."
```

---

#### The Shadow Weaver (ID: 3057)

| Property | Value |
|----------|-------|
| **Faction** | Obsidion Syndicate |
| **Archetype** | Shadow/Clone |
| **Ability Type** | Passive |
| **Ability** | Your creatures have Stealth |
| **Flavor** | *"Shadows don't lie. They simply don't tell the whole truth."* |

```yaml
- id: 3057
  name: "The Shadow Weaver"
  card_type: commander
  faction: Obsidion
  rarity: Legendary
  passive_ability:
    description: "Your creatures have Stealth"
    effect:
      type: grant_keyword
      keyword: Stealth
  flavor: "Shadows don't lie. They simply don't tell the whole truth."
```

---

#### Void Archon (ID: 3058)

| Property | Value |
|----------|-------|
| **Faction** | Obsidion Syndicate |
| **Archetype** | Quick/Burst |
| **Ability Type** | Passive |
| **Ability** | Your creatures have Quick |
| **Flavor** | *"Time is just another resource to exploit."* |

```yaml
- id: 3058
  name: "Void Archon"
  card_type: commander
  faction: Obsidion
  rarity: Legendary
  passive_ability:
    description: "Your creatures have Quick"
    effect:
      type: grant_keyword
      keyword: Quick
  flavor: "Time is just another resource to exploit."
```

---

### 7.4 Commander Summary Table

| ID | Name | Faction | Type | Ability |
|----|------|---------|------|---------|
| 1056 | The High Artificer | Argentum | Triggered | StartOfTurn: Summon 1/1 Brass Cog |
| 1057 | The Sanctum Healer | Argentum | Passive | Creatures have Regenerate |
| 1058 | Siege Marshal Vex | Argentum | Passive | Creatures have +1 Attack |
| 1059 | The Grand Architect | Argentum | Passive | Creatures have Fortify |
| 2060 | The Broodmother | Symbiote | Triggered | OnCreaturePlayed (Rush): Summon 1/1 Rush Broodling |
| 2061 | Plague Sovereign | Symbiote | Triggered | OnAllyDeath: 1 damage to enemy commander |
| 2062 | Alpha of the Hunt | Symbiote | Passive | Creatures have +1 Attack |
| 2063 | The Eternal Grove | Symbiote | Passive | Creatures have Regenerate |
| 3055 | The Blood Sovereign | Obsidion | Passive | Creatures have Lifesteal |
| 3056 | Shadow Emperor Kael | Obsidion | Triggered | OnEnemyDeath: Draw a card |
| 3057 | The Shadow Weaver | Obsidion | Passive | Creatures have Stealth |
| 3058 | Void Archon | Obsidion | Passive | Creatures have Quick |

---

## 8. Game Flow Changes

### 8.1 Game Setup (Updated)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GAME SETUP (v2.0)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. COMMANDER SELECTION                                                     │
│     • Each player selects a Commander                                      │
│     • Commander defines deck constraints (faction)                         │
│     • Commander placed in Command Zone (visible from start)                │
│                                                                             │
│  2. DECK PREPARATION                                                        │
│     • Both players submit their decks (30 cards, NO commander)             │
│     • Decks are arranged using Fair Order Algorithm                        │
│                                                                             │
│  3. STARTING STATE                                                          │
│     • Both commanders visible in Command Zones                             │
│     • Commander abilities ACTIVE from turn 1                               │
│     • Both players: 30 life, 1 max essence, 1 current essence             │
│     • Each player draws 4 cards                                            │
│                                                                             │
│  4. GAME BEGINS                                                             │
│     • Player 1 starts Turn 1 (skips draw)                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Turn Structure (Updated)

The turn structure remains largely the same, with additions for commander triggers:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TURN STRUCTURE (v2.0)                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  START OF TURN (Automatic)                                                  │
│  ─────────────────────────                                                  │
│    1. Increase max essence by 1 (cap at 10)                                │
│    2. Refill current essence to max                                        │
│    3. Reset action points to 3                                             │
│    4. Refresh all creatures (clear "exhausted" status)                     │
│    5. Draw 1 card (EXCEPTION: Player 1 skips on Turn 1)                    │
│    6. Tick down support durability by 1, remove if 0                       │
│    7. Trigger all "Start of Turn" effects (supports AND commander)    ★   │
│                                                                             │
│  MAIN PHASE (Player Decisions)                                              │
│  ─────────────────────────────                                              │
│    • Commander passive abilities ALWAYS active                         ★   │
│    • Commander triggered abilities fire on relevant events             ★   │
│    While action_points > 0 AND player chooses to act:                      │
│      • Play a Card (costs 1 AP + essence cost)                             │
│      • Attack with a Creature (costs 1 AP)                                 │
│      • Activate an Ability (costs AP as specified)                         │
│      • End Turn (costs 0 AP, forfeits remaining AP)                        │
│                                                                             │
│  END OF TURN (Automatic)                                                    │
│  ──────────────────────────                                                 │
│    1. Trigger all "End of Turn" effects (supports AND commander)       ★   │
│    2. Pass turn to opponent                                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Win Condition (Unchanged)

| Condition | Result |
|-----------|--------|
| Enemy commander life reaches 0 | You win (enemy retreats) |
| Turn 30 reached | Higher life wins (or draw if equal) |

---

## 9. UI Requirements

### 9.1 Command Zone Display

The UI must prominently display each commander:

**Required Elements:**
- Commander portrait/card art
- Commander name
- Commander life (= player life, displayed prominently)
- Ability text (always visible or on hover)
- Faction indicator

**Visual Hierarchy:**
```
┌──────────────────────────────────────┐
│  ┌─────────┐  THE GRAND ARCHITECT    │
│  │  [ART]  │  ───────────────────    │
│  │         │  Life: 24 / 30          │
│  │         │                         │
│  └─────────┘  Your creatures have    │
│               Fortify                │
│                                      │
│  [ARGENTUM COMBINE]                  │
└──────────────────────────────────────┘
```

### 9.2 Ability Indicators

- **Passive abilities:** Show persistent glow/indicator on affected creatures
- **Triggered abilities:** Show brief animation when trigger fires
- **Ability reminder:** Tooltip on hover showing full ability text

### 9.3 Face Attack Visualization

When a creature attacks the enemy commander:
- Draw attack line from creature to Command Zone
- Show damage number on commander portrait
- Animate life total decrease

---

## 10. Migration Plan

### 10.1 Data Migration

1. **Commander Cards:** Convert from creature type to commander type
2. **Deck Files:** Remove commander from card list, add separate `commander` field
3. **Card Database:** Add commander card type support

### 10.2 Engine Changes

| Component | Change Required |
|-----------|-----------------|
| `CardType` enum | Add `Commander` variant |
| `GameState` | Add `commander_p1`, `commander_p2` fields |
| `PlayerState` | Life already exists (no change) |
| Effect System | Add commander as passive effect source |
| Legal Actions | No changes (commanders can't be played/targeted) |
| State Tensor | Add commander ID fields (for AI) |

### 10.3 Deck File Format Update

**Before:**
```toml
id = "architect_fortify"
name = "The Grand Architect"
cards = [1059, 1030, 1040, ...]  # Commander was card 1059
```

**After:**
```toml
id = "architect_fortify"
name = "The Grand Architect"
commander = 1059  # Commander separate
cards = [1030, 1040, ...]  # 30 cards, no commander
```

### 10.4 Implementation Phases

| Phase | Tasks |
|-------|-------|
| **Phase 1** | Update card schema, convert commander cards in YAML |
| **Phase 2** | Engine: Add commander fields to GameState, load commanders |
| **Phase 3** | Engine: Implement commander passive effects |
| **Phase 4** | Engine: Implement commander triggered effects |
| **Phase 5** | Update deck files and validation |
| **Phase 6** | Update state tensor for AI |
| **Phase 7** | UI: Add Command Zone display |
| **Phase 8** | Testing and balance validation |

---

## Appendix A: Design Alternatives Considered

### A.1 Activated Abilities

**Considered:** Commanders have "Pay 2 AP: Do X" abilities.

**Rejected:** Adds complexity, makes commanders too active in gameplay. The goal is persistent identity, not another resource to manage.

### A.2 Commander Targeting

**Considered:** Some spells can target commanders directly.

**Rejected:** Would require extensive spell redesign and could make direct damage too powerful. Face attacks provide sufficient interaction.

### A.3 Commander on Board

**Considered:** Commander occupies a special 6th creature slot.

**Rejected:** Creates confusion about whether commander can attack/be attacked. Command Zone is cleaner.

---

## Appendix B: Balance Considerations

### B.1 Passive Ability Power Levels

Passive abilities that grant keywords should be balanced:

| Keyword | Power Level | Notes |
|---------|-------------|-------|
| Guard | Medium | Forces enemy to trade, but board position matters |
| Fortify | Low-Medium | Defensive, helps survive trades |
| Regenerate | Medium | Significant sustain, but doesn't prevent burst |
| Lifesteal | High | Strong sustain, scales with attack |
| Quick | High | Combat advantage, especially with high attack |
| Stealth | Medium-High | Protects creatures, enables ambush |

### B.2 Triggered Ability Frequency

| Trigger | Frequency | Notes |
|---------|-----------|-------|
| StartOfTurn | 1/turn | Reliable, scales with game length |
| OnCreaturePlayed | Variable | Depends on deck composition |
| OnAllyDeath | Variable | Synergizes with sacrifice/swarm |
| OnEnemyDeath | Variable | Rewards aggressive play |

---

*End of Design Document*
