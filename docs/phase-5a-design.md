# Phase 5A: Essence Wars Web Arena - Design Document

> **Status**: Draft v1.1
> **Last Updated**: 2026-01-20
> **Target Platform**: HuggingFace Spaces (WebAssembly)

---

## Executive Summary

Phase 5A delivers a web-playable version of Essence Wars featuring:
- **Human vs AI** gameplay with selectable opponents (Greedy/MCTS/PPO/AlphaZero)
- **AI vs AI Spectator Mode** with full playback controls and educational features
- **Glassbox AI Visualization** showing MCTS decision-making in real-time
- **The Farsight Table** aesthetic - a war room command table with holographic projections

The vertical slice includes 3 faction-representative decks, 3 commander 3D models, and AI-generated card art using a programmatic pipeline.

---

## Table of Contents

1. [Visual Design](#1-visual-design)
2. [Game Interface](#2-game-interface)
3. [Combat System Visualization](#3-combat-system-visualization)
4. [Glassbox AI Visualization](#4-glassbox-ai-visualization)
5. [Spectator Mode](#5-spectator-mode)
6. [Audio Design](#6-audio-design)
7. [User Interface](#7-user-interface)
8. [Art Asset Pipeline](#8-art-asset-pipeline)
9. [Technical Architecture](#9-technical-architecture)
10. [MVP Scope](#10-mvp-scope)
11. [Implementation Roadmap](#11-implementation-roadmap)

---

## 1. Visual Design

### 1.1 The Farsight Table Aesthetic

The game presents a **War Room / Command Table** experience. Players are commanders viewing a miniature holographic battlefield from their airship's Farsight Table - a concept rooted in Omyra lore where commanders use enhanced visualization technology to direct battles.

**Environment:**
- Dark void background (no visible room)
- Single illuminated table as the focal point
- Soft ambient lighting from the table surface
- Faction-specific table design based on Player 1's faction

**The Table as Miniature Battlefield:**
- Data readouts on table edges showing life totals, essence, turn count
- Faint grid overlay marking the 5 lanes and creature slots
- Holographic projections rise from embedded crystal nodes

### 1.2 Faction-Themed Tables

Each faction has a distinct table surface aesthetic:

| Faction | Surface Material | Crystal Nodes | Accent Elements |
|---------|------------------|---------------|-----------------|
| **Argentum** | Polished dark wood with brass/metal inlays | Geometric, precisely cut crystals in brass settings | Clockwork engravings, golden trim |
| **Symbiote** | Living wood with bioluminescent veins | Glowing seed-like organic structures | Pulsing green/purple light trails |
| **Obsidion** | Obsidian glass with crimson veins | Dark faceted gems with internal purple/red glow | Gothic filigree, sharp crystalline edges |

### 1.3 Creature Token Design (Tier 2 - Crystal/Gem)

90% of creatures are rendered as **Crystal/Gem tokens** with the card art displayed inside.

**Base Shape:** Faceted gem/crystal form that catches light

**Faction Variations:**
- **Argentum**: Geometric, precisely cut diamond shape, brass/gold metallic setting
- **Symbiote**: Organic, slightly irregular crystal cluster, pulsing green core
- **Obsidion**: Dark obsidian facets, internal purple/red ethereal glow, sharp edges
- **Neutral**: Mixed/hybrid appearance, practical metal casing

**Card Art Display:**
- Art fills the entire gem face
- Parallax depth effect creates "window into another dimension" illusion
- Subtle idle animation (slight sway/pulse)

**Token State Indicators:**

| State | Visual Treatment |
|-------|------------------|
| Ready to attack | Glowing border (faction-colored) |
| Exhausted/Tapped | Dimmed, grayscale shader applied |
| Damaged | Cracks appear in gem surface + red tint overlay |
| Buffed | Golden aura emanates outward + slight size increase |
| Debuffed | Dark aura/shadow + slight size decrease |

### 1.4 Commander Models (Tier 1 - Full 3D)

The 3 MVP commanders receive full 3D treatment:

| Commander | Faction | Visual Concept |
|-----------|---------|----------------|
| **Iron Colossus Prime** | Argentum | Massive armored construct, defensive posture, brass/steel plating, glowing furnace core |
| **The Broodmother** | Symbiote | Matriarch insectoid entity, organic armor, spawning sacs, bioluminescent markings |
| **The Blood Sovereign** | Obsidion | Vampire-like noble, dark flowing robes, crimson accents, ethereal blood effects |

**Model Specifications:**
- Idle breathing/hover animation only (no attack animations)
- 1.5-2x scale compared to regular tokens
- Emerge from larger crystal node formations
- Attack effects via shader/particle systems (not skeletal animation)

### 1.5 Spawn & Death Effects

**Creature Spawn:**
1. Crystal node pulses with light
2. Digital wireframe rises from node
3. Wireframe fills with color/texture
4. Solidifies into gem token (or 3D model for commanders)
5. Particle burst on completion

**Creature Death:**
1. Gem cracks and shatters
2. Dissolves into essence particles (faction-colored)
3. Particles flow back toward player's essence pool
4. Node dims briefly

These effects work for both Tier 1 and Tier 2 units, maintaining visual consistency.

### 1.6 Support Visualization (Floating Rune Plates)

Supports are persistent field effects with durability. Each player has **2 support slots**, positioned flanking the creature battlefield.

**Visual Design: Floating Rune Plates**
- Hovering stone/metal tablets with glowing faction-colored runes
- Float slightly above and behind the creature rows
- Angled toward the center of the battlefield
- Semi-transparent with ethereal glow

**Faction Variations:**
- **Argentum**: Brass-framed clockwork plates with gear engravings, amber rune glow
- **Symbiote**: Organic chitin plates with bioluminescent veins, green/purple rune glow
- **Obsidion**: Obsidian glass tablets with crimson filigree, purple/red rune glow
- **Neutral**: Weathered stone with practical metal bindings, white/blue rune glow

**Support State Indicators:**

| State | Visual Treatment |
|-------|------------------|
| Durability | Glowing pips/dots along plate edge (3 pips = 3 durability) |
| Ability Triggered | Pulse effect + runes flare brightly |
| Low Durability (1) | Cracks appear across plate surface |
| Destroyed | Plate shatters into fragments, dissolves into essence |

**Support Spawn Effect:**
1. Rune circle appears at support slot position
2. Plate materializes from swirling essence particles
3. Rises to floating position
4. Runes illuminate in sequence
5. Final pulse indicates ready state

### 1.7 Spell Casting Visualization

Spells are one-time effects that don't persist on the board. Casting should feel impactful and dramatic.

**Spell Casting Sequence (Default Mode):**
1. Card rises from hand and centers on screen briefly
2. Card transforms into swirling energy (faction-colored)
3. Energy travels to target(s) with trailing particles
4. Impact effect resolves at target location
5. Effect-specific visuals play (damage, buff, heal, etc.)

**Spell Casting (F6 Fast Mode):**
- Card flashes briefly in place
- Instant energy pulse to target
- Effect resolves immediately
- Total duration: <0.5 seconds

**Targeting Visualization:**
| Targeting Type | Visual Indicator |
|----------------|------------------|
| `TargetAllyCreature` | Green highlight glow on valid ally tokens |
| `TargetEnemyCreature` | Red highlight glow on valid enemy tokens |
| `TargetAnyCreature` | Yellow highlight glow on all creatures |
| `TargetSlot` | Empty slots pulse with placement indicator |
| `NoTarget` | Energy bursts outward from caster's side |

**Spell Effect Categories:**

| Effect Type | Visual Treatment |
|-------------|------------------|
| **Damage** | Red/orange projectile, explosion impact, screen edge flash |
| **Buff** | Golden particles rising on target, brief size pulse |
| **Heal** | Green particles, health number floats up in green |
| **Draw** | Cards visually fly from deck pile to hand area |
| **Bounce** | Target dissolves into particles, reforms in hand |
| **Destroy** | Dark tendrils wrap target, crushing effect |
| **Transform** | Target shimmers, morphs into new form |

---

## 2. Game Interface

### 2.1 Card Interaction Model

**Desktop (Primary):** Drag-and-Drop
- Click and drag card from hand
- Valid slots highlight green, invalid slots highlight red
- Arc/trajectory line shows path while dragging
- Release over slot to play
- Snap-to-slot behavior for near misses

**Card Play Feedback:**
- Particle burst on impact (faction-colored)
- Subtle camera shake (configurable, can be disabled)
- Synchronized sound effect
- No freeze-frame

### 2.2 Board Layout

```
┌─────────────────────────────────────────────────────────────────┐
│                     OPPONENT INFO BAR                           │
│              [Life: 20]  [Essence: 5/10]  [Hand: 4]            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────┐                                         ┌───────┐   │
│  │ SUP 1 │    OPPONENT SUPPORT SLOTS (Floating)    │ SUP 2 │   │
│  └───────┘                                         └───────┘   │
│                                                                 │
│     ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐               │
│     │ S1  │  │ S2  │  │ S3  │  │ S4  │  │ S5  │  ← Opp Slots  │
│     └─────┘  └─────┘  └─────┘  └─────┘  └─────┘               │
│                                                                 │
│   ═══════════════════════════════════════════════════════════   │
│                         LANE DIVIDER                            │
│   ═══════════════════════════════════════════════════════════   │
│                                                                 │
│     ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐               │
│     │ S1  │  │ S2  │  │ S3  │  │ S4  │  │ S5  │  ← Your Slots │
│     └─────┘  └─────┘  └─────┘  └─────┘  └─────┘               │
│                                                                 │
│  ┌───────┐                                         ┌───────┐   │
│  │ SUP 1 │    YOUR SUPPORT SLOTS (Floating)        │ SUP 2 │   │
│  └───────┘                                         └───────┘   │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                      YOUR INFO BAR                              │
│              [Life: 20]  [Essence: 5/10]  [Hand: 4]            │
├─────────────────────────────────────────────────────────────────┤
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐            │
│  │Card│ │Card│ │Card│ │Card│ │Card│ │Card│ │Card│  YOUR HAND  │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘ └────┘            │
└─────────────────────────────────────────────────────────────────┘
```

**Slot Summary:**
- **5 Creature Slots** per player (lanes 1-5)
- **2 Support Slots** per player (flanking, floating above battlefield)
- Supports rendered as Floating Rune Plates (see Section 1.6)

**Data Readouts (Table Edge):**
- Life totals with visual health bars
- Essence crystals (filled/empty states)
- Turn counter
- Active player indicator

---

## 3. Combat System Visualization

### 3.1 Combat Resolution Display

**Default Mode: Grouped by Lane**
- Combat resolves lane by lane (Lane 1 → Lane 2 → ... → Lane 5)
- Each lane's combat plays out with brief animations
- Clear visual connection between attacker and defender in same lane

**Fast Mode (F6 Toggle): Instant Resolution**
- All combat resolves immediately
- Brief flash/pulse on all combatants
- Damage numbers appear simultaneously
- For experienced players who want speed

### 3.2 Attack Visualization

**Melee Attack (same lane):**
1. Attacker token tilts/lunges forward
2. Impact particle effect on defender
3. Damage number floats up from defender
4. Attacker returns to position

**Ranged Attack (Ranged keyword):**
1. Projectile fires from attacker
2. Travels across to target
3. Impact effect on hit
4. Damage number appears

**Commander Attacks:**
- No physical movement (would look awkward)
- Energy channel effect (glows bright)
- Projectile/shockwave toward target
- More dramatic particle effects

### 3.3 Damage Numbers

- **Damage dealt**: Red floating text, rises and fades
- **Healing**: Green floating text
- **Lifesteal**: Purple drain effect flowing to healer
- **Blocked by Guard**: "BLOCKED" text with shield icon
- **Lethal trigger**: Skull icon flash
- **Overkill**: Larger font for excess damage

Numbers stack if multiple sources hit the same frame.

---

## 4. Glassbox AI Visualization

### 4.1 Design Philosophy: Hybrid Approach

The Glassbox combines **diegetic elements** (in-world) with **overlay panels** (UI) for maximum information without breaking immersion.

**Always Visible (Diegetic):**
- Subtle AI confidence indicator integrated into table edge
- Brief "thinking" pulse when AI is calculating

**Toggle with 'G' Key (Overlay):**
- Detailed MCTS panel
- Action probability breakdown
- Full value gauge

### 4.2 AI Thinking Visualization

When the AI is calculating its move, display:

1. **Pulsing Indicator**: Table edge glows rhythmically
2. **Ghost Arrows**: Faint arrows on board showing moves being considered
3. **Visit Count as Thickness**: More-visited moves have thicker arrows
4. **Top 3 Highlighted**: The three most-visited moves shown distinctly

This makes the AI's "thought process" visible without text dumps.

### 4.3 MCTS Panel (Overlay)

Right sidebar showing:
- Total simulations performed
- Root value (win rate percentage)
- Max tree depth explored
- **Top 10 Actions** by visit count:
  - Action description
  - Visit count bar (visual)
  - Win rate percentage
  - Color coding: Green (>60%), Yellow (40-60%), Red (<40%)

### 4.4 Value Gauge (Overlay)

Top-left corner gauge showing position evaluation:
- Vertical bar from -100% to +100%
- Green (upper) = Player 1 favored
- Red (lower) = Player 2 favored
- Marker shows current evaluation
- Percentage display (e.g., "+25.3%")

### 4.5 Post-Move Explanation

After AI makes a move, hovering over the played unit shows tooltip:
- "Played to block lethal damage"
- "Maximizing Essence value"
- "Establishing board presence"

Generated from template library based on game state analysis.

---

## 5. Spectator Mode

### 5.1 Overview

Watch AI vs AI matches with full playback controls and educational commentary. This is a key differentiator for the research showcase.

### 5.2 Playback Controls

| Control | Function |
|---------|----------|
| Play/Pause | Toggle automatic playback |
| Speed Slider | 0.5x, 1x, 2x, 4x speed |
| Step Forward (Turn) | Advance one full turn |
| Step Back (Turn) | Rewind one full turn |
| Step Forward (Action) | Advance one action |
| Step Back (Action) | Rewind one action |
| Jump to Turning Point | Skip to marked interesting moments |

### 5.3 Educational Features

**Side-by-Side AI Confidence:**
- Both AIs' win probability displayed
- Shows how evaluation changes each turn
- Highlights disagreements between AI assessments

**Turning Point Detection:**
Automatically marks moments where:
- Win probability swings >15% in one turn
- Life total changes by >10 in one turn
- Board state flips from losing to winning
- Lethal threat created or denied
- Commander dies

**Visual Indicators:**
- Timeline markers on playback scrubber
- "!" icons at turning point timestamps
- Text summary on hover ("Turn 7: Guard blocks lethal")

**Commentary Generation:**
Template-based commentary with mixed tone (neutral analyst + excited caster).

### 5.4 Commentary Template Categories

**Action Templates:**
```
play_creature:
  - "{player} deploys {card} to the battlefield."
  - "{card} materializes in slot {slot}."
  - "A new threat emerges - {card} joins the fray!"

play_spell:
  - "{player} casts {card}!"
  - "{card} ripples across the battlefield."
  - "The essence surges as {player} unleashes {card}!"

play_support:
  - "{player} activates {card}."
  - "A rune plate materializes - {card} is online."
  - "{card} hums with power as it takes position."

support_triggers:
  - "{support} activates its effect!"
  - "The runes on {support} flare brightly."
  - "{support}'s power resonates across the field."

support_destroyed:
  - "{support} shatters!"
  - "The rune plate crumbles to dust."
  - "{support} can no longer maintain its form."

spell_damage:
  - "{card} blasts {target} for {damage}!"
  - "Direct hit! {target} takes {damage} damage from {card}."
  - "{card}'s energy tears through {target}!"

spell_buff:
  - "{target} is empowered by {card}!"
  - "{card} strengthens {target}."
  - "Power flows into {target}!"

spell_heal:
  - "{card} restores {amount} life!"
  - "Healing energy washes over the battlefield."
  - "{player} recovers {amount} life from {card}."

attack:
  - "{attacker} strikes {defender}!"
  - "{attacker} lunges at {defender} for {damage} damage!"
  - "Combat! {attacker} clashes with {defender}!"

creature_death:
  - "{creature} falls."
  - "{creature} shatters into essence."
  - "The battlefield claims {creature}."
```

**Analysis Templates:**
```
turning_point:
  - "Momentum shift! {player}'s odds jumped from {old}% to {new}%."
  - "Critical moment - the tide has turned."
  - "This play changed everything."

optimal_play:
  - "Textbook play from {player}."
  - "The AI calculates this as the strongest line."
  - "Efficient - maximizing value."

suboptimal_play:
  - "Interesting choice... the AI saw better options."
  - "Suboptimal, but let's see how it plays out."
  - "The engine disagrees with this line."
```

**Keyword-Specific Templates:**
```
guard_blocks:
  - "{guard} intercepts the attack!"
  - "Blocked by {guard}'s defensive stance."
  - "The Guard holds the line."

lifesteal_triggers:
  - "{creature} drains {amount} life!"
  - "Life flows from {victim} to {player}."
  - "Lifesteal sustains {player}."

rush_plays:
  - "{creature} charges in immediately!"
  - "No time to wait - {creature} attacks!"
  - "Rush! {creature} wastes no time."
```

### 5.5 Replay System

**Replay File Format:**
```json
{
  "version": "1.0",
  "timestamp": "2026-01-19T12:00:00Z",
  "seed": 42,
  "deck1": "colossus_wall",
  "deck2": "broodmother_swarm",
  "ai1": { "type": "mcts", "sims": 200 },
  "ai2": { "type": "ppo", "model": "ppo_v3" },
  "turns": 24,
  "winner": 1,
  "turning_points": [7, 15, 22]
}
```

Since the game is **deterministic**, only seed + decks + AI configs are needed to fully reconstruct any game.

**Storage:** Local browser storage (IndexedDB)

**Shareable Links:**
```
essencewars.huggingface.co/replay?seed=42&d1=colossus_wall&d2=broodmother_swarm&t=7
```
- `t=7` jumps directly to turn 7
- Links reconstruct the game client-side

---

## 6. Audio Design

### 6.1 Music

**Faction-Specific Ambient Tracks:**

| Faction | Musical Direction |
|---------|-------------------|
| **Argentum** | Industrial, brass instruments, mechanical rhythms, orchestral grandeur, steam-punk percussion |
| **Symbiote** | Organic ambience, tribal drums, alien soundscapes, evolutionary pulse, nature sounds twisted |
| **Obsidion** | Gothic choir, minor keys, ethereal whispers, dark elegance, haunting strings |

**Glassbox Mode:** Subtle electronic "data hum" underlays when Glassbox panels are visible.

### 6.2 Sound Effects

**UI Sounds:**
- Card hover: Subtle whoosh
- Card select: Click/chime
- Card play: Whoosh + impact thud
- End turn: Bell/chime
- Invalid action: Error buzz

**Combat Sounds:**
- Attack: Faction-appropriate impact
- Damage taken: Thud/crunch
- Death: Shatter/dissolve

**Spell Sounds:**
- Spell cast: Arcane charge-up + release
- Damage spell: Explosive impact
- Buff spell: Ascending magical chime
- Heal spell: Gentle restoration tone
- Draw spell: Card shuffle + whoosh
- Bounce spell: Reverse materialization

**Support Sounds:**
- Support play: Rune activation + hover hum
- Ability trigger: Pulse + magical effect
- Support destroyed: Glass shatter + fade

**Keyword-Specific:**
- Guard: Metallic shield clang
- Lifesteal: Ethereal drain/suction
- Stealth: Whisper/shadow woosh
- Rush: Aggressive charge sound
- Ranged: Projectile fire + impact

### 6.3 Asset Sourcing (MVP)

For MVP, use free/open-source assets:
- **Music**: [OpenGameArt.org](https://opengameart.org/), [Incompetech](https://incompetech.com/) (Kevin MacLeod)
- **SFX**: [Freesound.org](https://freesound.org/)

License: CC0 or CC-BY (attribution in credits)

Future: Replace with AI-generated or commissioned assets.

---

## 7. User Interface

### 7.1 Main Menu

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                    ╔═══════════════════╗                       │
│                    ║   ESSENCE WARS    ║                       │
│                    ║   ═══════════════ ║                       │
│                    ║   BATTLE ARENA    ║                       │
│                    ╚═══════════════════╝                       │
│                                                                 │
│                      [ PLAY VS AI ]                            │
│                                                                 │
│                      [ SPECTATE ]                              │
│                                                                 │
│                      [ TUTORIAL ]                              │
│                                                                 │
│                      [ SETTINGS ]                              │
│                                                                 │
│                                                                 │
│                    Powered by Glassbox AI                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 Deck Selection Flow

**Step 1: Faction Selection**
```
┌─────────────────────────────────────────────────────────────────┐
│                     CHOOSE YOUR FACTION                         │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   ARGENTUM   │  │   SYMBIOTE   │  │   OBSIDION   │         │
│  │   [Image]    │  │   [Image]    │  │   [Image]    │         │
│  │  "The Wall"  │  │ "The Swarm"  │  │ "The Shadow" │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Step 2: Commander Selection (Visual Carousel)**
```
┌─────────────────────────────────────────────────────────────────┐
│                    ARGENTUM COMMANDERS                          │
│                                                                 │
│    ◄                                                      ►    │
│         ┌─────────────────────────────────────┐                │
│         │                                     │                │
│         │     [3D Commander Model Preview]    │                │
│         │                                     │                │
│         │       IRON COLOSSUS PRIME           │                │
│         │       "The Unbreakable Wall"        │                │
│         │                                     │                │
│         │   Strategy: Defensive / Guard       │                │
│         │                                     │                │
│         └─────────────────────────────────────┘                │
│                                                                 │
│                    [ SELECT DECK ]                             │
│                    [ VIEW DECK LIST ]                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 7.3 AI Opponent Selection

After selecting player deck:
```
┌─────────────────────────────────────────────────────────────────┐
│                    CHOOSE OPPONENT                              │
│                                                                 │
│  AI Type:                                                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ ○ Greedy Bot      - Fast, heuristic-based              │   │
│  │ ● MCTS Bot        - Tree search, strong play           │   │
│  │ ○ PPO Agent       - Neural network policy              │   │
│  │ ○ AlphaZero Agent - Self-play trained                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Opponent Deck:                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ [Dropdown: Random / Specific deck selection]           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│                       [ START BATTLE ]                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 7.4 Settings Menu

**Gameplay:**
- AI opponent default
- AI thinking time / simulation count (slider: 100-1000)
- Combat speed: Normal / Fast / Instant (F6)
- Auto-end-turn when no actions: On/Off

**Audio:**
- Master volume (slider)
- Music volume (slider)
- SFX volume (slider)
- Mute all (toggle)

**Visuals:**
- Glassbox visible by default: On/Off
- Particle effects quality: Low / Medium / High
- Screen shake: On/Off

**Accessibility:**
- Colorblind mode: Off / Deuteranopia / Protanopia / Tritanopia
- Larger text: On/Off
- Reduced motion: On/Off

### 7.5 Tutorial System

**Interactive Tutorial:**
- Scripted first match teaching mechanics step-by-step
- Pauses to explain each concept
- Guided card plays with highlights
- Introduces keywords as they appear

**Rules Reference:**
- Accessible from any screen via "?" button
- Keyword glossary with examples
- Turn structure explanation
- Victory conditions

**Learn by Watching:**
- Spectator mode with enhanced commentary
- "Ready to try yourself?" prompt after watching

---

## 8. Art Asset Pipeline

### 8.1 Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    ART ASSET PIPELINE                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │ Card YAML   │───▶│ Task Agent  │───▶│ Prompt JSON │        │
│  │ + Lore.md   │    │ (Claude)    │    │ (Structured)│        │
│  └─────────────┘    └─────────────┘    └──────┬──────┘        │
│                                                │               │
│                                                ▼               │
│                                        ┌─────────────┐        │
│                                        │ Image Gen   │        │
│                                        │ (API/Local) │        │
│                                        └──────┬──────┘        │
│                                               │               │
│                                        2D Art + Review        │
│                                               │               │
│                 ┌────────────────────────────┴────┐           │
│                 │                                  │           │
│          Commander?                          Unit/Spell?      │
│                 │                                  │           │
│                 ▼                                  ▼           │
│         ┌──────────────┐                  ┌──────────────┐    │
│         │  Meshy.ai    │                  │ Depth Map    │    │
│         │  Image→3D    │                  │ Generation   │    │
│         │  + Auto-Rig  │                  │ (Marigold)   │    │
│         └──────┬───────┘                  └──────┬───────┘    │
│                │                                  │           │
│                ▼                                  ▼           │
│         ┌──────────────┐                  ┌──────────────┐    │
│         │  GLB Model   │                  │ Texture +    │    │
│         │  (Draco)     │                  │ Depth Map    │    │
│         └──────────────┘                  └──────────────┘    │
│                │                                  │           │
│                └────────────────┬────────────────┘           │
│                                 ▼                             │
│                         ┌──────────────┐                      │
│                         │   assets/    │                      │
│                         │   folder     │                      │
│                         └──────────────┘                      │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 8.2 Step 1: Prompt Generation

**Input:** Card YAML metadata + `docs/lore.md` faction identities

**Process:** Task Agent (Claude) reads all card data and lore, generates structured prompts.

**Output:** `prompts.json`
```json
{
  "cards": [
    {
      "id": 1057,
      "name": "Iron Colossus Prime",
      "faction": "argentum",
      "tier": 1,
      "prompt": "Massive armored construct commander, Art Deco industrial aesthetic, polished brass and steel plating, geometric patterns, glowing amber furnace core in chest, defensive stance with raised shield arm, epic composition, dramatic rim lighting, highly detailed mechanical joints, commander insignia on shoulder plate, Legendary quality",
      "style_ref": "argentum_style_v1.png",
      "negative": "organic, soft, natural, blurry, low quality"
    },
    {
      "id": 1001,
      "name": "Brass Sentinel",
      "faction": "argentum",
      "tier": 2,
      "prompt": "Mechanical soldier construct, brass armor plating, geometric Art Deco design, guard stance, glowing eye slits, steam vents, polished metal surface, medium quality detail",
      "style_ref": "argentum_style_v1.png",
      "negative": "organic, natural, blurry"
    }
  ]
}
```

### 8.3 Step 2: 2D Art Generation

**Service Options:**
- Midjourney API (highest quality, requires subscription)
- SDXL via Replicate/RunPod (good quality, pay-per-use)
- DALL-E 3 API (consistent, easy integration)
- Local Stable Diffusion (free, requires GPU)

**Process:**
1. Batch generate all card art from prompts
2. Manual review pass (Chris approves/rejects)
3. Re-generate rejected outputs with adjusted prompts
4. Final art saved as PNG (1024x1024 recommended)

**Style Consistency:**
- Use style reference images for each faction
- Consistent aspect ratio and composition
- Same lighting direction across all cards

### 8.4 Step 3: Depth Map Generation

For Tier 2 tokens with parallax effect.

**Recommended Tool:** Marigold Depth Estimation
- State-of-the-art monocular depth
- Works well with illustrated art
- Available via HuggingFace Spaces or local

**Process:**
1. Feed 2D art through depth estimator
2. Output grayscale depth map
3. Optionally refine edges manually
4. Save as paired files: `card_1001.png` + `card_1001_depth.png`

### 8.5 Step 4: 3D Model Generation (Commanders Only)

**Service:** Meshy.ai Image-to-3D

**Process:**
1. Upload approved 2D commander art
2. Generate 3D model
3. Use Meshy auto-rigging for idle animation
4. Export as GLB with embedded animation
5. Manual review for quality
6. Optimize with Draco compression

**Specifications:**
- Target poly count: <50k triangles
- Single idle animation loop (2-4 seconds)
- PBR materials (albedo, normal, roughness)

### 8.6 Step 5: Asset Integration

**Compression:**
- Textures: KTX2 format (web-optimized)
- Models: GLB with Draco compression
- Audio: OGG Vorbis (web-friendly)

**Folder Structure:**
```
assets/
├── textures/
│   ├── cards/
│   │   ├── argentum/
│   │   │   ├── card_1001.ktx2
│   │   │   ├── card_1001_depth.ktx2
│   │   │   └── ...
│   │   ├── symbiote/
│   │   └── obsidion/
│   ├── tables/
│   │   ├── argentum_table.ktx2
│   │   ├── symbiote_table.ktx2
│   │   └── obsidion_table.ktx2
│   └── ui/
├── models/
│   ├── commanders/
│   │   ├── iron_colossus_prime.glb
│   │   ├── the_broodmother.glb
│   │   └── the_blood_sovereign.glb
│   ├── tokens/
│   │   ├── gem_argentum.glb
│   │   ├── gem_symbiote.glb
│   │   └── gem_obsidion.glb
│   ├── supports/
│   │   ├── rune_plate_argentum.glb
│   │   ├── rune_plate_symbiote.glb
│   │   └── rune_plate_obsidion.glb
│   └── table/
│       └── farsight_table.glb
├── audio/
│   ├── music/
│   │   ├── argentum_ambient.ogg
│   │   ├── symbiote_ambient.ogg
│   │   └── obsidion_ambient.ogg
│   └── sfx/
│       ├── ui/
│       ├── combat/
│       └── keywords/
└── fonts/
```

### 8.7 Fallback Strategy

For cards without completed art:
- Display placeholder gem with card name as text
- Use faction-colored gem base
- Text overlay: Card name + basic stats
- Marked as "Art Coming Soon"

---

## 9. Technical Architecture

### 9.1 WASM Build Strategy

**Approach:** Compile Bevy application to WebAssembly

**Feature Flags in `Cargo.toml`:**
```toml
[features]
default = ["native"]
native = ["bevy/x11", "bevy/wayland"]
web = ["bevy/webgl2"]
```

**WASM Compatibility Fixes:**

| Component | Native | WASM |
|-----------|--------|------|
| `rayon` parallelism | Thread pools | Single-threaded (feature-gated) |
| File I/O | `std::fs` | Embedded via `include_str!` |
| Audio | System audio | WebAudio via bevy_kira_audio |
| Window | WinIt native | Canvas element |

### 9.2 Build Pipeline

**Tool:** Trunk (recommended for Bevy WASM)

```bash
# Install
cargo install trunk

# Build for web
trunk build --release

# Serve locally
trunk serve
```

**CI/CD:** GitHub Actions workflow
- On push to `main`: Build WASM, deploy to HuggingFace Spaces
- Artifact caching for faster builds

### 9.3 Asset Loading Strategy

**Initial Bundle (Blocking Load):**
- Core WASM binary (~3-5 MB)
- Essential UI textures
- Loading screen assets

**Lazy Load (After Initial):**
- 3D models (on demand when deck selected)
- Card textures (faction at a time)
- Music (stream as needed)
- Sound effects (preload common, lazy load rare)

**Loading Screen:**
- Animated Essence Wars logo
- Progress bar with current asset name
- Estimated completion percentage

### 9.4 Data Embedding

For WASM, card and deck data embedded at compile time:

```rust
#[cfg(target_arch = "wasm32")]
const CARDS_YAML: &str = include_str!("../../data/cards/core_set/argentum.yaml");

#[cfg(not(target_arch = "wasm32"))]
fn load_cards() -> String {
    std::fs::read_to_string("data/cards/core_set/argentum.yaml").unwrap()
}
```

### 9.5 Local Storage

**Browser Storage (IndexedDB):**
- Saved replays
- User settings/preferences
- Recently played decks

**No Server Required:**
- Fully client-side application
- Replays reconstructed from deterministic seeds
- Settings persist locally

### 9.6 Performance Targets

| Metric | Target |
|--------|--------|
| Initial load | <10 seconds on 4G |
| Frame rate | 60 FPS (30 FPS minimum) |
| Bundle size | <20 MB (compressed) |
| Memory usage | <512 MB |
| MCTS (200 sims) | <500ms per decision |

---

## 10. MVP Scope

### 10.1 Must Have (Launch Blockers)

- [ ] 3 playable decks (Iron Colossus Prime, The Broodmother, The Blood Sovereign)
- [ ] Human vs AI with opponent selection (Greedy/MCTS/PPO/AlphaZero)
- [ ] Spectator Mode (AI vs AI) with playback controls
- [ ] Glassbox visualization (hybrid approach)
- [ ] Sound effects (UI + combat basics)
- [ ] 3 Tier 1 Commander 3D models
- [ ] Faction-themed Farsight Tables (3 variants)
- [ ] Tier 2 crystal gem tokens with parallax
- [ ] Basic tutorial
- [ ] Settings menu
- [ ] Works in browser (Chrome, Firefox, Safari, Edge)
- [ ] Deployed to HuggingFace Spaces

### 10.2 Should Have (Post-Launch Priority)

- [ ] All 12 commander decks selectable
- [ ] All 12 commander 3D models
- [ ] Full card art for 3 starter decks (~90 cards)
- [ ] Turning point detection and jump
- [ ] Commentary generation
- [ ] Replay export and sharing
- [ ] Faction-specific music tracks
- [ ] Full keyword sound effects

### 10.3 Nice to Have (Future)

- [ ] Full MCTS tree visualization
- [ ] Card art for all 300 cards
- [ ] Deck builder interface
- [ ] Match history and statistics
- [ ] Accessibility features (colorblind, large text)
- [ ] Mobile support
- [ ] Localization

### 10.4 Explicitly Deferred

- Multiplayer networking
- Account system / login
- Leaderboards
- Card collection / unlocks
- JRPG campaign elements
- Deck sharing

---

## 11. Implementation Roadmap

### Phase 5A-1: WASM Foundation ✅

**Goal:** Prove the technical pipeline works.

- [x] Add `web` feature flag to `cardgame` crate
- [x] Feature-gate `rayon` usage (parallel/sequential code paths)
- [x] Embed card/deck data for WASM builds (`embedded_data.rs`)
- [x] Configure Trunk build pipeline (`Trunk.toml`, `index.html`)
- [x] Get basic Bevy scene running in browser
- [x] Benchmark MCTS performance in WASM (~50 games/sec native baseline)
- [ ] Set up HuggingFace Space (deferred until polish complete)

**Implementation Notes (2026-01-20):**
- WASM binary size: ~29MB (release, wasm-opt)
- Required manual `main()` call via JS (wasm-bindgen quirk with Trunk)
- Fixed tonemapping (Reinhard, no LUT required)
- Fixed deck defaults to MVP decks
- All 634 cardgame tests pass

**Headless Mode Enhancement (2026-01-20):**
Enhanced `--headless` mode with three use cases:
- **Visual debugging** (default): Window with normal pacing for watching AI
- **Fast testing** (`--fast`): Skip visual timers for quick iteration
- **Benchmark mode** (`--fast --games N --json`): Multi-game runs with JSON stats

New CLI flags: `--fast`, `--games N`, `--json`, `--debug`

Files added/modified:
- `src/game/stats.rs` (new): `HeadlessStats` resource, JSON output structs
- `src/main.rs`: Extended CLI arguments
- `src/game/turn_loop.rs`: Fast mode (skip delays), debug logging
- `src/game/bridge.rs`: Deck ID storage for multi-game restart
- `src/ui/menu.rs`: `handle_headless_game_over` system for multi-game loop

### Phase 5A-2: Core Gameplay Loop

**Goal:** Playable game without polish.

- [ ] Implement full turn progression in Bevy
- [ ] Wire up AI opponent integration
- [ ] Click-to-play card interaction
- [ ] Basic combat visualization (no animations yet)
- [ ] Win/lose detection and game over screen
- [ ] Placeholder visuals (cubes/shapes)

### Phase 5A-3: Visual Foundation

**Goal:** The Farsight Table aesthetic.

- [ ] Design and model Farsight Table (3 variants)
- [ ] Implement crystal gem token mesh
- [ ] Create placeholder card art (text-based)
- [ ] Board layout with lane grid
- [ ] Basic lighting and camera setup
- [ ] HUD with life/essence/turn display

### Phase 5A-4: Art Pipeline Execution

**Goal:** Generate and integrate visual assets.

- [ ] Task Agent generates all prompts
- [ ] Generate 2D art for 3 starter decks
- [ ] Generate depth maps
- [ ] Create 3 commander 3D models (Meshy)
- [ ] Integrate assets into Bevy
- [ ] Implement parallax shader for tokens

### Phase 5A-5: Glassbox Integration

**Goal:** AI visualization working.

- [ ] Port existing egui Glassbox panels
- [ ] Add diegetic confidence indicator
- [ ] Implement "thinking" visualization (ghost arrows)
- [ ] Post-move explanation tooltips
- [ ] Toggle behavior ('G' key)

### Phase 5A-6: Spectator Mode

**Goal:** Full AI vs AI experience.

- [ ] Implement AI vs AI match flow
- [ ] Playback controls (play/pause/speed/step)
- [ ] Turning point detection
- [ ] Commentary template system
- [ ] Replay save/load (local storage)
- [ ] Shareable links

### Phase 5A-7: Audio & Polish

**Goal:** Production quality feel.

- [ ] Source and integrate music (3 faction tracks)
- [ ] Integrate UI sound effects
- [ ] Integrate combat sound effects
- [ ] Spawn/death particle effects
- [ ] Card play feedback (particles, shake, sound)
- [ ] Combat animations (grouped by lane)

### Phase 5A-8: UI & Onboarding

**Goal:** Complete user experience.

- [ ] Main menu design
- [ ] Faction-first deck selection flow
- [ ] Visual commander carousel
- [ ] AI opponent selection
- [ ] Settings menu (all options)
- [ ] Interactive tutorial
- [ ] Rules reference

### Phase 5A-9: Testing & Launch

**Goal:** Ship it.

- [ ] Cross-browser testing (Chrome, Firefox, Safari, Edge)
- [ ] Performance optimization
- [ ] Bug fixes
- [ ] Final HuggingFace Space configuration
- [ ] Write landing page / README
- [ ] Announce launch

---

## Appendices

### A. File References

| Document | Location |
|----------|----------|
| Faction Lore | `docs/lore.md` |
| Card Database | `data/cards/core_set/*.yaml` |
| Deck Definitions | `data/decks/{faction}/*.toml` |
| Engine Design | `docs/essence-wars-design.md` |
| JRPG Architecture | `docs/jrpg-architecture.md` |
| Glassbox Implementation | `docs/bevy-glassbox-implementation.md` |
| Existing 3D Client | `crates/essence-wars-3d/` |

### B. MVP Deck Contents

**Iron Colossus Prime** (`data/decks/argentum/colossus_wall.toml`)
- Commander: Iron Colossus Prime (ID: 1057)
- Strategy: Defensive Guard stacking
- Key cards: Guard creatures, Fortify effects, health buffs

**The Broodmother** (`data/decks/symbiote/broodmother_swarm.toml`)
- Commander: The Broodmother (ID: 2060)
- Strategy: Rush swarm aggression
- Key cards: Rush creatures, token generation, sacrifice effects

**The Blood Sovereign** (`data/decks/obsidion/sovereign_lifesteal.toml`)
- Commander: The Blood Sovereign (ID: 3055)
- Strategy: Lifesteal sustain
- Key cards: Lifesteal creatures, drain spells, life manipulation

### C. Estimated Asset Counts

| Asset Type | MVP Count | Full Game |
|------------|-----------|-----------|
| Card 2D Art (Creatures) | ~60 (3 decks) | ~210 |
| Card 2D Art (Spells) | ~15 (3 decks) | ~50 |
| Card 2D Art (Supports) | ~15 (3 decks) | ~40 |
| Depth Maps | ~90 | 300 |
| Commander 3D Models | 3 | 12 |
| Table Variants | 3 | 3 |
| Creature Token Gem Models | 3 (faction) | 4 (+neutral) |
| Support Rune Plate Models | 3 (faction) | 4 (+neutral) |
| Music Tracks | 3 | 3+ |
| Sound Effects | ~40 | ~60 |

### D. External Services

| Service | Purpose | Estimated Cost |
|---------|---------|----------------|
| Image Generation | 2D card art | ~$50-100 |
| Meshy.ai | 3D commander models | ~$30-50 |
| HuggingFace Spaces | Hosting | Free (community tier) |
| Freesound/OpenGameArt | Audio assets | Free (CC licensed) |

---

*Document Version: 1.0*
*Created: 2026-01-19*
*Authors: Chris + Claude (Opus 4.5)*
