# ESSENCE WARS
## A Strategic Card Game Design Document

**Version:** 1.2 (New Horizons Edition)
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
19. [AI Agent Architecture](#19-ai-agent-architecture)
20. [Glossary](#20-glossary)
21. [Quick Reference](#21-quick-reference)

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
| Symbiote Circles | 75 | 2000-2074 | "The Swarm" — Aggressive tempo |
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
| Maximum Essence | 0 (→1 T1) | 1 (→2 T1) | **FPA Compensation** |
| Current Essence | 0 | 0 | Refills to max each turn |
| Action Points | 0 | 0 | Becomes 3 on turn 1 |
| Hand Size | 4 cards | 4 cards | |

### First Player Advantage (FPA) Compensation

The player going first has a natural advantage due to earlier board development. To balance this:

- **Player 2 starts with +1 Maximum Essence** (2 essence on Turn 1 vs P1's 1 essence)
- This allows P2 to deploy a stronger creature or two 1-cost creatures on their first turn
- Testing shows this brings cross-faction win rates into the 45-55% target range
- The +1 essence advantage narrows over time as both players approach the 10 essence cap

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
| Attack with a Creature | 1 AP | None |
| End Turn | 0 AP | None |

### Action Order
You may take actions in any order. For example:
- Play a creature, attack with a different creature, play a spell
- Attack, attack, play a creature
- Play three cards (if you have the Essence)

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

- **Starting Essence (P1):** 0 Maximum / 0 Current → 1/1 after first turn start
- **Starting Essence (P2):** 1 Maximum / 0 Current → 2/2 after first turn start *(FPA compensation)*
- **Per Turn Gain:** +1 Maximum Essence (gained at the start of your turn)
- **Maximum Cap:** 10 Essence
- **Refill:** Current Essence refills to Maximum at the start of each turn
- **Carry-Over:** Unspent Essence is lost at end of turn (does not carry over)

## 7.3 Essence Curve

| Turn | P1 Max Essence | P2 Max Essence | Notes |
|------|----------------|----------------|-------|
| 1 | 1 | 2 | P2 has FPA compensation |
| 2 | 2 | 3 | Gap narrows |
| 3 | 3 | 4 | |
| 4 | 4 | 5 | |
| 5 | 5 | 6 | |
| 6 | 6 | 7 | |
| 7 | 7 | 8 | |
| 8 | 8 | 9 | |
| 9 | 9 | 10 | P2 caps first |
| 10+ | 10 | 10 | Both capped, parity reached |

*Note: P2's essence advantage diminishes as both players approach the 10 essence cap. By turn 10, both players have equal maximum essence.*

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

| Action | AP Cost |
|--------|---------|
| Play any card | 1 AP |
| Attack with a creature | 1 AP |
| End turn early | 0 AP |

## 8.4 Strategic Implications

With only 3 AP per turn, players must choose between:
- Playing multiple cheap cards vs. one expensive card + an attack
- Attacking with multiple creatures vs. developing their board
- Saving AP (impossible) vs. using all actions efficiently

**Example Turn Decisions:**
- *Aggressive:* Attack, Attack, Attack (3 creatures attack)
- *Developmental:* Play creature, Play creature, Attack
- *Defensive:* Play creature with Guard, Play removal spell, hold position

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

Keywords are special abilities that modify how creatures behave. Each keyword has a specific, consistent effect.

## 12.1 Combat Keywords

### RUSH
**"This creature can attack the turn it is played."**

- Rush creatures ignore Summoning Sickness
- They can attack immediately after being played
- Rush does not grant additional attacks; the creature still becomes Exhausted after attacking

**Strategic Use:** Rush creatures provide immediate impact. Use them to remove threats or push damage when you need something to happen NOW.

### RANGED
**"This creature can attack any enemy creature, regardless of lane position."**

- Ranged creatures are not limited to adjacent lanes
- A Ranged creature in Slot 1 can attack an enemy in Slot 5
- Ranged creatures BYPASS Guard (they can ignore Guard creatures and attack other targets)
- Face attack rules still apply: can only attack face if your direct lane is empty

**Strategic Use:** Ranged creatures are precision tools. Use them to eliminate key threats that are protected by positioning or Guard creatures.

### PIERCING
**"When this creature kills an enemy creature, excess damage is dealt to the enemy player."**

- Only triggers when the defending creature dies
- Excess damage = Attacker's Attack minus Defender's remaining Health
- Does not trigger if the defender survives

**Example:**
> Your 5/3 Piercing attacks enemy 2/2
> - Enemy takes 5 damage, has 2 Health → Dies
> - Excess damage: 5 - 2 = 3
> - Enemy player takes 3 damage!

**Strategic Use:** Piercing creatures punish chump-blocking. Even if the enemy throws creatures in front of your attacker, damage still gets through.

### GUARD
**"Enemy creatures in adjacent lanes must attack this creature first."**

- Guard only affects enemies that could attack this creature (within lane range)
- If a Guard is in your attack range, you MUST attack it (cannot attack other creatures or face)
- If multiple Guards are in range, you may choose which Guard to attack
- Ranged creatures ignore Guard

**Lane Protection Zones:**

| Guard in Slot | Protects Against Enemies in Slots |
|---------------|-----------------------------------|
| 1 | 1, 2 |
| 2 | 1, 2, 3 |
| 3 | 2, 3, 4 (best coverage!) |
| 4 | 3, 4, 5 |
| 5 | 4, 5 |

**Strategic Use:** Place Guard creatures in the center (Slot 3) for maximum protection. Use Guards to protect valuable creatures or your life total.

## 12.2 Utility Keywords

### LIFESTEAL
**"When this creature deals combat damage, heal your hero for that amount."**

- Triggers on any combat damage (to creatures or face)
- Heals for the full damage dealt, even if overkilling
- Cannot heal above maximum life (30)

**Example:**
> Your 4/3 Lifesteal attacks and kills an enemy 2/2
> - You deal 4 damage
> - You heal for 4 life

**Strategic Use:** Lifesteal creatures help you race. You deal damage while healing, making it hard for aggressive decks to keep up.

### LETHAL
**"Any damage this creature deals to another creature destroys it."**

- Works on any amount of damage (even 1)
- Only affects creatures, not players
- Triggers on combat damage
- The creature still takes damage normally from the defender

**Example:**
> Your 1/1 Lethal attacks enemy 10/10
> - Your creature deals 1 damage with Lethal → Enemy is destroyed!
> - Enemy deals 10 damage → Your creature is destroyed
> - Both die, but you traded a 1-cost for a 10-cost!

**Strategic Use:** Lethal creatures are the great equalizers. A tiny Lethal creature threatens the biggest enemies. Use them to remove expensive threats efficiently.

### SHIELD
**"The first time this creature would take damage, prevent that damage and remove Shield."**

- Absorbs the first instance of damage completely (even 100 damage becomes 0)
- After absorbing damage once, Shield is removed
- Shield does not regenerate (unless granted again by an effect)
- Works against combat damage AND spell damage

**Example:**
> Enemy 3/3 attacks your 2/2 Shield
> - Your creature would take 3 damage
> - Shield absorbs all 3 damage → Your creature takes 0 damage
> - Shield is removed
> - Your creature deals 2 damage to the attacker
> - Result: Your creature survives at 2/2 (no Shield), enemy is at 3/1

**Strategic Use:** Shield guarantees your creature survives at least one combat. Use it to protect key creatures or to win trades.

### QUICK
**"This creature deals combat damage before creatures without Quick."**

- Quick creatures strike first in combat
- If the Quick creature kills the defender, the defender deals no damage back
- If both creatures have Quick, damage is simultaneous
- Only affects creature combat, not face damage

**Example:**
> Your 3/2 Quick attacks enemy 4/4
> - Your Quick creature deals 3 damage first → Enemy becomes 4/1
> - Enemy survives, deals 4 damage back → Your creature becomes 3/-2 → Dies
> - Result: You died, but dealt your damage first (didn't help here)

**Better Example:**
> Your 3/2 Quick attacks enemy 2/2
> - Your Quick creature deals 3 damage first → Enemy becomes 2/-1 → Dies
> - Enemy is dead, deals no damage back
> - Result: Your creature survives at 3/2!

**Strategic Use:** Quick lets you trade up efficiently. A Quick creature can kill something and survive when it normally would have died in mutual combat.

### EPHEMERAL
**"This creature is destroyed at the end of your turn."**

- Triggers at the end of the owner's turn, not immediately
- Death triggers (OnDeath effects) still fire normally
- Can still attack if it has Rush
- Excellent for burst damage or one-time effects

**Example:**
> You play a 3/3 Rush + Ephemeral creature for only 1 mana
> - It attacks immediately for 3 damage
> - At end of turn, it dies
> - Great value for burst damage, no board presence next turn

**Strategic Use:** Ephemeral creatures trade lasting board presence for immediate impact. Use them for surprise attacks or when you need to close out a game.

### REGENERATE
**"At the start of your turn, this creature heals 2 health."**

- Triggers at the start of the owner's turn
- Cannot heal above maximum health
- Makes the creature very hard to remove through gradual damage
- Does not trigger if the creature is already at full health

**Example:**
> Your 2/4 Regenerate takes 3 damage, is now at 2/1
> - At start of your next turn: heals 2, becomes 2/3
> - Survives another attack!

**Strategic Use:** Regenerate creatures are excellent for attrition battles. They force opponents to either kill them in one hit or waste resources on repeated attacks.

### STEALTH
**"This creature cannot be targeted by enemy attacks or abilities. Stealth is removed when this creature attacks."**

- Enemy creatures cannot attack this creature
- Enemy targeted spells/abilities cannot target this creature
- YOUR OWN spells/abilities CAN still target it (friendly targeting allowed)
- Stealth is removed (broken) when the creature declares an attack
- Stealth masks Guard (a stealthed Guard cannot force enemies to attack it)

**Example:**
> You play a 3/2 Stealth creature
> - Enemy cannot attack it directly
> - Enemy cannot use "Deal 2 damage to target creature" on it
> - On your next turn, you attack → Stealth is removed
> - Now the creature can be targeted normally

**Strategic Use:** Stealth creatures guarantee at least one attack. Use them to set up powerful attacks or to protect key creatures until you're ready to strike.

### CHARGE
**"This creature deals +2 attack damage when attacking."**

- Bonus applies when this creature attacks (not when defending)
- The bonus damage applies to both creature and face attacks
- Works with Piercing (excess damage includes the Charge bonus)
- Does not increase the creature's displayed Attack stat

**Example:**
> Your 2/3 Charge attacks an enemy creature
> - Base attack: 2
> - Charge bonus: +2
> - Total damage dealt: 4

**Strategic Use:** Charge creatures hit harder than their stats suggest. They're excellent for trading up or pushing face damage.

### FRENZY
**"This creature gains +1 attack after each attack it makes this turn."**

- Stacking bonus that resets at end of turn
- Works with Quick (creature can attack twice, gaining +1 after first attack)
- Pairs well with effects that ready creatures
- Primarily a Symbiote keyword

**Example:**
> Your 3/4 Frenzy creature attacks twice (via Quick or readying effect)
> - First attack: deals 3 damage
> - Frenzy triggers: gains +1 attack (now 4/4)
> - Second attack: deals 4 damage

**Strategic Use:** Frenzy creatures reward multiple attacks per turn. Build around effects that ready creatures or grant Quick.

### VOLATILE
**"When this creature dies, deal 2 damage to all enemy creatures."**

- Triggers on death from any source (combat, spells, effects)
- Damage is dealt before the creature leaves the board
- Does not damage the enemy player, only creatures
- Can chain with other Volatile creatures dying
- Primarily a Symbiote keyword

**Example:**
> Your 2/2 Volatile creature dies in combat
> - Death trigger: deals 2 damage to ALL enemy creatures
> - Enemy board of 3/1, 2/1, 4/3 becomes 3/-1 (dead), 2/-1 (dead), 4/1

**Strategic Use:** Volatile creatures punish board-wide strategies. Even when killed, they take enemies down with them.

### FORTIFY
**"This creature takes 1 less damage from all sources (minimum 1)."**

- Reduces ALL incoming damage by 1
- Minimum damage is 1 (cannot reduce damage to 0)
- Stacks with other damage reduction effects
- Primarily an Argentum keyword

**Example:**
> Your 2/5 Fortify creature is attacked by a 3/3
> - Normal damage would be 3
> - Fortify reduces by 1 → takes 2 damage
> - Your creature survives at 2/3

**Strategic Use:** Fortify creatures are excellent tanks. They survive multiple small attacks and trade favorably against most threats.

### WARD
**"The first spell or ability that would target this creature has no effect. Ward is then removed."**

- Only blocks the FIRST targeted effect
- Does not block untargeted effects (AoE damage)
- Does not block combat damage
- Similar to Shield but for spells/abilities instead of damage
- Primarily an Obsidion keyword (for protecting key pieces)

**Example:**
> Your 4/4 Ward creature is targeted by "Deal 5 damage"
> - Ward absorbs the spell → no damage dealt
> - Ward is removed
> - Next spell will affect the creature normally

**Strategic Use:** Ward protects valuable creatures from removal. Force opponents to waste a spell before using their real removal.

## 12.3 Keyword Summary Table

| Keyword | Effect | Stat Cost* | Primary Faction |
|---------|--------|------------|-----------------|
| Rush | Attack immediately when played | ~1.0 stats | Symbiote |
| Ranged | Attack any enemy creature, bypass Guard | ~1.0-1.5 stats | Free-Walker |
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

When multiple keywords interact, follow these rules:

## 13.1 Ranged + Guard

**Ranged BYPASSES Guard.**

A Ranged creature can attack any enemy creature, even if Guard creatures are within range. This makes Ranged a direct counter to Guard-based defensive strategies.

## 13.2 Quick + Lethal

**EXTREMELY POWERFUL COMBINATION!**

A creature with both Quick and Lethal can:
1. Strike first (Quick)
2. Kill the defender with any damage (Lethal)
3. The defender dies before dealing damage back
4. Your creature survives!

A 1/1 Quick+Lethal can kill a 10/10 and walk away unharmed.

*Design Note: This combination should be rare and expensive.*

## 13.3 Piercing + Lethal

**Does NOT combo as strongly as you might think.**

Piercing calculates excess damage based on the defender's actual Health, not the Lethal effect. 

**Example:**
> Your 2/1 Piercing+Lethal attacks enemy 8/8
> - Lethal triggers: Enemy is destroyed
> - Piercing check: Your Attack (2) - Enemy Health (8) = -6 → No piercing damage
> - Result: Enemy dies but no face damage (Piercing doesn't benefit from Lethal)

## 13.4 Shield Interactions

Shield prevents damage, which affects several keywords:

| Attacker Has | Result When Hitting Shield |
|--------------|---------------------------|
| Lethal | No damage dealt → Lethal doesn't trigger → Defender survives (without Shield) |
| Piercing | No damage dealt → No kill → No piercing damage |
| Lifesteal | No damage dealt → No healing |

**Example:**
> Your 1/1 Lethal attacks enemy 2/2 Shield
> - Shield absorbs the 1 damage → 0 damage dealt
> - Lethal requires damage to be dealt → Doesn't trigger
> - Enemy survives at 2/2 (without Shield)
> - Enemy deals 2 damage → Your creature dies

## 13.5 Quick + Shield

Quick creature attacks Shield creature:
1. Quick deals damage first
2. Shield absorbs the damage
3. Shield is removed
4. Defender survives, deals damage back (not blocked by Quick since they survived)

## 13.6 Complete Interaction Matrix (Core 8 Keywords)

```
             │ Rush │Ranged│Pierce│Guard │LifeS │Lethal│Shield│Quick │
─────────────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┤
Rush         │  -   │  ✓   │  ✓   │  ✓   │  ✓   │  ✓   │  ✓   │  ✓   │
Ranged       │  ✓   │  -   │  ✓   │BYPASS│  ✓   │  ✓   │  ✓   │  ✓   │
Piercing     │  ✓   │  ✓   │  -   │  ✓   │  ✓   │  x   │  x   │  ✓   │
Guard        │  ✓   │BYPSD │  ✓   │  -   │  ✓   │  ✓   │  ✓   │  ✓   │
Lifesteal    │  ✓   │  ✓   │  ✓   │  ✓   │  -   │  ✓   │  x   │  ✓   │
Lethal       │  ✓   │  ✓   │  x   │  ✓   │  ✓   │  -   │BLOCKED│COMBO!│
Shield       │  ✓   │  ✓   │BLOCKS│  ✓   │BLOCKS│BLOCKS│  -   │  ✓   │
Quick        │  ✓   │  ✓   │  ✓   │  ✓   │  ✓   │COMBO!│  ✓   │  -   │

Legend:
✓ = Works independently, no special interaction
x = Does not combo effectively
BYPASS/BYPSD = One keyword bypasses the other
BLOCKS/BLOCKED = One keyword blocks/is blocked by the other
COMBO! = Especially powerful combination
```

## 13.6.1 Extended Keyword Interactions

### Frenzy Interactions

| Keyword | Interaction |
|---------|-------------|
| Quick | **COMBO!** Quick + Frenzy can attack twice per turn if creature readies; second attack gets +1 |
| Rush | Works independently; Frenzy creature can attack turn 1 but only once |
| Lifesteal | Works well; each attack heals AND increases next attack |

### Volatile Interactions

| Keyword | Interaction |
|---------|-------------|
| Ephemeral | **COMBO!** Ephemeral dies at end of turn, triggering Volatile damage guaranteed |
| Lethal | Works independently; Lethal kills attackers, Volatile punishes board-wide |
| Shield | Volatile damage is blocked by Shield (one instance) |

### Fortify Interactions

| Keyword | Interaction |
|---------|-------------|
| Guard | **COMBO!** Fortify + Guard creates an extremely durable wall |
| Regenerate | **COMBO!** Takes less damage AND heals; very hard to remove |
| Lethal | Fortify does NOT reduce Lethal to 0; Lethal still kills |
| Shield | Works independently; Shield blocks first hit entirely, Fortify reduces subsequent |

### Ward Interactions

| Keyword | Interaction |
|---------|-------------|
| Stealth | **COMBO!** Stealth prevents targeting until attack, Ward blocks first spell after |
| Shield | Works independently; Ward blocks abilities, Shield blocks damage |
| Guard | Works well; Ward protects your Guard from removal spells |

## 13.7 Stealth + Guard

**Stealth MASKS Guard.**

A creature with both Stealth and Guard cannot be targeted by enemies while stealthed. The Guard keyword is effectively inactive until Stealth is broken (when the creature attacks).

**Example:**
> You play a creature with Stealth + Guard
> - Enemies cannot target it (Stealth)
> - Guard does NOT force enemies to attack it (masked by Stealth)
> - When it attacks, Stealth breaks
> - NOW Guard is active and enemies must attack it

## 13.8 Ephemeral + Rush

**POWERFUL BURST COMBO!**

Ephemeral + Rush creatures can attack immediately and die at end of turn anyway. This allows for extremely aggressive stats at low cost, since the creature was going to die regardless.

**Example:**
> Ghost Wolf (1-cost 3/3 Rush + Ephemeral)
> - Play for just 1 mana
> - Attack immediately for 3 damage
> - Dies at end of turn
> - Incredible burst value!

## 13.9 Charge + Piercing

**ENHANCED PIERCING DAMAGE!**

When a Charge creature kills a defender, the excess damage for Piercing includes the +2 Charge bonus.

**Example:**
> Your 3/3 Charge + Piercing attacks enemy 2/2
> - Damage dealt: 3 + 2 (Charge) = 5
> - Enemy has 2 health → Dies
> - Piercing excess: 5 - 2 = 3 damage to face!

## 13.10 Regenerate + Damage Trading

**EXCELLENT FOR ATTRITION!**

Regenerate creatures are very efficient in repeated small trades. They heal 2 HP at start of your turn, making them hard to remove through chip damage.

**Counter Strategy:** Kill Regenerate creatures in one hit, or they'll keep coming back.

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
- If life totals are equal, the game is a draw

## 14.4 Simultaneous Events

If both players would win at the same time (e.g., both reduced to 0 life in the same combat), the game is a **draw**.

## 14.5 Win Condition Summary

| Condition | Description | Priority |
|-----------|-------------|----------|
| Life to Zero | Reduce opponent to 0 life | Checked immediately |
| Victory Points | Deal 50 total damage | Checked immediately |
| Turn Limit | Higher life after Turn 30 | End of Turn 30 |
| Draw | Equal conditions | Fallback |

---

# 15. CARD ANATOMY

## 15.1 Creature Card Layout

```
╔═════════════════════════════════════════════════════════════╗
║                                                             ║
║   CARD NAME                              ESSENCE COST       ║
║   ───────────────────────────────────    ┌─────────────┐   ║
║                                          │     💎      │   ║
║                                          │     5       │   ║
║                                          └─────────────┘   ║
║   ┌───────────────────────────────────────────────────┐    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                  [ARTWORK]                        │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   └───────────────────────────────────────────────────┘    ║
║                                                             ║
║   TYPE: Creature — Tag                                      ║
║                                                             ║
║   ┌─────────────────────────────────────────────────────┐  ║
║   │                                                     │  ║
║   │   [KEYWORDS]                                        │  ║
║   │                                                     │  ║
║   │   Ability text goes here. Describes what the        │  ║
║   │   creature does when certain conditions are met.    │  ║
║   │                                                     │  ║
║   └─────────────────────────────────────────────────────┘  ║
║                                                             ║
║   ┌───────────┐                           ┌───────────┐    ║
║   │  ATTACK   │                           │  HEALTH   │    ║
║   │    ⚔️     │                           │    ❤️     │    ║
║   │    4      │                           │    5      │    ║
║   └───────────┘                           └───────────┘    ║
║                                                             ║
╚═════════════════════════════════════════════════════════════╝
```

### Creature Card Elements

| Element | Location | Description |
|---------|----------|-------------|
| Card Name | Top Left | The creature's name |
| Essence Cost | Top Right | Cost to play (in blue gem) |
| Artwork | Center | Illustration of the creature |
| Type Line | Below Art | "Creature — [Tags]" |
| Keywords | Text Box Top | Bolded keywords in brackets |
| Ability Text | Text Box | Description of special abilities |
| Attack | Bottom Left | Red sword/axe icon with number |
| Health | Bottom Right | Green/red heart icon with number |

## 15.2 Spell Card Layout

```
╔═════════════════════════════════════════════════════════════╗
║                                                             ║
║   CARD NAME                              ESSENCE COST       ║
║   ───────────────────────────────────    ┌─────────────┐   ║
║                                          │     💎      │   ║
║                                          │     3       │   ║
║                                          └─────────────┘   ║
║   ┌───────────────────────────────────────────────────┐    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                  [ARTWORK]                        │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   └───────────────────────────────────────────────────┘    ║
║                                                             ║
║   TYPE: Spell                                               ║
║                                                             ║
║   ┌─────────────────────────────────────────────────────┐  ║
║   │                                                     │  ║
║   │   Effect text goes here. Describes exactly what     │  ║
║   │   happens when this spell is cast.                  │  ║
║   │                                                     │  ║
║   │                                                     │  ║
║   │                                                     │  ║
║   └─────────────────────────────────────────────────────┘  ║
║                                                             ║
╚═════════════════════════════════════════════════════════════╝
```

### Spell Card Elements

| Element | Location | Description |
|---------|----------|-------------|
| Card Name | Top Left | The spell's name |
| Essence Cost | Top Right | Cost to play (in blue gem) |
| Artwork | Center | Illustration of the spell effect |
| Type Line | Below Art | "Spell" |
| Effect Text | Text Box | What the spell does when played |

## 15.3 Support Card Layout

```
╔═════════════════════════════════════════════════════════════╗
║                                                             ║
║   CARD NAME                              ESSENCE COST       ║
║   ───────────────────────────────────    ┌─────────────┐   ║
║                                          │     💎      │   ║
║                                          │     4       │   ║
║                                          └─────────────┘   ║
║   ┌───────────────────────────────────────────────────┐    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                  [ARTWORK]                        │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   │                                                   │    ║
║   └───────────────────────────────────────────────────┘    ║
║                                                             ║
║   TYPE: Support                                             ║
║                                                             ║
║   ┌─────────────────────────────────────────────────────┐  ║
║   │                                                     │  ║
║   │   Effect text goes here. Describes the ongoing      │  ║
║   │   benefit this support provides while in play.      │  ║
║   │                                                     │  ║
║   │                                                     │  ║
║   │                                                     │  ║
║   └─────────────────────────────────────────────────────┘  ║
║                                                             ║
║                                           ┌───────────┐    ║
║                                           │DURABILITY │    ║
║                                           │    ⏳     │    ║
║                                           │    3      │    ║
║                                           └───────────┘    ║
║                                                             ║
╚═════════════════════════════════════════════════════════════╝
```

### Support Card Elements

| Element | Location | Description |
|---------|----------|-------------|
| Card Name | Top Left | The support's name |
| Essence Cost | Top Right | Cost to play (in blue gem) |
| Artwork | Center | Illustration of the support |
| Type Line | Below Art | "Support" |
| Effect Text | Text Box | The ongoing effect or trigger |
| Durability | Bottom Right | Hourglass icon with number |

---

# 16. CARD DATABASE

The New Horizons Edition contains **300 cards** organized by faction. For the complete card database with all stats, effects, and abilities, see:

**📖 [cards-new-horizons.md](cards-new-horizons.md)** — Complete Card Reference

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

## 16.3 Legendary Commanders

Each faction has **4 Legendary Commanders** designed to be deck-building centerpieces:

### Argentum Combine Commanders

| ID | Name | Cost | Stats | Keywords | Ability |
|----|------|------|-------|----------|---------|
| 1056 | The High Artificer | 6 | 3/5 | — | OnPlay: Summon two 2/2 Construct tokens |
| 1057 | Iron Colossus Prime | 7 | 2/10 | Guard | All friendly creatures have +0/+2 |
| 1058 | Siege Marshal Vex | 6 | 5/4 | Piercing | OnAttack: Deal 2 damage to enemy player |
| 1059 | The Grand Architect | 6 | 3/6 | Fortify | All friendly creatures have Fortify |

### Symbiote Circles Commanders

| ID | Name | Cost | Stats | Keywords | Ability |
|----|------|------|-------|----------|---------|
| 2060 | The Broodmother | 6 | 3/5 | Rush | OnAttack: Summon a 2/2 Rush Broodling |
| 2061 | Plague Sovereign | 6 | 4/4 | Volatile | OnAllyDeath: Deal 1 damage to enemy player |
| 2062 | Alpha of the Hunt | 5 | 4/3 | Frenzy | All friendly creatures have +1 Attack |
| 2063 | The Eternal Grove | 6 | 2/8 | Regenerate | All friendly creatures have Regenerate |

### Obsidion Syndicate Commanders

| ID | Name | Cost | Stats | Keywords | Ability |
|----|------|------|-------|----------|---------|
| 3055 | The Blood Sovereign | 6 | 4/5 | Lifesteal | All friendly creatures have Lifesteal |
| 3056 | Shadow Emperor Kael | 6 | 5/4 | Stealth, Quick | OnKill: Return this to hand |
| 3057 | The Shadow Weaver | 6 | 3/4 | Stealth | OnPlay: Summon two 2/2 Ephemeral Stealth Shadow Clones |
| 3058 | Void Archon | 5 | 4/4 | Quick | All friendly creatures have Quick |

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

- **Deck Size:** 30 cards (standard competitive format)
- **Card Copies:** Maximum 2 copies of any non-Legendary card per deck
- **Legendary Limit:** 1 copy of each Legendary card
- **Composition:** ~21 faction cards + ~9 neutral splash cards (70/30 split)

## 17.2 Argentum Combine Decks (4)

### 🏗️ The High Artificer — Token/Construct

**Deck ID:** `artificer_tokens`
**Commander:** The High Artificer (1056) — 6-cost 3/5, OnPlay: Summon two 2/2 Constructs
**Strategy:** Flood the board with Construct tokens, buff them with support cards

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Token Swarm | Assembly Line, Construct tokens | Build wide board, overwhelm with numbers |

### 🏰 Iron Colossus Prime — Guard/Wall

**Deck ID:** `colossus_wall`
**Commander:** Iron Colossus Prime (1057) — 7-cost 2/10 Guard, All allies +0/+2
**Strategy:** Create an impenetrable wall of high-HP Guards

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Defensive Wall | Shield Bearer, Fortress Golem | Wall up, outlast, win through attrition |

### ⚔️ Siege Marshal Vex — Piercing/Aggro

**Deck ID:** `vex_piercing`
**Commander:** Siege Marshal Vex (1058) — 6-cost 5/4 Piercing, OnAttack: 2 face damage
**Strategy:** Aggressive Piercing damage that bypasses blockers

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Piercing Aggro | Steam Knight, Siege Cannon | Push damage through Guards, finish with commander |

### 🔧 The Grand Architect — Fortify/Control

**Deck ID:** `architect_fortify`
**Commander:** The Grand Architect (1059) — 6-cost 3/6 Fortify, All allies have Fortify
**Strategy:** Damage reduction makes every creature a durable threat

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Fortify Control | Armored Sentinel, Field Medic | Reduce incoming damage, grind out value |

---

## 17.3 Symbiote Circles Decks (4)

### 🐛 The Broodmother — Rush/Swarm

**Deck ID:** `broodmother_swarm`
**Commander:** The Broodmother (2060) — 6-cost 3/5 Rush, OnAttack: Summon 2/2 Rush Broodling
**Strategy:** Aggressive Rush creatures, token generation, overwhelming tempo

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Rush Swarm | Broodling x3, Pack Hunter x3 | Fast pressure, generate tokens, never let up |

### ☠️ Plague Sovereign — Volatile/Death

**Deck ID:** `plague_volatile`
**Commander:** Plague Sovereign (2061) — 6-cost 4/4 Volatile, OnAllyDeath: 1 face damage
**Strategy:** Death triggers and board-wide punishment

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Death Triggers | Volatile Spore, Plague Carrier | Trade aggressively, punish enemy board |

### 🐺 Alpha of the Hunt — Frenzy/Aggro

**Deck ID:** `alpha_frenzy`
**Commander:** Alpha of the Hunt (2062) — 5-cost 4/3 Frenzy, All allies +1 Attack
**Strategy:** Attack buffs and Frenzy creatures for snowballing damage

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Frenzy Aggro | Feral Stalker, Rabid Hunter | Stack attack buffs, multiple attacks per turn |

### 🌳 The Eternal Grove — Regenerate/Midrange

**Deck ID:** `grove_regenerate`
**Commander:** The Eternal Grove (2063) — 6-cost 2/8 Regenerate, All allies Regenerate
**Strategy:** Outlast through healing, impossible to remove through chip damage

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Regenerate Value | Regenerating Ooze, Hive Guardian | Trade efficiently, heal back, win the long game |

---

## 17.4 Obsidion Syndicate Decks (4)

### 🩸 The Blood Sovereign — Lifesteal/Sustain

**Deck ID:** `sovereign_lifesteal`
**Commander:** The Blood Sovereign (3055) — 6-cost 4/5 Lifesteal, All allies Lifesteal
**Strategy:** Sustain through combat, race opponents while healing

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Lifesteal Control | Blood Acolyte, Vampire Lord | Attack for damage AND healing, outlast aggro |

### 🗡️ Shadow Emperor Kael — Stealth/Assassin

**Deck ID:** `kael_assassin`
**Commander:** Shadow Emperor Kael (3056) — 6-cost 5/4 Stealth, Quick, OnKill: Bounce
**Strategy:** Untargetable assassins, precision removal, hit-and-run tactics

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Stealth Assassin | Shadow Blade, Silent Assassin | Strike from stealth, remove key threats |

### 👤 The Shadow Weaver — Shadow Clone

**Deck ID:** `shadow_weaver`
**Commander:** The Shadow Weaver (3057) — 6-cost 3/4 Stealth, OnPlay: Summon 2 Shadow Clones
**Strategy:** Ephemeral shadow tokens, hit-and-run tactics

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Shadow Tokens | Shadow Clone tokens, Stealth creatures | Burst damage from clones, disappear before retaliation |

### ⚡ Void Archon — Quick/Burst

**Deck ID:** `archon_burst`
**Commander:** Void Archon (3058) — 5-cost 4/4 Quick, All allies Quick
**Strategy:** Strike first in every combat, win all trades

| Archetype | Key Cards | Gameplan |
|-----------|-----------|----------|
| Quick Burst | Blood Seeker, Ritual Master | Every creature strikes first, dominate combat |

---

## 17.5 Deck File Location

All decks are defined in TOML files organized by faction:

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
│   ├── Symbiote Circles    "The Swarm"    [Aggressive, Adaptive]         │
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

### 🌿 SYMBIOTE CIRCLES — "The Swarm"

**Thematic Identity:** Growth, Adaptation, Evolution
**Lore:** Biopunk Fantasy ecosystem. "Adapt or Perish."

| Aspect | Definition |
|--------|------------|
| **Primary Keywords** | Rush, Lethal, Regenerate |
| **Secondary Keywords** | Ranged (acid spitters) |
| **Archetypes** | Beasts, Parasites, Healers, Swarm |
| **Strengths** | Tempo, efficient trading, sustained pressure |
| **Weaknesses** | Low board control, vulnerable to AoE |
| **Avoid** | Guard, Shield |

**Playstyle:** Aggressive tempo with Rush creatures. Trade efficiently using Lethal. Regenerate provides staying power for key threats.

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

## 18.3 Deck Composition

Standard deck construction follows the **Faction Core + Neutral Splash** model:

```
STANDARD DECK: 20 cards
├── Faction Core: 14 cards (70%)    ← Primary faction identity
└── Neutral Splash: 6 cards (30%)   ← Free-Walker utility
```

| Deck Type | Composition | Strategy |
|-----------|-------------|----------|
| `argentum_control` | 14 Argentum + 6 FW | Wall up, outlast, utility removal |
| `symbiote_aggro` | 14 Symbiote + 6 FW | Fast pressure, Charge finishers |
| `obsidion_burst` | 14 Obsidion + 6 FW | Setup + burst, Ranged precision |

**Why 14/6 Split?**
- **70% Faction Core:** Maintains clear faction identity and keyword focus
- **30% Neutral Splash:** Provides flexibility without diluting theme
- Free-Walkers fill gaps that factions intentionally lack

## 18.4 Keyword Distribution by Faction

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

## 18.5 Balance Philosophy

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

- **"Anti-X" Decks:** No deck should exist solely to counter another faction
- **Dominant Strategies:** No single faction/deck should exceed 60% win rate
- **Unplayable Factions:** No faction should fall below 40% win rate

---

# 19. AI AGENT ARCHITECTURE

## 19.1 Overview

Essence Wars is designed for AI research, with a comprehensive agent architecture that supports both specialized and generalized play.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         AGENT HIERARCHY                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   SPECIALISTS (Faction-Optimized)                                        │
│   ├── Agent-Argentum   → Tuned for defensive, high-HP strategies        │
│   ├── Agent-Symbiote   → Tuned for aggressive tempo strategies          │
│   └── Agent-Obsidion   → Tuned for burst/control strategies             │
│                                                                          │
│   GENERALIST (Cross-Faction)                                             │
│   └── Agent-Generalist → Balanced across all factions                   │
│       - Can play any deck competently                                    │
│       - Benchmark for specialist comparison                              │
│       - Trained against all specialists + mirror play                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## 19.2 Agent Types

### Specialist Agents

Specialist agents are **optimized for a specific faction**. They have weights/policies tuned to maximize performance with that faction's deck and playstyle.

| Agent | Faction | Deck Binding | Optimization Focus |
|-------|---------|--------------|-------------------|
| Agent-Argentum | Argentum Combine | `argentum_*` decks only | Guard value, HP preservation, survival |
| Agent-Symbiote | Symbiote Circles | `symbiote_*` decks only | Rush value, Lethal trades, board presence |
| Agent-Obsidion | Obsidion Syndicate | `obsidion_*` decks only | Lifesteal value, burst damage, Stealth setup |

**Key Rule:** Specialists are **bound to their faction's decks**. An Argentum specialist should not play a Symbiote deck—this would be suboptimal and wastes computation.

### Generalist Agent

The Generalist agent is **balanced across all factions**. It can play any deck competently and serves as a benchmark.

| Property | Value |
|----------|-------|
| Deck Binding | Any deck |
| Training | Against all specialists equally + mirror play |
| Purpose | Benchmark, flexible opponent, "universal player" |

## 19.3 Training Pipeline

### Specialist Training

Each specialist is trained against its optimal opponents:

```
SPECIALIST TRAINING
├── Agent-Argentum trains vs:
│   ├── Agent-Symbiote (cross-faction)
│   ├── Agent-Obsidion (cross-faction)
│   └── Agent-Argentum (mirror, for robustness)
│
├── Agent-Symbiote trains vs:
│   ├── Agent-Argentum (cross-faction)
│   ├── Agent-Obsidion (cross-faction)
│   └── Agent-Symbiote (mirror)
│
└── Agent-Obsidion trains vs:
    ├── Agent-Argentum (cross-faction)
    ├── Agent-Symbiote (cross-faction)
    └── Agent-Obsidion (mirror)
```

### Generalist Training

The Generalist trains against **all specialists equally plus itself**:

```
GENERALIST TRAINING
Agent-Generalist trains vs:
├── Agent-Argentum (25%)
├── Agent-Symbiote (25%)
├── Agent-Obsidion (25%)
└── Agent-Generalist (25%, mirror play)
```

This ensures the Generalist:
- Can handle any faction's playstyle
- Doesn't overfit to one opponent type
- Learns robust, general strategies

## 19.4 Agent Implementation

This architecture applies to **all agent types**:

| Agent Type | MCTS | PPO | AlphaZero |
|------------|------|-----|-----------|
| Argentum Specialist | Weight file | Policy network | Value+Policy network |
| Symbiote Specialist | Weight file | Policy network | Value+Policy network |
| Obsidion Specialist | Weight file | Policy network | Value+Policy network |
| Generalist | Weight file | Policy network | Value+Policy network |

### File Organization

```
data/weights/                     # MCTS/Greedy weights
├── specialists/
│   ├── argentum.toml
│   ├── symbiote.toml
│   └── obsidion.toml
└── generalist.toml

models/                           # Trained neural networks
├── ppo/
│   ├── specialists/
│   │   ├── argentum/
│   │   ├── symbiote/
│   │   └── obsidion/
│   └── generalist/
└── alphazero/
    ├── specialists/
    │   ├── argentum/
    │   ├── symbiote/
    │   └── obsidion/
    └── generalist/
```

## 19.5 Balance Testing Matrix

### Test Configurations

| Test Type | Purpose | Configuration |
|-----------|---------|---------------|
| **Deck Balance** | Are faction decks balanced? | Generalist vs Generalist, all deck matchups |
| **Specialist Quality** | Do specialists outperform generalists? | Specialist vs Generalist, same deck |
| **Meta Health** | Overall competitive landscape | Full specialist tournament |

### Standard Test Suite

**Quick Balance Check** (during development):
```
3 decks × 3 decks = 9 matchups
Agent: Generalist only
Games: 50-100 per matchup
Time: ~5 minutes
```

**Specialist Validation** (after training):
```
Specialist vs Specialist round-robin
3 matchups (Arg vs Sym, Arg vs Obs, Sym vs Obs)
Games: 100 per matchup
Time: ~10 minutes
```

**Full Tournament** (nightly/weekly):
```
All meaningful permutations:
├── 9 deck matchups (3×3 including mirrors)
├── 3 agent configurations per matchup:
│   ├── Generalist vs Generalist (baseline)
│   ├── Specialist vs Specialist (optimal)
│   └── Specialist vs Generalist (advantage test)
└── Total: 27 test cases
Games: 100 per test case
Time: ~30-60 minutes
```

## 19.6 Success Metrics

| Metric | Target | Meaning |
|--------|--------|---------|
| Faction Balance | 45-55% win rates | No dominant faction |
| Specialist Advantage | >5% vs Generalist | Specialization is rewarded |
| Training Convergence | Stable fitness | Agent has learned |
| Meta Diversity | All factions viable | Healthy competitive landscape |

---

# 20. GLOSSARY

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

# 21. QUICK REFERENCE

## 21.1 Turn Structure

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

## 21.2 Action Costs

| Action | Cost |
|--------|------|
| Play any card | 1 AP + Essence Cost |
| Attack with creature | 1 AP |
| End turn | Free |

## 21.3 Lane Attack Ranges

| Your Slot | Attack Range |
|-----------|--------------|
| 1 | Slots 1, 2 |
| 2 | Slots 1, 2, 3 |
| 3 | Slots 2, 3, 4 |
| 4 | Slots 3, 4, 5 |
| 5 | Slots 4, 5 |

## 21.4 Keyword Quick Reference

| Keyword | One-Line Summary | Faction |
|---------|------------------|---------|
| Rush | Attack immediately when played | Symbiote |
| Ranged | Attack any enemy, bypass Guard | Free-Walker |
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

## 21.5 Win Conditions

1. **Enemy life ≤ 0** → You win
2. **50 Victory Points** → You win
3. **Turn 30** → Higher life wins (draw if tied)

---

# APPENDIX A: DESIGN NOTES FOR DEVELOPERS

## A.1 Balance Philosophy

- Vanilla creatures follow the formula: **Total Stats ≈ (Cost × 2) + 1**
- Keywords "cost" stat points (Rush ≈ 1 point, Guard ≈ 0.5 points, etc.)
- Combat keywords (Rush, Ranged, Piercing, Guard) are worth less than utility keywords
- The Quick+Lethal combination should be rare and expensive

## A.2 Suggested Expansions

Future expansions could introduce:
- New keywords (carefully limited to maintain clarity)
- Multi-color or faction systems
- Legendary unique cards (one per deck)
- Environment or terrain effects
- Alternative game modes

## A.3 Physical Component Recommendations

For production as a physical card game:
- **Card size:** Standard poker size (63mm × 88mm)
- **Card stock:** 300+ gsm with linen finish
- **Token types:** Damage (1s, 3s, 5s), Status (Exhausted, Shield), Buff/Debuff (+1/+1)
- **Life/Essence trackers:** Spin-down dice or sliding track
- **Game board:** Foldable playmat with clearly marked zones

---

*End of Document*

**ESSENCE WARS: NEW HORIZONS EDITION** — A Game of Perfect Information and Strategic Depth

© 2026 — Game Design Document v1.2

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01 | Initial design document (43 cards, 12 keywords) |
| 1.1 | 2025-06 | Added faction system, AI architecture |
| 1.2 | 2026-01 | **New Horizons Edition** — 300 cards, 16 keywords, 12 Commander Decks, Phase 4 engine features |
