# Card Game Engine Design Document

> **Version:** 1.3 (New Horizons Edition)
> **Last Updated:** 2026-01-17
> **Status:** Implementation Complete - 300 cards, 16 keywords

This document is the single source of truth for all game rules, parameters, and engine specifications.

---

## Table of Contents

1. [Game Overview](#1-game-overview)
2. [Core Parameters](#2-core-parameters)
3. [Game Flow](#3-game-flow)
4. [Board & Lanes](#4-board--lanes)
5. [Actions](#5-actions)
6. [Combat](#6-combat)
7. [Keywords](#7-keywords)
8. [Card Types](#8-card-types)
9. [Effects](#9-effects)
10. [Win Conditions](#10-win-conditions)
11. [AI Interface](#11-ai-interface)
12. [Appendix](#12-appendix)

---

## 1. Game Overview

### 1.1 Design Philosophy

This is a **deterministic, perfect-information card game** designed for AI research and human play. Think of it as **"Chess with Cards"** — both players have complete knowledge of the game state at all times, and the only randomness occurs during initial setup.

### 1.2 Core Design Goals

| Goal | Description |
|------|-------------|
| **AI Trainability** | Bounded action space, fast state cloning, clear reward signals |
| **Perfect Information** | No hidden cards, no random effects during gameplay |
| **Strategic Depth** | Lane-based positioning, keyword interactions, resource management |
| **Bounded Complexity** | Max ~50 legal actions per turn, games end by turn 30 |

### 1.3 What Makes This Game Unique

- **Open Information:** Both players see all cards in both hands AND both decks (in order)
- **Lane Combat:** Creatures attack specific board positions, not freely chosen targets
- **Action Points:** Limited actions per turn force meaningful decisions
- **Deterministic Play:** After initial setup, the game is fully deterministic

---

## 2. Core Parameters

All numeric constants in one place:

### 2.1 Player Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Starting Life | 30 | Both players |
| Maximum Life | 30 | Cannot heal above starting |
| Starting Hand Size | 4 | Cards drawn during setup |
| Maximum Hand Size | 10 | Excess cards are discarded (burned) |
| Action Points per Turn | 3 | Refreshed each turn |

### 2.2 Resource Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Starting Essence | 1 | Player 1 starts with 1, Player 2 starts with 1 |
| Essence Gain per Turn | +1 | Added to maximum at turn start |
| Maximum Essence | 10 | Cap on essence pool |

### 2.3 Board Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Creature Slots | 5 | Per player, numbered 1-5 (or 0-4 internally) |
| Support Slots | 2 | Per player |

### 2.4 Game Length Parameters

| Parameter | Default | Adjustable | Notes |
|-----------|---------|------------|-------|
| Deck Size (Starter) | 20 | Yes | For testing and starter decks |
| Deck Size (Standard) | 30 | Yes | For competitive play |
| Turn Limit | 30 | Yes | Total turns (not per player) |

### 2.5 Balance Formula

For creature stat budgeting:

```
Vanilla Stats = (Essence Cost × 2) + 1

Example:
  1-cost creature: 3 total stats (e.g., 2/1 or 1/2)
  3-cost creature: 7 total stats (e.g., 4/3 or 3/4)
  5-cost creature: 11 total stats (e.g., 5/6 or 6/5)

Keywords cost approximately:
  Rush:      1.0 stat points
  Ranged:    1.0 stat points
  Guard:     0.5 stat points
  Shield:    1.0 stat points
  Quick:     1.5 stat points
  Lethal:    2.0 stat points
  Lifesteal: 1.5 stat points
  Piercing:  0.5-1.0 stat points
```

---

## 3. Game Flow

### 3.1 Game Setup

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GAME SETUP                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. DECK PREPARATION                                                        │
│     • Both players submit their decks (20 or 30 cards)                     │
│     • Decks are arranged using the Fair Order Algorithm (see 3.2)          │
│     • Alternatively: Decks shuffled with shared seed (for random mode)     │
│                                                                             │
│  2. STARTING HANDS                                                          │
│     • Each player draws 4 cards from the top of their deck                 │
│                                                                             │
│  3. STARTING RESOURCES                                                      │
│     • Both players: 30 life, 1 max essence, 1 current essence              │
│     • Player 1: Goes first, but SKIPS first turn card draw                 │
│     • Player 2: Goes second, draws normally on their first turn            │
│                                                                             │
│  4. GAME BEGINS                                                             │
│     • Player 1 starts Turn 1                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Fair Deck Order Algorithm

> **Purpose:** Remove luck from initial draws while maintaining strategic deck building.

The Fair Order Algorithm ensures both players have playable opening hands and reasonable card distribution. This is an area for future refinement, but the initial approach:

**Option A: Curve-Based Sorting (Simple)**
```
1. Sort each player's deck by essence cost (ascending)
2. Interleave: take 1 card from each cost bracket in rotation
3. Result: Both players draw a mix of costs naturally

Example 20-card deck distribution:
  Cards 1-5:   Mix of 1-2 cost cards
  Cards 6-10:  Mix of 2-3 cost cards
  Cards 11-15: Mix of 3-4 cost cards
  Cards 16-20: Mix of 5+ cost cards
```

**Option B: Deterministic Shuffle (Seeded)**
```
1. Use a shared seed for both decks
2. Apply identical shuffle algorithm to both
3. Result: Reproducible games, but still "random" feel
```

**Option C: Mirror Setup (Symmetric)**
```
1. Both players use identical deck lists
2. Deck order is identical
3. Result: Pure skill test, no deck advantage
```

**Selected for v1:** Option B (Deterministic Shuffle) — simplest to implement, allows deck diversity, and produces reproducible games for AI training. The seed is stored in GameState for perfect reproducibility.

### 3.3 Turn Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TURN STRUCTURE                                       │
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
│    7. Trigger all "Start of Turn" effects                                  │
│                                                                             │
│  MAIN PHASE (Player Decisions)                                              │
│  ─────────────────────────────                                              │
│    While action_points > 0 AND player chooses to act:                      │
│      • Play a Card (costs 1 AP + essence cost)                             │
│      • Attack with a Creature (costs 1 AP)                                 │
│      • Activate an Ability (costs AP as specified)                         │
│      • End Turn (costs 0 AP, forfeits remaining AP)                        │
│                                                                             │
│  END OF TURN (Automatic)                                                    │
│  ──────────────────────────                                                 │
│    1. Trigger all "End of Turn" effects                                    │
│    2. Pass turn to opponent                                                │
│                                                                             │
│  NOTE: There is NO separate "combat phase" — attacks are individual        │
│        actions that can be interleaved with playing cards.                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.4 Turn Sequence Diagram

```
Turn 1 (Player 1)           Turn 2 (Player 2)           Turn 3 (Player 1)
─────────────────           ─────────────────           ─────────────────
Essence: 1                  Essence: 2                  Essence: 3
AP: 3                       AP: 3                       AP: 3
Draw: SKIP                  Draw: 1 card                Draw: 1 card
Hand: 4 cards               Hand: 5 cards               Hand: 5 cards
                                                        (assuming 1 played)
```

---

## 4. Board & Lanes

### 4.1 Board Layout

```
                         PLAYER TWO'S SIDE
    ┌─────────────────────────────────────────────────────┐
    │                   SUPPORT ZONE                       │
    │              [Slot A]     [Slot B]                  │
    ├─────────────────────────────────────────────────────┤
    │                   CREATURE ZONE                      │
    │     [1]      [2]      [3]      [4]      [5]        │
    │                                                      │
    └─────────────────────────────────────────────────────┘
                              │
                         COMBAT LANES
                              │
    ┌─────────────────────────────────────────────────────┐
    │                   CREATURE ZONE                      │
    │     [1]      [2]      [3]      [4]      [5]        │
    │                                                      │
    ├─────────────────────────────────────────────────────┤
    │                   SUPPORT ZONE                       │
    │              [Slot A]     [Slot B]                  │
    └─────────────────────────────────────────────────────┘
                         PLAYER ONE'S SIDE
```

### 4.2 Lane Adjacency Rules

Creatures can attack their own lane OR adjacent lanes:

```
YOUR SLOT    CAN ATTACK ENEMY SLOTS
────────────────────────────────────
   1         →  1, 2
   2         →  1, 2, 3
   3         →  2, 3, 4        (Center has most reach!)
   4         →  3, 4, 5
   5         →  4, 5
```

**Visual representation:**

```
Your Slot 1:    Your Slot 3:    Your Slot 5:
    ╲               │               ╱
     ╲           ╱  │  ╲           ╱
      ↘         ↙   ↓   ↘         ↙
    [1] [2]    [2] [3] [4]    [4] [5]
    Enemy      Enemy          Enemy
```

### 4.3 Face Attack Rules

A creature can attack the enemy player's face (life total) **only if its direct lane is empty**.

```
Your creature in Slot 3:
  • Enemy Slot 3 is EMPTY → Can attack face
  • Enemy Slot 3 has creature → CANNOT attack face (must attack creature)

Note: Adjacent slots do NOT block face attacks.
      Only the direct opposite slot matters.
```

### 4.4 Creature Placement

When playing a creature card:
- Player chooses which empty slot (1-5) to place it
- Once placed, creatures do NOT move (unless a card effect moves them)
- Slot choice is strategic — center slots have more attack reach but face more attackers

---

## 5. Actions

### 5.1 Action Types

| Action | AP Cost | Additional Cost | Description |
|--------|---------|-----------------|-------------|
| Play Card | 1 | Essence cost of card | Play a card from hand |
| Attack | 1 | None | Attack with one creature |
| Activate Ability | Varies | As specified | Use a creature's activated ability |
| End Turn | 0 | None | Pass, forfeit remaining AP |

### 5.2 Play Card Action

```
PLAY CARD
─────────
Cost: 1 AP + Card's Essence Cost

Requirements:
  • Card is in your hand
  • You have enough essence
  • You have at least 1 AP
  • (For creatures) An empty creature slot exists
  • (For supports) An empty support slot exists
  • (For targeted spells) A valid target exists

Resolution:
  1. Remove card from hand
  2. Deduct essence cost from current essence
  3. Deduct 1 AP
  4. Place card on board (creatures/supports) or resolve effect (spells)
  5. Trigger "On Play" effects
```

### 5.3 Attack Action

```
ATTACK
──────
Cost: 1 AP

Requirements:
  • Creature is in one of your slots
  • Creature is NOT exhausted
  • Creature does NOT have summoning sickness (unless Rush)
  • Creature has Attack > 0
  • A valid target exists (creature in range OR empty direct lane for face)

Resolution:
  1. Deduct 1 AP
  2. Mark attacker as exhausted
  3. Resolve combat (see Section 6)
```

### 5.4 End Turn Action

```
END TURN
────────
Cost: 0 AP

Effect:
  • Immediately ends your turn
  • All remaining AP is forfeited
  • Triggers "End of Turn" effects
  • Turn passes to opponent
```

### 5.5 Action Index Mapping (For AI)

Actions are mapped to a fixed-size index space for neural network output:

```
INDEX RANGE    ACTION TYPE
───────────────────────────────────────────────────────────
0              End Turn

1-50           Play Card from Hand
               = hand_index (0-9) × 5 + slot/target (0-4)
               [Note: Simplified — actual targeting is more complex]

51-80          Attack
               = attacker_slot (0-4) × 6 + target (0-5)
               where target 0 = Face, 1-5 = enemy slots

81-100         Activate Ability (reserved for future)

───────────────────────────────────────────────────────────
MAX_ACTIONS = 128 (power of 2 for efficient masking)
```

**Detailed Play Card Indexing:**

```
For creatures (need slot target):
  Base: 1
  Index = 1 + (hand_index × 5) + target_slot
  Range: 1-50 (10 hand positions × 5 slots)

For spells (need various targets):
  Base: 51
  Target types:
    0 = No target
    1-5 = Enemy creature slots
    6-10 = Friendly creature slots
    11 = Enemy face
    12 = Own face
  Index = 51 + (hand_index × 13) + target_type
  Range: 51-180

For supports (no target needed):
  Index = 181 + hand_index
  Range: 181-190
```

**Actual implementation may vary — the key requirement is:**
- Fixed-size action space
- Deterministic mapping between Action enum and index
- Ability to generate legal action mask efficiently

---

## 6. Combat

### 6.1 Combat Overview

Combat in this game is:
- **Attacker-initiated:** Only the active player attacks
- **Lane-constrained:** Attackers can only hit adjacent lanes
- **Simultaneous damage:** Both creatures deal damage at the same time (unless Quick)
- **Persistent damage:** Damage stays until healed or creature dies

### 6.2 Attack Resolution Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ATTACK RESOLUTION                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. DECLARE ATTACK                                                          │
│     • Attacker chooses creature and target                                 │
│     • Verify target is valid (in range, Guard rules)                       │
│                                                                             │
│  2. TRIGGER "ON ATTACK" EFFECTS                                            │
│     • Attacker's OnAttack abilities trigger                                │
│                                                                             │
│  3. DETERMINE COMBAT TYPE                                                   │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │ IF target is FACE:                                              │    │
│     │   → Deal attacker's Attack as damage to enemy player           │    │
│     │   → Apply Lifesteal if attacker has it                         │    │
│     │   → DONE                                                        │    │
│     │                                                                 │    │
│     │ IF target is CREATURE:                                          │    │
│     │   → Continue to step 4                                          │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  4. CHECK QUICK KEYWORD                                                     │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │ IF attacker has Quick AND defender does NOT have Quick:         │    │
│     │   → Attacker deals damage first                                 │    │
│     │   → If defender dies, defender deals NO damage back             │    │
│     │   → If defender survives, defender deals damage                 │    │
│     │                                                                 │    │
│     │ IF defender has Quick AND attacker does NOT have Quick:         │    │
│     │   → Defender deals damage first                                 │    │
│     │   → If attacker dies, attacker deals NO damage                  │    │
│     │   → If attacker survives, attacker deals damage                 │    │
│     │                                                                 │    │
│     │ IF both have Quick OR neither has Quick:                        │    │
│     │   → Simultaneous damage (both deal damage at once)              │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  5. DEAL COMBAT DAMAGE                                                      │
│     For each creature dealing damage:                                      │
│       a. Check if target has Shield                                        │
│          → If yes: Remove Shield, deal 0 damage, skip to (f)              │
│       b. Apply damage to target's health                                   │
│       c. Check Lethal keyword                                              │
│          → If attacker has Lethal AND damage > 0: Target dies             │
│       d. Trigger "On Damage Dealt" effects                                 │
│       e. Trigger "On Damage Taken" effects                                 │
│       f. Continue                                                          │
│                                                                             │
│  6. CHECK DEATHS                                                            │
│     • Any creature with health ≤ 0 dies                                    │
│     • Trigger "On Death" effects                                           │
│     • Remove dead creatures from board                                     │
│                                                                             │
│  7. APPLY PIERCING (if applicable)                                         │
│     IF attacker has Piercing AND defender DIED:                            │
│       → Calculate excess damage = Attacker's Attack - Defender's Health   │
│       → Deal excess damage to enemy face                                   │
│     (Piercing does NOT trigger if defender survived)                       │
│                                                                             │
│  8. APPLY LIFESTEAL (if applicable)                                        │
│     IF attacker has Lifesteal AND damage_actually_dealt > 0:               │
│       → Heal attacker's controller for damage_actually_dealt               │
│     IMPORTANT: Lifesteal does NOT trigger if:                              │
│       • Damage was absorbed by Shield (damage_actually_dealt = 0)         │
│       • Attacker has 0 Attack                                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Guard Keyword Rules

```
GUARD RULES
───────────
When attacking, if ANY creature with Guard is in your attack range,
you MUST attack a Guard creature. You cannot attack non-Guard creatures
or face while a Guard is in range.

Example:
  Your creature in Slot 2 can attack enemy slots 1, 2, 3.
  Enemy has: Slot 1 (Guard), Slot 2 (no Guard), Slot 3 (no Guard)

  Legal targets: Only Slot 1 (the Guard)
  Illegal: Slot 2, Slot 3, Face

Multiple Guards in range:
  If slots 1 AND 3 both have Guard, you can choose either one.
```

### 6.4 Ranged Keyword Rules

```
RANGED RULES
────────────
Creatures with Ranged can attack ANY enemy creature slot (1-5),
ignoring normal lane adjacency restrictions.

HOWEVER:
  • Ranged creatures still respect Guard (must attack Guard if any exists)
  • Ranged creatures can only attack face if their direct lane is empty
    (Ranged does NOT bypass the face-attack lane rule)

Example:
  Your Ranged creature in Slot 1.
  Enemy has: Slot 3 (creature), Slot 5 (creature), Slot 1 (empty)

  Legal targets: Slot 3, Slot 5, or Face (since direct lane 1 is empty)
```

### 6.5 Summoning Sickness

```
SUMMONING SICKNESS
──────────────────
Creatures cannot attack on the turn they are played.

Exception: Creatures with the Rush keyword ignore summoning sickness
           and can attack immediately.

Implementation:
  • Track turn_played on each creature
  • Creature can attack if: turn_played < current_turn OR has_rush
```

### 6.6 Exhaustion

```
EXHAUSTION
──────────
After a creature attacks, it becomes "exhausted" and cannot attack again
this turn.

Exhaustion clears at the start of your turn.

Note: Playing a creature does NOT exhaust it (but summoning sickness
      prevents attacking anyway).
```

---

## 7. Keywords

### 7.1 Keyword Definitions

| Keyword | Effect | Stat Cost |
|---------|--------|-----------|
| **Rush** | Can attack the turn it's played (ignores summoning sickness) | ~1.0 |
| **Ranged** | Can attack any enemy creature slot (not just adjacent) | ~1.0 |
| **Guard** | Enemies in attack range MUST attack this creature first | ~0.5 |
| **Shield** | The first time this would take damage, prevent it and lose Shield | ~1.0 |
| **Quick** | Deals combat damage before creatures without Quick | ~1.5 |
| **Lethal** | Any damage dealt by this creature destroys the target | ~2.0 |
| **Lifesteal** | When this deals damage, heal your hero for that amount | ~1.5 |
| **Piercing** | When this kills a creature, excess damage hits enemy face | ~0.5-1.0 |
| **Ephemeral** | Dies at end of owner's turn (triggers OnDeath effects) | ~-1.5 |
| **Regenerate** | Heals 2 HP at start of owner's turn | ~1.0 |
| **Stealth** | Cannot be targeted by enemy attacks/spells; breaks when attacking | ~1.5 |
| **Charge** | +2 attack damage when attacking | ~1.0 |
| **Frenzy** | +1 attack after each attack this turn | ~1.0 |
| **Volatile** | Deal 2 damage to all enemy creatures when this dies | ~0.5 |
| **Fortify** | Take 1 less damage from all sources (minimum 1) | ~1.0 |
| **Ward** | First spell/ability targeting this has no effect; Ward is removed | ~1.0 |

### 7.2 Keyword Interaction Matrix

```
┌───────────┬─────────────────────────────────────────────────────────────────┐
│ SCENARIO  │ RESOLUTION                                                      │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Shield    │ Shield absorbs damage → 0 damage dealt                         │
│ vs        │ Lethal requires damage > 0 to trigger                          │
│ Lethal    │ RESULT: Target survives, loses Shield                          │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Shield    │ Shield absorbs damage → 0 damage dealt                         │
│ vs        │ Lifesteal heals for damage dealt (0)                           │
│ Lifesteal │ RESULT: No healing occurs                                      │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Piercing  │ Piercing only triggers if defender DIES                        │
│ vs        │ If Guard survives, no piercing damage to face                  │
│ Guard     │ RESULT: Must kill Guard to pierce through                      │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Quick     │ Both have same speed → Simultaneous damage                     │
│ vs        │ RESULT: Both creatures deal damage at the same time            │
│ Quick     │                                                                 │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Quick     │ Quick creature deals damage first                              │
│ +         │ If defender dies, Quick+Lethal creature survives unscathed    │
│ Lethal    │ RESULT: Extremely powerful combination                         │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Ranged    │ Ranged ignores lane restrictions for creature targeting        │
│ vs        │ Ranged does NOT ignore Guard — must still attack Guards       │
│ Guard     │ RESULT: Ranged can pick WHICH Guard to attack if multiple     │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Lethal    │ Lethal requires damage > 0 to trigger                          │
│ with      │ A creature with 0 Attack cannot trigger Lethal                 │
│ 0 Attack  │ RESULT: No kill effect (0 damage = no Lethal)                  │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Lifesteal │ Lifesteal heals for damage dealt (attacker's Attack value)    │
│ +         │ Piercing deals excess to face                                  │
│ Piercing  │ RESULT: Heal for Attack value, pierce excess to face          │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Stealth   │ Stealth masks Guard until broken                               │
│ +         │ A stealthed Guard cannot be targeted                           │
│ Guard     │ RESULT: Stealth takes priority until creature attacks          │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Stealth   │ Stealth only blocks ENEMY targeting                            │
│ Targeting │ Friendly spells/abilities can target your stealthed creatures │
│           │ RESULT: Can buff/heal your own stealthed creatures             │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Charge    │ Charge grants +2 attack damage when attacking                  │
│ +         │ Bonus applies to both creature and face attacks                │
│ Piercing  │ RESULT: Piercing excess calculated with Charge bonus included  │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Ephemeral │ Ephemeral death triggers OnDeath effects                       │
│ +         │ Death occurs at end of owner's turn, not immediately           │
│ OnDeath   │ RESULT: Useful for "suicide bomber" style creatures            │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           │                                                                 │
│ Ephemeral │ Ephemeral creature CAN attack (if it has Rush)                 │
│ +         │ Gets full turn of action before dying                          │
│ Rush      │ RESULT: Strong burst damage, no board presence next turn       │
│           │                                                                 │
└───────────┴─────────────────────────────────────────────────────────────────┘
```

### 7.3 Keyword Implementation (Bitfield)

```rust
// Keywords packed into a u16 for efficiency (supports up to 16 keywords)
pub struct Keywords(u16);

impl Keywords {
    // Original 8 keywords (bits 0-7)
    pub const RUSH: u16      = 0x0001;  // bit 0
    pub const RANGED: u16    = 0x0002;  // bit 1
    pub const PIERCING: u16  = 0x0004;  // bit 2
    pub const GUARD: u16     = 0x0008;  // bit 3
    pub const LIFESTEAL: u16 = 0x0010;  // bit 4
    pub const LETHAL: u16    = 0x0020;  // bit 5
    pub const SHIELD: u16    = 0x0040;  // bit 6
    pub const QUICK: u16     = 0x0080;  // bit 7

    // Phase 1.5 keywords (bits 8-11)
    pub const EPHEMERAL: u16   = 0x0100;  // bit 8
    pub const REGENERATE: u16  = 0x0200;  // bit 9
    pub const STEALTH: u16     = 0x0400;  // bit 10
    pub const CHARGE: u16      = 0x0800;  // bit 11

    // Phase 4 keywords (bits 12-15)
    pub const FRENZY: u16    = 0x1000;  // bit 12
    pub const VOLATILE: u16  = 0x2000;  // bit 13
    pub const FORTIFY: u16   = 0x4000;  // bit 14
    pub const WARD: u16      = 0x8000;  // bit 15
}

// Check with single bitwise AND: keywords.0 & Keywords::RUSH != 0
// Combine with bitwise OR: keywords.0 | Keywords::RUSH
```

---

## 8. Card Types

### 8.1 Creatures

```
CREATURE CARD
─────────────
Properties:
  • Name: Display name
  • Cost: Essence cost to play (0-10)
  • Attack: Damage dealt in combat (0-15)
  • Health: Damage required to kill (1-15)
  • Keywords: Set of keyword flags
  • Abilities: List of triggered/activated abilities
  • Tags: Creature types (Soldier, Beast, Mage, etc.)

Behavior:
  • Placed in a creature slot when played
  • Can attack (with restrictions) once per turn
  • Removed when health reaches 0
  • Affected by support passive effects
```

### 8.2 Spells

```
SPELL CARD
──────────
Properties:
  • Name: Display name
  • Cost: Essence cost to play (0-10)
  • Effects: List of effects to apply
  • Targeting: What the spell can target

Behavior:
  • Resolved immediately when played
  • Goes to discard (removed from game) after resolution
  • Does NOT persist on board

Targeting Types:
  • NoTarget: Effect applies automatically (e.g., "Draw 2 cards")
  • TargetCreature: Must select a creature (with optional filters)
  • TargetPlayer: Must select a player
  • TargetAny: Can target creature or player
```

### 8.3 Supports

```
SUPPORT CARD
────────────
Properties:
  • Name: Display name
  • Cost: Essence cost to play (0-10)
  • Durability: Turns until support expires (1-10)
  • Passive Effects: Ongoing effects while in play
  • Triggered Effects: Effects that trigger on conditions

Behavior:
  • Placed in a support slot when played
  • Durability decreases by 1 at start of owner's turn
  • Removed when durability reaches 0
  • Passive effects apply to all friendly creatures continuously

Example Passive Effects:
  • "Your creatures have +1 Attack"
  • "Your creatures have Rush"
  • "Your creatures have +2 Health"
```

---

## 9. Effects

### 9.1 Trigger Types

| Trigger | When It Fires |
|---------|---------------|
| **OnPlay** | When this card is played from hand |
| **OnAttack** | When this creature declares an attack |
| **OnDealDamage** | When this creature deals damage |
| **OnTakeDamage** | When this creature takes damage |
| **OnKill** | When this creature kills another creature |
| **OnDeath** | When this creature dies |
| **OnAllyPlayed** | When another friendly creature is played |
| **OnAllyDeath** | When another friendly creature dies |
| **StartOfTurn** | At the start of your turn |
| **EndOfTurn** | At the end of your turn |

### 9.2 Effect Types

```
DAMAGE EFFECTS
  • Deal X damage to target creature
  • Deal X damage to target player
  • Deal X damage to all enemy creatures
  • Deal X damage to all creatures

HEALING EFFECTS
  • Restore X health to target creature
  • Restore X health to target player
  • Restore X health to all friendly creatures

STAT MODIFICATION
  • Give target creature +X/+Y (Attack/Health)
  • Give target creature +X Attack
  • Give target creature +Y Health
  • Set target creature's Attack to X
  • Set target creature's Health to Y

KEYWORD MANIPULATION
  • Give target creature [Keyword]
  • Remove [Keyword] from target creature
  • Silence target creature (remove all keywords and abilities)

CARD FLOW
  • Draw X cards
  • Discard X cards (random or chosen)

CREATURE MANIPULATION
  • Destroy target creature
  • Bounce target creature (return to owner's hand)
  • Summon a [Token] creature
  • Move target creature to another slot

RESOURCE MANIPULATION
  • Gain X essence (temporary, this turn)
  • Gain X action points
  • Refresh target creature (remove exhausted)
```

### 9.3 Effect Resolution Order

Effects use a **queue-based resolution system**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         EFFECT QUEUE SYSTEM                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. When an effect is created, it's added to the END of the queue         │
│                                                                             │
│  2. Effects are processed one at a time from the FRONT of the queue       │
│                                                                             │
│  3. If processing an effect creates new effects (triggers), they go       │
│     to the END of the queue                                                │
│                                                                             │
│  4. After an effect resolves, check for creature deaths                   │
│     • Death triggers are queued                                            │
│     • Dead creatures are removed                                           │
│                                                                             │
│  5. Continue until queue is empty                                          │
│                                                                             │
│  EXAMPLE:                                                                   │
│    Play "Warlord Titan" (OnPlay: Deal 3 to all enemies)                   │
│    Queue: [Titan OnPlay]                                                   │
│    Process Titan OnPlay → Queue: [Dmg to Slot1, Dmg to Slot2, Dmg to Slot3]│
│    Process Dmg to Slot1 → Creature at Slot1 takes 3, triggers OnTakeDmg   │
│    Queue: [Dmg to Slot2, Dmg to Slot3, OnTakeDmg effect]                  │
│    ... continue until queue empty ...                                      │
│    Finally: Remove all creatures with health ≤ 0                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.4 Targeting Rules

```
TARGET VALIDATION
─────────────────
When a card or ability requires a target:

1. Determine valid target set based on targeting rule:
   • TargetCreature { owner: Ally } → Only your creatures
   • TargetCreature { owner: Enemy } → Only enemy creatures
   • TargetCreature { max_health: 4 } → Creatures with current health ≤ 4
   • TargetAny → Any creature or player

2. If no valid targets exist, the card CANNOT be played
   (Exception: NoTarget spells can always be played)

3. Player must select a target from the valid set

4. If target becomes invalid before resolution (e.g., creature dies),
   the effect fizzles (does nothing)
```

### 9.5 Creature Filters (Phase 4)

Effects can be filtered to only affect creatures matching specific criteria:

```
FILTER FIELDS
─────────────
  • max_health: u8    → Target must have current health ≤ value
  • min_health: u8    → Target must have current health ≥ value
  • has_keyword: u16  → Target must have the keyword (bit value)
  • lacks_keyword: u16 → Target must NOT have the keyword

USAGE IN YAML
─────────────
  effects:
    - type: damage
      amount: 3
      filter:
        max_health: 4
        has_keyword: 1  # Guard (bit value)

BEHAVIOR
────────
  • For targeted effects: Filter is applied during legal action generation
  • For AoE effects: Filter is applied during effect resolution
  • Filtering respects creature's CURRENT state, not base stats
```

### 9.6 Conditional Triggers (Phase 4)

Spells and abilities can have bonus effects that trigger based on outcomes:

```
CONDITIONS
──────────
  • target_died    → Primary target was destroyed by this effect

YAML SYNTAX
───────────
  effects:
    - type: damage
      amount: 3
  conditional_effects:
    - condition: target_died
      effects:
        - type: draw
          count: 1

BEHAVIOR
────────
  1. Primary effects resolve first
  2. Engine tracks results (did target die?)
  3. If condition is met, bonus effects are queued
  4. Bonus effects resolve after primary effects
```

### 9.7 Bounce Effect (Phase 4)

Return a creature from the battlefield to its owner's hand:

```
BOUNCE BEHAVIOR
───────────────
  1. Creature is removed from the board
  2. Original card is added to owner's hand
  3. If hand is full (20 cards), card is discarded
  4. All buffs, damage, and applied effects are removed
     (creature returns as a fresh card)

YAML SYNTAX
───────────
  # Single target bounce
  effects:
    - type: bounce

  # Mass bounce with filter
  effects:
    - type: bounce
      filter:
        max_health: 3

USE CASES
─────────
  • Tempo-based control (remove blocker, enemy must replay)
  • Self-bounce for value (re-trigger OnPlay effects)
  • Removal for buffed creatures (reset to base stats)
```

---

## 10. Win Conditions

### 10.1 Primary Win Condition

```
LIFE TOTAL VICTORY
──────────────────
A player wins immediately when their opponent's life total reaches 0 or below.

If both players reach 0 or below simultaneously:
  → Game is a DRAW
```

### 10.2 Turn Limit Tiebreaker (Mode A)

```
TURN LIMIT RESOLUTION
─────────────────────
If the game reaches turn 30 without a winner:

1. Compare life totals
2. Player with higher life total wins
3. If life totals are equal → DRAW

Win Reason: "Turn Limit - Higher Life"
```

### 10.3 Victory Points (Mode C - Future)

```
VICTORY POINTS (Designed for future implementation)
───────────────────────────────────────────────────
Track total damage dealt by each player (cumulative, even through healing).

Alternative win: First player to deal 50 total damage wins.

Implementation note:
  • Track `total_damage_dealt: u16` per player
  • Increment whenever player deals damage to enemy (face or creatures)
  • Check threshold after each damage event
```

### 10.4 Game Result Encoding

```rust
pub enum GameResult {
    Win { winner: PlayerId, reason: WinReason },
    Draw,
}

pub enum WinReason {
    LifeReachedZero,
    TurnLimitHigherLife,
    VictoryPointsReached,  // Future
    Concession,
}
```

---

## 11. AI Interface

### 11.1 State Tensor Specification

The game state must be convertible to a fixed-size tensor for neural network input.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         STATE TENSOR LAYOUT                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  GLOBAL STATE (10 floats)                                                   │
│  ─────────────────────────                                                  │
│    [0]  current_turn / 30.0           (normalized)                         │
│    [1]  active_player                 (0.0 or 1.0)                         │
│    [2]  active_player_ap / 3.0        (normalized)                         │
│    [3]  active_player_essence / 10.0  (normalized)                         │
│    [4]  active_player_max_essence / 10.0                                   │
│    [5]  active_player_life / 30.0                                          │
│    [6]  opponent_essence / 10.0                                            │
│    [7]  opponent_max_essence / 10.0                                        │
│    [8]  opponent_life / 30.0                                               │
│    [9]  reserved                                                           │
│                                                                             │
│  CREATURE SLOTS (5 slots × 2 players × 12 features = 120 floats)           │
│  ────────────────────────────────────────────────────────────────          │
│  For each of 10 slots (5 per player):                                      │
│    [0]  occupied                      (0.0 or 1.0)                         │
│    [1]  attack / 15.0                 (normalized)                         │
│    [2]  current_health / 15.0         (normalized)                         │
│    [3]  max_health / 15.0             (normalized)                         │
│    [4]  can_attack                    (0.0 or 1.0)                         │
│    [5]  has_rush                      (0.0 or 1.0)                         │
│    [6]  has_ranged                    (0.0 or 1.0)                         │
│    [7]  has_guard                     (0.0 or 1.0)                         │
│    [8]  has_shield                    (0.0 or 1.0)                         │
│    [9]  has_quick                     (0.0 or 1.0)                         │
│    [10] has_lethal                    (0.0 or 1.0)                         │
│    [11] has_lifesteal + has_piercing  (packed: 0.0, 0.5, 1.0)             │
│                                                                             │
│  SUPPORT SLOTS (2 slots × 2 players × 4 features = 16 floats)              │
│  ─────────────────────────────────────────────────────────────             │
│  For each of 4 slots (2 per player):                                       │
│    [0]  occupied                      (0.0 or 1.0)                         │
│    [1]  card_id / MAX_CARDS           (normalized card identifier)         │
│    [2]  durability / 10.0             (normalized)                         │
│    [3]  reserved                                                           │
│                                                                             │
│  HAND ENCODING (10 cards × 2 players × 3 features = 60 floats)             │
│  ──────────────────────────────────────────────────────────────            │
│  For each of 20 hand slots (10 per player):                                │
│    [0]  occupied                      (0.0 or 1.0)                         │
│    [1]  card_id / MAX_CARDS           (normalized)                         │
│    [2]  essence_cost / 10.0           (normalized)                         │
│                                                                             │
│  DECK ENCODING (30 cards × 2 players × 2 features = 120 floats)            │
│  ───────────────────────────────────────────────────────────────           │
│  For each of 60 deck slots (30 per player):                                │
│    [0]  occupied                      (0.0 or 1.0)                         │
│    [1]  card_id / MAX_CARDS           (normalized)                         │
│                                                                             │
│  TOTAL: ~326 floats (round up to 512 for padding)                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 11.1.1 Card Embedding Approach (Recommended)

Instead of raw card IDs, use learned embeddings for better generalization:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CARD EMBEDDING SYSTEM                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  CONCEPT                                                                    │
│  ───────                                                                    │
│  Each card_id → lookup in embedding table → 16-dimensional vector          │
│                                                                             │
│  EMBEDDING TABLE                                                            │
│  ───────────────                                                            │
│  Shape: [MAX_CARDS × EMBEDDING_DIM] = [256 × 16]                           │
│  Initialized: Random small values (e.g., uniform [-0.1, 0.1])              │
│  Updated: During RL training via backpropagation                           │
│                                                                             │
│  HOW IT WORKS                                                               │
│  ────────────                                                               │
│  1. Rust engine outputs raw card IDs in state tensor                       │
│  2. Python training code has nn.Embedding layer                            │
│  3. Card IDs → embedding lookup → dense vectors → rest of network          │
│  4. Training updates both network AND embedding table                      │
│                                                                             │
│  WHAT THE NETWORK LEARNS                                                    │
│  ───────────────────────                                                    │
│  • Similar cards get similar embeddings (clustering)                       │
│  • "Rush creatures" cluster together                                       │
│  • "Removal spells" cluster together                                       │
│  • Cost/stat relationships encoded implicitly                              │
│                                                                             │
│  BENEFITS FOR LARGE CARD POOLS                                              │
│  ─────────────────────────────                                              │
│  • New cards can be added without retraining from scratch                  │
│  • Transfer learning: similar cards help learn new ones                    │
│  • Enables deck archetypes (Aggro, Control, etc.) to emerge naturally     │
│                                                                             │
│  STATE TENSOR ADJUSTMENT                                                    │
│  ───────────────────────                                                    │
│  With embeddings, the Rust engine still outputs card IDs as integers.      │
│  The embedding lookup happens in the neural network's first layer.         │
│  No change to Rust code needed — just output card IDs.                     │
│                                                                             │
│  EMBEDDING DIM RECOMMENDATIONS                                              │
│  ─────────────────────────────                                              │
│  • Small card pool (<100):   8-dimensional                                 │
│  • Medium card pool (<500):  16-dimensional                                │
│  • Large card pool (500+):   32-dimensional                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Implementation Note:** The Rust engine outputs raw card IDs. The Python training
code wraps them with `nn.Embedding`. This separation keeps the engine simple and fast.

### 11.2 Action Space Specification

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ACTION SPACE                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Fixed size: 256 actions (MAX_ACTIONS)                                      │
│                                                                             │
│  INDEX    ACTION                                                            │
│  ─────    ──────                                                            │
│  0        End Turn                                                          │
│                                                                             │
│  1-50     Play Card (creature to slot)                                     │
│           = 1 + hand_idx * 5 + slot_idx                                    │
│           hand_idx: 0-9, slot_idx: 0-4                                     │
│                                                                             │
│  51-80    Attack                                                            │
│           = 51 + attacker_slot * 6 + target                                │
│           attacker_slot: 0-4                                               │
│           target: 0=face, 1-5=enemy slots                                  │
│                                                                             │
│  81-180   Play Spell (with target)                                         │
│           = 81 + hand_idx * 10 + target_type                               │
│           target_type: varies by spell                                     │
│                                                                             │
│  181-200  Play Support                                                      │
│           = 181 + hand_idx                                                 │
│                                                                             │
│  201-255  Reserved for abilities                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 11.3 Legal Action Mask

```rust
/// Returns a boolean mask indicating which actions are legal
pub fn get_legal_action_mask(state: &GameState) -> [bool; 256] {
    let mut mask = [false; 256];

    // End Turn is always legal (if game not over)
    if state.result.is_none() {
        mask[0] = true;
    }

    // For each legal action, set mask[action.to_index()] = true
    for action in get_legal_actions(state) {
        mask[action.to_index()] = true;
    }

    mask
}
```

### 11.4 Required Engine API

```rust
/// Core trait that AI agents interact with
pub trait GameEngine {
    /// Create a new game with given decks and random seed
    fn new_game(
        &self,
        deck_one: &[CardId],
        deck_two: &[CardId],
        seed: u64,
    ) -> GameState;

    /// Get all legal actions for current player
    fn get_legal_actions(&self, state: &GameState) -> Vec<Action>;

    /// Get legal actions as a bitmask (for neural networks)
    fn get_legal_action_mask(&self, state: &GameState) -> [bool; MAX_ACTIONS];

    /// Apply an action, mutating the game state
    fn apply_action(&mut self, state: &mut GameState, action: Action);

    /// Clone the game state (must be fast for MCTS!)
    fn clone_state(&self, state: &GameState) -> GameState;

    /// Check if game is over
    fn is_terminal(&self, state: &GameState) -> bool;

    /// Get game result (if terminal)
    fn get_result(&self, state: &GameState) -> Option<GameResult>;

    /// Convert state to tensor for neural network
    fn state_to_tensor(&self, state: &GameState) -> StateTensor;

    /// Get reward for terminal state
    fn get_reward(&self, state: &GameState, player: PlayerId) -> f32;
}
```

### 11.5 Reward Signal

```
REWARD FUNCTION
───────────────
For terminal states:

  Win:  +1.0
  Loss: -1.0
  Draw:  0.0

For non-terminal states:
  0.0 (no intermediate reward)

This sparse reward is standard for game-playing agents.
For faster training, consider shaped rewards (future work):
  • +0.01 per damage dealt
  • +0.05 per creature killed
  • -0.01 per own creature lost
```

---

## 12. Appendix

### 12.1 Worked Example: Complete Turn

```
═══════════════════════════════════════════════════════════════════════════════
                           WORKED EXAMPLE TURN
═══════════════════════════════════════════════════════════════════════════════

INITIAL STATE (Start of Player 1's Turn 5):
───────────────────────────────────────────

Player 1:                          Player 2:
  Life: 26                           Life: 22
  Essence: 5/5                       Essence: 4/5 (not their turn)
  AP: 3                              AP: 0
  Hand: 4 cards                      Hand: 5 cards

Board:
                    PLAYER 2
    Support: [War Drums (D:2)]  [Empty]
    Creatures:
        [1]         [2]         [3]         [4]         [5]
        Empty       2/3         Empty       4/2         Empty
                    Guard

                    PLAYER 1
    Creatures:
        [1]         [2]         [3]         [4]         [5]
        3/2         Empty       4/4         2/1         Empty
        Piercing                            Rush

    Support: [Empty]  [Empty]

Player 1's Hand:
  [0] Centaur Charger (3-cost, 3/3 Rush)
  [1] Lightning Bolt (3-cost, Deal 4 damage)
  [2] Village Guard (1-cost, 1/2)
  [3] Execute (2-cost, Destroy creature with ≤4 health)

───────────────────────────────────────────────────────────────────────────────

ACTION 1: Attack with Slot 1 creature (3/2 Piercing) → Enemy Slot 2 (2/3 Guard)
───────────────────────────────────────────────────────────────────────────────

Why this target?
  • Slot 1 can attack enemy slots 1 or 2
  • Enemy slot 2 has Guard → MUST attack it

Combat Resolution:
  • Simultaneous damage (neither has Quick)
  • Player 1's creature deals 3 damage → Enemy 2/3 becomes 2/0 → DIES
  • Enemy creature deals 2 damage → Player 1's 3/2 becomes 3/0 → DIES
  • Piercing check: Enemy died, excess damage = 3 - 3 = 0 → No pierce

Result:
  • Player 1: Slot 1 now empty, AP = 2
  • Player 2: Slot 2 now empty

───────────────────────────────────────────────────────────────────────────────

ACTION 2: Play Centaur Charger (3/3 Rush) → Slot 2
───────────────────────────────────────────────────────────────────────────────

Cost: 1 AP + 3 Essence
  • AP: 2 → 1
  • Essence: 5 → 2

Creature enters Slot 2 with Rush (can attack immediately)

War Drums effect: +1 Attack to enemy creatures
  • Wait, War Drums is PLAYER 2's support
  • It gives +1 Attack to PLAYER 2's creatures, not Player 1's
  • So Centaur Charger is still 3/3

Result:
  • Player 1: Slot 2 has 3/3 Rush, AP = 1, Essence = 2

───────────────────────────────────────────────────────────────────────────────

ACTION 3: Attack with Slot 2 creature (3/3 Rush) → Enemy Face
───────────────────────────────────────────────────────────────────────────────

Valid because:
  • Slot 2 can attack enemy slots 1, 2, or 3
  • Enemy slot 2 (direct lane) is empty → Can attack face
  • No Guards in range (slots 1, 2, 3 all empty or no Guard)

Resolution:
  • Deal 3 damage to Player 2's face
  • Player 2 life: 22 → 19

Result:
  • Player 1: AP = 0, Slot 2 creature now exhausted

───────────────────────────────────────────────────────────────────────────────

TURN ENDS (0 AP remaining or player passes)
───────────────────────────────────────────────────────────────────────────────

Final State:

Player 1:                          Player 2:
  Life: 26                           Life: 19
  Essence: 2/5                       Essence: 4/5
  AP: 0                              AP: 0
  Hand: 3 cards                      Hand: 5 cards

Board:
                    PLAYER 2
    Support: [War Drums (D:2)]  [Empty]
    Creatures:
        [1]         [2]         [3]         [4]         [5]
        Empty       Empty       Empty       4/2         Empty
                                            (+1 Atk from War Drums = 5/2)

                    PLAYER 1
    Creatures:
        [1]         [2]         [3]         [4]         [5]
        Empty       3/3         4/4         2/1         Empty
                    (exh)                   Rush

═══════════════════════════════════════════════════════════════════════════════
```

### 12.2 Card Data Loading Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    YAML CARD DATA SYSTEM                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  FILE STRUCTURE                                                             │
│  ──────────────                                                             │
│  data/                                                                      │
│    └── cards/                                                               │
│        └── core_set/                                                        │
│            ├── argentum.yaml     (75 cards - IDs 1000-1074)                │
│            ├── symbiote.yaml     (75 cards - IDs 2000-2074)                │
│            ├── obsidion.yaml     (75 cards - IDs 3000-3074)                │
│            └── neutral.yaml      (75 cards - IDs 4000-4074)                │
│                                                                             │
│  LOADING PROCESS                                                            │
│  ───────────────                                                            │
│  1. Engine startup calls CardDatabase::load("data/cards/")                 │
│  2. All YAML files in sets/ are parsed                                     │
│  3. Cards are validated against schema                                     │
│  4. CardDatabase struct is built with all cards                            │
│  5. Wrapped in Arc<> for zero-cost sharing                                 │
│                                                                             │
│  PERFORMANCE                                                                │
│  ───────────                                                                │
│  • Parsing: ~1-10ms (one time at startup)                                  │
│  • Card lookup: O(1) array index (during gameplay)                         │
│  • Memory: ~100 bytes per card definition                                  │
│  • Current: 300 cards ≈ 30KB memory (trivial)                              │
│                                                                             │
│  BENEFITS                                                                   │
│  ────────                                                                   │
│  • Edit cards without recompiling                                          │
│  • Easy to add new sets/expansions                                         │
│  • Human-readable format for designers                                     │
│  • Version control friendly (diff-able)                                    │
│  • Can validate cards against design rules                                 │
│                                                                             │
│  RUST IMPLEMENTATION                                                        │
│  ───────────────────                                                        │
│  use serde::{Deserialize, Serialize};                                      │
│  use serde_yaml;                                                           │
│                                                                             │
│  impl CardDatabase {                                                        │
│      pub fn load(path: &str) -> Result<Self, CardLoadError> {              │
│          let mut cards = Vec::new();                                       │
│          for entry in fs::read_dir(path)? {                                │
│              let yaml = fs::read_to_string(entry.path())?;                 │
│              let set: CardSet = serde_yaml::from_str(&yaml)?;              │
│              cards.extend(set.cards);                                      │
│          }                                                                  │
│          cards.sort_by_key(|c| c.id);  // Ensure ID order                  │
│          Ok(Self { cards: Arc::new(cards) })                               │
│      }                                                                      │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 12.3 Card Data Format

```yaml
# Example card definitions (for data-driven loading)

creatures:
  - id: 1
    name: "Eager Recruit"
    cost: 1
    attack: 2
    health: 1
    keywords: []
    abilities: []
    tags: [Soldier]

  - id: 3
    name: "Nimble Scout"
    cost: 1
    attack: 1
    health: 1
    keywords: [Rush]
    abilities: []
    tags: [Soldier]

  - id: 18
    name: "Ambush Predator"
    cost: 3
    attack: 2
    health: 2
    keywords: [Rush, Lethal]
    abilities: []
    tags: [Beast]

  - id: 23
    name: "Storm Mage"
    cost: 4
    attack: 3
    health: 3
    keywords: []
    abilities:
      - trigger: OnPlay
        target: TargetCreature
        effects:
          - type: Damage
            amount: 2
    tags: [Mage]

spells:
  - id: 32
    name: "Quick Strike"
    cost: 1
    targeting: TargetCreature
    effects:
      - type: Damage
        amount: 2

  - id: 38
    name: "Obliterate"
    cost: 5
    targeting: TargetCreature
    effects:
      - type: Destroy

  # Phase 4: Spell with filter
  - id: 1025
    name: "Execute"
    cost: 2
    targeting: TargetEnemyCreature
    effects:
      - type: destroy
        filter:
          max_health: 4

  # Phase 4: Spell with conditional trigger
  - id: 3030
    name: "Soul Harvest"
    cost: 3
    targeting: TargetEnemyCreature
    effects:
      - type: damage
        amount: 3
    conditional_effects:
      - condition: target_died
        effects:
          - type: heal
            amount: 3

  # Phase 4: Bounce spell
  - id: 4015
    name: "Temporal Shift"
    cost: 3
    targeting: TargetEnemyCreature
    effects:
      - type: bounce

supports:
  - id: 40
    name: "War Drums"
    cost: 3
    durability: 3
    passive_effects:
      - target: AllAllyCreatures
        modifier: AttackBonus
        amount: 1
```

### 12.4 Engine Configuration Options

```rust
/// Configurable game parameters
pub struct GameConfig {
    /// Starting life total for each player
    pub starting_life: u16,           // Default: 30

    /// Maximum life (healing cap)
    pub max_life: u16,                // Default: 30

    /// Cards drawn at game start
    pub starting_hand_size: u8,       // Default: 4

    /// Maximum cards in hand
    pub max_hand_size: u8,            // Default: 10

    /// Action points per turn
    pub ap_per_turn: u8,              // Default: 3

    /// Starting essence (turn 1)
    pub starting_essence: u8,         // Default: 1

    /// Maximum essence
    pub max_essence: u8,              // Default: 10

    /// Turn limit (0 = no limit)
    pub turn_limit: u16,              // Default: 30

    /// Deck size for validation
    pub deck_size: u8,                // Default: 20 (starter) or 30 (standard)

    /// Whether Player 1 draws on turn 1
    pub player_one_draws_turn_one: bool,  // Default: false

    /// Victory points threshold (0 = disabled)
    pub victory_points_threshold: u16,    // Default: 0 (Mode A)

    /// Deck visibility mode
    pub deck_visibility: DeckVisibility,  // Default: FullOrder
}

pub enum DeckVisibility {
    /// Both players see exact deck order
    FullOrder,
    /// Both players see remaining cards but not order
    ContentsOnly,
    /// Decks are hidden (requires different AI approach)
    Hidden,
}
```

### 12.5 Card Pool Reference

Current card pool: **300 cards** across 4 factions (New Horizons Edition)

| Faction | ID Range | Cards | Identity |
|---------|----------|-------|----------|
| Argentum Combine | 1000-1074 | 75 | Guard, Piercing, Shield, Fortify - "The Wall" |
| Symbiote Circles | 2000-2074 | 75 | Rush, Lethal, Regenerate, Frenzy, Volatile - "The Swarm" |
| Obsidion Syndicate | 3000-3074 | 75 | Lifesteal, Stealth, Quick, Ward - "The Shadow" |
| Free-Walkers (Neutral) | 4000-4074 | 75 | Ranged, Charge - "The Toolbox" |

**Commander Decks:** 12 pre-built decks (4 per faction) with Legendary Commanders.

See `data/cards/core_set/` for complete card definitions.
See `docs/cards-new-horizons.md` for the complete card database reference.

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-12 | Initial consolidated design document |
| 1.1 | 2026-01-13 | Phase 1.5 keywords (Ephemeral, Regenerate, Stealth, Charge) |
| 1.2 | 2026-01-16 | Phase 4 engine enhancements: Creature Filters, Conditional Triggers, Bounce Effect, Frenzy/Volatile keywords |
| 1.3 | 2026-01-17 | **New Horizons Edition**: 300 cards, 16 keywords (added Fortify, Ward), 12 Commander Decks |

---

*End of Design Document*
