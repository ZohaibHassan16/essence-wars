# Commander System Implementation Roadmap

> **Target Version:** 0.8.0
> **Created:** 2026-01-25
> **Status:** Planning Complete - Ready for Implementation

---

## Overview

This document outlines the implementation plan for the Commander System Rework, as designed in `docs/design-commanders.md`. The implementation is divided into 9 sequential phases, each building on the previous.

### Key Decisions

| Decision | Choice |
|----------|--------|
| Phase execution | **Sequential** (not parallel) |
| Backwards compatibility | **Clean cut** (no legacy support) |
| Balance validation | **Automated arena tournaments** |
| Version | **0.8.0** |

---

## Systems Affected

| System | Location | Changes Required |
|--------|----------|------------------|
| **Card Schema** | `data/cards/core_set/*.yaml` | New commander card type |
| **Deck Format** | `data/decks/**/*.toml` | Separate commander field |
| **Core Engine** | `crates/cardgame/src/core/` | CardType, GameState, effects |
| **Effect System** | `crates/cardgame/src/engine/` | Commander passives & triggers |
| **Card Loading** | `crates/cardgame/src/cards/` | Load commander type |
| **AI Interface** | `crates/cardgame/src/bots/` | State tensor, evaluation |
| **MCP Server** | `crates/essence-wars-mcp/` | Display commander in state |
| **Tauri UI** | `crates/essence-wars-ui/` | Command Zone component |
| **Documentation** | `docs/` | design-engine.md, CLAUDE.md |

---

## Phase 1: Documentation & Schema Definition

**Goal:** Establish the contract before writing code

**Estimated Scope:** Documentation only

### Tasks

- [ ] **1.1** Update `docs/design-engine.md` with new commander rules
  - Add Section 8.4: Commander Cards
  - Update Section 3.1: Game Setup (commander selection)
  - Update Section 10: Win Conditions (commander retreat terminology)
  - Add Command Zone to board layout diagram

- [ ] **1.2** Define commander YAML schema formally
  - Document in design-commanders.md (already done)
  - Create example entries for reference

- [ ] **1.3** Update `CLAUDE.md` with new commander information
  - Update Card System section
  - Update Deck System section
  - Add Commander System section
  - Update State Tensor section (preview of changes)

- [ ] **1.4** Document state tensor changes for AI
  - Define new tensor layout with commander fields
  - Document in design-engine.md Section 11.1

### Acceptance Criteria

- [ ] All documentation reflects the new commander design
- [ ] Schema is formally defined and documented
- [ ] CLAUDE.md is up to date for AI assistant context

### Deliverable

All documentation serves as specification for implementation phases.

---

## Phase 2: Card Data Migration

**Goal:** Convert commander cards to new format in YAML

**Estimated Scope:** 4 YAML files, 12 commander cards

### Tasks

- [ ] **2.1** Add commander card type support to YAML structure
  - Commanders go in same faction files but with `card_type: commander`

- [ ] **2.2** Create new commander entries in `data/cards/core_set/argentum.yaml`
  - The High Artificer (1056)
  - The Sanctum Healer (1057)
  - Siege Marshal Vex (1058)
  - The Grand Architect (1059)

- [ ] **2.3** Create new commander entries in `data/cards/core_set/symbiote.yaml`
  - The Broodmother (2060)
  - Plague Sovereign (2061)
  - Alpha of the Hunt (2062)
  - The Eternal Grove (2063)

- [ ] **2.4** Create new commander entries in `data/cards/core_set/obsidion.yaml`
  - The Blood Sovereign (3055)
  - Shadow Emperor Kael (3056)
  - The Shadow Weaver (3057)
  - Void Archon (3058)

- [ ] **2.5** Keep old creature commander entries temporarily
  - Comment out but preserve for reference during engine transition
  - Will be removed in Phase 6

### Acceptance Criteria

- [ ] All 12 commanders defined in new YAML format
- [ ] YAML files parse without errors
- [ ] Old creature entries preserved (commented) for transition

### Deliverable

Commander cards ready in YAML format (engine can't load them yet).

---

## Phase 3: Engine - Core Types & Loading

**Goal:** Engine can recognize and load commander cards

**Estimated Scope:** Core type changes, card loading

### Tasks

- [ ] **3.1** Add `Commander` variant to `CardType` enum
  - Location: `crates/cardgame/src/core/cards.rs` (or similar)
  - Update any match statements that handle CardType

- [ ] **3.2** Create `CommanderCard` struct
  ```rust
  pub struct CommanderCard {
      pub id: CardId,
      pub name: String,
      pub faction: Faction,
      pub ability: CommanderAbility,
      pub flavor: Option<String>,
  }

  pub enum CommanderAbility {
      Passive(PassiveAbility),
      Triggered(TriggeredAbility),
  }
  ```

- [ ] **3.3** Update `CardDatabase` to parse and store commanders
  - Add `commanders: HashMap<CardId, CommanderCard>` field
  - Update YAML parsing to handle `card_type: commander`
  - Add `get_commander(id: CardId) -> Option<&CommanderCard>`

- [ ] **3.4** Add commander fields to `GameState`
  ```rust
  pub struct GameState {
      // ... existing fields ...
      pub commander_p1: CardId,
      pub commander_p2: CardId,
  }
  ```

- [ ] **3.5** Update `new_game()` to accept commander IDs
  - Add `commander_p1: CardId, commander_p2: CardId` parameters
  - Initialize commander fields in GameState
  - Validate commanders exist in CardDatabase

- [ ] **3.6** Create unit tests for commander loading
  - Test: Commander cards parse correctly
  - Test: GameState initializes with commanders
  - Test: Invalid commander ID rejected

### Acceptance Criteria

- [ ] `CardType::Commander` exists and is handled everywhere
- [ ] `CommanderCard` struct defined with all fields
- [ ] `CardDatabase` loads all 12 commanders
- [ ] `GameState` tracks both players' commanders
- [ ] `new_game()` accepts and validates commander IDs
- [ ] All existing tests still pass
- [ ] New unit tests pass

### Deliverable

Engine loads commanders, GameState tracks them, but abilities don't work yet.

---

## Phase 4: Engine - Passive Effects

**Goal:** Commander passive abilities work

**Estimated Scope:** Effect system integration

### Commanders with Passive Abilities (8 total)

| Commander | Ability |
|-----------|---------|
| The Sanctum Healer | Creatures have Regenerate |
| Siege Marshal Vex | Creatures have +1 Attack |
| The Grand Architect | Creatures have Fortify |
| Alpha of the Hunt | Creatures have +1 Attack |
| The Eternal Grove | Creatures have Regenerate |
| The Blood Sovereign | Creatures have Lifesteal |
| The Shadow Weaver | Creatures have Stealth |
| Void Archon | Creatures have Quick |

### Tasks

- [ ] **4.1** Create `CommanderPassiveEffect` processing
  - Similar pattern to support passive effects
  - Always active (no durability, no removal)
  - Location: `crates/cardgame/src/engine/effects.rs` (or similar)

- [ ] **4.2** Integrate into creature stat/keyword calculation
  - When calculating creature stats, apply commander passive
  - Order: Base stats → Support passives → Commander passive

- [ ] **4.3** Implement keyword granting passives
  - Regenerate (The Sanctum Healer)
  - Fortify (The Grand Architect)
  - Regenerate (The Eternal Grove)
  - Lifesteal (The Blood Sovereign)
  - Stealth (The Shadow Weaver)
  - Quick (Void Archon)

- [ ] **4.4** Implement stat buff passives
  - +1 Attack (Siege Marshal Vex)
  - +1 Attack (Alpha of the Hunt)

- [ ] **4.5** Unit tests for each passive commander
  - Test: Keyword granted to all creatures
  - Test: Stat buff applied to all creatures
  - Test: Passive applies to newly played creatures
  - Test: Passive doesn't affect enemy creatures

### Acceptance Criteria

- [ ] All 8 passive commanders functional
- [ ] Passives apply to all friendly creatures
- [ ] Passives apply immediately when creatures enter play
- [ ] Passives don't affect enemy creatures
- [ ] All existing tests still pass
- [ ] New unit tests for each passive commander

### Deliverable

8 passive commanders fully functional.

---

## Phase 5: Engine - Triggered Effects

**Goal:** Commander triggered abilities work

**Estimated Scope:** Trigger system extension

### Commanders with Triggered Abilities (4 total)

| Commander | Trigger | Effect |
|-----------|---------|--------|
| The High Artificer | StartOfTurn | Summon 1/1 Brass Cog |
| The Broodmother | OnCreaturePlayed (if Rush) | Summon 1/1 Rush Broodling |
| Plague Sovereign | OnAllyDeath | Deal 1 damage to enemy commander |
| Shadow Emperor Kael | OnEnemyDeath | Draw a card |

### Tasks

- [ ] **5.1** Review existing trigger system
  - Identify where triggers are processed
  - Understand effect queue integration

- [ ] **5.2** Add new trigger types if needed
  - `OnCreaturePlayed` - when owner plays a creature
  - `OnEnemyDeath` - when enemy creature dies
  - (StartOfTurn and OnAllyDeath likely exist already)

- [ ] **5.3** Hook commander triggers into effect queue
  - Commander triggers should fire alongside support/creature triggers
  - Ensure proper ordering (commander triggers after the event)

- [ ] **5.4** Implement The High Artificer
  - Trigger: StartOfTurn
  - Effect: Summon 1/1 Brass Cog token
  - Test: Token appears at start of each turn

- [ ] **5.5** Implement The Broodmother
  - Trigger: OnCreaturePlayed
  - Condition: Played creature has Rush
  - Effect: Summon 1/1 Broodling with Rush
  - Test: Token appears only when Rush creature played

- [ ] **5.6** Implement Plague Sovereign
  - Trigger: OnAllyDeath
  - Effect: Deal 1 damage to enemy commander
  - Test: Enemy life decreases when ally dies

- [ ] **5.7** Implement Shadow Emperor Kael
  - Trigger: OnEnemyDeath
  - Effect: Draw a card
  - Test: Card drawn when enemy creature dies

- [ ] **5.8** Unit tests for each triggered commander
  - Test: Trigger fires on correct event
  - Test: Condition checked (Broodmother)
  - Test: Effect resolves correctly
  - Test: Multiple triggers in one turn work

### Acceptance Criteria

- [ ] All 4 triggered commanders functional
- [ ] New trigger types work correctly
- [ ] Conditional triggers work (Broodmother)
- [ ] Triggers integrate with effect queue properly
- [ ] All existing tests still pass
- [ ] New unit tests for each triggered commander

### Deliverable

All 12 commanders fully functional.

---

## Phase 6: Deck Format Migration

**Goal:** Decks use new format with separate commander

**Estimated Scope:** 12 deck files, deck loading code

### New Deck Format

```toml
# OLD FORMAT
id = "architect_fortify"
name = "The Grand Architect"
cards = [1059, 1030, 1040, ...]  # Commander was in cards list

# NEW FORMAT
id = "architect_fortify"
name = "The Grand Architect"
commander = 1059                  # Commander separate
cards = [1030, 1040, ...]         # 30 cards, no commander
```

### Tasks

- [ ] **6.1** Update deck TOML schema
  - Add required `commander: u16` field
  - Update deck struct definition

- [ ] **6.2** Update deck loading code
  - Parse new `commander` field
  - Pass commander to game initialization
  - Location: deck loading module

- [ ] **6.3** Update deck validation
  - Validate: Exactly 30 cards (not 31)
  - Validate: Commander ID exists and is commander type
  - Validate: Commander faction matches deck faction

- [ ] **6.4** Migrate Argentum deck files
  - `data/decks/argentum/architect_fortify.toml`
  - `data/decks/argentum/artificer_tokens.toml`
  - `data/decks/argentum/colossus_wall.toml`
  - `data/decks/argentum/vex_piercing.toml`

- [ ] **6.5** Migrate Symbiote deck files
  - `data/decks/symbiote/broodmother_swarm.toml`
  - `data/decks/symbiote/plague_volatile.toml`
  - `data/decks/symbiote/alpha_frenzy.toml`
  - `data/decks/symbiote/grove_regenerate.toml`

- [ ] **6.6** Migrate Obsidion deck files
  - `data/decks/obsidion/sovereign_lifesteal.toml`
  - `data/decks/obsidion/kael_assassin.toml`
  - `data/decks/obsidion/shadow_weaver.toml`
  - `data/decks/obsidion/archon_burst.toml`

- [ ] **6.7** Remove old commander creature cards from YAML
  - Delete commented creature versions from Phase 2
  - Clean up any references

- [ ] **6.8** Update arena CLI
  - Handle new deck format in `--list-decks`
  - Display commander info in deck listing
  - Pass commander to game initialization

- [ ] **6.9** Update any other deck consumers
  - Tune binary
  - Validate binary
  - Any other binaries that load decks

### Acceptance Criteria

- [ ] New deck format defined and documented
- [ ] Deck loading handles new format
- [ ] Deck validation enforces new rules
- [ ] All 12 deck files migrated
- [ ] Old creature commanders removed from YAML
- [ ] Arena CLI works with new format
- [ ] All binaries work with new format
- [ ] All tests pass

### Deliverable

All decks use new format, clean codebase with no legacy commander creatures.

---

## Phase 7: AI Integration

**Goal:** Bots understand commanders

**Estimated Scope:** State tensor, bot evaluation, MCP

### Tasks

- [ ] **7.1** Update state tensor layout
  - Add commander ID fields for both players
  - Document new tensor indices
  - Current: ~326 floats → New: ~330 floats (estimate)

  ```
  NEW FIELDS (4 floats):
    [326] player1_commander_id / MAX_COMMANDERS
    [327] player2_commander_id / MAX_COMMANDERS
    [328] reserved
    [329] reserved
  ```

- [ ] **7.2** Update `get_state_tensor()` implementation
  - Include commander IDs in tensor output
  - Ensure backwards compatibility notes in docs

- [ ] **7.3** Update state tensor documentation
  - `docs/design-engine.md` Section 11.1
  - `CLAUDE.md` State Tensor section

- [ ] **7.4** Review GreedyBot evaluation
  - Consider if commander abilities affect evaluation
  - Passive stat buffs: Already reflected in creature stats
  - Triggered abilities: May need heuristic adjustments
  - Decision: Document findings, implement if needed

- [ ] **7.5** Update MCP `show_state` tool
  - Display both commanders in state output
  - Show commander name and ability
  - Show commander life (same as player life)

- [ ] **7.6** Update MCP `list_decks` tool
  - Show commander for each deck
  - Display commander ability summary

- [ ] **7.7** Validate bots work correctly
  - Run arena matches with all commanders
  - Ensure no crashes or errors
  - Check for obvious strategic issues

### Acceptance Criteria

- [ ] State tensor includes commander information
- [ ] State tensor documentation updated
- [ ] MCP shows commanders in game state
- [ ] MCP shows commanders in deck listing
- [ ] Bots function correctly with new system
- [ ] All tests pass

### Deliverable

AI agents see commander state, MCP displays commanders properly.

---

## Phase 8: Tauri UI

**Goal:** Visual representation of Command Zone

**Estimated Scope:** Svelte components, layout changes

### Tasks

- [ ] **8.1** Design Command Zone component
  - Sketch layout and visual hierarchy
  - Decide on information density
  - Plan hover/tooltip behavior

- [ ] **8.2** Create CommandZone Svelte component
  - Display commander card art (or placeholder)
  - Display commander name
  - Display ability text
  - Display life total prominently
  - Location: `crates/essence-wars-ui/src/components/`

- [ ] **8.3** Update game board layout
  - Add Command Zone above/below creature slots
  - Adjust spacing for new element
  - Ensure responsive design

- [ ] **8.4** Implement hover/tooltip for ability
  - Full ability text on hover
  - Faction indicator
  - Commander lore/flavor (optional)

- [ ] **8.5** Style Command Zone
  - Match existing UI aesthetic
  - Faction-specific color accents
  - Clear visual distinction from creature slots

- [ ] **8.6** Animate face attacks
  - Draw attack line to Command Zone
  - Show damage number animation
  - Flash/pulse on damage taken

- [ ] **8.7** Handle commander ability triggers visually
  - Brief glow/animation when trigger fires
  - Optional: Toast/notification for effect

- [ ] **8.8** Test UI across different states
  - Game start (both commanders visible)
  - Mid-game (various life totals)
  - Face attacks (animation)
  - Triggered abilities (visual feedback)

### Acceptance Criteria

- [ ] Command Zone visible for both players
- [ ] Commander name and ability displayed
- [ ] Life total prominently shown
- [ ] Hover shows full ability text
- [ ] Face attacks animate correctly
- [ ] Triggered abilities have visual feedback
- [ ] UI is responsive and polished

### Deliverable

Command Zone fully integrated into Tauri UI.

---

## Phase 9: Testing & Validation

**Goal:** Ensure everything works correctly and is balanced

**Estimated Scope:** Testing, arena tournaments, polish

### Tasks

- [ ] **9.1** Run full test suite
  ```bash
  cargo nextest run --status-level=fail
  ```
  - All ~629+ tests must pass
  - No new warnings

- [ ] **9.2** Run Clippy
  ```bash
  ./scripts/run-clippy.sh
  ```
  - No new warnings or errors

- [ ] **9.3** Automated arena tournaments
  - Run all commander matchups
  ```bash
  # All 12 commanders vs all 12 commanders
  # 144 matchups × 100 games = 14,400 games
  for deck1 in $(cargo run --release --bin arena -- --list-decks | grep id); do
    for deck2 in $(cargo run --release --bin arena -- --list-decks | grep id); do
      cargo run --release --bin arena -- \
        --deck1 $deck1 --deck2 $deck2 \
        --bot1 mcts --bot2 mcts \
        --games 100 --progress
    done
  done
  ```

- [ ] **9.4** Analyze tournament results
  - Calculate win rates per commander
  - Identify outliers (>60% or <40% overall win rate)
  - Document findings

- [ ] **9.5** Balance adjustments (if needed)
  - Tweak commander abilities if severe imbalance found
  - Re-run affected matchups to validate

- [ ] **9.6** Manual playtesting in Tauri app
  - Play several games with each commander
  - Verify UI displays correctly
  - Check for edge cases

- [ ] **9.7** Performance validation
  - Run benchmarks
  ```bash
  cargo bench -p cardgame
  ```
  - Ensure no significant regression
  - Document any changes

- [ ] **9.8** Update version to 0.8.0
  - Root `Cargo.toml`: `version = "0.8.0"`
  - `crates/cardgame/src/version.rs`: Update test
  - Verify with `cargo test`

- [ ] **9.9** Final documentation review
  - All docs accurate and complete
  - CHANGELOG updated
  - README updated if needed

### Acceptance Criteria

- [ ] All tests pass
- [ ] Clippy clean
- [ ] Arena tournaments complete
- [ ] No severe balance issues (or documented/addressed)
- [ ] Manual playtesting complete
- [ ] No performance regression
- [ ] Version bumped to 0.8.0
- [ ] Documentation complete

### Deliverable

Stable, tested, balanced v0.8.0 with full commander system.

---

## Dependency Graph

```
Phase 1 (Docs)
    │
    ▼
Phase 2 (Card Data)
    │
    ▼
Phase 3 (Core Types)
    │
    ▼
Phase 4 (Passives)
    │
    ▼
Phase 5 (Triggers)
    │
    ▼
Phase 6 (Deck Migration)
    │
    ▼
Phase 7 (AI)
    │
    ▼
Phase 8 (UI)
    │
    ▼
Phase 9 (Testing)
    │
    ▼
  v0.8.0 Release
```

---

## Progress Tracking

### Overall Status

| Phase | Status | Started | Completed |
|-------|--------|---------|-----------|
| Phase 1: Documentation | Not Started | - | - |
| Phase 2: Card Data | Not Started | - | - |
| Phase 3: Core Types | Not Started | - | - |
| Phase 4: Passives | Not Started | - | - |
| Phase 5: Triggers | Not Started | - | - |
| Phase 6: Deck Migration | Not Started | - | - |
| Phase 7: AI Integration | Not Started | - | - |
| Phase 8: UI | Not Started | - | - |
| Phase 9: Testing | Not Started | - | - |

### Notes

_Use this section to track blockers, decisions, and learnings during implementation._

---

## Appendix: File Inventory

### Files to Create

| File | Phase | Description |
|------|-------|-------------|
| (none - all modifications to existing files) | | |

### Files to Modify

| File | Phase | Changes |
|------|-------|---------|
| `docs/design-engine.md` | 1 | Add commander rules |
| `docs/CLAUDE.md` | 1 | Update context |
| `data/cards/core_set/argentum.yaml` | 2, 6 | Add commanders, remove old |
| `data/cards/core_set/symbiote.yaml` | 2, 6 | Add commanders, remove old |
| `data/cards/core_set/obsidion.yaml` | 2, 6 | Add commanders, remove old |
| `crates/cardgame/src/core/cards.rs` | 3 | CardType::Commander |
| `crates/cardgame/src/core/state.rs` | 3 | Commander fields |
| `crates/cardgame/src/cards/database.rs` | 3 | Load commanders |
| `crates/cardgame/src/engine/effects.rs` | 4, 5 | Commander effects |
| `crates/cardgame/src/engine/triggers.rs` | 5 | New triggers |
| `data/decks/argentum/*.toml` | 6 | New format |
| `data/decks/symbiote/*.toml` | 6 | New format |
| `data/decks/obsidion/*.toml` | 6 | New format |
| `crates/cardgame/src/bin/arena.rs` | 6 | Handle new decks |
| `crates/cardgame/src/ai/tensor.rs` | 7 | Commander in tensor |
| `crates/essence-wars-mcp/src/tools.rs` | 7 | Show commanders |
| `crates/essence-wars-ui/src/components/` | 8 | CommandZone component |
| `crates/essence-wars-ui/src/routes/` | 8 | Layout update |
| `Cargo.toml` | 9 | Version 0.8.0 |

---

*End of Roadmap*
