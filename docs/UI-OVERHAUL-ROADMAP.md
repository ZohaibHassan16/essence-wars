# UI Overhaul Roadmap - Epic Commander Experience

> **Target Version:** 0.9.0
> **Created:** 2026-01-25
> **Status:** In Progress - Phase 3 Complete

---

## Overview

This document outlines the implementation plan for a major UI overhaul focusing on two key areas:

1. **Board Layout Rework** - Commanders displayed as large, prominent card zones on the left edge
2. **Menu Rework** - Wizard-style deck selection with visual appeal and faction theming

### Design Goals

- Showcase commanders as the premium centerpiece mechanic
- Create an epic, immersive experience that makes players want to play
- Move away from "debugging app" aesthetic to polished game UI
- Consistent visual language across Human vs AI and AI vs AI modes
- Responsive design for various viewport sizes (default: 2560x1600)

### Key Decisions

| Decision | Choice |
|----------|--------|
| Commander card width | **250px** default, responsive scaling |
| Commander position | **Left edge** (opponent top-left, player bottom-left) |
| Menu flow | **Wizard style** (Step 1 → 2 → 3 → Start) |
| Menu consistency | **Same UI** for Human vs AI and AI vs AI |
| Deck flavor data | **TOML files** (not hardcoded) |

---

## ✅ Phase 1: Data Layer Enhancement [COMPLETE]

**Goal:** Ensure all deck files have rich descriptions for the new UI

**Estimated Scope:** 12 TOML file updates, backend DTO updates

### Tasks

- [x] **1.1** Audit existing deck descriptions
  - Review all 12 deck TOML files
  - Identify missing or placeholder descriptions

- [x] **1.2** Write compelling deck descriptions
  - Each description should be 1-2 sentences
  - Convey playstyle and deck fantasy
  - Make players excited to try the deck

- [x] **1.3** Add `playstyle` field to deck TOML schema
  ```toml
  id = "artificer_tokens"
  name = "Artificer Tokens"
  commander = 5000
  description = "Overwhelm your enemies with an endless tide of mechanical constructs. The High Artificer's workshop never stops producing."
  playstyle = "Token Swarm"  # NEW: Short archetype tag
  cards = [...]
  ```

- [x] **1.4** Update `DeckDefinition` struct in Rust
  - Add `playstyle: String` field (with `#[serde(default)]`)
  - Update `DeckInfo` DTO with playstyle

- [x] **1.5** Update frontend `DeckInfo` TypeScript interface
  - Add `playstyle: string` field

### Deck Descriptions to Write

| Deck ID | Commander | Description Theme |
|---------|-----------|-------------------|
| artificer_tokens | High Artificer | Token swarm, endless production |
| sanctum_healer | Sanctum Healer | Healing wall, outlast opponents |
| vex_piercing | Siege Marshal Vex | Piercing assault, break through guards |
| architect_fortify | Grand Architect | Fortified defense, immovable wall |
| broodmother_swarm | Broodmother | Rush aggression, broodling generation |
| plague_volatile | Plague Sovereign | Death synergy, face damage |
| alpha_frenzy | Alpha of the Hunt | Frenzy stacking, snowball attacks |
| grove_regenerate | Eternal Grove | Regeneration, sustain and heal |
| sovereign_lifesteal | Blood Sovereign | Lifesteal, drain opponents dry |
| kael_assassin | Shadow Emperor Kael | Card advantage, enemy death triggers |
| shadow_weaver | Shadow Weaver | Stealth, surprise attacks |
| archon_burst | Void Archon | Quick keyword, burst damage |

### Acceptance Criteria

- [x] All 12 decks have compelling descriptions (1-2 sentences)
- [x] All 12 decks have playstyle tags
- [x] Backend loads and serves playstyle data
- [x] Frontend types updated
- [x] Existing tests pass (668 tests)

---

## ✅ Phase 2: Commander Card Component [COMPLETE]

**Goal:** Create a large, visually impressive commander card component

**Estimated Scope:** 1 new Svelte component, CSS styling

### Design Specification

```
┌─────────────────────────────┐
│                             │
│      ┌─────────────────┐    │
│      │                 │    │
│      │    PORTRAIT     │    │  ← Large portrait (full art)
│      │     (150px)     │    │
│      │                 │    │
│      └─────────────────┘    │
│                             │
│   ═══════════════════════   │  ← Faction-colored divider
│                             │
│   THE HIGH ARTIFICER        │  ← Name (bold, faction color)
│                             │
│   ┌─────────────────────┐   │
│   │      30 HP          │   │  ← Life total (large, prominent)
│   └─────────────────────┘   │
│                             │
│   At the start of your      │
│   turn, summon a 1/1        │  ← Ability text (readable)
│   Brass Cog.                │
│                             │
│   ═══════════════════════   │
│   ARGENTUM                  │  ← Faction badge
└─────────────────────────────┘
        250px wide
```

### Tasks

- [x] **2.1** Create `CommanderCardLarge.svelte` component
  - Location: `src/lib/components/board/CommanderCardLarge.svelte`
  - Props: `commander: CommanderDto`, `life: number`, `maxLife: number`, `isActive: boolean`, `isPlayer: boolean`
  - Added: `essence`, `maxEssence` for essence orb display

- [x] **2.2** Implement portrait display
  - Use full portrait art from `static/portrait/{name}.webp`
  - Aspect ratio handling for various portrait sizes
  - Faction-colored border/frame

- [x] **2.3** Implement life display
  - Large, prominent HP badge
  - Color coding: green when healthy, yellow when damaged, red when critical
  - Flash animation when life changes (red for damage, green for healing)

- [x] **2.4** Implement ability text display
  - Readable font size with line-clamp-3
  - Keyword highlighting in popup
  - Full keyword definitions in hover popup

- [x] **2.5** Implement faction styling
  - Faction-specific gradients for background (Argentum gold, Symbiote green, Obsidion cyan)
  - Faction-colored borders and accents
  - Faction badge at bottom with display name

- [x] **2.6** Implement hover popup
  - Triggered on hover
  - Shows full ability text
  - Keyword definitions (Rush, Guard, Lifesteal, etc.)
  - Stats summary

- [x] **2.7** Implement active turn indicator
  - Glowing border/shadow when it's this player's turn
  - Subtle pulse animation via CSS

- [x] **2.8** Add CSS variables for commander card sizing
  ```css
  --commander-card-width: 250px;
  --commander-portrait-height: 150px;
  ```
  - Added responsive breakpoints for all viewport sizes

### Acceptance Criteria

- [x] Commander card displays all required information
- [x] Faction styling looks distinct and polished
- [x] Hover popup shows full details
- [x] Active turn is clearly visible
- [x] Life changes animate smoothly
- [x] Component is responsive (180px-300px based on viewport)

---

## ✅ Phase 3: Board Layout Rework [COMPLETE]

**Goal:** Restructure GameBoard with commanders on the left edge

**Estimated Scope:** Major layout changes to GameBoard.svelte and SpectatorPlayback.svelte

### New Layout Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Header Bar - minimal, just back button and match title]           │
├────────────┬────────────────────────────────────────────────┬──────┤
│            │                                                │      │
│  OPPONENT  │   [Opponent Hand - fanned cards]               │  S   │
│  COMMANDER │                                                │  I   │
│            ├────────────────────────────────────────────────┤  D   │
│  250px     │                                                │  E   │
│  wide      │   [S1] [1] [2] [3] [4] [5] [S2]               │  B   │
│            │        Opponent Battlefield                    │  A   │
│            │                                                │  R   │
├────────────┼────────────────────────────────────────────────┤      │
│            │                                                │  AI  │
│            │        ═══ Turn 7 | Your Turn ═══              │ Hint │
│            │                                                │      │
├────────────┼────────────────────────────────────────────────┤ Log  │
│            │                                                │      │
│  YOUR      │   [S1] [1] [2] [3] [4] [5] [S2]               │      │
│  COMMANDER │        Player Battlefield                      │      │
│            │                                                │      │
│  250px     ├────────────────────────────────────────────────┤      │
│  wide      │                                                │      │
│            │   [Player Hand - fanned cards]                 │      │
│            │                                                │      │
│            ├────────────────────────────────────────────────┤      │
│            │   [Action Bar: Undo | End Turn | Quit | Audio] │      │
└────────────┴────────────────────────────────────────────────┴──────┘
```

### Tasks

- [x] **3.1** Refactor GameBoard.svelte layout
  - Add left column for commander cards
  - Use CSS Grid or Flexbox for 3-column layout
  - Maintain existing battlefield and hand components

- [x] **3.2** Remove CommandZone from info bars
  - Removed the small CommandZone components from GameBoard
  - Compact stats now displayed below/above commander cards

- [x] **3.3** Integrate CommanderCardLarge components
  - Opponent commander: top-left area
  - Player commander: bottom-left area
  - Wire up life totals and active turn state

- [x] **3.4** Update header bar
  - Simplified to: Back button, Match title (decks), Audio controls
  - Compact stats moved to left column near commander cards

- [x] **3.5** Adjust battlefield spacing
  - Account for narrower main area (viewport - 250px - sidebar)
  - Creatures and supports fit comfortably
  - Uses CSS variables for responsive sizing

- [x] **3.6** Update SpectatorPlayback.svelte
  - Applied same 3-column layout
  - Both commanders visible with P1/P2 labels
  - Result badges and save replay in action bar

- [x] **3.7** Add responsive breakpoints for commander card
  - Responsive sizing from 180px (short displays) to 300px (4K)
  - Height-based media queries in app.css

### Acceptance Criteria

- [x] Commanders prominently displayed on left edge
- [x] Board still playable and usable
- [x] Responsive at multiple viewport sizes
- [x] SpectatorPlayback matches GameBoard layout
- [x] All existing game functionality works (668 tests pass)

---

## Phase 4: Deck Selection Component

**Goal:** Create a beautiful, visual deck selection component

**Estimated Scope:** New components for deck cards and faction tabs

### Design Specification - Deck Card

```
┌─────────────────────────┐
│    ┌───────────────┐    │
│    │               │    │
│    │   COMMANDER   │    │
│    │   PORTRAIT    │    │
│    │               │    │
│    └───────────────┘    │
│                         │
│   THE HIGH ARTIFICER    │  ← Commander name
│   ─────────────────     │
│   "Artificer Tokens"    │  ← Deck name
│                         │
│   Token Swarm           │  ← Playstyle tag
│                         │
│   ✨ SELECTED           │  ← Selection indicator (if selected)
└─────────────────────────┘
       ~180px wide
```

### Tasks

- [ ] **4.1** Create `DeckCard.svelte` component
  - Location: `src/lib/components/menu/DeckCard.svelte`
  - Props: `deck: DeckInfo`, `commander: CommanderDto`, `isSelected: boolean`, `onSelect: () => void`

- [ ] **4.2** Create `FactionTabs.svelte` component
  - Location: `src/lib/components/menu/FactionTabs.svelte`
  - Props: `selectedFaction: string`, `onSelect: (faction: string) => void`
  - Tabs: Argentum, Symbiote, Obsidion
  - Faction icons/colors for each tab

- [ ] **4.3** Create `DeckGrid.svelte` component
  - Location: `src/lib/components/menu/DeckGrid.svelte`
  - Displays deck cards in a grid (4 per faction)
  - Filters by selected faction tab

- [ ] **4.4** Create `DeckPreview.svelte` component
  - Location: `src/lib/components/menu/DeckPreview.svelte`
  - Shows detailed info for selected deck:
    - Commander portrait (larger)
    - Commander name and ability
    - Deck name and description
    - Playstyle
    - Card count

- [ ] **4.5** Style deck cards with faction theming
  - Faction-colored borders
  - Hover effects (lift, glow)
  - Selection state (prominent border, checkmark)

- [ ] **4.6** Add selection animations
  - Smooth transitions when selecting/deselecting
  - Card "pops" slightly when selected

### Acceptance Criteria

- [ ] Deck cards display commander portraits
- [ ] Faction tabs filter decks correctly
- [ ] Selection state is clear and animated
- [ ] Preview shows full deck details
- [ ] Faction styling is consistent and polished

---

## Phase 5: Wizard Flow Implementation

**Goal:** Create the step-by-step deck selection wizard

**Estimated Scope:** New wizard container, step components, state management

### Wizard Flow

```
Step 1: "Choose Your Commander"
  - Faction tabs
  - Deck grid (4 cards per faction)
  - Deck preview panel
  - "Next: Choose Opponent" button

Step 2: "Choose Your Opponent"
  - Same UI as Step 1
  - Shows "You selected: [deck]" summary
  - "Next: Game Options" button

Step 3: "Game Options" (AI vs AI has extra options)
  - Bot selection (for opponent / both players)
  - Turn order (Human vs AI only)
  - Advanced options toggle
  - "Start Match" button
```

### Tasks

- [ ] **5.1** Create `DeckSelectionWizard.svelte` container
  - Location: `src/lib/components/menu/DeckSelectionWizard.svelte`
  - Manages wizard state (current step, selections)
  - Props: `mode: 'human-vs-ai' | 'ai-vs-ai'`

- [ ] **5.2** Create `WizardStep.svelte` component
  - Wrapper for each step with consistent styling
  - Step indicator (1, 2, 3)
  - Navigation buttons

- [ ] **5.3** Implement Step 1: Player Deck Selection
  - Full-screen experience
  - Faction tabs + deck grid
  - Deck preview
  - "Next" button (disabled until selection)

- [ ] **5.4** Implement Step 2: Opponent/P2 Deck Selection
  - Same UI as Step 1
  - Summary of previous selection
  - Different header text

- [ ] **5.5** Implement Step 3: Game Options
  - Bot selection (styled cards, not dropdowns)
  - Turn order toggle (Human vs AI)
  - Advanced options (collapsible)
  - "Start Match" CTA

- [ ] **5.6** Add step transition animations
  - Slide/fade between steps
  - Progress indicator

- [ ] **5.7** Implement back navigation
  - "Back" button on each step
  - Preserves selections when going back

### Acceptance Criteria

- [ ] Wizard flows smoothly through all steps
- [ ] Selections persist across steps
- [ ] Back navigation works correctly
- [ ] Mode differences handled (human-vs-ai vs ai-vs-ai)
- [ ] Animations are smooth and polished

---

## Phase 6: Menu Integration

**Goal:** Replace existing SetupScreen and SpectatorSetup with new wizard

**Estimated Scope:** Refactor existing screens, maintain routing

### Tasks

- [ ] **6.1** Refactor SetupScreen.svelte
  - Replace current UI with DeckSelectionWizard
  - Mode: 'human-vs-ai'
  - Maintain existing game start logic

- [ ] **6.2** Refactor SpectatorSetup.svelte
  - Replace current UI with DeckSelectionWizard
  - Mode: 'ai-vs-ai'
  - Include bot selection for both players
  - Include MCTS/Alpha-Beta options

- [ ] **6.3** Update MainMenu navigation
  - Ensure routing to new screens works
  - Update any preview/teaser text

- [ ] **6.4** Handle loading states
  - Show loading while fetching decks/bots
  - Graceful error handling

- [ ] **6.5** Add background visuals to menu
  - Subtle faction artwork or patterns
  - Particle effects or ambient animation
  - Dark overlay for readability

- [ ] **6.6** Delete old components (cleanup)
  - Remove deprecated setup components
  - Remove old CommandZone (small version)

### Acceptance Criteria

- [ ] Human vs AI setup uses new wizard
- [ ] AI vs AI setup uses new wizard
- [ ] All game modes launch correctly
- [ ] Loading and error states handled
- [ ] Menu feels polished and epic

---

## Phase 7: Polish & Responsive Design

**Goal:** Fine-tune styling, animations, and responsive behavior

**Estimated Scope:** CSS refinements, animation polish, testing

### Tasks

- [ ] **7.1** Responsive testing and fixes
  - Test at: 1280x720, 1920x1080, 2560x1440, 2560x1600, 3840x2160
  - Adjust breakpoints as needed
  - Ensure all elements remain usable

- [ ] **7.2** Animation polish
  - Turn transition effects
  - Card hover effects
  - Commander ability trigger visuals (glow/pulse)
  - Life change animations

- [ ] **7.3** Sound integration
  - Menu navigation sounds
  - Selection confirmation sounds
  - Match start fanfare

- [ ] **7.4** Keyboard navigation
  - Tab through deck cards
  - Enter to select
  - Escape to go back
  - Ensure accessibility

- [ ] **7.5** Performance optimization
  - Lazy load commander portraits
  - Optimize animations (will-change, GPU layers)
  - Test on lower-end hardware

- [ ] **7.6** Visual polish pass
  - Consistent shadows and borders
  - Color harmony check
  - Typography hierarchy
  - Icon consistency

- [ ] **7.7** Error state styling
  - Network errors
  - Missing data
  - Invalid states

### Acceptance Criteria

- [ ] UI works at all target resolutions
- [ ] Animations are smooth (60fps)
- [ ] Sounds enhance experience
- [ ] Keyboard navigation works
- [ ] No performance regressions
- [ ] Visual consistency throughout

---

## Phase 8: Testing & Documentation

**Goal:** Ensure quality and update documentation

**Estimated Scope:** Testing, docs, version bump

### Tasks

- [ ] **8.1** Manual testing checklist
  - [ ] Human vs AI: full game flow
  - [ ] AI vs AI: full spectator flow
  - [ ] All 12 decks selectable
  - [ ] All 4 bot types work
  - [ ] Commander abilities display correctly
  - [ ] Life totals update correctly
  - [ ] Hover popups show full info

- [ ] **8.2** Run test suite
  ```bash
  cargo nextest run --status-level=fail
  pnpm check
  ```

- [ ] **8.3** Update CLAUDE.md
  - Document new UI components
  - Update setup flow description
  - Add UI-OVERHAUL-ROADMAP.md reference

- [ ] **8.4** Update screenshots
  - New board layout
  - New deck selection wizard
  - Both game modes

- [ ] **8.5** Version bump to 0.9.0
  - Root Cargo.toml
  - Update version tests

- [ ] **8.6** Create CHANGELOG entry
  - Summarize UI overhaul
  - List key features

### Acceptance Criteria

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Screenshots current
- [ ] Version 0.9.0 tagged
- [ ] CHANGELOG complete

---

## Dependency Graph

```
Phase 1 (Data)
    │
    ▼
Phase 2 (Commander Card Component)
    │
    ▼
Phase 3 (Board Layout)
    │
    ├─────────────────┐
    ▼                 ▼
Phase 4 (Deck Cards)  (can parallel)
    │
    ▼
Phase 5 (Wizard Flow)
    │
    ▼
Phase 6 (Menu Integration)
    │
    ▼
Phase 7 (Polish)
    │
    ▼
Phase 8 (Testing)
    │
    ▼
  v0.9.0 Release
```

**Note:** Phases 2-3 (Board) and Phases 4-5 (Menu) can be developed in parallel after Phase 1 is complete.

---

## Progress Tracking

### Overall Status

| Phase | Status | Started | Completed |
|-------|--------|---------|-----------|
| Phase 1: Data Layer | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 2: Commander Card | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 3: Board Layout | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 4: Deck Selection | Not Started | - | - |
| Phase 5: Wizard Flow | Not Started | - | - |
| Phase 6: Menu Integration | Not Started | - | - |
| Phase 7: Polish | Not Started | - | - |
| Phase 8: Testing | Not Started | - | - |

---

## Appendix: File Inventory

### Files to Create

| File | Phase | Description |
|------|-------|-------------|
| `src/lib/components/board/CommanderCardLarge.svelte` | 2 | Large commander display for board |
| `src/lib/components/menu/DeckCard.svelte` | 4 | Deck selection card |
| `src/lib/components/menu/FactionTabs.svelte` | 4 | Faction tab navigation |
| `src/lib/components/menu/DeckGrid.svelte` | 4 | Grid of deck cards |
| `src/lib/components/menu/DeckPreview.svelte` | 4 | Selected deck preview |
| `src/lib/components/menu/DeckSelectionWizard.svelte` | 5 | Wizard container |
| `src/lib/components/menu/WizardStep.svelte` | 5 | Step wrapper |

### Files to Modify

| File | Phase | Changes |
|------|-------|---------|
| `data/decks/**/*.toml` | 1 | Add/improve descriptions, add playstyle |
| `crates/cardgame/src/decks.rs` | 1 | Add playstyle field |
| `src/lib/api/types.ts` | 1 | Add playstyle to DeckInfo |
| `src/app.css` | 2 | Add commander card CSS variables |
| `src/lib/components/GameBoard.svelte` | 3 | Major layout restructure |
| `src/lib/components/SpectatorPlayback.svelte` | 3 | Layout restructure |
| `src/lib/components/SetupScreen.svelte` | 6 | Replace with wizard |
| `src/lib/components/SpectatorSetup.svelte` | 6 | Replace with wizard |
| `CLAUDE.md` | 8 | Document new UI |

### Files to Delete

| File | Phase | Reason |
|------|-------|--------|
| `src/lib/components/board/CommandZone.svelte` | 3 | Replaced by CommanderCardLarge |

---

## Design Reference: Mockups

### Board Layout (ASCII)

```
┌─────────────────────────────────────────────────────────────────────┐
│ ← Back   The Sanctum Healer vs The Broodmother            🔊 🎵    │
├──────────┬──────────────────────────────────────────────────┬──────┤
│          │  ┌───┐┌───┐┌───┐┌───┐┌───┐┌───┐┌───┐            │      │
│ ┌──────┐ │  │ ? ││ ? ││ ? ││ ? ││ ? ││ ? ││ ? │   Hand: 7  │  AI  │
│ │      │ │  └───┘└───┘└───┘└───┘└───┘└───┘└───┘            │ Hint │
│ │ Brood│ ├──────────────────────────────────────────────────┤      │
│ │mother│ │  ┌──┐  ┌────┐┌────┐┌────┐┌────┐┌────┐  ┌──┐    │      │
│ │      │ │  │S1│  │ 1  ││ 2  ││ 3  ││ 4  ││ 5  │  │S2│    │ ──── │
│ │ 28HP │ │  └──┘  └────┘└────┘└────┘└────┘└────┘  └──┘    │      │
│ │      │ ├──────────────────────────────────────────────────┤Action│
│ │ Abil │ │            ══ Turn 16 | P1 Turn ══               │ Log  │
│ └──────┘ ├──────────────────────────────────────────────────┤      │
│          │  ┌──┐  ┌────┐┌────┐┌────┐┌────┐┌────┐  ┌──┐    │      │
│ ┌──────┐ │  │S1│  │ 1  ││ 2  ││ 3  ││ 4  ││ 5  │  │S2│    │      │
│ │      │ │  └──┘  └────┘└────┘└────┘└────┘└────┘  └──┘    │      │
│ │Sanctm│ ├──────────────────────────────────────────────────┤      │
│ │Healer│ │  ┌─────────────────────────────────────────┐     │      │
│ │      │ │  │     Player Hand (fanned cards)          │     │      │
│ │ 13HP │ │  └─────────────────────────────────────────┘     │      │
│ │      │ ├──────────────────────────────────────────────────┤      │
│ │ Abil │ │  [Undo]  [End Turn (Space)]  [Quit]  🔊         │      │
│ └──────┘ │                                                  │      │
└──────────┴──────────────────────────────────────────────────┴──────┘
```

### Menu Wizard Step 1 (ASCII)

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                    ⚔️  CHOOSE YOUR COMMANDER  ⚔️                    │
│                           Step 1 of 3                               │
│                                                                     │
│   ┌──────────────┬──────────────┬──────────────┐                   │
│   │  ⚙️ ARGENTUM │  🌿 SYMBIOTE │  💀 OBSIDION │                   │
│   └──────────────┴──────────────┴──────────────┘                   │
│                                                                     │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐              │
│   │ ▓▓▓▓▓▓▓ │  │ ▓▓▓▓▓▓▓ │  │ ▓▓▓▓▓▓▓ │  │ ▓▓▓▓▓▓▓ │              │
│   │ ▓     ▓ │  │ ▓     ▓ │  │ ▓     ▓ │  │ ▓     ▓ │              │
│   │ ▓▓▓▓▓▓▓ │  │ ▓▓▓▓▓▓▓ │  │ ▓▓▓▓▓▓▓ │  │ ▓▓▓▓▓▓▓ │              │
│   │         │  │         │  │         │  │         │              │
│   │  High   │  │ Sanctum │  │  Siege  │  │  Grand  │              │
│   │Artificer│  │ Healer  │  │ Marshal │  │Architect│              │
│   │         │  │         │  │         │  │         │              │
│   │  Token  │  │ Healing │  │Piercing │  │ Fortify │              │
│   │  Swarm  │  │  Wall   │  │ Assault │  │ Defense │              │
│   │         │  │         │  │         │  │         │              │
│   │✨SELECT │  │         │  │         │  │         │              │
│   └─────────┘  └─────────┘  └─────────┘  └─────────┘              │
│                                                                     │
│   ═══════════════════════════════════════════════════════════════  │
│                                                                     │
│   SELECTED: The High Artificer                                     │
│   "Artificer Tokens" - Token Swarm                                 │
│                                                                     │
│   Overwhelm your enemies with an endless tide of mechanical        │
│   constructs. The High Artificer's workshop never stops.           │
│                                                                     │
│                           ┌─────────────────────┐                  │
│   ← Back                  │  Next: Opponent →   │                  │
│                           └─────────────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of Roadmap*
