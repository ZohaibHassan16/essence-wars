# ESSENCE WARS
## A Strategic Card Game Design Document

**Version:** 1.6 (New Horizons Edition)
**Last Updated:** January 2026

---

# TABLE OF CONTENTS

1. [Game Overview](#1-game-overview)
2. [Design Philosophy](#2-design-philosophy)
3. [Components](#3-components)
4. [Game Setup](#4-game-setup)
5. [The Game Board](#5-the-game-board)
6. [Turn Structure](#6-turn-structure)
7. [Resource System: Essence](#7-resource-system-essence)
8. [Action Points](#8-action-points)
9. [Card Types](#9-card-types)
10. [Playing Cards](#10-playing-cards)
11. [Combat System](#11-combat-system)
12. [Keywords](#12-keywords)
13. [Keyword Interactions](#13-keyword-interactions)
14. [Win Conditions](#14-win-conditions)
15. [Card Anatomy](#15-card-anatomy)
16. [Card Database](#16-card-database)
17. [Commander Decks](#17-commander-decks)
18. [Faction System](#18-faction-system)
19. [Glossary](#19-glossary)
20. [Quick Reference](#20-quick-reference)

---

# 1. GAME OVERVIEW

## 1.1 Introduction

**Essence Wars** is a strategic two-player card game where players summon creatures, cast spells, and deploy powerful supports to defeat their opponent. The game features a unique lane-based combat system that creates positional strategy while remaining accessible and fast-paced.

## 1.2 Game Summary

- **Players:** 2
- **Age:** 12+
- **Play Time:** 15-30 minutes
- **Deck Size:** 30-40 cards (recommended 30 for starter games)

## 1.3 Objective

Reduce your opponent's life total from 30 to 0, or achieve an alternate victory condition before the game's turn limit.

## 1.4 Key Features

- **Lane-Based Combat:** Creatures occupy specific board positions and can only attack adjacent lanes, creating meaningful positional decisions.
- **Perfect Information:** All cards are visible to both players, including hands and decks. Strategy comes from outthinking your opponent, not from hidden information.
- **Guaranteed Resources:** No resource cards in your deck means no "bad draws" — every game has consistent pacing.
- **Action Point System:** Limited actions per turn force meaningful choices about what to do each turn.
- **Sixteen Keywords:** A focused set of keywords creates strategic depth without overwhelming complexity.

---

# 2. DESIGN PHILOSOPHY

## 2.1 Core Principles

### Clarity Over Complexity
Every rule should be understandable after a single explanation. When in doubt, choose the simpler implementation.

### Meaningful Decisions
Every turn should present interesting choices. Avoid situations where the "correct" play is obvious.

### Positional Strategy
The lane system creates a spatial dimension to strategy. Where you place creatures matters as much as which creatures you play.

### Accessible Depth
Easy to learn, difficult to master. New players can enjoy the game immediately, while experienced players discover deeper strategic layers.

### Deterministic Outcomes
Combat resolution is predictable. Players can plan ahead with certainty about outcomes.

## 2.2 What This Game Is NOT

- **Not a collectible card game:** No randomized booster packs. All cards are available to all players.
- **Not a luck-based game:** No dice rolls, no coin flips. Randomness is limited to initial deck shuffling.
- **Not a hidden information game:** Both players can see all cards at all times.
- **Not a reaction-based game:** No instant-speed responses or interrupts. Each player takes their full turn before the other acts.

---

# 3. COMPONENTS

## 3.1 Required Components

### Per Player
- **1 Deck** of 30-40 cards
- **1 Life Counter** (tracking 0-30+)
- **1 Essence Counter** (tracking 0-10)
- **1 Action Point Counter** (tracking 0-5)

### Shared
- **1 Game Board** (see Section 5)
- **1 Turn Counter** (tracking turns 1-30)
- **Status Tokens:**
  - Exhausted markers (to indicate creatures that have attacked)
  - Damage counters (1s and 5s recommended)
  - Shield tokens
  - Buff/Debuff tokens (+1/+1, -1/-1, etc.)

## 3.2 Card Breakdown (New Horizons Edition)

The New Horizons Edition contains **300 cards** organized across three factions plus neutral cards:

| Faction | Cards | ID Range | Identity |
|---------|-------|----------|----------|
| Argentum Combine | 75 | 1000-1074 | "The Wall" — Defensive constructs |
| Symbiote Circles | 75 | 2000-2074 | "The Pack" — Aggressive tempo |
| Obsidion Syndicate | 75 | 3000-3074 | "The Shadow" — Burst and control |
| Free-Walkers (Neutral) | 75 | 4000-4074 | "The Toolbox" — Utility splash |
| **Total** | **300** | | |

**Card Type Distribution (approximate):**

| Card Type | Quantity | Percentage |
|-----------|----------|------------|
| Creatures | 172 | 57% |
| Spells | 64 | 21% |
| Supports | 64 | 21% |
| **Total** | **300** | **100%** |

*For the complete card database, see [cards-new-horizons.md](cards-new-horizons.md).*

---

# 4. GAME SETUP

## 4.1 Setup Procedure

1. **Choose Decks:** Each player selects a deck of 20-30 cards.

2. **Set Life Totals:** Both players set their life counters to **30**.

3. **Prepare Essence:** Both players set their Essence counters to **0** (this will become 1 when the first turn begins).

4. **Shuffle Decks:** Each player thoroughly shuffles their deck and places it face-up in their deck zone. (Remember: this is a perfect information game!)

5. **Reveal Decks:** Both players may examine both decks at any time during the game.

6. **Draw Starting Hands:** Each player draws **4 cards** from their deck.

7. **Determine First Player:** Players may use any fair method (coin flip, dice roll, mutual agreement). The first player has a slight advantage, which is offset by drawing one fewer card on their first turn (they skip their first draw).

8. **Begin Play:** The first player begins their turn.

## 4.2 Starting Resources

| Resource | Player 1 | Player 2 | Notes |
|----------|----------|----------|-------|
| Life | 30 | 30 | |
| Maximum Essence | 0 (→1 T1) | 0 (→1 T1) | Both start equal |
| Current Essence | 0 | 0 | Refills to max each turn |
| Action Points | 0 | 0 | Becomes 3 on turn 1 |
| Hand Size | 4 cards | 6 cards | **FPA Compensation** |

### First Player Advantage (FPA) Compensation

The player going first has a natural advantage due to earlier board development. To balance this:

- **Player 2 draws 2 bonus cards** (6 cards vs P1's 4 cards)
- This gives P2 more options and flexibility to respond to P1's early plays
- Testing shows this brings win rates into the 45-55% target range
- The card advantage provides immediate flexibility without affecting the essence curve

---

# 5. THE GAME BOARD

## 5.1 Board Layout

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║    PLAYER TWO'S SIDE                                                      ║
║    ┌─────────────────────────────────────────────────────────────────┐   ║
║    │  SUPPORT ZONE          │            │          SUPPORT ZONE     │   ║
║    │     [Slot 1]           │            │            [Slot 2]       │   ║
║    └─────────────────────────────────────────────────────────────────┘   ║
║                                                                           ║
║    ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          ║
║    │ SLOT 1  │ │ SLOT 2  │ │ SLOT 3  │ │ SLOT 4  │ │ SLOT 5  │          ║
║    │         │ │         │ │ (CENTER)│ │         │ │         │          ║
║    │Creature │ │Creature │ │Creature │ │Creature │ │Creature │          ║
║    └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘          ║
║         │           │           │           │           │                ║
║         │╲         ╱│╲         ╱│╲         ╱│╲         ╱│                ║
║         │ ╲       ╱ │ ╲       ╱ │ ╲       ╱ │ ╲       ╱ │    LANE       ║
║         │  ╲     ╱  │  ╲     ╱  │  ╲     ╱  │  ╲     ╱  │    COMBAT    ║
║         │   ╲   ╱   │   ╲   ╱   │   ╲   ╱   │   ╲   ╱   │    ZONE      ║
║         │    ╲ ╱    │    ╲ ╱    │    ╲ ╱    │    ╲ ╱    │                ║
║         │     ╳     │     ╳     │     ╳     │     ╳     │                ║
║         │    ╱ ╲    │    ╱ ╲    │    ╱ ╲    │    ╱ ╲    │                ║
║         │   ╱   ╲   │   ╱   ╲   │   ╱   ╲   │   ╱   ╲   │                ║
║         │  ╱     ╲  │  ╱     ╲  │  ╱     ╲  │  ╱     ╲  │                ║
║         │ ╱       ╲ │ ╱       ╲ │ ╱       ╲ │ ╱       ╲ │                ║
║         │╱         ╲│╱         ╲│╱         ╲│╱         ╲│                ║
║         ▼           ▼           ▼           ▼           ▼                ║
║    ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          ║
║    │ SLOT 1  │ │ SLOT 2  │ │ SLOT 3  │ │ SLOT 4  │ │ SLOT 5  │          ║
║    │         │ │         │ │ (CENTER)│ │         │ │         │          ║
║    │Creature │ │Creature │ │Creature │ │Creature │ │Creature │          ║
║    └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘          ║
║                                                                           ║
║    ┌─────────────────────────────────────────────────────────────────┐   ║
║    │  SUPPORT ZONE          │            │          SUPPORT ZONE     │   ║
║    │     [Slot 1]           │            │            [Slot 2]       │   ║
║    └─────────────────────────────────────────────────────────────────┘   ║
║    PLAYER ONE'S SIDE                                                      ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

## 5.2 Board Zones

### Creature Slots (5 per player)
Each player has 5 creature slots arranged in a horizontal row. These are numbered 1 through 5, with Slot 3 being the center position.

- **Slot 1:** Left edge
- **Slot 2:** Left-center
- **Slot 3:** Center (most strategically valuable)
- **Slot 4:** Right-center
- **Slot 5:** Right edge

### Support Slots (2 per player)
Each player has 2 support slots located behind their creature row. Support cards placed here provide ongoing effects.

### Deck Zone (1 per player)
Where each player's deck is placed, face-up. Players may examine both decks at any time.

### Hand Zone (1 per player)
Cards in a player's hand. Hands are public information and may be examined by either player.

### Discard Pile (1 per player)
Where spent spells and destroyed cards go. This pile is also public information.

## 5.3 Lane Combat Visualization

The diagonal lines on the board represent attack lanes. A creature in a given slot can attack:
- The enemy creature directly across (same slot number)
- Enemy creatures in adjacent slots (±1 from their slot number)

**Lane Attack Ranges:**

| Your Slot | Can Attack Enemy Slots |
|-----------|------------------------|
| 1 | 1, 2 |
| 2 | 1, 2, 3 |
| 3 | 2, 3, 4 |
| 4 | 3, 4, 5 |
| 5 | 4, 5 |

*Note: Slot 3 (Center) is the most powerful defensive position because it can be attacked by enemies in slots 2, 3, and 4, but a creature with Guard there protects the widest area.*

---

# 6. TURN STRUCTURE

## 6.1 Turn Overview

Each turn follows this sequence:

```
┌─────────────────────────────────────────────────────────────────┐
│                         TURN STRUCTURE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. START PHASE (Automatic)                                     │
│     ├── Increase Maximum Essence by 1 (cap: 10)                │
│     ├── Refill Current Essence to Maximum                      │
│     ├── Set Action Points to 3                                 │
│     ├── Remove "Exhausted" status from all your creatures      │
│     ├── Draw 1 card from your deck                             │
│     ├── Reduce Durability of your Supports by 1                │
│     └── Remove any Supports with 0 Durability                  │
│                                                                 │
│  2. MAIN PHASE (Player Actions)                                 │
│     └── Take any number of actions until you run out of        │
│         Action Points or choose to end your turn               │
│                                                                 │
│  3. END PHASE (Automatic)                                       │
│     ├── Trigger any "End of Turn" effects                      │
│     └── Pass turn to opponent                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 6.2 Start Phase Details

The Start Phase happens automatically and cannot be interrupted.

### Essence Increase
Your Maximum Essence increases by 1, to a maximum of 10. Then your Current Essence refills to match your Maximum.

| Turn | Maximum Essence |
|------|-----------------|
| 1 | 1 |
| 2 | 2 |
| 3 | 3 |
| ... | ... |
| 10+ | 10 |

### Action Points Reset
Your Action Points reset to 3 at the start of each turn.

### Creature Refresh
All "Exhausted" markers are removed from your creatures. Creatures that attacked last turn can attack again this turn.

### Card Draw
Draw the top card of your deck. If your hand already has 10 cards (maximum hand size), the drawn card is discarded instead.

### Support Durability
Each Support you control loses 1 Durability. If a Support reaches 0 Durability, it is removed from the game and placed in your discard pile.

## 6.3 Main Phase Details

During the Main Phase, you may take actions by spending Action Points (AP) and Essence.

### Available Actions

| Action | AP Cost | Additional Cost |
|--------|---------|-----------------|
| Play a Creature | 1 AP | Card's Essence cost |
| Play a Spell | 1 AP | Card's Essence cost |
| Play a Support | 1 AP | Card's Essence cost |
| Attack with a Creature | **0 AP** | None (free action) |
| End Turn | 0 AP | None |

**Key Rule:** Attacks are free! Only playing cards costs AP. This allows aggressive plays where you attack with all your creatures AND play cards.

### Action Order
You may take actions in any order. For example:
- Play a creature, attack with all your creatures, play another creature
- Attack, attack, attack, play a card
- Play three cards (if you have the Essence), then attack with existing creatures

### Ending Your Turn
You may end your turn at any time, even if you have remaining Action Points. Say "End turn" or "Pass" to signal this.

## 6.4 End Phase Details

The End Phase happens automatically after you declare the end of your turn.

- Any "End of Turn" triggered abilities activate.
- Play then passes to your opponent.

---

# 7. RESOURCE SYSTEM: ESSENCE

## 7.1 Overview

Essence is the primary resource used to play cards. Unlike some other card games, you do not draw resource cards — Essence generation is automatic and guaranteed.

## 7.2 Essence Generation

- **Starting Essence:** 0 Maximum / 0 Current → 1/1 after first turn start (both players equal)
- **Per Turn Gain:** +1 Maximum Essence (gained at the start of your turn)
- **Maximum Cap:** 10 Essence
- **Refill:** Current Essence refills to Maximum at the start of each turn
- **Carry-Over:** Unspent Essence is lost at end of turn (does not carry over)

## 7.3 Essence Curve

| Turn | Max Essence | Notes |
|------|-------------|-------|
| 1 | 1 | Early game - cheap creatures |
| 2 | 2 | |
| 3 | 3 | Mid-game begins |
| 4 | 4 | |
| 5 | 5 | |
| 6 | 6 | Late-game threshold |
| 7-9 | 7-9 | |
| 10+ | 10 | Capped |

*Note: Both players follow the same essence curve. FPA compensation is via bonus cards, not essence (see Section 4.2).*

## 7.4 Design Rationale

The automatic Essence system provides several benefits:

1. **No "Mana Screw":** Every player always has resources to work with.
2. **Consistent Pacing:** Games follow a predictable arc from early to late game.
3. **Deckbuilding Focus:** Deck construction focuses on strategy, not resource ratios.
4. **Reduced Variance:** Game outcomes depend more on decisions than luck.

---

# 8. ACTION POINTS

## 8.1 Overview

Action Points (AP) limit how many things you can do each turn. This creates meaningful decisions about what to prioritize.

## 8.2 Action Point Economy

- **Starting AP:** 3 per turn (refreshes each turn)
- **Maximum AP:** Typically 3, but some effects may grant additional AP
- **Carry-Over:** Unspent AP is lost at end of turn

## 8.3 Action Costs

| Action | AP Cost | Essence Cost |
|--------|---------|--------------|
| Play any card | 1 AP | Card's essence cost |
| Attack with a creature | **0 AP** | None |
| Commander's Insight | 0 AP | 4 Essence |
| End turn early | 0 AP | None |

**Note:** Attacks are free actions! This is a key design choice that enables aggressive board-wide attacks while still limiting card plays.

## 8.4 Commander's Insight (Catch-Up Mechanic)

Commander's Insight is a special action available in the late game to help struggling players:

**Cost:** 0 AP + 4 Essence
**Effect:** Draw 1 card

**Requirements (ALL must be met):**
- Turn 10 or later
- 0-1 cards in hand
- 4+ essence available
- Behind on creatures OR behind on life (strict inequality, ties don't qualify)
- Not already used this turn

**Example:**
> Turn 12. You have 1 card in hand, 18 life (opponent: 24), 2 creatures (opponent: 3).
> Commander's Insight is available because you're behind on both life AND creatures.
> You pay 4 essence and draw a card, hoping for an answer.

**Design Rationale:**
- **Late game only (Turn 10+):** Prevents early game abuse and ensures the mechanic only matters when games go long
- **Low hand size (≤1):** Targets players in "top-deck mode" who are truly struggling
- **Behind condition:** Players who are winning or tied cannot use it — this is strictly a catch-up mechanic
- **Free action (0 AP):** Players can actually use the drawn card immediately
- **Once per turn:** Prevents infinite loops or excessive card advantage

## 8.5 Strategic Implications

With only 3 AP per turn (and free attacks), players must choose between:
- Playing multiple cheap cards vs. one expensive card
- Developing board vs. holding cards for later
- AP is the limiting factor for card plays, not attacks

**Example Turn Decisions:**
- *Aggressive:* Play a creature, attack with ALL your creatures (attacks are free!)
- *Developmental:* Play creature, Play creature, Play creature, attack with existing creatures
- *Defensive:* Play creature with Guard, Play removal spell, attack opportunistically

---

# 9. CARD TYPES

## 9.1 Overview

There are three card types in Essence Wars:

| Type | Persistence | Slots Used | Primary Role |
|------|-------------|------------|--------------|
| Creature | Permanent (until destroyed) | Creature Slots (1-5) | Combat, board presence |
| Spell | One-time | None (discarded after use) | Immediate effects |
| Support | Temporary (has Durability) | Support Slots (1-2) | Ongoing effects |

## 9.2 Creatures

Creatures are the backbone of your strategy. They occupy board slots, engage in combat, and persist until destroyed.

### Creature Properties
- **Attack:** How much damage this creature deals in combat
- **Health:** How much damage this creature can take before dying
- **Keywords:** Special abilities (see Section 12)
- **Abilities:** Triggered or activated effects

### Creature Rules
- Creatures enter play in a specific slot (1-5)
- Newly played creatures cannot attack the turn they are played (Summoning Sickness) unless they have **Rush**
- Creatures that attack become "Exhausted" and cannot attack again until your next turn
- Creatures cannot move between slots once placed (unless a card effect allows it)
- When a creature's Health reaches 0 or less, it is destroyed and placed in the discard pile

### Damage on Creatures
Damage dealt to creatures persists until the creature is healed or destroyed. A creature with 5 maximum Health that has taken 3 damage has 2 current Health remaining.

## 9.3 Spells

Spells are one-time effects that happen immediately when played. After resolving, the spell is placed in the discard pile.

### Spell Properties
- **Cost:** Essence required to play
- **Effect:** What happens when the spell is played
- **Targeting:** What the spell can target (if any)

### Spell Rules
- Spells resolve immediately upon being played
- If a spell has a target, you must choose a valid target when playing it
- If no valid target exists, the spell cannot be played
- After resolution, spells go to the discard pile

## 9.4 Supports

Supports are persistent effects that occupy Support Slots. They provide ongoing benefits but have limited duration.

### Support Properties
- **Cost:** Essence required to play
- **Durability:** How many turns the Support lasts
- **Effect:** The ongoing benefit or triggered ability

### Support Rules
- Each player has 2 Support Slots
- Supports lose 1 Durability at the start of your turn
- When Durability reaches 0, the Support is removed and discarded
- Effects are active as long as the Support is in play
- You may play a new Support even if your slots are full; you must first discard an existing Support

---

# 10. PLAYING CARDS

## 10.1 General Procedure

To play any card:

1. **Announce:** Declare which card you are playing
2. **Pay Costs:** Spend the required Essence AND 1 Action Point
3. **Choose Targets/Placement:** If required, select targets or placement slot
4. **Resolve:** Apply the card's effects or place it on the board

## 10.2 Playing Creatures

1. Check that you have an empty Creature Slot
2. Pay the Essence cost and 1 AP
3. Choose which slot (1-5) to place the creature
4. Place the creature card in that slot
5. Resolve any "When played" (OnPlay) abilities
6. The creature has Summoning Sickness (cannot attack this turn) unless it has Rush

### Summoning Sickness
Creatures cannot attack on the turn they are played. This is called "Summoning Sickness." At the start of your next turn, the creature is ready to attack.

**Exception:** Creatures with the **Rush** keyword can attack immediately.

## 10.3 Playing Spells

1. Check that you can meet the spell's targeting requirements (if any)
2. Pay the Essence cost and 1 AP
3. Choose targets (if required)
4. Resolve the spell's effects
5. Place the spell in your discard pile

### Targeting Rules
- **"Target creature":** Choose any creature on the board
- **"Target enemy creature":** Choose a creature controlled by your opponent
- **"Target ally creature":** Choose a creature you control
- **"Target creature or player":** Choose any creature OR either player
- **"All enemy creatures":** Affects all creatures your opponent controls (no choice)

## 10.4 Playing Supports

1. Check that you have an empty Support Slot (or are willing to discard one)
2. Pay the Essence cost and 1 AP
3. Place the Support in an empty Support Slot
4. The Support's effects are now active
5. The Support will lose 1 Durability at the start of each of your turns

---

# 11. COMBAT SYSTEM

## 11.1 Overview

Combat in Essence Wars uses a **lane-based system** where positioning matters. Creatures can only attack enemies within their reach, and the defending player has no opportunity to block — defense is accomplished by strategic creature placement.

## 11.2 Lane Attack Ranges

Each creature can attack enemies in adjacent lanes based on their slot position:

```
YOUR CREATURES:     Slot 1    Slot 2    Slot 3    Slot 4    Slot 5
                       │         │         │         │         │
Can Attack:         ───┼─────────┼─────────┼─────────┼─────────┼───
                       │╲       ╱│╲       ╱│╲       ╱│╲       ╱│
                       │ ╲     ╱ │ ╲     ╱ │ ╲     ╱ │ ╲     ╱ │
                       │  ╲   ╱  │  ╲   ╱  │  ╲   ╱  │  ╲   ╱  │
                       ▼   ╲ ╱   ▼   ╲ ╱   ▼   ╲ ╱   ▼   ╲ ╱   ▼
ENEMY CREATURES:    Slot 1    Slot 2    Slot 3    Slot 4    Slot 5
```

| Your Creature Slot | Can Attack Enemy Slots | Can Attack Face? |
|-------------------|------------------------|------------------|
| 1 | 1, 2 | Only if enemy Slot 1 is empty |
| 2 | 1, 2, 3 | Only if enemy Slot 2 is empty |
| 3 | 2, 3, 4 | Only if enemy Slot 3 is empty |
| 4 | 3, 4, 5 | Only if enemy Slot 4 is empty |
| 5 | 4, 5 | Only if enemy Slot 5 is empty |

## 11.3 Attacking Procedure

To attack with a creature:

1. **Check Eligibility:** The creature must:
   - Not be Exhausted (hasn't attacked this turn)
   - Not have Summoning Sickness (unless it has Rush)
   - Have at least 1 Attack

2. **Spend AP:** Pay 1 Action Point

3. **Choose Target:** Select a valid target:
   - An enemy creature within lane range, OR
   - The enemy player's face (only if your direct lane is empty)

4. **Check Guard:** If any enemy creature with Guard is within your attack range, you MUST attack that creature (see Section 12.4)

5. **Resolve Combat:** Apply damage based on whether you're attacking a creature or face

6. **Mark Exhausted:** The attacking creature becomes Exhausted

## 11.4 Attacking a Creature

When your creature attacks an enemy creature, combat is resolved simultaneously:

1. Your creature deals damage equal to its Attack to the defender
2. The defending creature deals damage equal to its Attack to your creature
3. Both creatures take damage at the same time
4. Any creature reduced to 0 or less Health is destroyed

**Example:**
> Your 4/3 attacks enemy 3/4
> - Your creature deals 4 damage → Enemy creature becomes 3/0 → Dies
> - Enemy creature deals 3 damage → Your creature becomes 4/0? No wait...
> 
> Let me recalculate:
> - Your 4/3 (4 Attack, 3 Health)
> - Enemy 3/4 (3 Attack, 4 Health)
> - Your creature takes 3 damage → 4/0? No, 4 attack / (3-3=0) health → Dies
> - Enemy takes 4 damage → (4-4=0) health → Dies
> - Both creatures are destroyed!

## 11.5 Attacking Face

When your creature attacks the enemy player directly:

1. Your creature deals damage equal to its Attack to the enemy player's life total
2. Your creature takes no damage in return
3. The enemy player's life is reduced accordingly

**Requirement:** You can only attack face if the enemy's slot directly across from your creature is empty.

**Example:**
> Your creature in Slot 3 can attack the enemy's face ONLY if enemy Slot 3 is empty.
> (Enemy creatures in Slots 2 and 4 do NOT block face attacks from your Slot 3)

## 11.6 Combat Resolution Order

For most combats, damage is **simultaneous**. However, the **Quick** keyword changes this:

### Standard Combat (Simultaneous)
Both creatures deal damage at the same time. Both may die.

### Quick Combat (Sequential)
1. The Quick creature deals its damage first
2. If the defender dies, it deals no damage back
3. If the defender survives, it deals its damage

### Quick vs. Quick
If both creatures have Quick, combat is simultaneous (they cancel out).

## 11.7 Combat Example

**Situation:**
- You control a 3/4 creature in Slot 2
- Enemy controls a 2/3 creature in Slot 1 and a 4/2 creature in Slot 3

**Your Options:**
- Attack the 2/3 in Slot 1 (within range: slots 1, 2, 3)
- Attack the 4/2 in Slot 3 (within range: slots 1, 2, 3)
- Cannot attack face (your direct lane, Slot 2, is... wait, there's no creature in enemy Slot 2)

Actually, let me re-read. The enemy has creatures in Slots 1 and 3, not Slot 2.

**Corrected Options:**
- Attack the 2/3 in Slot 1 (within range)
- Attack the 4/2 in Slot 3 (within range)
- Attack face (enemy Slot 2 is empty, your Slot 2 creature can hit face!)

**If you attack the 2/3:**
- Your 3/4 deals 3 damage → Enemy 2/3 becomes 2/0 → Dies
- Enemy 2/3 deals 2 damage → Your 3/4 becomes 3/2 → Survives
- Result: You killed their creature and yours survived with 2 Health!

---

# 12. KEYWORDS

Keywords are special abilities that modify how creatures behave.

## 12.1 Combat Keywords

### RUSH
**"This creature can attack the turn it is played."**
- Ignores Summoning Sickness
- Does not grant additional attacks

### RANGED
**"This creature can attack any enemy creature. Does not take counter-attack damage."**
- Can target any slot (not just adjacent lanes)
- **Still respects Guard** — must attack Guard creatures if present
- Does not take counter-attack damage in combat

### PIERCING
**"When this creature kills an enemy creature, excess damage is dealt to the enemy player."**
- Excess = Attack minus Defender's remaining Health
- Only triggers when defender dies

### GUARD
**"Enemy creatures in adjacent lanes must attack this creature first."**
- If a Guard is in range, you MUST attack it
- Multiple Guards: attacker chooses which to attack
- Slot 3 has best coverage (protects against slots 2, 3, 4)

## 12.2 Utility Keywords

### LIFESTEAL
**"When this creature deals combat damage, heal your commander for that amount."**
- Triggers on any combat damage (creatures or face)
- Cannot heal above 30 life

### LETHAL
**"Any damage this creature deals to another creature destroys it."**
- Works on any amount of damage (even 1)
- Only affects creatures, not players
- Creature still takes counter-attack damage

### SHIELD
**"The first time this creature would take damage, prevent that damage and remove Shield."**
- Blocks first damage instance completely
- Works against combat AND spell damage

### QUICK
**"This creature deals combat damage before creatures without Quick."**
- If Quick creature kills defender, no counter-attack
- Both Quick: simultaneous damage

### EPHEMERAL
**"This creature is destroyed at the end of your turn."**
- Death triggers still fire normally
- Often paired with Rush for burst damage

### REGENERATE
**"At the start of your turn, this creature heals 2 health."**
- Cannot heal above maximum health

### STEALTH
**"This creature cannot be targeted by enemy attacks or abilities. Removed when attacking."**
- Enemy cannot attack or target with spells
- YOUR spells can still target it
- Stealth masks Guard

### CHARGE
**"This creature deals +2 attack damage when attacking."**
- Only when attacking (not defending)
- Works with Piercing

### FRENZY
**"This creature gains +1 attack after each attack this turn."**
- Resets at end of turn
- Pairs with readying effects

### VOLATILE
**"When this creature dies, deal 2 damage to all enemy creatures."**
- Triggers on death from any source
- Does not damage enemy player

### FORTIFY
**"This creature takes 1 less damage from all sources (minimum 1)."**
- Cannot reduce damage to 0

### WARD
**"The first spell or ability that would target this creature has no effect. Ward is then removed."**
- Does not block AoE or combat damage
- Similar to Shield but for targeted effects

## 12.3 Keyword Summary Table

| Keyword | Effect | Stat Cost* | Primary Faction |
|---------|--------|------------|-----------------|
| Rush | Attack immediately when played | ~1.0 stats | Symbiote |
| Ranged | Attack any slot, no counter-attack | ~1.0-1.5 stats | Free-Walker |
| Piercing | Excess damage to face when killing | ~0.5-1.0 stats | Argentum |
| Guard | Adjacent enemies must attack this | ~0.5 stats | Argentum |
| Lifesteal | Heal when dealing combat damage | ~1.0-1.5 stats | Obsidion |
| Lethal | Any damage to creatures kills them | ~1.5-2.0 stats | Symbiote |
| Shield | Absorb first damage instance | ~1.0 stats | Argentum |
| Quick | Deal combat damage first | ~1.0-1.5 stats | Obsidion |
| Ephemeral | Dies at end of your turn | ~-1.5 stats (bonus) | Obsidion |
| Regenerate | Heal 2 at start of your turn | ~1.0 stats | Symbiote |
| Stealth | Can't be targeted by enemies until attacking | ~1.5 stats | Obsidion |
| Charge | +2 attack damage when attacking | ~1.0 stats | Free-Walker |
| Frenzy | +1 attack after each attack this turn | ~1.0 stats | Symbiote |
| Volatile | Deal 2 damage to all enemies on death | ~0.5 stats | Symbiote |
| Fortify | Take 1 less damage (minimum 1) | ~1.0 stats | Argentum |
| Ward | Block first targeted spell/ability | ~1.0 stats | Obsidion |

*Stat Cost indicates how many stat points (Attack + Health) a creature "loses" to have this keyword. A vanilla 3-cost creature has ~7 stats; a 3-cost with Rush has ~6 stats. Ephemeral has negative cost (bonus stats) because the creature self-destructs.

---

# 13. KEYWORD INTERACTIONS

## 13.1 Key Interactions

| Combination | Result |
|-------------|--------|
| **Ranged + Guard** | Ranged does NOT bypass Guard. Must still attack Guards. Benefit: global range + no counter-attack |
| **Quick + Lethal** | **COMBO!** Strike first, kill with any damage, survive. A 1/1 can kill a 10/10 |
| **Piercing + Lethal** | No synergy. Piercing calculates from actual health, not Lethal's instant kill |
| **Charge + Piercing** | **COMBO!** +2 Charge damage counts toward Piercing excess |
| **Ephemeral + Rush** | **COMBO!** Attack immediately, die anyway. Allows aggressive stats at low cost |
| **Ephemeral + Volatile** | **COMBO!** Guaranteed 2 AoE damage at end of turn |
| **Fortify + Guard** | **COMBO!** Extremely durable wall |
| **Fortify + Regenerate** | **COMBO!** Takes less damage AND heals back |
| **Stealth + Guard** | Stealth MASKS Guard. Guard inactive until Stealth breaks |
| **Stealth + Ward** | **COMBO!** Protected from targeting, then spell immunity |

## 13.2 Shield Blocks Keywords

Shield prevents damage entirely, which affects:

| Attacker Has | Result |
|--------------|--------|
| Lethal | Lethal doesn't trigger (no damage dealt) |
| Piercing | No kill, no piercing |
| Lifesteal | No damage, no healing |

## 13.3 Quick + Shield

When Quick creature attacks Shield creature: Quick deals damage → Shield absorbs → Defender survives → Defender hits back (Quick doesn't help since defender lived)

---

# 14. WIN CONDITIONS

## 14.1 Primary Win Condition: Life Reduction

**Reduce your opponent's life total to 0 or less.**

This is the most common way to win. Deal 30 damage to your opponent (or enough to reduce them from their current life to 0).

## 14.2 Alternate Win Condition: Victory Points

**Be the first player to deal 50 total damage.**

All damage you deal to the enemy player is tracked as "Victory Points." If you reach 50 Victory Points, you win immediately.

This prevents stalemates where both players are at low life but unable to finish the game.

## 14.3 Turn Limit Tiebreaker

**After Turn 30 (15 full rounds), the player with more life wins.**

If the game reaches Turn 30 (meaning each player has taken 15 turns), the game ends immediately:
- The player with higher life wins
- **If life totals are equal, Player 1 wins** (slight first-player advantage as tiebreaker)

## 14.4 Simultaneous Events

If both players would reach 0 life in the same combat, the game is a **draw**.

## 14.5 Win Condition Summary

| Condition | Description | Priority |
|-----------|-------------|----------|
| Life to Zero | Reduce opponent to 0 life | Checked immediately |
| Victory Points | Deal 50 total damage | Checked immediately |
| Turn Limit | Higher life after Turn 30; P1 wins ties | End of Turn 30 |
| Draw | Both reach 0 life simultaneously | Only on mutual death |

---

# 15. CARD ANATOMY

## 15.1 Creature Cards

| Element | Description |
|---------|-------------|
| Card Name | The creature's name |
| Essence Cost | Cost to play |
| Type Line | "Creature — [Tags]" |
| Keywords | Special abilities (Rush, Guard, etc.) |
| Ability Text | Triggered or activated abilities |
| Attack | Damage dealt in combat |
| Health | Damage required to destroy |

## 15.2 Spell Cards

| Element | Description |
|---------|-------------|
| Card Name | The spell's name |
| Essence Cost | Cost to play |
| Type Line | "Spell" |
| Effect Text | What happens when cast (then discarded) |

## 15.3 Support Cards

| Element | Description |
|---------|-------------|
| Card Name | The support's name |
| Essence Cost | Cost to play |
| Type Line | "Support" |
| Effect Text | Ongoing effect or trigger |
| Durability | Turns remaining before removal |

---

# 16. CARD DATABASE

The New Horizons Edition contains **300 cards** organized by faction. 


## 16.1 Card Organization

Cards are organized in YAML files by faction:

```
data/cards/core_set/
├── argentum.yaml     # IDs 1000-1074 (75 cards)
├── symbiote.yaml     # IDs 2000-2074 (75 cards)
├── obsidion.yaml     # IDs 3000-3074 (75 cards)
└── neutral.yaml      # IDs 4000-4074 (75 cards)
```

## 16.2 Card ID Ranges

| Faction | ID Range | Reserved For |
|---------|----------|--------------|
| Argentum Combine | 1000-1999 | Future expansions |
| Symbiote Circles | 2000-2999 | Future expansions |
| Obsidion Syndicate | 3000-3999 | Future expansions |
| Free-Walkers (Neutral) | 4000-4999 | Future expansions |

## 16.3 Commander System

Commanders are a special card type that defines your deck's identity. Unlike regular creatures, commanders exist in a **Command Zone** and provide persistent abilities throughout the game.

### Commander Rules

- **Command Zone:** Commanders are not on the battlefield — they exist in a separate Command Zone
- **Commander Life = Player Life:** Your commander's life total is your life total (30). Face attacks damage the commander/player.
- **Cannot be Targeted:** Commanders cannot be targeted by attacks, spells, or abilities
- **Persistent Abilities:** Commanders provide either **Passive** abilities (always active) or **Triggered** abilities (fire on specific events)
- **Not in Deck:** Commanders are chosen separately from your deck and do not count toward deck size

### Commander ID Range

Commanders use a separate ID range: **5000-5011** (not part of the regular card set).

| Faction | Commander IDs |
|---------|---------------|
| Argentum Combine | 5000-5003 |
| Symbiote Circles | 5004-5007 |
| Obsidion Syndicate | 5008-5011 |

### Argentum Combine Commanders

| ID | Name | Ability Type | Effect |
|----|------|--------------|--------|
| 5000 | The High Artificer | Triggered | StartOfTurn: Summon a 2/2 Brass Cog |
| 5001 | The Sanctum Healer | Passive | Creatures have Ward and +0/+2 |
| 5002 | Siege Marshal Vex | Passive | Creatures have +2 Attack |
| 5003 | The Grand Architect | Passive | Creatures have Fortify and +0/+2 |

### Symbiote Circles Commanders

| ID | Name | Ability Type | Effect |
|----|------|--------------|--------|
| 5004 | The Broodmother | Passive | All creatures have Rush |
| 5005 | Plague Sovereign | Triggered | OnAllyDeath: Deal 2 damage to enemy commander |
| 5006 | Alpha of the Hunt | Triggered | OnAttack: Give all your creatures +1/+0 |
| 5007 | The Eternal Grove | Triggered | StartOfTurn: Give all your creatures +1/+1 |

### Obsidion Syndicate Commanders

| ID | Name | Ability Type | Effect |
|----|------|--------------|--------|
| 5008 | The Blood Sovereign | Passive | Creatures have Lifesteal and +0/+1 |
| 5009 | The Deathmaster | Passive | Creatures with Lethal have Quick |
| 5010 | The Shadow Weaver | Passive | All creatures have Stealth |
| 5011 | Void Archon | Passive | All creatures have Quick |

## 16.4 Card Rarity Distribution

| Rarity | Per Faction | Total |
|--------|-------------|-------|
| Common | ~30 | ~120 |
| Uncommon | ~25 | ~100 |
| Rare | ~15 | ~60 |
| Legendary | ~5 | ~20 |
| **Total** | **75** | **300** |

---

# 17. COMMANDER DECKS

The New Horizons Edition features **12 pre-built Commander Decks** — each built around a Legendary Commander with synergistic cards.

## 17.1 Deck Construction Rules

- **Deck Size:** 29 cards + 1 commander = 30 total
- **Commander:** Chosen separately, not part of the 29-card deck
- **Card Copies:** Maximum 2 copies of any card per deck
- **Composition:** Typically ~21 faction cards + ~8 neutral splash cards (70/30 split)

## 17.2 Argentum Combine Decks (4)

### 🏗️ The High Artificer — Token/Construct

**Deck ID:** `artificer_tokens`
**Commander:** The High Artificer (5000) — StartOfTurn: Summon 2/2 Brass Cog
**Strategy:** Flood the board with Construct tokens, buff them with support cards

### 💚 The Sanctum Healer — Ward/Tank

**Deck ID:** `sanctum_healer`
**Commander:** The Sanctum Healer (5001) — Creatures have Ward and +0/+2
**Strategy:** Spell-immune creatures with massive health pools

### ⚔️ Siege Marshal Vex — Aggro

**Deck ID:** `vex_piercing`
**Commander:** Siege Marshal Vex (5002) — Creatures have +2 Attack
**Strategy:** Aggressive damage with heavily buffed creatures

### 🔧 The Grand Architect — Fortify/Control

**Deck ID:** `architect_fortify`
**Commander:** The Grand Architect (5003) — Creatures have Fortify and +0/+2
**Strategy:** Damage reduction + health buff makes every creature extremely durable

---

## 17.3 Symbiote Circles Decks (4)

### 🐺 The Broodmother — Rush/Pack

**Deck ID:** `broodmother_pack`
**Commander:** The Broodmother (5004) — All creatures have Rush
**Strategy:** Every creature attacks immediately; overwhelming pack aggression

### ☠️ Plague Sovereign — Death Triggers

**Deck ID:** `plague_volatile`
**Commander:** Plague Sovereign (5005) — OnAllyDeath: Deal 2 damage to enemy commander
**Strategy:** Death triggers and board-wide punishment

### 🐺 Alpha of the Hunt — Frenzy/Aggro

**Deck ID:** `alpha_frenzy`
**Commander:** Alpha of the Hunt (5006) — OnAttack: Give all creatures +1/+0
**Strategy:** Snowballing attack buffs with every combat

### 🌳 The Eternal Grove — Growth/Midrange

**Deck ID:** `grove_regenerate`
**Commander:** The Eternal Grove (5007) — StartOfTurn: Give all creatures +1/+1
**Strategy:** Creatures grow stronger every turn; inevitable late-game dominance

---

## 17.4 Obsidion Syndicate Decks (4)

### 🩸 The Blood Sovereign — Lifesteal/Sustain

**Deck ID:** `sovereign_lifesteal`
**Commander:** The Blood Sovereign (5008) — Creatures have Lifesteal and +0/+1
**Strategy:** Sustain through combat while dealing damage

### 🗡️ The Deathmaster — Lethal Assassins

**Deck ID:** `deathmaster_assassin`
**Commander:** The Deathmaster (5009) — Creatures with Lethal have Quick
**Strategy:** Lethal creatures strike first, killing enemies without taking damage

### 👤 The Shadow Weaver — Stealth

**Deck ID:** `shadow_weaver`
**Commander:** The Shadow Weaver (5010) — All creatures have Stealth
**Strategy:** Untargetable creatures for safe attacks

### ⚡ Void Archon — Quick/Burst

**Deck ID:** `archon_burst`
**Commander:** Void Archon (5011) — All creatures have Quick
**Strategy:** Strike first in every combat

---

## 17.5 Deck File Location

All decks are defined in TOML files organized by faction:

```
data/decks/
├── argentum/
│   ├── artificer_tokens.toml
│   ├── sanctum_healer.toml
│   ├── vex_piercing.toml
│   └── architect_fortify.toml
├── symbiote/
│   ├── broodmother_pack.toml
│   ├── plague_volatile.toml
│   ├── alpha_frenzy.toml
│   └── grove_regenerate.toml
└── obsidion/
    ├── sovereign_lifesteal.toml
    ├── deathmaster_assassin.toml
    ├── shadow_weaver.toml
    └── archon_burst.toml
```

## 17.6 Balance Status

All 12 commander decks have been validated for competitive balance:

| Faction | Win Rate Range | Status |
|---------|----------------|--------|
| Argentum | 52-58% | ✅ Balanced |
| Symbiote | 46-52% | ✅ Balanced |
| Obsidion | 44-50% | ✅ Balanced |

**Cross-faction delta:** <10% (target achieved)

---

# 18. FACTION SYSTEM

## 18.1 Overview

Essence Wars features a **faction-based card system** that provides thematic identity and strategic focus. Cards are organized into three true factions plus a neutral category.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         FACTION HIERARCHY                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   TRUE FACTIONS (Primary Identity)                                       │
│   ├── Argentum Combine    "The Wall"     [Defensive, Industrial]        │
│   ├── Symbiote Circles    "The Pack"     [Aggressive, Primal]           │
│   └── Obsidion Syndicate  "The Shadow"   [Burst, Control]               │
│                                                                          │
│   NEUTRAL CARDS (Supplemental)                                           │
│   └── Free-Walkers        "The Toolbox"  [Utility, Flexible]            │
│       - Can be splashed into any faction deck                            │
│       - Provides answers and flexibility                                 │
│       - Similar to "colorless/artifact" cards in other games            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## 18.2 True Factions

### 🏛️ ARGENTUM COMBINE — "The Wall"

**Thematic Identity:** Order, Industry, Defense
**Lore:** Art Deco Steampunk civilization. "Structure is Safety."

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Guard, Piercing, Shield |
| **Secondary Keywords** | Regenerate (rare) |
| **Archetypes** | Soldiers, Constructs, Engineers |
| **Strengths** | High HP, defensive formations, outlasting opponents |
| **Weaknesses** | Low burst damage, slow tempo |
| **Avoid** | Rush, Lethal, Ephemeral, Stealth |

**Playstyle:** Wall up with Guard creatures, heal through damage, grind opponents down through superior board presence.

---

### 🌿 SYMBIOTE CIRCLES — "The Pack"

**Thematic Identity:** Primal Nature, Pack Bond, The Hunt
**Lore:** Urza's Saga-era nature magic. "The Pack Endures. Hunt as One."

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Rush, Lethal, Regenerate |
| **Secondary Keywords** | Frenzy, Volatile |
| **Archetypes** | Wolves, Great Cats, Serpents, Druids, Treants |
| **Strengths** | Tempo, efficient trading, sustained pressure |
| **Weaknesses** | Low board control, vulnerable to AoE |
| **Avoid** | Guard, Shield |

**Playstyle:** Aggressive pack tactics with Rush creatures. Trade efficiently using Lethal predators. Regenerate provides staying power for ancient forest guardians.

---

### 🔮 OBSIDION SYNDICATE — "The Glass Cannon"

**Thematic Identity:** Knowledge, Ambition, Power
**Lore:** Gothic Cyber-Magic underworld. "Power is Personal."

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Lifesteal, Stealth, Ephemeral, Quick |
| **Secondary Keywords** | Lethal (assassins) |
| **Archetypes** | Mages, Cultists, Assassins, Undead, Spirits |
| **Strengths** | Burst damage, life manipulation, precision removal |
| **Weaknesses** | Low creature stats, fragile board presence |
| **Avoid** | Guard, Regenerate |

**Playstyle:** Setup-based burst damage. Use Ephemeral creatures for tempo, Stealth for guaranteed damage, and Lifesteal to sustain through self-inflicted costs.

---

### ⚖️ FREE-WALKERS — "The Toolbox" (Neutral)

**Thematic Identity:** Mercenaries, Flexibility, Profit
**Lore:** Rugged frontier survivors. "No Flag. Just Gold."

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Ranged, Charge |
| **Secondary Keywords** | Any (neutral access) |
| **Archetypes** | Giants, Hunters, Mercenaries, Scouts |
| **Strengths** | Flexibility, precision damage, gap-filling |
| **Weaknesses** | No strong faction identity, jack-of-all-trades |
| **Special Rule** | Can be splashed into ANY faction deck |

**Role:** Free-Walker cards are **neutral utility cards** that can be added to any faction deck. They fill gaps, provide answers, and add flexibility without diluting faction identity.

## 18.3 Keyword Distribution by Faction

| Keyword | Argentum | Symbiote | Obsidion | Free-Walker |
|---------|:--------:|:--------:|:--------:|:-----------:|
| Rush | ✗ | ★★★ | ★ | ★ |
| Ranged | ★ | ★ | ✗ | ★★★ |
| Piercing | ★★ | ✗ | ✗ | ★★ |
| Guard | ★★★ | ✗ | ✗ | ★ |
| Lifesteal | ✗ | ✗ | ★★★ | ✗ |
| Lethal | ✗ | ★★★ | ★ | ✗ |
| Shield | ★★ | ✗ | ✗ | ★ |
| Quick | ✗ | ✗ | ★★★ | ★ |
| Ephemeral | ✗ | ✗ | ★★★ | ✗ |
| Regenerate | ★ | ★★★ | ✗ | ✗ |
| Stealth | ✗ | ✗ | ★★★ | ✗ |
| Charge | ✗ | ✗ | ✗ | ★★★ |
| **Frenzy** | ✗ | ★★★ | ✗ | ✗ |
| **Volatile** | ✗ | ★★★ | ✗ | ✗ |
| **Fortify** | ★★★ | ✗ | ✗ | ✗ |
| **Ward** | ✗ | ✗ | ★★ | ★ |

**Legend:** ★★★ Primary | ★★ Secondary | ★ Rare | ✗ Avoided

**New Horizons Edition Keywords:**
- **Frenzy** and **Volatile** are Symbiote signature mechanics (death/aggression theme)
- **Fortify** is Argentum's signature defensive mechanic (damage reduction)
- **Ward** protects key Obsidion pieces from removal

## 18.4 Balance Philosophy

### Design Goals

1. **Faction Identity:** Each faction should feel distinct and have clear strengths/weaknesses
2. **No Hard Counters:** Avoid strict rock-paper-scissors relationships
3. **Slight Asymmetry OK:** Perfect 50/50 balance is not required; ±5% variance acceptable
4. **Neutral as Glue:** Free-Walkers should enable faction decks, not replace them

### Balance Targets

| Matchup Type | Target Win Rate |
|--------------|-----------------|
| Faction vs Faction | 45-55% |
| Mirror Match | 50% (by definition) |
| Same Deck, Different Agents | Agent skill difference |

### What We Avoid

- **Dominant Strategies:** No single faction/deck should exceed 60% win rate
- **Unplayable Factions:** No faction should fall below 40% win rate

---

# 19. GLOSSARY

| Term | Definition |
|------|------------|
| **Action Point (AP)** | Resource spent to take actions. Players receive 3 AP per turn. |
| **Adjacent Lane** | The lanes immediately next to a given lane. Slot 3 is adjacent to Slots 2 and 4. |
| **Attack** | A creature's stat determining how much damage it deals in combat. |
| **Combat Damage** | Damage dealt during creature-to-creature combat or face attacks. |
| **Creature** | A card type that occupies board slots and engages in combat. |
| **Current Essence** | The amount of Essence available to spend this turn. |
| **Damage** | A reduction of Health (for creatures) or Life (for players). |
| **Deck** | The pile of cards a player draws from. Visible to all players. |
| **Destroy** | Remove a creature from the board and place it in the discard pile. |
| **Discard Pile** | Where spent spells and destroyed cards go. Public information. |
| **Durability** | How many turns a Support lasts before being removed. |
| **Essence** | The primary resource used to play cards. |
| **Exhausted** | A creature that has attacked this turn and cannot attack again. |
| **Face** | The player themselves as an attack target (reduces Life). |
| **Guard** | Keyword: Adjacent enemies must attack this creature first. |
| **Hand** | Cards held by a player. Public information. |
| **Health** | A creature's stat determining how much damage it can take. |
| **Keyword** | A special ability word that modifies how a creature behaves. |
| **Lane** | The vertical attack path between opposing creature slots. |
| **Lethal** | Keyword: Any damage dealt destroys the target creature. |
| **Life** | A player's health total. Starting value is 30. |
| **Lifesteal** | Keyword: Combat damage dealt heals your hero. |
| **Maximum Essence** | The cap on how much Essence you can have (increases each turn to 10). |
| **OnPlay** | Trigger: Activates when the card is played from hand. |
| **Piercing** | Keyword: Excess damage to creatures goes to the enemy player. |
| **Quick** | Keyword: This creature deals combat damage first. |
| **Ranged** | Keyword: Can attack any enemy creature, bypasses Guard. |
| **Rush** | Keyword: Can attack the turn it is played. |
| **Shield** | Keyword: First damage instance is absorbed and prevented. |
| **Slot** | A position on the board where a creature or support is placed. |
| **Spell** | A card type that has an immediate effect and is then discarded. |
| **StartOfTurn** | Trigger: Activates at the beginning of your turn. |
| **Summoning Sickness** | Creatures cannot attack the turn they are played (unless they have Rush). |
| **Support** | A card type that provides ongoing effects with limited duration. |
| **Tag** | A creature subtype (e.g., Soldier, Beast, Mage) for thematic grouping. |
| **Target** | The selection of what a spell or ability affects. |
| **Turn** | One player's complete cycle of phases (Start, Main, End). |
| **Vanilla** | A creature with no keywords or abilities, just stats. |
| **Victory Points** | Total damage dealt to the enemy player (tracked for alternate win condition). |
| **Commander's Insight** | A catch-up mechanic allowing struggling players to draw a card for 4 essence (requires Turn 10+, ≤1 hand, behind on creatures OR life). |
| **Frenzy** | Keyword: +1 attack after each attack this turn. |
| **Volatile** | Keyword: Deal 2 damage to all enemy creatures when this creature dies. |
| **Fortify** | Keyword: Take 1 less damage from all sources (minimum 1). |
| **Ward** | Keyword: First spell/ability targeting this has no effect; then Ward is removed. |
| **Commander** | A Legendary creature designed as a deck's centerpiece with powerful abilities. |
| **Token** | A creature created by an effect, not from a card. |
| **Bounce** | Return a creature to its owner's hand. |
| **Conditional Effect** | An effect that triggers only if a condition is met (e.g., "if target died"). |
| **Filter** | Criteria that restrict which creatures an effect can target (e.g., "max health ≤ 3"). |

---

# 20. QUICK REFERENCE

## 20.1 Turn Structure

1. **START PHASE**
   - +1 Maximum Essence (cap 10)
   - Refill Current Essence
   - Reset to 3 Action Points
   - Refresh all your creatures
   - Draw 1 card
   - Reduce Support Durabilities by 1

2. **MAIN PHASE**
   - Take actions (1 AP each)
   - Play cards (pay Essence + 1 AP)
   - Attack with creatures (1 AP each)
   - End turn when ready

3. **END PHASE**
   - Resolve end-of-turn effects
   - Pass to opponent

## 20.2 Action Costs

| Action | Cost |
|--------|------|
| Play any card | 1 AP + Essence Cost |
| Attack with creature | **0 AP** (free!) |
| Commander's Insight | 0 AP + 4 Essence (late game catch-up) |
| End turn | Free |

## 20.3 Lane Attack Ranges

| Your Slot | Attack Range |
|-----------|--------------|
| 1 | Slots 1, 2 |
| 2 | Slots 1, 2, 3 |
| 3 | Slots 2, 3, 4 |
| 4 | Slots 3, 4, 5 |
| 5 | Slots 4, 5 |

## 20.4 Keyword Quick Reference

| Keyword | One-Line Summary | Faction |
|---------|------------------|---------|
| Rush | Attack immediately when played | Symbiote |
| Ranged | Attack any slot, no counter-attack | Free-Walker |
| Piercing | Overkill damage hits face | Argentum |
| Guard | Force adjacent enemies to attack this | Argentum |
| Lifesteal | Heal when dealing damage | Obsidion |
| Lethal | Any damage kills creatures | Symbiote |
| Shield | Block first damage, one time | Argentum |
| Quick | Deal damage first in combat | Obsidion |
| Ephemeral | Dies at end of your turn | Obsidion |
| Regenerate | Heal 2 at start of your turn | Symbiote |
| Stealth | Can't be targeted until attacking | Obsidion |
| Charge | +2 attack damage when attacking | Free-Walker |
| Frenzy | +1 attack after each attack this turn | Symbiote |
| Volatile | Deal 2 AoE damage on death | Symbiote |
| Fortify | Take 1 less damage (min 1) | Argentum |
| Ward | Block first targeted spell/ability | Obsidion |

## 20.5 Win Conditions

1. **Enemy life ≤ 0** → You win
2. **50 Victory Points** → You win
3. **Turn 30** → Higher life wins (P1 wins ties)

---

*End of Document*

**ESSENCE WARS: NEW HORIZONS EDITION** — A Game of Perfect Information and Strategic Depth
