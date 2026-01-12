# Christian! Let's Design Your Combat System! ⚔️

Oh, I'm *so* glad you're vibing with the "Chess with Cards" direction — and yes, the accumulated Victory Points option is genuinely elegant! Having multiple win condition modes as different game variants is a brilliant way to add replayability without complicating the core engine. Start with the simplest (life reduction + tiebreaker), validate everything works, then unlock the other modes. Smart thinking! 🎯

Now, let's dive into the juicy part — **combat design!** You're absolutely right that MTG's declare attackers → declare blockers → damage assignment → responses creates an exponential complexity nightmare. The number of possible attack/block configurations with just 5 creatures on each side is astronomical, and we want none of that chaos here.

Let me walk you through several combat system architectures, analyze them for our AI-friendliness and human enjoyment criteria, and then we can land on something that feels *right*.

---

## 🎭 The Core Combat Philosophy Question

Before we pick a system, we need to decide something fundamental about the *feel* of combat:

**Who has agency during combat?**

| Model | Attacker Agency | Defender Agency | Complexity |
|-------|-----------------|-----------------|------------|
| **Hearthstone** | Full (pick any target) | None (pray they miss you) | Medium |
| **MTG** | Full (choose attackers) | Full (choose blockers) | Very High |
| **Runeterra** | Medium (declare attack) | High (assign blockers to lanes) | High |
| **Lanes (fixed)** | Low (attack your lane) | Low (can't redirect) | Very Low |
| **Lanes (flexible)** | Medium (some targeting) | Low-Medium | Low-Medium |

For your goals, I'm going to advocate for a system where **the active player has meaningful choices, but those choices are bounded**, and **the defending player has limited-to-no reactive decisions during the opponent's turn**. This keeps the game tree manageable while preserving strategic depth.

---

## 🏛️ Combat System Options

### Option A: The Pure Lane System (Most AI-Friendly)

```
         YOUR BOARD
    ┌─────┬─────┬─────┬─────┬─────┐
    │  1  │  2  │  3  │  4  │  5  │  ← Your creature slots
    └──┬──┴──┬──┴──┬──┴──┬──┴──┬──┘
       │     │     │     │     │
       ↓     ↓     ↓     ↓     ↓     ← Combat flows vertically
    ┌──┴──┬──┴──┬──┴──┬──┴──┬──┴──┐
    │  1  │  2  │  3  │  4  │  5  │  ← Enemy creature slots
    └─────┴─────┴─────┴─────┴─────┘
         ENEMY BOARD
                 ↓
           ENEMY FACE
```

**How it works:**
- Each creature slot has a corresponding "lane"
- When a creature attacks, it *must* attack the creature directly across from it
- If the opposing lane is empty, the attack hits the enemy's face directly
- No targeting choice for attacks — completely deterministic given board state

**Action Space:**
```
Attack actions = Number of your creatures that CAN attack
               = Maximum 5 (one per slot)
               = Each is binary: attack or don't attack
```

**Pros:**
- ✅ Incredibly bounded action space (5 binary attack decisions max)
- ✅ Easy for AI to learn — clear spatial reasoning
- ✅ Placement becomes strategic ("which lane do I contest?")
- ✅ Defense is implicit — you defend a lane by placing a creature there
- ✅ Perfect for parallel MCTS simulations

**Cons:**
- ❌ Might feel too deterministic/predictable for humans
- ❌ Less "outplay" potential during combat itself
- ❌ Corner cases: What if your creature is stronger? Weaker? (need rules)

**Human Feel:** Like a tug-of-war across multiple fronts. You're battling for lane control, trying to clear lanes to push damage through.

---

### Option B: Lanes with Adjacent Reach (My Favorite 💜)

```
         YOUR BOARD
    ┌─────┬─────┬─────┬─────┬─────┐
    │  1  │  2  │  3  │  4  │  5  │
    └──┬──┴──┬──┴──┬──┴──┬──┴──┬──┘
       │╲   ╱│╲   ╱│╲   ╱│╲   ╱│
       │ ╲ ╱ │ ╲ ╱ │ ╲ ╱ │ ╲ ╱ │      ← Can attack own lane OR adjacent!
       │  ╳  │  ╳  │  ╳  │  ╳  │
       │ ╱ ╲ │ ╱ ╲ │ ╱ ╲ │ ╱ ╲ │
       ↓╱   ╲↓╱   ╲↓╱   ╲↓╱   ╲↓
    ┌──┴──┬──┴──┬──┴──┬──┴──┬──┴──┐
    │  1  │  2  │  3  │  4  │  5  │
    └─────┴─────┴─────┴─────┴─────┘
         ENEMY BOARD
```

**How it works:**
- Creatures can attack their **own lane** OR **one adjacent lane**
- Lane 1 can attack enemy lanes 1 or 2
- Lane 3 can attack enemy lanes 2, 3, or 4 (center has most reach!)
- If ALL targetable lanes are empty, creature attacks face

**Targeting options per slot:**

| Your Slot | Can Target Enemy Slots | Options |
|-----------|----------------------|---------|
| 1 | 1, 2 | 2 |
| 2 | 1, 2, 3 | 3 |
| 3 | 2, 3, 4 | 3 |
| 4 | 3, 4, 5 | 3 |
| 5 | 4, 5 | 2 |

**Action Space:**
```
Attack actions = Sum of (valid targets per attacking creature)
               = Worst case: 5 creatures × 3 targets average = ~13 attack options
               = But often fewer due to empty lanes
```

**Pros:**
- ✅ Still bounded (max ~15 attack actions total)
- ✅ Creates meaningful attack targeting decisions
- ✅ Positioning matters MORE — center lanes are premium real estate
- ✅ Allows "flanking" strategy — attack weak creatures from the side
- ✅ Defender has implicit agency via placement, not reactive choices

**Cons:**
- ❌ Slightly more complex than pure lanes
- ❌ Center slots might be too dominant (but can balance with keywords!)

**Human Feel:** Like commanding troops on a battlefield — you're managing fronts, flanking, concentrating force. The center is valuable but exposed to more attackers.

---

### Option C: Priority Targeting (No Lanes, Soft Constraints)

```
         YOUR BOARD
    ┌─────┬─────┬─────┬─────┬─────┐
    │  A  │  B  │  C  │  D  │  E  │  ← No fixed positions
    └──┬──┴──┬──┴──┬──┴──┬──┴──┬──┘
       │     │     │     │     │
       └─────┴─────┼─────┴─────┘
                   ↓
           Target Selection
           (with constraints)
                   ↓
    ┌─────┬─────┬─────┬─────┬─────┐
    │  X  │  Y  │  Z  │     │     │  ← Enemy creatures
    └─────┴─────┴─────┴─────┴─────┘
```

**How it works:**
- No lanes — positions don't matter
- Creatures with **Guard** (like Taunt in Hearthstone) MUST be attacked first
- If no Guards exist, you can attack any creature OR face
- Multiple Guards? You can attack any of them (still restricted pool)

**Action Space:**
```
If enemy has Guards:    Targets = Number of Guards (1-5)
If no Guards:           Targets = All enemy creatures + Face (1-6)
Attack actions = Your creatures × Valid targets = up to 5 × 6 = 30
```

**Pros:**
- ✅ Familiar to CCG players (Hearthstone-like)
- ✅ Guard/Taunt is intuitive protection mechanic
- ✅ Flexible targeting allows for skill expression

**Cons:**
- ❌ Higher action space than lane systems
- ❌ Positioning is meaningless (loses a strategic axis)
- ❌ Without lanes, board layout is arbitrary

**Human Feel:** Classic CCG combat. Satisfying when you find lethal through Guard creatures.

---

### Option D: Front Row / Back Row (Simplified Positioning)

```
         YOUR BOARD
    ┌─────────────────────────────┐
    │     BACK ROW (Protected)    │  ← Can't be attacked while front exists
    │   [  ] [  ] [  ]            │     (unless Ranged attackers)
    ├─────────────────────────────┤
    │     FRONT ROW (Exposed)     │  ← Must be attacked first
    │   [  ] [  ] [  ] [  ] [  ]  │
    └─────────────────────────────┘
```

**How it works:**
- Board has 2 rows: Front (3-5 slots) and Back (2-3 slots)
- Front row creatures MUST be cleared before attacking back row or face
- Back row is safe but has limited slots
- Some creatures have **Ranged**: can attack back row directly

**Action Space:**
```
If front row has creatures: Targets = Front row creatures only
If front row empty:         Targets = Back row + Face
Significantly bounded by row mechanics
```

**Pros:**
- ✅ Creates tank/damage-dealer dynamics naturally
- ✅ Bounded targeting (row restrictions)
- ✅ Strategic depth in placement (protect key creatures)

**Cons:**
- ❌ Less granular than lanes
- ❌ Might feel like "solved" — always protect your best creature?
- ❌ Ranged keyword adds complexity

**Human Feel:** Like protecting your healers behind your tanks in an RPG. Intuitive for gamers.

---

## 📊 Comparison Matrix

| Criteria | Pure Lanes | Adjacent Lanes | Priority Target | Front/Back |
|----------|-----------|----------------|-----------------|------------|
| **Max Attack Actions** | 5 | ~15 | ~30 | ~15 |
| **AI Trainability** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Human Skill Expression** | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Positional Strategy** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐ |
| **Simplicity** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Comeback Potential** | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |

---

## 💜 My Recommendation: Option B — Lanes with Adjacent Reach

Here's why I think this is your sweet spot, Christian:

1. **Bounded but Meaningful:** ~15 max attack actions is totally tractable for MCTS and AlphaZero while still offering real choices.

2. **Emergent Strategy:** The lane system creates natural strategic concepts:
   - "Lane control" — dominating specific lanes
   - "Flanking" — attacking from the side
   - "Center pressure" — the mid-lane tension
   - "Wide vs. tall" — spreading creatures vs. stacking power

3. **Positional Keywords:** You can design creatures with positioning abilities:
   - **Ranged:** Can attack ANY enemy lane (not just adjacent)
   - **Immobile:** Cannot attack adjacent lanes, only its own
   - **Sweeping:** Attacks ALL valid lanes simultaneously
   - **Shift:** Can move to an adjacent empty slot as an action

4. **Visual Clarity:** The board state is immediately readable — both for humans staring at a screen AND for neural networks looking at a state tensor.

5. **Defense Without Reaction:** You "defend" by placing creatures in contested lanes. No reactive blocking phase needed!

---

## ⚔️ Detailed Combat Mechanics for Adjacent Lane System

Let me flesh this out into concrete rules:

### Combat Flow

```
COMBAT IS NOT A SEPARATE PHASE!
Each attack is an individual Action Point expenditure.

┌─────────────────────────────────────────────────────────────┐
│  TURN STRUCTURE                                             │
│                                                             │
│  Start of Turn                                              │
│    → Gain Mana (automatic)                                  │
│    → Draw Card (automatic)                                  │
│    → Refresh all creatures (remove "exhausted" status)      │
│                                                             │
│  Main Phase (spend Action Points freely)                    │
│    → Play a card from hand (costs AP + Mana)               │
│    → Attack with a creature (costs 1 AP, creature exhausts)│
│    → Activate an ability (costs AP as specified)           │
│    → Pass (ends turn, forfeits remaining AP)               │
│                                                             │
│  No separate "combat phase" — attacks interleave with plays │
└─────────────────────────────────────────────────────────────┘
```

### Attack Resolution

```rust
// Pseudocode for attack resolution
fn resolve_attack(attacker: Creature, target: Target) -> AttackResult {
    match target {
        Target::Creature(defender) => {
            // Simultaneous damage exchange!
            defender.health -= attacker.attack;
            attacker.health -= defender.attack;
            
            // Check deaths
            let attacker_died = attacker.health <= 0;
            let defender_died = defender.health <= 0;
            
            // Trigger on-death effects if applicable
            // ...
        }
        Target::Face => {
            // Direct damage to enemy life total
            enemy.life -= attacker.attack;
            // Attacker takes no damage
        }
    }
    
    // Mark attacker as exhausted (can't attack again this turn)
    attacker.exhausted = true;
}
```

### Key Decision: Simultaneous vs. Attacker-First Damage

We have two options here:

**Simultaneous Damage (MTG-style):**
```
Your 3/2 attacks their 2/3
Result: Your creature takes 2 damage (survives at 3/0... wait, that's wrong)

Let me redo:
Your 3/3 attacks their 2/2
Result: Both deal damage simultaneously
        Your creature: 3/3 → takes 2 → 3/1 (survives)
        Their creature: 2/2 → takes 3 → 2/-1 (dies)
```

**Attacker-First (Hearthstone-style... actually HS is simultaneous too):**

You know what, both major CCGs use simultaneous! Let's go with that — it's intuitive and creates interesting trades.

### Damage Persistence

Another key question: **Does damage heal at end of turn?**

| Option | Pros | Cons |
|--------|------|------|
| **Damage persists** (MTG) | Creates attrition gameplay, wounded creatures matter | Board states more complex, harder to track |
| **Damage heals** (Hearthstone) | Simpler board state, cards feel more "fresh" each turn | Less attrition, harder to wear down big creatures |

**My recommendation:** **Damage persists.** It creates more strategic depth (weakening creatures matters!) and fits the "Chess with Cards" vibe better. Your creatures feel more like persistent units on a battlefield.

### Summoning Sickness

**Should newly played creatures be able to attack immediately?**

| Option | Pros | Cons |
|--------|------|------|
| **Summoning Sickness** (can't attack on play) | Defensive play is viable, placement is commitment | Slower gameplay, charge/haste becomes keyword |
| **No Sickness** (can attack immediately) | Aggressive, dynamic, immediate impact | Hard to stabilize, snowbally |

**My recommendation:** **Summoning Sickness by default.** This makes placement a real commitment ("I'm putting this creature HERE, it'll be stuck defending this lane until my next turn"). You can then have a **Rush** keyword for creatures that CAN attack immediately.

### Can Creatures Attack Face Directly?

With the adjacent lane system, we need to clarify when face attacks are allowed:

**Option 1: Only through empty lanes**
```
Your creature in Slot 3 can attack face ONLY if enemy slots 2, 3, AND 4 are all empty.
```
This is very defensive — face is well protected.

**Option 2: Can always choose face OR creatures**
```
Your creature can always attack face as an option, but attacking creatures first might be strategically better.
```
This is very aggressive — face is always vulnerable.

**Option 3: Must clear your lane first (Recommended)**
```
Your creature in Slot 3 can attack face IF enemy slot 3 is empty.
Adjacent slots (2, 4) don't block face attacks, only direct lane.
```

I recommend **Option 3** — it creates clear "breakthrough" moments when you clear a lane, but adjacent creatures still threaten each other.

---

## 🎮 Complete Combat Example

Let me illustrate how a turn might play out:

```
GAME STATE AT START OF YOUR TURN:

Your Board:                     Your Stats:
[2/3] [   ] [4/2] [   ] [3/1]   Life: 24
  1     2     3     4     5     Mana: 5
                                AP: 3
  ↕     ↕     ↕     ↕     ↕     Hand: 4 cards

[1/1] [3/4] [   ] [2/2] [   ]   Enemy Stats:
  1     2     3     4     5     Life: 18
Enemy Board:

LEGAL ATTACK ACTIONS:
• Creature in Slot 1 (2/3): Can attack → Enemy 1 or Enemy 2
• Creature in Slot 3 (4/2): Can attack → Face (slot 3 empty!) or Enemy 2 or Enemy 4
• Creature in Slot 5 (3/1): Can attack → Enemy 4 or Face (slot 5 empty!)

YOUR TURN:
1. [1 AP] Attack with Slot 3 creature (4/2) → Enemy Face
   → Enemy takes 4 damage! (Life: 18 → 14)
   → Slot 3 creature now exhausted

2. [1 AP] Play "Goblin Scout" (2/1, costs 2 mana) → Place in Slot 2
   → Mana: 5 → 3
   → Slot 2 now has creature (but summoning sickness!)

3. [1 AP] Attack with Slot 5 creature (3/1) → Enemy Slot 4 (2/2)
   → Simultaneous damage:
      Your 3/1: takes 2 damage → dies (3/-1)
      Their 2/2: takes 3 damage → dies (2/-1)
   → Both creatures removed!

END TURN (0 AP remaining)
```

---

## 📋 Combat System Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COMBAT SYSTEM SPECIFICATION                      │
├─────────────────────────────────────────────────────────────────────┤
│  BOARD LAYOUT                                                       │
│  • 5 creature slots per player, arranged in lanes                  │
│  • 2 support slots per player (position doesn't affect combat)     │
│  • Slot positions: 1 (left edge) through 5 (right edge)            │
│                                                                     │
│  ATTACK TARGETING                                                   │
│  • Creatures attack their own lane OR adjacent lanes               │
│  • Slot 1: targets enemy 1, 2                                      │
│  • Slot 2: targets enemy 1, 2, 3                                   │
│  • Slot 3: targets enemy 2, 3, 4                                   │
│  • Slot 4: targets enemy 3, 4, 5                                   │
│  • Slot 5: targets enemy 4, 5                                      │
│  • If your direct lane is empty, can attack face                   │
│                                                                     │
│  COMBAT RESOLUTION                                                  │
│  • Attacking costs 1 Action Point                                  │
│  • Damage is simultaneous (both creatures deal damage)             │
│  • Creatures with 0 or less health die immediately                 │
│  • Damage persists (does not heal at end of turn)                  │
│                                                                     │
│  CREATURE STATUS                                                    │
│  • Summoning Sickness: Can't attack on the turn played            │
│  • Exhausted: Can't attack again this turn (after attacking)       │
│  • Both statuses clear at start of your turn                       │
│                                                                     │
│  COMBAT KEYWORDS (Examples)                                         │
│  • Rush: Ignores summoning sickness                                │
│  • Ranged: Can attack ANY enemy slot (not just adjacent)           │
│  • Piercing: Excess damage goes to face when killing creature      │
│  • Guard: Enemies in adjacent slots MUST attack this first         │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🎯 What's Next?

With combat nailed down, I think we should tackle one of these next:

1. **Card Anatomy & Stats:** What numbers do creatures have? What makes a spell a spell?

2. **Mana/Resource System:** Starting mana, gain rate, card costs — the economic backbone.

3. **Effect Taxonomy:** What can cards actually DO? (Damage, heal, draw, buff, summon, etc.)

4. **Keywords Deep Dive:** Design the 10-15 core keywords that create gameplay variety.

I'm personally leaning toward **Card Anatomy** next — once we know what a "creature" looks like as data, everything else flows from that. But you know your priorities better than I do!

What calls to you? 💜