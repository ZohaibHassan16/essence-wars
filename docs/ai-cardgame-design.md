# ESSENCE WARS
## A Strategic Card Game Design Document

**Version:** 1.0  
**Last Updated:** January 2025

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
16. [Starter Set Card List](#16-starter-set-card-list)
17. [Sample Decks](#17-sample-decks)
18. [Glossary](#18-glossary)
19. [Quick Reference](#19-quick-reference)

---

# 1. GAME OVERVIEW

## 1.1 Introduction

**Essence Wars** is a strategic two-player card game where players summon creatures, cast spells, and deploy powerful supports to defeat their opponent. The game features a unique lane-based combat system that creates positional strategy while remaining accessible and fast-paced.

## 1.2 Game Summary

- **Players:** 2
- **Age:** 12+
- **Play Time:** 15-30 minutes
- **Deck Size:** 20-30 cards (recommended 20 for starter games)

## 1.3 Objective

Reduce your opponent's life total from 30 to 0, or achieve an alternate victory condition before the game's turn limit.

## 1.4 Key Features

- **Lane-Based Combat:** Creatures occupy specific board positions and can only attack adjacent lanes, creating meaningful positional decisions.
- **Perfect Information:** All cards are visible to both players, including hands and decks. Strategy comes from outthinking your opponent, not from hidden information.
- **Guaranteed Resources:** No resource cards in your deck means no "bad draws" — every game has consistent pacing.
- **Action Point System:** Limited actions per turn force meaningful choices about what to do each turn.
- **Eight Keywords:** A focused set of keywords creates strategic depth without overwhelming complexity.

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
- **1 Deck** of 20-30 cards
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

## 3.2 Card Breakdown (Starter Set)

| Card Type | Quantity | Percentage |
|-----------|----------|------------|
| Creatures | 31 | 72% |
| Spells | 8 | 19% |
| Supports | 4 | 9% |
| **Total** | **43** | **100%** |

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

| Resource | Starting Value |
|----------|----------------|
| Life | 30 |
| Maximum Essence | 0 (becomes 1 on turn 1) |
| Current Essence | 0 (refills to max each turn) |
| Action Points | 0 (becomes 3 on turn 1) |
| Hand Size | 4 cards |

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

- **Starting Essence:** 0 Maximum / 0 Current
- **Per Turn Gain:** +1 Maximum Essence (gained at the start of your turn)
- **Maximum Cap:** 10 Essence
- **Refill:** Current Essence refills to Maximum at the start of each turn
- **Carry-Over:** Unspent Essence is lost at end of turn (does not carry over)

## 7.3 Essence Curve

| Turn | Max Essence | Cumulative Total* |
|------|-------------|-------------------|
| 1 | 1 | 1 |
| 2 | 2 | 3 |
| 3 | 3 | 6 |
| 4 | 4 | 10 |
| 5 | 5 | 15 |
| 6 | 6 | 21 |
| 7 | 7 | 28 |
| 8 | 8 | 36 |
| 9 | 9 | 45 |
| 10+ | 10 | 55+ |

*Cumulative total represents total Essence available over the course of the game if all Essence is spent each turn.

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

## 12.3 Keyword Summary Table

| Keyword | Effect | Stat Cost* |
|---------|--------|------------|
| Rush | Attack immediately when played | ~1.0 stats |
| Ranged | Attack any enemy creature, bypass Guard | ~1.0-1.5 stats |
| Piercing | Excess damage to face when killing | ~0.5-1.0 stats |
| Guard | Adjacent enemies must attack this | ~0.5 stats |
| Lifesteal | Heal when dealing combat damage | ~1.0-1.5 stats |
| Lethal | Any damage to creatures kills them | ~1.5-2.0 stats |
| Shield | Absorb first damage instance | ~1.0 stats |
| Quick | Deal combat damage first | ~1.0-1.5 stats |

*Stat Cost indicates how many stat points (Attack + Health) a creature "loses" to have this keyword. A vanilla 3-cost creature has ~7 stats; a 3-cost with Rush has ~6 stats.

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

## 13.6 Complete Interaction Matrix

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

# 16. STARTER SET CARD LIST

The complete starter set contains 43 cards: 31 Creatures, 8 Spells, and 4 Supports.

## 16.1 Creatures by Cost

### 1-Cost Creatures (4 cards)

| # | Name | Stats | Keywords | Ability | Notes |
|---|------|-------|----------|---------|-------|
| 01 | Eager Recruit | 2/1 | — | — | Aggressive vanilla |
| 02 | Village Guard | 1/2 | — | — | Defensive vanilla |
| 03 | Nimble Scout | 1/1 | Rush | — | Immediate impact |
| 04 | Toxic Spider | 1/1 | Lethal | — | Trades with anything |

### 2-Cost Creatures (6 cards)

| # | Name | Stats | Keywords | Ability | Notes |
|---|------|-------|----------|---------|-------|
| 05 | Iron Defender | 1/4 | Guard | — | Early wall |
| 06 | Frontier Ranger | 2/2 | Ranged | — | Flexible targeting |
| 07 | Young Knight | 2/3 | — | — | Efficient vanilla |
| 08 | Shielded Squire | 2/2 | Shield | — | Survives first hit |
| 09 | Blood Cultist | 3/2 | — | OnPlay: Deal 2 damage to yourself | High stats, self-damage |
| 10 | Medic Apprentice | 1/3 | — | OnPlay: Restore 2 health to your hero | Healing on entry |

### 3-Cost Creatures (8 cards)

| # | Name | Stats | Keywords | Ability | Notes |
|---|------|-------|----------|---------|-------|
| 11 | Centaur Charger | 3/3 | Rush | — | Immediate threat |
| 12 | Blade Dancer | 3/2 | Quick | — | Wins trades |
| 13 | Veteran Guardian | 2/5 | Guard | — | Solid wall |
| 14 | Highland Archer | 3/2 | Ranged | — | Sniper |
| 15 | War Elephant | 4/3 | — | — | Efficient vanilla |
| 16 | Piercing Striker | 4/2 | Piercing | — | Damage gets through |
| 17 | Battle Priest | 2/4 | — | StartOfTurn: Restore 1 health to your hero | Recurring heal |
| 18 | Ambush Predator | 2/2 | Rush, Lethal | — | Immediate removal |

### 4-Cost Creatures (5 cards)

| # | Name | Stats | Keywords | Ability | Notes |
|---|------|-------|----------|---------|-------|
| 19 | Armored Knight | 4/5 | — | — | Premium vanilla |
| 20 | Siege Breaker | 5/3 | Piercing | — | Heavy piercing |
| 21 | Vampire Lord | 4/3 | Lifesteal | — | Sustain machine |
| 22 | Fortress Golem | 2/7 | Guard | — | Massive wall |
| 23 | Storm Mage | 3/3 | — | OnPlay: Deal 2 damage to target creature | Removal on a body |

### 5-Cost Creatures (4 cards)

| # | Name | Stats | Keywords | Ability | Notes |
|---|------|-------|----------|---------|-------|
| 24 | Royal Champion | 5/6 | — | — | Big vanilla |
| 25 | Assassin Queen | 3/3 | Quick, Lethal | — | Kills anything, survives! |
| 26 | Warhost Captain | 4/4 | — | OnPlay: Give all other ally creatures +1/+1 | Team buffer |
| 27 | Sniper Marksman | 5/3 | Ranged | — | High-powered sniper |

### 6+ Cost Creatures (4 cards)

| # | Name | Cost | Stats | Keywords | Ability | Notes |
|---|------|------|-------|----------|---------|-------|
| 28 | Guardian Angel | 6 | 4/6 | Lifesteal, Shield | — | Ultimate stabilizer |
| 29 | Siege Commander | 6 | 5/5 | Rush, Piercing | — | Immediate heavy damage |
| 30 | Tower Sentinel | 7 | 5/9 | Guard | — | The biggest wall |
| 31 | Warlord Titan | 8 | 8/8 | — | OnPlay: Deal 3 damage to all enemy creatures | Board clear finisher |

## 16.2 Spells (8 cards)

| # | Name | Cost | Effect | Notes |
|---|------|------|--------|-------|
| 32 | Quick Strike | 1 | Deal 2 damage to target creature | Cheap removal |
| 33 | Arcane Intellect | 3 | Draw 2 cards | Card advantage |
| 34 | Lightning Bolt | 3 | Deal 4 damage to target creature or enemy hero | Versatile damage |
| 35 | Battle Rage | 2 | Give target creature +3/+1 and Rush this turn | Combat trick |
| 36 | Execute | 2 | Destroy target creature with 4 or less health | Conditional removal |
| 37 | Mass Heal | 4 | Restore 3 health to your hero and all ally creatures | Board-wide heal |
| 38 | Obliterate | 5 | Destroy target creature | Unconditional removal |
| 39 | Flame Wave | 6 | Deal 3 damage to all enemy creatures | Board clear |

## 16.3 Supports (4 cards)

| # | Name | Cost | Durability | Effect | Notes |
|---|------|------|------------|--------|-------|
| 40 | War Drums | 3 | 3 | Your creatures have +1 Attack | Aggro buff |
| 41 | Healing Fountain | 4 | 4 | StartOfTurn: Restore 2 health to your hero | Sustained healing |
| 42 | Tactical Command | 5 | 3 | Your creatures have Rush | Everything attacks immediately |
| 43 | Barrier Field | 4 | 3 | Your creatures have +2 Health | Defensive buff |

---

# 17. SAMPLE DECKS

## 17.1 Deck Construction Rules

- **Deck Size:** 20-30 cards (20 recommended for starters)
- **Card Copies:** Maximum 2 copies of any card per deck
- **No Restrictions:** All cards can be mixed freely

## 17.2 Aggressive Assault Deck (20 cards)

*Strategy: Fast creatures, Rush damage, Piercing to push damage through blockers*

| Quantity | Card Name | Cost | Type |
|----------|-----------|------|------|
| 2 | Eager Recruit | 1 | Creature |
| 2 | Nimble Scout | 1 | Creature |
| 2 | Blood Cultist | 2 | Creature |
| 2 | Shielded Squire | 2 | Creature |
| 2 | Frontier Ranger | 2 | Creature |
| 2 | Centaur Charger | 3 | Creature |
| 2 | Piercing Striker | 3 | Creature |
| 2 | Siege Breaker | 4 | Creature |
| 1 | Siege Commander | 6 | Creature |
| 1 | Warhost Captain | 5 | Creature |
| 2 | Lightning Bolt | 3 | Spell |

**Average Essence Cost:** 2.65 (very low curve)

**Gameplan:**
1. Deploy cheap, aggressive creatures early
2. Use Rush creatures for immediate damage
3. Push damage through blockers with Piercing
4. Finish with Siege Commander or Lightning Bolt to face

## 17.3 Iron Fortress Deck (20 cards)

*Strategy: Survive the early game with Guards, heal through damage, win with big finishers*

| Quantity | Card Name | Cost | Type |
|----------|-----------|------|------|
| 2 | Toxic Spider | 1 | Creature |
| 2 | Iron Defender | 2 | Creature |
| 2 | Medic Apprentice | 2 | Creature |
| 2 | Veteran Guardian | 3 | Creature |
| 2 | Battle Priest | 3 | Creature |
| 2 | Fortress Golem | 4 | Creature |
| 2 | Storm Mage | 4 | Creature |
| 1 | Assassin Queen | 5 | Creature |
| 1 | Royal Champion | 5 | Creature |
| 1 | Guardian Angel | 6 | Creature |
| 1 | Warlord Titan | 8 | Creature |
| 1 | Flame Wave | 6 | Spell |
| 1 | Obliterate | 5 | Spell |

**Average Essence Cost:** 3.55 (higher curve)

**Gameplan:**
1. Wall up with Guard creatures in the early game
2. Heal with Battle Priest and Medic Apprentice
3. Use Toxic Spider and Storm Mage to remove threats
4. Stabilize with Guardian Angel
5. Close the game with Warlord Titan

---

# 18. GLOSSARY

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

---

# 19. QUICK REFERENCE

## 19.1 Turn Structure

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

## 19.2 Action Costs

| Action | Cost |
|--------|------|
| Play any card | 1 AP + Essence Cost |
| Attack with creature | 1 AP |
| End turn | Free |

## 19.3 Lane Attack Ranges

| Your Slot | Attack Range |
|-----------|--------------|
| 1 | Slots 1, 2 |
| 2 | Slots 1, 2, 3 |
| 3 | Slots 2, 3, 4 |
| 4 | Slots 3, 4, 5 |
| 5 | Slots 4, 5 |

## 19.4 Keyword Quick Reference

| Keyword | One-Line Summary |
|---------|------------------|
| Rush | Attack immediately |
| Ranged | Attack any enemy, bypass Guard |
| Piercing | Overkill damage hits face |
| Guard | Force adjacent enemies to attack this |
| Lifesteal | Heal when dealing damage |
| Lethal | Any damage kills creatures |
| Shield | Block first damage, one time |
| Quick | Deal damage first in combat |

## 19.5 Win Conditions

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

**ESSENCE WARS** — A Game of Perfect Information and Strategic Depth

© 2025 — Game Design Document v1.0
