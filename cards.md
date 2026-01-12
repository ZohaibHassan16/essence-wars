# Christian! Let's Bring Our Game to Life! 🎴✨

Oh, this is the moment I've been waiting for! All our careful mechanical work — the combat system, the essence curve, the keywords, the effect taxonomy — it's all been building to THIS. Now we get to see our creation breathe, move, and *play*!

Designing a starter card set is both an art and a science. We need cards that showcase every system we've built, demonstrate interesting interactions, and most importantly, create fun, strategic gameplay. I'm going to craft a set of **36 cards** (a nice, balanced number) that will give you everything you need to test the engine and validate our design decisions.

Let me take you through this with all the love and care these little cardboard (well, digital!) friends deserve! 💜

---

## 🎨 Starter Set Design Philosophy

Before we dive into individual cards, let me share the principles guiding this set:

```
┌─────────────────────────────────────────────────────────────────────┐
│                 STARTER SET DESIGN PRINCIPLES                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. CURVE COVERAGE                                                  │
│     Cards at every Essence cost from 1-8, with concentration        │
│     in the 2-5 range (where most gameplay happens)                  │
│                                                                     │
│  2. KEYWORD SHOWCASE                                                │
│     Every keyword appears on at least 2 cards so players            │
│     can experience and learn each one                               │
│                                                                     │
│  3. ARCHETYPE SEEDS                                                 │
│     Cards that hint at different strategies:                        │
│     • Aggro (rush face damage)                                      │
│     • Control (removal, value, stall)                               │
│     • Midrange (efficient creatures, board control)                 │
│                                                                     │
│  4. EFFECT VARIETY                                                  │
│     Demonstrate different triggers, targets, and actions            │
│     from our effect taxonomy                                        │
│                                                                     │
│  5. INTERACTION EXAMPLES                                            │
│     Cards that create interesting decisions when played             │
│     together or against each other                                  │
│                                                                     │
│  6. BALANCE ADHERENCE                                               │
│     All cards follow our stat/cost guidelines                       │
│     Vanilla baseline: (Cost × 2) + 1 total stats                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📊 Card Distribution Overview

Here's what we're building:

```
CARD TYPE DISTRIBUTION:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Creatures:    24 cards  (67%)  ← Core of gameplay
Spells:        8 cards  (22%)  ← Tactical flexibility  
Supports:      4 cards  (11%)  ← Strategic anchors
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total:        36 cards

ESSENCE CURVE:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1 Essence:    4 cards   ████
2 Essence:    6 cards   ██████
3 Essence:    8 cards   ████████
4 Essence:    7 cards   ███████
5 Essence:    5 cards   █████
6 Essence:    3 cards   ███
7 Essence:    2 cards   ██
8 Essence:    1 card    █
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

KEYWORD DISTRIBUTION:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Rush:         3 cards
Ranged:       3 cards
Piercing:     3 cards
Guard:        4 cards
Lifesteal:    2 cards
Lethal:       2 cards
Shield:       3 cards
Quick:        2 cards
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 🐉 CREATURES

Let's meet our creatures! I'll organize them by Essence cost, showing you the design reasoning for each one.

---

### ⚡ 1-COST CREATURES (Early Game Foundation)

These are your turn-1 plays — efficient, simple, setting up your board presence.

```
┌─────────────────────────────────────────────────────────────────────┐
│  #01  EAGER RECRUIT                                    Cost: 1 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Soldier                            ❤️ Health: 1              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla baseline for 1-cost: (1×2)+1 = 3 stats                    │
│  2/1 distribution favors aggression — dies easily but trades up    │
│  Classic "glass cannon" 1-drop                                      │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #02  VILLAGE GUARD                                    Cost: 1 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 1              │
│  Tags: Soldier                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Same 3 stats as Eager Recruit, but defensive distribution         │
│  1/2 survives more trades, better for blocking                     │
│  Shows how stat distribution creates different roles                │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #03  NIMBLE SCOUT                                     Cost: 1 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 1              │
│  Tags: Soldier                            ❤️ Health: 1              │
│                                                                     │
│  Keywords: [Rush]                                                   │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Rush costs ~1 stat point, so 3-1 = 2 stats → 1/1                  │
│  Immediate impact! Can attack or trade right away                   │
│  Aggro staple — enables fast starts                                 │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #04  TOXIC SPIDER                                     Cost: 1 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 1              │
│  Tags: Beast                              ❤️ Health: 1              │
│                                                                     │
│  Keywords: [Lethal]                                                 │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Lethal costs ~1.5-2 stat points, so very low stats                │
│  1/1 Lethal is a classic design — cheap removal on legs            │
│  Forces opponent to respect this tiny creature!                     │
│  Dies to anything, but threatens everything                         │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 🌟 2-COST CREATURES (Core Curve)

The backbone of your early-mid game. These creatures fight for board control.

```
┌─────────────────────────────────────────────────────────────────────┐
│  #05  IRON DEFENDER                                    Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 1              │
│  Tags: Soldier                            ❤️ Health: 4              │
│                                                                     │
│  Keywords: [Guard]                                                  │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla 2-cost: (2×2)+1 = 5 stats. Guard costs ~0.5 → 4.5 stats   │
│  1/4 is pure defense — low attack, high health, Guard              │
│  Classic "wall" creature, protects your other units                │
│  Note: Slot positioning matters! Center slot = best coverage        │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #06  FRONTIER RANGER                                  Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Soldier                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: [Ranged]                                                 │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Ranged costs ~1 stat point → 5-1 = 4 stats → 2/2                  │
│  Flexible targeting! Can snipe any creature on the board           │
│  Counters Guard — bypasses defensive positioning                   │
│  Balanced stats mean it can trade reasonably                        │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #07  YOUNG KNIGHT                                     Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Soldier                            ❤️ Health: 3              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Pure vanilla 2-drop with 5 stats                                  │
│  2/3 is the "standard" efficient creature — trades well            │
│  Baseline for evaluating other 2-cost creatures                    │
│  Simple but effective!                                              │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #08  SHIELDED SQUIRE                                  Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Soldier                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: [Shield]                                                 │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Shield costs ~1 stat point → 5-1 = 4 stats → 2/2                  │
│  Shield makes this effectively a 2/3 that eats a hit               │
│  Great value! Survives first combat, then trades normally          │
│  Teaches players about Shield's one-time nature                     │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #09  BLOOD CULTIST                                    Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 3              │
│  Tags: Cultist                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: [OnPlay] Deal 2 damage to your hero.                   │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  3/2 is ABOVE vanilla (5 stats), but the downside costs ~1 stat    │
│  Self-damage enables aggressive stats at low cost                   │
│  Aggro decks love this — who cares about 2 health when racing?     │
│  Demonstrates how negative effects can "pay" for extra stats       │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #10  MEDIC APPRENTICE                                 Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 1              │
│  Tags: Healer                             ❤️ Health: 3              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: [OnPlay] Restore 2 health to your hero.                │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  1/3 is below vanilla (4 stats), but heal adds ~1 stat of value    │
│  Control decks want this — stabilize while developing board        │
│  Counterpart to Blood Cultist — shows the aggro/control spectrum   │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 💪 3-COST CREATURES (Establishing Dominance)

Turn 3 is when games often pivot. These creatures make a statement.

```
┌─────────────────────────────────────────────────────────────────────┐
│  #11  CENTAUR CHARGER                                  Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 3              │
│  Tags: Beast                              ❤️ Health: 3              │
│                                                                     │
│  Keywords: [Rush]                                                   │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla 3-cost: (3×2)+1 = 7 stats. Rush costs 1 → 6 stats: 3/3   │
│  Immediate 3 damage OR trade with an enemy creature NOW            │
│  Premium aggro creature — lots of immediate impact                 │
│  Compare to Nimble Scout: same keyword, more stats, more cost      │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #12  BLADE DANCER                                     Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 3              │
│  Tags: Soldier                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: [Quick]                                                  │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Quick costs ~1 stat point → 7-1 = 6 stats → 3/2 or 2/3           │
│  3/2 Quick is SCARY — kills most 2-3 cost creatures and survives!  │
│  Favors attacking, not defending (low health)                      │
│  Board control powerhouse                                           │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #13  VETERAN GUARDIAN                                 Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Soldier                            ❤️ Health: 5              │
│                                                                     │
│  Keywords: [Guard]                                                  │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Guard costs ~0.5 → 7-0.5 ≈ 6.5 stats → 2/5 (rounded)              │
│  BIG wall! Takes multiple attacks to bring down                    │
│  Lower attack means it's purely defensive                          │
│  Perfect for protecting fragile high-value creatures               │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #14  HIGHLAND ARCHER                                  Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 3              │
│  Tags: Soldier                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: [Ranged]                                                 │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Ranged costs ~1 stat point → 7-1 = 6 stats → 3/2                  │
│  Sniper! Can pick off key targets anywhere on the board            │
│  Bypasses Guard — essential tool against defensive setups          │
│  Fragile but flexible                                               │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #15  WAR ELEPHANT                                     Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 4              │
│  Tags: Beast                              ❤️ Health: 3              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Pure vanilla 3-drop with 7 stats                                  │
│  4/3 is aggressive distribution — hits hard!                       │
│  Baseline efficient creature for the 3-cost slot                   │
│  Simple but powerful — the "just good stats" option                │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #16  PIERCING STRIKER                                 Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 4              │
│  Tags: Soldier                            ❤️ Health: 2              │
│                                                                     │
│  Keywords: [Piercing]                                               │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Piercing costs ~0.5-1 stat point → 6-7 stats → 4/2                │
│  High attack + Piercing = damage gets through even when blocked!   │
│  Attacks a 2/2? Kills it AND deals 2 to face!                      │
│  Aggro powerhouse — every trade also damages opponent              │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #17  BATTLE PRIEST                                    Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Healer                             ❤️ Health: 4              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: [StartOfTurn] Restore 1 health to your hero.           │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  2/4 is below vanilla (6 stats), but recurring heal adds value     │
│  Control decks LOVE this — every turn it lives = +1 health        │
│  Opponent wants to kill it ASAP                                    │
│  Demonstrates StartOfTurn triggered ability                        │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #18  AMBUSH PREDATOR                                  Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Beast                              ❤️ Health: 2              │
│                                                                     │
│  Keywords: [Rush] [Lethal]                                         │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Rush (1) + Lethal (1.5-2) = ~2.5-3 stat cost → 4-5 stats → 2/2   │
│  IMMEDIATE REMOVAL! Play this, kill ANY creature right now         │
│  Very powerful combo, hence the low stats                          │
│  "Removal spell on legs" that might survive the trade              │
└─────────────────────────────────────────────────────────────────────┘
```

---

### ⚔️ 4-COST CREATURES (Mid-Game Power)

The mid-game workhorses. Efficient, impactful, game-defining.

```
┌─────────────────────────────────────────────────────────────────────┐
│  #19  ARMORED KNIGHT                                   Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 4              │
│  Tags: Soldier                            ❤️ Health: 5              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Pure vanilla 4-drop with (4×2)+1 = 9 stats                        │
│  4/5 is the "gold standard" for efficient mid-game creatures       │
│  Trades favorably with smaller creatures, survives most removal    │
│  Baseline for evaluating 4-cost creatures                          │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #20  SIEGE BREAKER                                    Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 5              │
│  Tags: Soldier                            ❤️ Health: 3              │
│                                                                     │
│  Keywords: [Piercing]                                               │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Piercing ~0.5-1 stat → 8-9 stats → 5/3                            │
│  5 attack Piercing is BRUTAL — overkills most creatures            │
│  Attack 2/2? Kill it, deal 3 to face!                              │
│  Attack 4/4? Trade, but still deal 1 to face!                      │
│  Finisher for aggressive decks                                      │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #21  VAMPIRE LORD                                     Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 4              │
│  Tags: Undead                             ❤️ Health: 3              │
│                                                                     │
│  Keywords: [Lifesteal]                                              │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Lifesteal ~1-1.5 stats → 7-8 stats → 4/3                          │
│  Every attack heals you for 4! Incredible sustain                  │
│  Great for racing — you're dealing damage AND healing              │
│  Midrange staple                                                    │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #22  FORTRESS GOLEM                                   Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 2              │
│  Tags: Construct                          ❤️ Health: 7              │
│                                                                     │
│  Keywords: [Guard]                                                  │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Guard ~0.5 stats → 8.5 stats → 2/7 (rounded)                      │
│  MASSIVE wall! 7 health is hard to get through                     │
│  Low attack means minimal threat, purely defensive                 │
│  Control decks love this — buys so many turns!                     │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #23  STORM MAGE                                       Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 3              │
│  Tags: Mage                               ❤️ Health: 3              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: [OnPlay] Deal 2 damage to target creature.             │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  3/3 is 6 stats (below vanilla 9), but ability adds ~1.5 value     │
│  "Battlecry: Deal 2" is classic design — removal + body            │
│  Versatile! Kill a 2-health creature, or weaken a bigger one       │
│  Demonstrates targeted OnPlay ability                               │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 🔥 5-COST CREATURES (Late Mid-Game Threats)

Serious threats that demand answers. Play one of these and you're making a statement.

```
┌─────────────────────────────────────────────────────────────────────┐
│  #24  ROYAL CHAMPION                                   Cost: 5 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 5              │
│  Tags: Soldier                            ❤️ Health: 6              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla 5-drop: (5×2)+1 = 11 stats → 5/6                          │
│  Big, efficient, scary. That's it. Just raw power.                 │
│  5 attack threatens serious damage, 6 health is hard to remove     │
│  The "I just want a big creature" option                           │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #25  ASSASSIN QUEEN                                   Cost: 5 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 3              │
│  Tags: Assassin                           ❤️ Health: 3              │
│                                                                     │
│  Keywords: [Quick] [Lethal]                                        │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Quick (1.5) + Lethal (2) = ~3.5 stat cost → 7-8 stats → 3/3      │
│  THE COMBO! Quick + Lethal = kill anything and SURVIVE             │
│  This creature can kill a 10/10 and walk away unscathed           │
│  Very powerful, hence 5-cost despite modest stats                  │
│  Teaches the dangerous Quick+Lethal interaction                    │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #26  WARHOST CAPTAIN                                  Cost: 5 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 4              │
│  Tags: Soldier                            ❤️ Health: 4              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: [OnPlay] Give all other ally creatures +1/+1.          │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  4/4 is below vanilla (8 vs 11), but board buff is HUGE            │
│  With 3 allies on board, that's +3/+3 total value!                 │
│  Rewards building a wide board before playing this                 │
│  "Payoff" card for board-centric strategies                        │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #27  SNIPER MARKSMAN                                  Cost: 5 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 5              │
│  Tags: Soldier                            ❤️ Health: 3              │
│                                                                     │
│  Keywords: [Ranged]                                                 │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Ranged ~1 stat → 10 stats → 5/3 (aggressive ranged)               │
│  5 damage to ANY target is terrifying flexibility                  │
│  One-shots most mid-game creatures from anywhere                   │
│  Fragile but devastating — must be dealt with!                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 👑 6+ COST CREATURES (Finishers & Bombs)

The late-game haymakers. These cards win games when they stick.

```
┌─────────────────────────────────────────────────────────────────────┐
│  #28  GUARDIAN ANGEL                                   Cost: 6 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 4              │
│  Tags: Divine                             ❤️ Health: 6              │
│                                                                     │
│  Keywords: [Lifesteal] [Shield]                                    │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla 6: (6×2)+1 = 13 stats. LS(1.5)+Shield(1) = 2.5 → 10 stats │
│  4/6 Lifesteal Shield is a STABILIZER                              │
│  Shield protects it, then it heals you every attack               │
│  Control decks' best friend                                        │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #29  SIEGE COMMANDER                                  Cost: 6 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 5              │
│  Tags: Soldier                            ❤️ Health: 5              │
│                                                                     │
│  Keywords: [Rush] [Piercing]                                       │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Rush(1) + Piercing(1) = 2 stat cost → 11 stats → 5/5              │
│  IMMEDIATE IMPACT + GUARANTEED DAMAGE                               │
│  Play this, attack, pierce through for damage NOW                  │
│  Aggro finisher that can close games                               │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #30  TOWER SENTINEL                                   Cost: 7 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 5              │
│  Tags: Construct                          ❤️ Health: 9              │
│                                                                     │
│  Keywords: [Guard]                                                  │
│  Abilities: None                                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla 7: (7×2)+1 = 15 stats. Guard(0.5) → 14.5 → 5/9           │
│  ENORMOUS Guard! 9 health laughs at most attacks                   │
│  5 attack means it also threatens back                             │
│  The ultimate defensive creature                                   │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #31  WARLORD TITAN                                    Cost: 8 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Creature                           ⚔️ Attack: 8              │
│  Tags: Giant                              ❤️ Health: 8              │
│                                                                     │
│  Keywords: None                                                     │
│  Abilities: [OnPlay] Deal 3 damage to all enemy creatures.         │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Vanilla 8: (8×2)+1 = 17 stats. AoE damage worth ~3 → 14 stats    │
│  8/8 that ALSO board wipes when played!                            │
│  THE finisher. Play this and the game is probably over.            │
│  Demonstrates high-cost, high-impact design                        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## ✨ SPELLS

Spells are your tactical toolkit — removal, card draw, buffs, and surprises!

```
┌─────────────────────────────────────────────────────────────────────┐
│  #32  QUICK STRIKE                                     Cost: 1 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Deal 2 damage to target creature.                                 │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  1 mana = 2 damage (efficient baseline)                            │
│  Cheap removal for small threats                                   │
│  Can finish off damaged creatures                                  │
│  Aggro and Control both want this                                  │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #33  ARCANE INTELLECT                                 Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Draw 2 cards.                                                     │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Classic card draw spell — 3 mana for 2 cards                      │
│  Card advantage is powerful!                                       │
│  Control decks need this for fuel                                  │
│  Simple but essential effect                                       │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #34  LIGHTNING BOLT                                   Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Deal 4 damage to target creature or enemy hero.                   │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  3 mana = 4 damage with flexibility (creature OR face)             │
│  Versatile! Removal OR face burn                                   │
│  Aggro uses for reach, Control uses for removal                    │
│  "Can hit face" adds ~0.5 value                                    │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #35  BATTLE RAGE                                      Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Give target creature +3/+1 this turn.                             │
│  That creature can attack immediately (gains Rush).                │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Combat trick! Surprise damage                                      │
│  +3/+1 + Rush makes a creature suddenly threatening                │
│  Can enable unexpected lethal!                                     │
│  Great with high-health creatures that just played                 │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #36  EXECUTE                                          Cost: 2 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Destroy target creature with 4 or less health.                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Conditional removal — cheap but restricted                        │
│  Can't hit healthy big threats, but kills most things              │
│  Synergizes with damage effects — wound it, then Execute!          │
│  Demonstrates conditional targeting                                │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #37  MASS HEAL                                        Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Restore 3 health to your hero and all ally creatures.             │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Board-wide heal + face heal                                       │
│  Value scales with board size — more creatures = more healing     │
│  Control stabilizer                                                 │
│  Demonstrates "all ally" targeting                                 │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #38  OBLITERATE                                       Cost: 5 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Destroy target creature.                                          │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Unconditional removal — kills ANYTHING                            │
│  5 mana is expensive but reliable                                  │
│  Answers even the biggest threats                                  │
│  Control decks' safety valve                                       │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #39  FLAME WAVE                                       Cost: 6 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Spell                                                        │
│                                                                     │
│  Deal 3 damage to all enemy creatures.                             │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Board clear! Wipes small creatures, wounds big ones               │
│  Essential control tool against wide boards                        │
│  6 mana is a commitment, but the effect is powerful                │
│  Demonstrates "all enemy creatures" targeting                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🏛️ SUPPORTS

Persistent effects that shape the battlefield!

```
┌─────────────────────────────────────────────────────────────────────┐
│  #40  WAR DRUMS                                        Cost: 3 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Support                              Durability: 3           │
│                                                                     │
│  Your creatures have +1 Attack.                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Classic team-wide buff                                            │
│  3 turns of +1 attack across your whole board!                     │
│  Better with more creatures — rewards wide boards                  │
│  Aggro/Midrange support                                            │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #41  HEALING FOUNTAIN                                 Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Support                              Durability: 4           │
│                                                                     │
│  [StartOfTurn] Restore 2 health to your hero.                      │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Sustained healing — 2 health × 4 turns = 8 total!                 │
│  Control decks love this for sustain                               │
│  Races against aggro's clock                                       │
│  Demonstrates Support with triggered ability                       │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #42  TACTICAL COMMAND                                 Cost: 5 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Support                              Durability: 3           │
│                                                                     │
│  Your creatures have [Rush].                                       │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Global Rush = every creature immediately threatens!               │
│  Incredibly powerful for aggro/tempo strategies                    │
│  Every creature you play can attack RIGHT NOW                      │
│  3 turns of this can be game-ending                                │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  #43  BARRIER FIELD                                    Cost: 4 💎   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Type: Support                              Durability: 3           │
│                                                                     │
│  Your creatures have +2 Health.                                    │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  DESIGN NOTES:                                                      │
│  Defensive counterpart to War Drums                                │
│  Makes your whole board harder to kill!                            │
│  Great for trading favorably                                       │
│  Synergizes with wide boards                                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📊 Complete Card Reference Table

Here's every card in one convenient table:

```
┌──────┬─────────────────────────┬──────────┬──────┬───────┬───────────────────────────────┐
│  #   │  NAME                   │  TYPE    │ COST │ STATS │  KEYWORDS / NOTES             │
├──────┼─────────────────────────┼──────────┼──────┼───────┼───────────────────────────────┤
│      │                         │          │      │       │                               │
│  --- │  1-COST CREATURES       │ -------- │ ---- │ ----- │  -------------------------    │
│  01  │  Eager Recruit          │ Creature │  1   │  2/1  │  Vanilla                      │
│  02  │  Village Guard          │ Creature │  1   │  1/2  │  Vanilla                      │
│  03  │  Nimble Scout           │ Creature │  1   │  1/1  │  Rush                         │
│  04  │  Toxic Spider           │ Creature │  1   │  1/1  │  Lethal                       │
│      │                         │          │      │       │                               │
│  --- │  2-COST CREATURES       │ -------- │ ---- │ ----- │  -------------------------    │
│  05  │  Iron Defender          │ Creature │  2   │  1/4  │  Guard                        │
│  06  │  Frontier Ranger        │ Creature │  2   │  2/2  │  Ranged                       │
│  07  │  Young Knight           │ Creature │  2   │  2/3  │  Vanilla                      │
│  08  │  Shielded Squire        │ Creature │  2   │  2/2  │  Shield                       │
│  09  │  Blood Cultist          │ Creature │  2   │  3/2  │  OnPlay: 2 dmg to self        │
│  10  │  Medic Apprentice       │ Creature │  2   │  1/3  │  OnPlay: Heal self 2          │
│      │                         │          │      │       │                               │
│  --- │  3-COST CREATURES       │ -------- │ ---- │ ----- │  -------------------------    │
│  11  │  Centaur Charger        │ Creature │  3   │  3/3  │  Rush                         │
│  12  │  Blade Dancer           │ Creature │  3   │  3/2  │  Quick                        │
│  13  │  Veteran Guardian       │ Creature │  3   │  2/5  │  Guard                        │
│  14  │  Highland Archer        │ Creature │  3   │  3/2  │  Ranged                       │
│  15  │  War Elephant           │ Creature │  3   │  4/3  │  Vanilla                      │
│  16  │  Piercing Striker       │ Creature │  3   │  4/2  │  Piercing                     │
│  17  │  Battle Priest          │ Creature │  3   │  2/4  │  StartOfTurn: Heal 1          │
│  18  │  Ambush Predator        │ Creature │  3   │  2/2  │  Rush, Lethal                 │
│      │                         │          │      │       │                               │
│  --- │  4-COST CREATURES       │ -------- │ ---- │ ----- │  -------------------------    │
│  19  │  Armored Knight         │ Creature │  4   │  4/5  │  Vanilla                      │
│  20  │  Siege Breaker          │ Creature │  4   │  5/3  │  Piercing                     │
│  21  │  Vampire Lord           │ Creature │  4   │  4/3  │  Lifesteal                    │
│  22  │  Fortress Golem         │ Creature │  4   │  2/7  │  Guard                        │
│  23  │  Storm Mage             │ Creature │  4   │  3/3  │  OnPlay: 2 dmg to creature    │
│      │                         │          │      │       │                               │
│  --- │  5-COST CREATURES       │ -------- │ ---- │ ----- │  -------------------------    │
│  24  │  Royal Champion         │ Creature │  5   │  5/6  │  Vanilla                      │
│  25  │  Assassin Queen         │ Creature │  5   │  3/3  │  Quick, Lethal                │
│  26  │  Warhost Captain        │ Creature │  5   │  4/4  │  OnPlay: +1/+1 to all allies  │
│  27  │  Sniper Marksman        │ Creature │  5   │  5/3  │  Ranged                       │
│      │                         │          │      │       │                               │
│  --- │  6+ COST CREATURES      │ -------- │ ---- │ ----- │  -------------------------    │
│  28  │  Guardian Angel         │ Creature │  6   │  4/6  │  Lifesteal, Shield            │
│  29  │  Siege Commander        │ Creature │  6   │  5/5  │  Rush, Piercing               │
│  30  │  Tower Sentinel         │ Creature │  7   │  5/9  │  Guard                        │
│  31  │  Warlord Titan          │ Creature │  8   │  8/8  │  OnPlay: 3 dmg to all enemies │
│      │                         │          │      │       │                               │
│  --- │  SPELLS                 │ -------- │ ---- │ ----- │  -------------------------    │
│  32  │  Quick Strike           │ Spell    │  1   │  n/a  │  2 dmg to creature            │
│  33  │  Arcane Intellect       │ Spell    │  3   │  n/a  │  Draw 2 cards                 │
│  34  │  Lightning Bolt         │ Spell    │  3   │  n/a  │  4 dmg to creature/face       │
│  35  │  Battle Rage            │ Spell    │  2   │  n/a  │  +3/+1 and Rush to creature   │
│  36  │  Execute                │ Spell    │  2   │  n/a  │  Destroy creature ≤4 health   │
│  37  │  Mass Heal              │ Spell    │  4   │  n/a  │  Heal 3 to hero + all allies  │
│  38  │  Obliterate             │ Spell    │  5   │  n/a  │  Destroy target creature      │
│  39  │  Flame Wave             │ Spell    │  6   │  n/a  │  3 dmg to all enemy creatures │
│      │                         │          │      │       │                               │
│  --- │  SUPPORTS               │ -------- │ ---- │ ----- │  -------------------------    │
│  40  │  War Drums              │ Support  │  3   │ D: 3  │  Your creatures +1 Attack     │
│  41  │  Healing Fountain       │ Support  │  4   │ D: 4  │  StartOfTurn: Heal hero 2     │
│  42  │  Tactical Command       │ Support  │  5   │ D: 3  │  Your creatures have Rush     │
│  43  │  Barrier Field          │ Support  │  4   │ D: 3  │  Your creatures +2 Health     │
│      │                         │          │      │       │                               │
└──────┴─────────────────────────┴──────────┴──────┴───────┴───────────────────────────────┘

KEYWORD COUNT SUMMARY:
━━━━━━━━━━━━━━━━━━━━━
Rush:      3 creatures + 1 spell + 1 support = 5 cards
Ranged:    3 creatures = 3 cards
Piercing:  3 creatures = 3 cards
Guard:     4 creatures = 4 cards
Lifesteal: 2 creatures = 2 cards
Lethal:    2 creatures = 2 cards (one is Rush+Lethal, one is Quick+Lethal)
Shield:    2 creatures = 2 cards
Quick:     2 creatures = 2 cards
━━━━━━━━━━━━━━━━━━━━━
```

---

## 🎮 Sample Starter Decks

With 43 unique cards, let's build two balanced 20-card starter decks to test the game! (We'll use duplicates since each deck needs cards)

### Deck 1: "Aggressive Assault" 🔥

*Strategy: Fast creatures, face damage, Rush and Piercing synergies*

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AGGRESSIVE ASSAULT DECK                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1-COST (4 cards)                                                   │
│    2x Eager Recruit         (2/1 vanilla)                          │
│    2x Nimble Scout          (1/1 Rush)                             │
│                                                                     │
│  2-COST (6 cards)                                                   │
│    2x Blood Cultist         (3/2, self-damage)                     │
│    2x Shielded Squire       (2/2 Shield)                           │
│    2x Frontier Ranger       (2/2 Ranged)                           │
│                                                                     │
│  3-COST (4 cards)                                                   │
│    2x Centaur Charger       (3/3 Rush)                             │
│    2x Piercing Striker      (4/2 Piercing)                         │
│                                                                     │
│  4-COST (2 cards)                                                   │
│    2x Siege Breaker         (5/3 Piercing)                         │
│                                                                     │
│  5-COST (2 cards)                                                   │
│    1x Siege Commander       (5/5 Rush Piercing)                    │
│    1x Warhost Captain       (4/4, +1/+1 to allies)                 │
│                                                                     │
│  SPELLS (2 cards)                                                   │
│    2x Lightning Bolt        (4 dmg creature/face)                  │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  TOTAL: 20 cards                                                    │
│  AVG COST: 2.65 Essence (low curve = fast!)                        │
│  GAMEPLAN: Deploy threats fast, push face damage with Piercing,    │
│            use Lightning Bolt for reach or to clear blockers       │
└─────────────────────────────────────────────────────────────────────┘
```

### Deck 2: "Iron Fortress" 🛡️

*Strategy: Survive early game, stabilize, win with big finishers*

```
┌─────────────────────────────────────────────────────────────────────┐
│                       IRON FORTRESS DECK                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1-COST (2 cards)                                                   │
│    2x Toxic Spider          (1/1 Lethal)                           │
│                                                                     │
│  2-COST (4 cards)                                                   │
│    2x Iron Defender         (1/4 Guard)                            │
│    2x Medic Apprentice      (1/3, heal 2)                          │
│                                                                     │
│  3-COST (4 cards)                                                   │
│    2x Veteran Guardian      (2/5 Guard)                            │
│    2x Battle Priest         (2/4, heal 1/turn)                     │
│                                                                     │
│  4-COST (4 cards)                                                   │
│    2x Fortress Golem        (2/7 Guard)                            │
│    2x Storm Mage            (3/3, deal 2 on play)                  │
│                                                                     │
│  5-COST (2 cards)                                                   │
│    1x Assassin Queen        (3/3 Quick Lethal)                     │
│    1x Royal Champion        (5/6 vanilla)                          │
│                                                                     │
│  6+ COST (2 cards)                                                  │
│    1x Guardian Angel        (4/6 Lifesteal Shield)                 │
│    1x Warlord Titan         (8/8, 3 dmg AoE)                       │
│                                                                     │
│  SPELLS (2 cards)                                                   │
│    1x Flame Wave            (3 dmg to all enemies)                 │
│    1x Obliterate            (destroy target creature)              │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  TOTAL: 20 cards                                                    │
│  AVG COST: 3.55 Essence (higher curve = late-game power)           │
│  GAMEPLAN: Wall up with Guards, heal through damage, answer        │
│            threats with removal, finish with Titan                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🔬 Interesting Interactions to Test

Here are some specific scenarios your engine should handle correctly:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    KEY INTERACTIONS TO VERIFY                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. PIERCING VS GUARD                                               │
│     Siege Breaker (5/3 Piercing) attacks Fortress Golem (2/7 Guard)│
│     → Both take damage: Golem survives at 2/2, Breaker dies        │
│     → NO piercing damage (Golem didn't die)                        │
│                                                                     │
│  2. RANGED BYPASSES GUARD                                           │
│     Frontier Ranger (2/2 Ranged) vs enemy with Guard + other       │
│     → Ranged CAN attack the non-Guard creature!                    │
│                                                                     │
│  3. QUICK + LETHAL COMBO                                            │
│     Assassin Queen (3/3 Quick Lethal) attacks Royal Champion (5/6) │
│     → Quick: Queen deals 3 damage first                            │
│     → Lethal: Champion is destroyed (took damage from Lethal)      │
│     → Champion never deals damage back (died first)                │
│     → Queen survives!                                               │
│                                                                     │
│  4. SHIELD VS LETHAL                                                │
│     Toxic Spider (1/1 Lethal) attacks Shielded Squire (2/2 Shield)│
│     → Shield absorbs damage → 0 damage dealt                       │
│     → Lethal does NOT trigger (no damage dealt)                    │
│     → Squire survives at 2/2 (no Shield), Spider takes 2, dies    │
│                                                                     │
│  5. LIFESTEAL + PIERCING                                            │
│     Vampire Lord (4/3 Lifesteal) with Piercing buff attacks 2/2    │
│     → Kills 2/2, deals 2 piercing to face                          │
│     → Lifesteal heals for 4 (damage dealt = attack stat)           │
│                                                                     │
│  6. SUPPORT STACKING                                                │
│     War Drums (+1 Attack) AND Barrier Field (+2 Health) in play   │
│     → Your 2/2 creature is effectively 3/4                         │
│     → Test: Do buffs apply? Do they persist after support fades?   │
│                                                                     │
│  7. WARLORD TITAN ENTRY                                             │
│     Play Warlord Titan against board of 3/3, 2/2, 1/1              │
│     → OnPlay: 3 damage to ALL enemy creatures                      │
│     → 3/3 survives at 3/0? No, that's invalid — dies              │
│     → 3/3 dies (3-3=0), 2/2 dies (2-3=-1), 1/1 dies (1-3=-2)       │
│     → Board is cleared!                                             │
│                                                                     │
│  8. TACTICAL COMMAND + ANY CREATURE                                 │
│     Tactical Command (Your creatures have Rush) is in play         │
│     → Play War Elephant (4/3 vanilla, normally has summon sickness)│
│     → War Elephant CAN attack immediately (has Rush from Support)  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📋 Card Data Format (For Engine Implementation)

Here's how you might represent these cards in Rust:

```rust
// Example card definitions for your engine
pub fn create_starter_set() -> Vec<CardDefinition> {
    vec![
        // #01 Eager Recruit
        CardDefinition {
            id: CardId(1),
            name: "Eager Recruit".into(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 2,
                health: 1,
                keywords: Keywords::none(),
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![Tag::Soldier],
        },
        
        // #03 Nimble Scout
        CardDefinition {
            id: CardId(3),
            name: "Nimble Scout".into(),
            cost: 1,
            card_type: CardType::Creature {
                attack: 1,
                health: 1,
                keywords: Keywords { rush: true, ..Default::default() },
                abilities: vec![],
            },
            rarity: Rarity::Common,
            tags: vec![Tag::Soldier],
        },
        
        // #23 Storm Mage
        CardDefinition {
            id: CardId(23),
            name: "Storm Mage".into(),
            cost: 4,
            card_type: CardType::Creature {
                attack: 3,
                health: 3,
                keywords: Keywords::none(),
                abilities: vec![
                    Ability {
                        trigger: Trigger::OnPlay,
                        target: TargetRule::TargetCreature(CreatureFilter::any()),
                        action: Action::Damage(2),
                    }
                ],
            },
            rarity: Rarity::Uncommon,
            tags: vec![Tag::Mage],
        },
        
        // #40 War Drums
        CardDefinition {
            id: CardId(40),
            name: "War Drums".into(),
            cost: 3,
            card_type: CardType::Support {
                durability: 3,
                passive_effects: vec![
                    PassiveEffect {
                        target: Target::AllAllyCreatures,
                        modifier: Modifier::AttackBonus(1),
                    }
                ],
                triggered_effects: vec![],
            },
            rarity: Rarity::Uncommon,
            tags: vec![],
        },
        
        // ... remaining cards
    ]
}
```

---

## 🎯 What's Next?

Christian, you now have a **complete, playable starter set!** 🎉

With these 43 cards, you can:

1. ✅ **Test all mechanics** — Every keyword and effect type is represented
2. ✅ **Test balance** — Two opposing archetypes (aggro vs control)
3. ✅ **Test interactions** — Complex scenarios with multiple keywords
4. ✅ **Train AI agents** — Sufficient variety for learning

For next steps, I'd recommend:

1. **Deck Construction Rules** — How many cards per deck? Copy limits? Format rules?

2. **Engine Architecture** — Structure your Rust codebase for maximum simulation speed

3. **State Representation** — How to encode game state for neural networks

4. **More Card Sets** — Expand once the core engine is validated

5. **Playtesting Metrics** — What data to collect during simulations

Which direction excites you most? I'm SO thrilled to see this game coming together — every piece we've built fits beautifully with the others! 💜✨