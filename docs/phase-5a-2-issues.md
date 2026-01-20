# Phase 5A-2 Issues Tracker

> **Created**: 2026-01-20
> **Status**: Active
> **Goal**: Track and resolve all issues blocking Phase 5A-2 (Core Gameplay Loop) completion

---

## Summary

**Overall Phase 5A-2 Status: ~75% Complete**

| Category | Status | Blockers |
|----------|--------|----------|
| Turn Progression | 90% | Lane-grouped combat |
| AI Integration | 90% | ~~Introspection pipeline~~, bot selection UI |
| Card Interaction | 40% | Drag-and-drop, spell targeting |
| Rendering | 50% | Support slots, visual effects |
| Game Over | 100% | ~~Win reason display~~, ~~draw condition~~, ~~Play Again~~ |
| Glassbox | 50% | ~~Data pipeline~~ (basic wired, full MCTS tree TBD) |

**Phase 1 Quick Wins: COMPLETE** (5/5 issues resolved)

---

## Critical Issues

### ISSUE-001: Introspection Pipeline Broken
- **Severity**: HIGH (blocks Phase 5A-5 Glassbox)
- **Location**: `crates/essence-wars-3d/src/game/turn_loop.rs:247`
- **Problem**: TODO comment - bot decisions never captured for Glassbox visualization
- **Details**:
  - `IntrospectionConfig::full()` is set in `BotConfig` but never used
  - `GameBridge::last_decision` field exists but is never populated
  - MCTS tree snapshots unavailable to UI
- **Fix**: After `client.select_bot_action(&mut bot)`, capture `bot.last_decision()` and store in `GameBridge`
- **Effort**: 30 min
- **Status**: [x] COMPLETED (2026-01-20)
- **Notes**: Basic pipeline wired - captures timing, action, PolicyOutput. Full MCTS tree snapshot requires implementing `AnalyzableBot` for `MctsBot` (future enhancement).

### ISSUE-002: Support Slots Not Rendered
- **Severity**: HIGH (feature incomplete)
- **Location**: `crates/essence-wars-3d/src/rendering/board.rs`
- **Problem**: Design specifies 2 support slots per player, zero implementation
- **Details**:
  - Game engine supports supports (durability, triggered effects)
  - 3D client has no visualization for support cards
  - Design calls for "Floating Rune Plates" with durability pips
- **Fix**: Add support slot meshes (floating planes), render support cards, show durability
- **Effort**: 3-4 hrs
- **Status**: [ ] Open

### ISSUE-003: Spell Targeting Incomplete
- **Severity**: HIGH (feature incomplete)
- **Location**: `crates/essence-wars-3d/src/ui/player_input.rs:282`
- **Problem**: TODO comment - spells cast without proper targeting
- **Details**:
  - Creatures can be played to slots
  - Spells with targeting (TargetAllyCreature, TargetEnemyCreature, etc.) not handled
  - Currently spells just cast without target selection
- **Fix**: Implement target selection UI for spells based on `targeting` field
- **Effort**: 2-3 hrs
- **Status**: [ ] Open

### ISSUE-004: Drag-and-Drop Not Implemented
- **Severity**: HIGH (design mismatch)
- **Location**: `crates/essence-wars-3d/src/ui/player_input.rs`
- **Problem**: Design specifies drag-and-drop, implementation uses click-confirm modal
- **Details**:
  - Current: Click card → modal dialog → click slot button
  - Design: Drag card from hand → valid slots highlight → drop on slot
  - Missing: trajectory line, snap-to-slot, visual feedback during drag
- **Fix**: Implement egui drag payload system or 3D raycasting for drag-and-drop
- **Effort**: 4-6 hrs
- **Status**: [ ] Open
- **Decision**: User confirmed: implement drag-and-drop

---

## Medium Issues

### ISSUE-005: Win Reason Not Displayed
- **Severity**: MEDIUM
- **Location**: `crates/essence-wars-3d/src/ui/menu.rs:173`
- **Problem**: `GameResult::Win { winner, reason }` - reason field ignored with `..`
- **Details**:
  - Win reasons: LifeReachedZero, TurnLimitHigherLife, VictoryPointsReached
  - Game over screen shows winner but not why they won
- **Fix**: Pattern match on `reason`, display human-readable text
- **Effort**: 10 min
- **Status**: [x] COMPLETED (2026-01-20)

### ISSUE-006: Draw Condition Never Set
- **Severity**: MEDIUM
- **Location**: `crates/cardgame/src/core/engine/game_engine.rs`
- **Problem**: `GameResult::Draw` defined but never used
- **Details**:
  - If both players reach 0 life simultaneously, arbitrary winner chosen
  - `GameResult::Draw` enum variant exists but code never sets it
  - UI has placeholder for draw screen but it's unreachable
- **Fix**: Add check in `check_life_victory()` for simultaneous deaths
- **Effort**: 15 min
- **Status**: [x] COMPLETED (2026-01-20)

### ISSUE-007: Bot Type Selection Missing
- **Severity**: MEDIUM
- **Location**: `crates/essence-wars-3d/src/ui/menu.rs`
- **Problem**: Only MCTS bot available, no UI to select Greedy/PPO/AlphaZero
- **Details**:
  - Design Section 7.3 shows radio buttons for AI type selection
  - `BotConfig` hardcoded to MCTS for both players
  - Greedy bot exists in cardgame but never instantiated in 3D client
- **Fix**: Add AI type dropdown/radio in menu, wire to `BotConfig`
- **Effort**: 1-2 hrs
- **Status**: [ ] Open

### ISSUE-008: Combat Not Grouped by Lane
- **Severity**: MEDIUM
- **Location**: `crates/essence-wars-3d/src/game/turn_loop.rs`
- **Problem**: Combat events processed individually, not lane-by-lane
- **Details**:
  - Design Section 3.1: "Combat resolves lane by lane (Lane 1 → Lane 2 → ... → Lane 5)"
  - Current: All combat events fire as they come from engine
  - Missing: Visual pause between lane combats
- **Fix**: Buffer combat events, group by lane, add delays between lanes
- **Effort**: 2-3 hrs
- **Status**: [ ] Open

---

## Low Priority Issues

### ISSUE-009: Creature Meshes Are Placeholders
- **Severity**: LOW (acceptable for Phase 5A-2)
- **Location**: `crates/essence-wars-3d/src/rendering/creatures.rs`
- **Problem**: Creatures render as capsules, not gem/crystal tokens
- **Details**:
  - Design Section 1.3: Faceted gem/crystal form with card art inside
  - Current: Simple `Capsule3d` (0.4 × 0.8) with player colors
  - Missing: Faction-specific variations, card art display, parallax effect
- **Fix**: Replace capsule with low-poly gem mesh, add faction materials
- **Effort**: 1-2 hrs
- **Status**: [ ] Open (Phase 5A-3)

### ISSUE-010: Spawn/Death Effects Missing
- **Severity**: LOW (visual polish)
- **Location**: `crates/essence-wars-3d/src/rendering/creatures.rs`
- **Problem**: Creatures spawn/despawn instantly with no visual feedback
- **Details**:
  - Design Section 1.5: Wireframe rise, particle burst, shatter on death
  - Current: Instant spawn/despawn
- **Fix**: Add fade-in/out, scale animation, particle systems
- **Effort**: 2-3 hrs
- **Status**: [ ] Open (Phase 5A-3)

### ISSUE-011: Damage Number Styling Missing
- **Severity**: LOW (visual polish)
- **Location**: `crates/essence-wars-3d/src/rendering/combat.rs`
- **Problem**: Damage numbers are basic, no color coding or icons
- **Details**:
  - Design Section 3.3: Red for damage, green for heal, purple for lifesteal
  - "BLOCKED" text with shield icon, skull for lethal
  - Current: Plain floating numbers
- **Fix**: Add color parameters, icon sprites
- **Effort**: 1-2 hrs
- **Status**: [ ] Open (Phase 5A-3)

### ISSUE-012: No Farsight Table Aesthetic
- **Severity**: LOW (Phase 5A-3 scope)
- **Location**: `crates/essence-wars-3d/src/rendering/board.rs`
- **Problem**: Board is plain brown cuboid, not faction-themed Farsight Table
- **Details**:
  - Design Section 1.1-1.2: Faction-specific table variants
  - Argentum: dark wood + brass, Symbiote: living wood, Obsidion: obsidian glass
  - Crystal nodes, data readouts on edges, holographic projection effect
- **Fix**: Create/import table 3D models, faction materials
- **Effort**: 4-8 hrs
- **Status**: [ ] Open (Phase 5A-3)

### ISSUE-013: "Play Again" Button Missing
- **Severity**: LOW (UX improvement)
- **Location**: `crates/essence-wars-3d/src/ui/menu.rs`
- **Problem**: Must return to menu to start new game with same settings
- **Details**:
  - Game over screen only has "Return to Menu"
  - Should have "Play Again" for quick rematch
- **Fix**: Add button that calls `bridge.restart_with_seed(new_seed)`
- **Effort**: 5 min
- **Status**: [x] COMPLETED (2026-01-20)

---

## Code Quality Issues

### ISSUE-014: Hand Display Overlap During Human Turn
- **Severity**: LOW (code smell)
- **Location**: `crates/essence-wars-3d/src/ui/hand.rs` and `player_input.rs`
- **Problem**: Both hand panels render simultaneously during human turns
- **Details**:
  - `hand.rs`: Spectator hand display (panel "player_hand") - runs ALWAYS during Playing
  - `player_input.rs`: Interactive hand display (panel "human_hand") - runs only on human turn
  - Different panel IDs = two bottom panels stacked during human turn
  - `hand.rs` is NOT dead code - it's the spectator view for AI vs AI mode
- **Decision**: Option A - Add exclusion condition to `hand.rs`
- **Fix**:
  1. Add `.run_if(not(is_human_turn))` to `hand.rs` system
  2. Show CURRENT player's hand (not always P1) for better spectating
  3. Export `is_human_turn` from `player_input.rs` for reuse
- **Effort**: 15 min
- **Status**: [x] COMPLETED (2026-01-20)

### ISSUE-015: HeadlessStats Not Shared with UI
- **Severity**: LOW
- **Location**: `crates/essence-wars-3d/src/ui/menu.rs`
- **Problem**: Single-game stats not displayed on game over screen
- **Details**:
  - `HeadlessStats` tracks games, wins, turns, actions
  - `draw_game_over()` doesn't access this data
  - Could show action count, game duration for single games too
- **Fix**: Extract stats display from headless-only to shared component
- **Effort**: 30 min
- **Status**: [ ] Open

---

## Resolution Order (Recommended)

### Phase 1: Quick Wins (< 1 hour total) ✅ COMPLETE
1. [x] ISSUE-005: Display win reason (10 min)
2. [x] ISSUE-006: Implement draw condition (15 min)
3. [x] ISSUE-013: Add "Play Again" button (5 min)
4. [x] ISSUE-014: Fix hand display overlap + spectator mode (15 min)
5. [x] ISSUE-001: Wire introspection pipeline (30 min)

### Phase 2: Core Features (2-4 hours)
6. [ ] ISSUE-007: Bot type selection UI (1-2 hrs)
7. [ ] ISSUE-003: Spell targeting (2-3 hrs)

### Phase 3: Major Features (4-8 hours)
8. [ ] ISSUE-004: Drag-and-drop interaction (4-6 hrs)
9. [ ] ISSUE-002: Support slot rendering (3-4 hrs)
10. [ ] ISSUE-008: Lane-grouped combat (2-3 hrs)

### Phase 4: Visual Polish (Phase 5A-3)
11. [ ] ISSUE-009: Gem/crystal creature meshes
12. [ ] ISSUE-010: Spawn/death effects
13. [ ] ISSUE-011: Damage number styling
14. [ ] ISSUE-012: Farsight Table aesthetic

---

## Files Reference

| File | Purpose | Issues |
|------|---------|--------|
| `src/game/turn_loop.rs` | Turn execution, AI integration | 001, 008 |
| `src/game/bridge.rs` | Engine ↔ Bevy bridge | 001 |
| `src/ui/menu.rs` | Menu, game over screens | 005, 006, 007, 013 |
| `src/ui/player_input.rs` | Human card interaction | 003, 004 |
| `src/ui/hand.rs` | Hand display (possibly dead) | 014 |
| `src/rendering/board.rs` | Board and slot rendering | 002, 012 |
| `src/rendering/creatures.rs` | Creature visualization | 009, 010 |
| `src/rendering/combat.rs` | Combat animations | 011 |
| `cardgame/.../game_engine.rs` | Win/lose detection | 006 |

---

*Last Updated: 2026-01-20 (Phase 1 Quick Wins completed)*
