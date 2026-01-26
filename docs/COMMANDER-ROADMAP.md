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
| **Commander Data** | `data/commanders/*.yaml` | NEW: Commander definitions |
| **Card Data** | `data/cards/core_set/*.yaml` | Replace commander creatures with regular creatures |
| **Deck Format** | `data/decks/**/*.toml` | Separate commander field |
| **Portrait Assets** | `crates/essence-wars-ui/static/portrait/` | Rename from IDs to names |
| **Core Engine** | `crates/cardgame/src/core/` | CardType, GameState, effects |
| **Effect System** | `crates/cardgame/src/engine/` | Commander passives & triggers |
| **Card Loading** | `crates/cardgame/src/cards/` | Load commander type from new location |
| **AI Interface** | `crates/cardgame/src/bots/` | State tensor, evaluation |
| **MCP Server** | `crates/essence-wars-mcp/` | Display commander in state |
| **Tauri UI** | `crates/essence-wars-ui/` | Command Zone component |
| **Documentation** | `docs/` | design-engine.md, CLAUDE.md |

---

## ✅ Phase 1: Documentation & Schema Definition [COMPLETE]

**Goal:** Establish the contract before writing code

**Estimated Scope:** Documentation only

### Tasks

- [x] **1.1** Update `docs/design-engine.md` with new commander rules
  - Add Section 8.4: Commander Cards
  - Update Section 3.1: Game Setup (commander selection)
  - Update Section 10: Win Conditions (commander retreat terminology)
  - Add Command Zone to board layout diagram

- [x] **1.2** Define commander YAML schema formally
  - Document in design-commanders.md (already done)
  - Create example entries for reference

- [x] **1.3** Update `CLAUDE.md` with new commander information
  - Update Card System section
  - Update Deck System section
  - Add Commander System section
  - Update State Tensor section (preview of changes)

- [x] **1.4** Document state tensor changes for AI
  - Define new tensor layout with commander fields
  - Document in design-engine.md Section 11.1

### Acceptance Criteria

- [x] All documentation reflects the new commander design
- [x] Schema is formally defined and documented
- [x] CLAUDE.md is up to date for AI assistant context

### Deliverable

All documentation serves as specification for implementation phases.

---

## ✅ Phase 2: Card Data & Asset Migration [COMPLETE]

**Goal:** Create commander YAML files in new location, replace old commander creatures with regular creatures, rename portrait assets

**Estimated Scope:** 3 new YAML files, 12 commander definitions, 12 replacement creatures, 12 portrait renames

### New File Structure

```
data/
├── cards/
│   └── core_set/
│       ├── argentum.yaml    # IDs 1056-1059 become regular creatures
│       ├── symbiote.yaml    # IDs 2060-2063 become regular creatures
│       ├── obsidion.yaml    # IDs 3055-3058 become regular creatures
│       └── neutral.yaml
└── commanders/              # NEW FOLDER
    ├── argentum.yaml        # 4 Argentum commanders
    ├── symbiote.yaml        # 4 Symbiote commanders
    └── obsidion.yaml        # 4 Obsidion commanders

crates/essence-wars-ui/static/portrait/
├── the_high_artificer.webp      # Renamed from 1056.webp
├── the_sanctum_healer.webp      # Renamed from 1057.webp
├── siege_marshal_vex.webp       # Renamed from 1058.webp
├── the_grand_architect.webp     # Renamed from 1059.webp
├── the_broodmother.webp         # Renamed from 2060.webp
├── plague_sovereign.webp        # Renamed from 2061.webp
├── alpha_of_the_hunt.webp       # Renamed from 2062.webp
├── the_eternal_grove.webp       # Renamed from 2063.webp
├── the_blood_sovereign.webp     # Renamed from 3055.webp
├── shadow_emperor_kael.webp     # Renamed from 3056.webp
├── the_shadow_weaver.webp       # Renamed from 3057.webp
└── void_archon.webp             # Renamed from 3058.webp
```

### Tasks

#### 2A: Create Commander YAML Files

- [ ] **2.1** Create `data/commanders/` folder

- [ ] **2.2** Create `data/commanders/argentum.yaml`
  ```yaml
  commanders:
    - id: 1056
      name: "The High Artificer"
      card_type: commander
      faction: Argentum
      rarity: Legendary
      triggered_ability:
        trigger: StartOfTurn
        description: "At the start of your turn, summon a 1/1 Brass Cog"
        effects:
          - type: summon_token
            token: { name: "Brass Cog", attack: 1, health: 1, keywords: [] }
      flavor: "The Combine doesn't build soldiers. We manufacture victory."

    - id: 1057
      name: "The Sanctum Healer"
      # ... (see design-commanders.md for full definitions)

    - id: 1058
      name: "Siege Marshal Vex"
      # ...

    - id: 1059
      name: "The Grand Architect"
      # ...
  ```

- [ ] **2.3** Create `data/commanders/symbiote.yaml`
  - The Broodmother (2060)
  - Plague Sovereign (2061)
  - Alpha of the Hunt (2062)
  - The Eternal Grove (2063)

- [ ] **2.4** Create `data/commanders/obsidion.yaml`
  - The Blood Sovereign (3055)
  - Shadow Emperor Kael (3056)
  - The Shadow Weaver (3057)
  - Void Archon (3058)

#### 2B: Replace Commander Creatures with Regular Creatures

The old commander creature IDs will be reused for new regular creatures (Common/Uncommon), maintaining gender balance.

- [ ] **2.5** Replace Argentum commander creatures (1056-1059)

  | Old ID | Old Name | Gender | New Creature Name | Rarity | Stats | Notes |
  |--------|----------|--------|-------------------|--------|-------|-------|
  | 1056 | The High Artificer | Male | TBD | Uncommon | ~4-5 cost | Male, Construct tag |
  | 1057 | The Sanctum Healer | Female | TBD | Uncommon | ~4-5 cost | Female, Medic tag |
  | 1058 | Siege Marshal Vex | Male | TBD | Common | ~3-4 cost | Male, Soldier tag |
  | 1059 | The Grand Architect | Female | TBD | Common | ~3-4 cost | Female, Engineer tag |

- [ ] **2.6** Replace Symbiote commander creatures (2060-2063)

  | Old ID | Old Name | Gender | New Creature Name | Rarity | Stats | Notes |
  |--------|----------|--------|-------------------|--------|-------|-------|
  | 2060 | The Broodmother | Female | TBD | Uncommon | ~4-5 cost | Female, Beast tag |
  | 2061 | Plague Sovereign | Ambiguous | TBD | Uncommon | ~4-5 cost | Ambiguous, Parasite tag |
  | 2062 | Alpha of the Hunt | Male | TBD | Common | ~3-4 cost | Male, Beast tag |
  | 2063 | The Eternal Grove | Non-gendered | TBD | Common | ~3-4 cost | Structure tag |

- [ ] **2.7** Replace Obsidion commander creatures (3055-3058)

  | Old ID | Old Name | Gender | New Creature Name | Rarity | Stats | Notes |
  |--------|----------|--------|-------------------|--------|-------|-------|
  | 3055 | The Blood Sovereign | Ambiguous | TBD | Uncommon | ~4-5 cost | Ambiguous, Noble tag |
  | 3056 | Shadow Emperor Kael | Male | TBD | Uncommon | ~4-5 cost | Male, Assassin tag |
  | 3057 | The Shadow Weaver | Female | TBD | Common | ~3-4 cost | Female, Mage tag |
  | 3058 | Void Archon | Ambiguous | TBD | Common | ~3-4 cost | Ambiguous, Mage tag |

#### 2C: Rename Portrait Assets

- [ ] **2.8** Rename portrait files to use commander names (snake_case)
  ```bash
  cd crates/essence-wars-ui/static/portrait/
  mv 1056.webp the_high_artificer.webp
  mv 1057.webp the_sanctum_healer.webp
  mv 1058.webp siege_marshal_vex.webp
  mv 1059.webp the_grand_architect.webp
  mv 2060.webp the_broodmother.webp
  mv 2061.webp plague_sovereign.webp
  mv 2062.webp alpha_of_the_hunt.webp
  mv 2063.webp the_eternal_grove.webp
  mv 3055.webp the_blood_sovereign.webp
  mv 3056.webp shadow_emperor_kael.webp
  mv 3057.webp the_shadow_weaver.webp
  mv 3058.webp void_archon.webp
  ```

#### 2D: Update Deck Files

- [ ] **2.9** Rename `data/decks/argentum/colossus_wall.toml` to `sanctum_healer.toml`
  - Update deck name and description for The Sanctum Healer

### Acceptance Criteria

- [ ] `data/commanders/` folder exists with 3 YAML files
- [ ] All 12 commanders defined in new format
- [ ] All 12 old commander IDs replaced with new regular creatures
- [ ] Gender balance maintained in replacement creatures
- [ ] All 12 portrait files renamed to use commander names
- [ ] Deck file renamed from colossus_wall to sanctum_healer
- [ ] YAML files parse without errors

### Deliverable

- Commander data in new location (`data/commanders/`)
- Old IDs repurposed as regular creatures
- Portrait assets renamed and ready for UI
- Engine can't load new format yet (that's Phase 3)

---

## ✅ Phase 3: Engine - Core Types & Loading [COMPLETE]

**Goal:** Engine can recognize and load commander cards

**Estimated Scope:** Core type changes, card loading

### Tasks

- [x] **3.1** Add `Commander` types to `cards.rs`
  - Created separate commander types instead of CardType variant
  - Added: Faction, CommanderPassiveEffect, CommanderPassiveAbility
  - Added: CommanderTriggerCondition, CommanderTrigger, CommanderTriggeredAbility
  - Added: CommanderAbility (enum of Passive/Triggered), CommanderDefinition, CommanderSet

- [x] **3.2** Create `CommanderDefinition` struct
  ```rust
  pub struct CommanderDefinition {
      pub id: u16,
      pub name: String,
      pub faction: Faction,
      pub rarity: Rarity,
      pub ability: CommanderAbility,  // #[serde(flatten)]
      pub flavor: Option<String>,
  }
  ```

- [x] **3.3** Update `CardDatabase` to parse and store commanders
  - Added `commanders: Arc<Vec<CommanderDefinition>>` field
  - Added `commander_to_index: Arc<Vec<Option<usize>>>` for O(1) lookup
  - Added `load_commanders_from_directory()` method
  - Added `load_with_commanders()` convenience method
  - Added `get_commander(id: CardId) -> Option<&CommanderDefinition>`
  - Added `iter_commanders()` and `commander_ids()` iterators

- [x] **3.4** Add commander fields to `GameState`
  ```rust
  pub struct GameState {
      // ... existing fields ...
      pub commander_p1: Option<CardId>,  // serde(default)
      pub commander_p2: Option<CardId>,
  }
  ```
  - Added `get_commander(player)` and `set_commander(player, id)` methods

- [x] **3.5** Update `start_game()` to accept commander IDs
  - Added `start_game_with_commanders()` method
  - Added `start_game_full()` method (with mode + commanders)
  - Added `get_commander_def(player)` method
  - Validates commanders exist before starting game

- [x] **3.6** Create unit tests for commander loading
  - 9 new tests in `tests/unit/cards_tests.rs`:
    - test_commander_yaml_parsing
    - test_commander_triggered_ability_yaml
    - test_load_commanders_from_directory
    - test_card_database_with_commanders
    - test_load_with_commanders_convenience
    - test_commander_iterator
    - test_commander_ability_helpers
    - test_duplicate_commander_id_detection

### Acceptance Criteria

- [x] Commander types defined (not CardType variant, but separate structs)
- [x] `CommanderDefinition` struct defined with all fields
- [x] `CardDatabase` loads all 12 commanders from `data/commanders/`
- [x] `GameState` tracks both players' commanders (as Option<CardId>)
- [x] `start_game_with_commanders()` accepts and validates commander IDs
- [x] All 648 existing tests pass
- [x] New unit tests pass

### Notes

- Commander IDs changed to new range 5000-5011 to avoid conflict with card IDs
- Used Option<CardId> for backwards compatibility with existing tests
- Fixed pre-existing bug in lifesteal_trace.rs (token handling)
- Updated golden test values in regression_tests.rs

### Deliverable

Engine loads commanders, GameState tracks them, but abilities don't work yet.

---

## ✅ Phase 4: Engine - Passive Effects [COMPLETE]

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

- [x] **4.1** Create `CommanderPassiveEffect` processing
  - Added to `crates/cardgame/src/core/engine/passive.rs`
  - `apply_commander_passive_to_creature()` handles both keyword grants and stat buffs
  - `apply_commander_passive_to_new_creature()` called when creatures enter play

- [x] **4.2** Integrate into creature stat/keyword calculation
  - Modified `crates/cardgame/src/core/engine/game_engine.rs`
  - Applied after support passives when creature is played (line 637-642)
  - Order: Base stats → Support passives → Commander passive

- [x] **4.3** Implement keyword granting passives
  - Regenerate (The Sanctum Healer)
  - Fortify (The Grand Architect)
  - Regenerate (The Eternal Grove)
  - Lifesteal (The Blood Sovereign)
  - Stealth (The Shadow Weaver)
  - Quick (Void Archon)

- [x] **4.4** Implement stat buff passives
  - +1 Attack (Siege Marshal Vex)
  - +1 Attack (Alpha of the Hunt)

- [x] **4.5** Unit tests for each passive commander
  - Created `crates/cardgame/tests/unit/commander_tests.rs` with 13 tests:
    - test_sanctum_healer_grants_regenerate
    - test_grand_architect_grants_fortify
    - test_eternal_grove_grants_regenerate
    - test_blood_sovereign_grants_lifesteal
    - test_shadow_weaver_grants_stealth
    - test_void_archon_grants_quick
    - test_siege_marshal_vex_grants_attack_bonus
    - test_alpha_of_the_hunt_grants_attack_bonus
    - test_commander_passive_does_not_affect_enemy
    - test_commander_passive_applies_to_multiple_creatures
    - test_triggered_commander_has_no_passive_effect
    - test_commander_stat_buff_stacks_with_creature_base
    - test_commander_passive_preserved_after_combat

### Acceptance Criteria

- [x] All 8 passive commanders functional
- [x] Passives apply to all friendly creatures
- [x] Passives apply immediately when creatures enter play
- [x] Passives don't affect enemy creatures
- [x] All existing tests still pass (661 tests pass)
- [x] New unit tests for each passive commander

### Notes

- Commander passives use the same pattern as support passives but are simpler:
  - No removal needed (commanders are permanent)
  - Applied after support passives in creature creation flow
- `CommanderPassiveEffect` types match `CommanderAbility::Passive` struct
- All 13 commander tests verify keyword grants, stat buffs, and edge cases

### Deliverable

8 passive commanders fully functional.

---

## ✅ Phase 5: Engine - Triggered Effects [COMPLETE]

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

- [x] **5.1** Review existing trigger system
  - Identified trigger processing in game_engine.rs (start_turn), effect_queue.rs (deaths), combat.rs (deaths)
  - Understood effect queue integration

- [x] **5.2** Add new trigger types if needed
  - Added `OnCreaturePlayed` - when owner plays a creature
  - Added `OnEnemyDeath` - when enemy creature dies
  - Added `EffectSource::Commander { owner }` variant

- [x] **5.3** Hook commander triggers into effect queue
  - Added `process_commander_start_of_turn_triggers()` in game_engine.rs
  - Added OnCreaturePlayed check in `execute_play_card()` Creature arm
  - Added OnAllyDeath and OnEnemyDeath in `process_creature_death()` (combat.rs)
  - Added OnAllyDeath and OnEnemyDeath in `process_deaths()` (effect_queue.rs)

- [x] **5.4** Implement The High Artificer
  - Trigger: StartOfTurn
  - Effect: Summon 1/1 Brass Cog token
  - Test: Token appears at start of each turn ✓

- [x] **5.5** Implement The Broodmother
  - Trigger: OnCreaturePlayed
  - Condition: Played creature has Rush
  - Effect: Summon 1/1 Broodling with Rush
  - Test: Token appears only when Rush creature played ✓

- [x] **5.6** Implement Plague Sovereign
  - Trigger: OnAllyDeath
  - Effect: Deal 1 damage to enemy commander
  - Test: Enemy life decreases when ally dies ✓

- [x] **5.7** Implement Shadow Emperor Kael
  - Trigger: OnEnemyDeath
  - Effect: Draw a card
  - Test: Card drawn when enemy creature dies ✓

- [x] **5.8** Unit tests for each triggered commander
  - 8 new tests in `tests/unit/commander_tests.rs`:
    - test_high_artificer_summons_brass_cog_on_turn_start
    - test_high_artificer_summons_multiple_tokens_over_turns
    - test_broodmother_summons_broodling_when_rush_creature_played
    - test_broodmother_does_not_summon_when_non_rush_creature_played
    - test_plague_sovereign_deals_damage_on_ally_death
    - test_shadow_emperor_kael_draws_on_enemy_death
    - test_shadow_emperor_kael_multiple_deaths_multiple_draws

### Acceptance Criteria

- [x] All 4 triggered commanders functional
- [x] New trigger types work correctly
- [x] Conditional triggers work (Broodmother)
- [x] Triggers integrate with effect queue properly
- [x] All existing tests still pass (660 tests)
- [x] New unit tests for each triggered commander

### Notes

- Commander triggers are processed in two places:
  - `game_engine.rs`: StartOfTurn and OnCreaturePlayed
  - `combat.rs` and `effect_queue.rs`: OnAllyDeath and OnEnemyDeath
- Added helper functions in `passive.rs`:
  - `collect_commander_start_of_turn_effects()`
  - `collect_commander_creature_played_effects()`
  - `collect_commander_ally_death_effects()`
  - `collect_commander_enemy_death_effects()`
- Total commander tests: 20 (13 passive + 7 triggered)

### Deliverable

All 12 commanders fully functional.

---

## ✅ Phase 6: Deck Format Migration [COMPLETE]

**Goal:** Decks use new format with separate commander

**Estimated Scope:** 12 deck files, deck loading code

### New Deck Format

```toml
# OLD FORMAT
id = "architect_fortify"
name = "The Grand Architect"
cards = [1059, 1030, 1040, ...]  # Commander was in cards list (30 total)

# NEW FORMAT
id = "architect_fortify"
name = "The Grand Architect"
commander = 5003                  # Commander ID from commanders database
cards = [1030, 1040, ...]         # 29 cards, no commander (30 total with commander)
```

### Tasks

- [x] **6.1** Update deck TOML schema
  - Added required `commander: u16` field to DeckDefinition
  - Added `commander_id()` helper method

- [x] **6.2** Update deck loading code
  - Parsing of `commander` field automatic via serde
  - DeckDefinition now includes commander field

- [x] **6.3** Update deck validation
  - Validate: Exactly 29 cards (+ 1 commander = 30 total)
  - Validate: Commander ID exists in commander database
  - Added InvalidDeckSize and InvalidCommander error variants

- [x] **6.4** Migrate Argentum deck files
  - `architect_fortify.toml` - commander = 5003 (The Grand Architect)
  - `artificer_tokens.toml` - commander = 5000 (The High Artificer)
  - `sanctum_healer.toml` - commander = 5001 (The Sanctum Healer)
  - `vex_piercing.toml` - commander = 5002 (Siege Marshal Vex)

- [x] **6.5** Migrate Symbiote deck files
  - `broodmother_swarm.toml` - commander = 5004 (The Broodmother)
  - `plague_volatile.toml` - commander = 5005 (Plague Sovereign)
  - `alpha_frenzy.toml` - commander = 5006 (Alpha of the Hunt)
  - `grove_regenerate.toml` - commander = 5007 (The Eternal Grove)

- [x] **6.6** Migrate Obsidion deck files
  - `sovereign_lifesteal.toml` - commander = 5008 (The Blood Sovereign)
  - `kael_assassin.toml` - commander = 5009 (Shadow Emperor Kael)
  - `shadow_weaver.toml` - commander = 5010 (The Shadow Weaver)
  - `archon_burst.toml` - commander = 5011 (Void Archon)

- [x] **6.7** Update arena CLI
  - New deck format works with `--list-decks` automatically
  - Shows card count (29 cards per deck)

- [x] **6.8** Update test files
  - Updated `tests/unit/decks_tests.rs` with new commander IDs
  - Updated `src/validation/matchup.rs` to load commanders for validation
  - Updated `src/validation/executor.rs` to load commanders for validation
  - Updated `tests/regression_tests.rs` golden values for new deck format

### Acceptance Criteria

- [x] New deck format defined and documented
- [x] Deck loading handles new format
- [x] Deck validation enforces new rules (29 cards + commander)
- [x] All 12 deck files migrated
- [x] Arena CLI works with new format
- [x] All binaries work with new format
- [x] All 668 tests pass

### Deliverable

All decks use new format with separate commander field.

---

## ✅ Phase 7: AI Integration [COMPLETE]

**Goal:** Bots understand commanders

**Estimated Scope:** State tensor, bot evaluation, MCP

### Tasks

- [x] **7.1** Update state tensor layout
  - Added commander ID fields at indices 326-327
  - Updated STATE_TENSOR_SIZE from 326 to 328
  - Updated CARD_ID_NORMALIZER from 5000.0 to 6000.0 (handles commander IDs up to 5011)

  ```
  NEW FIELDS (2 floats):
    [326] player1_commander_id / 6000.0 (normalized)
    [327] player2_commander_id / 6000.0 (normalized)
  ```

- [x] **7.2** Update `get_state_tensor()` implementation
  - Added `encode_commander_ids()` function in tensor.rs
  - Commander IDs encoded at fixed positions (326-327)
  - Card embeddings now stop at index 325 to leave room

- [x] **7.3** Update state tensor documentation
  - Updated `CLAUDE.md` State Tensor section with full layout
  - Updated Bot Trait documentation with new tensor size
  - Added commander offset helper function

- [x] **7.4** Review GreedyBot evaluation
  - Confirmed: Commander passive effects already reflected in creature stats
  - No changes needed - passives apply when creatures enter play
  - Triggered abilities are difficult to evaluate heuristically (deferred)

- [x] **7.5** Update MCP `show_state` tool
  - Board rendering now shows commander name for both players
  - Format: `[Commander Name]` displayed in player info line

- [x] **7.6** Update MCP `list_decks` tool
  - Each deck now shows its commander
  - Format: `**Deck Name** (deck_id) - Commander: **Commander Name**`

- [x] **7.7** Validate bots work correctly
  - Arena matches run successfully with all commanders
  - No crashes or errors observed
  - All 668 tests pass

### Acceptance Criteria

- [x] State tensor includes commander information (indices 326-327)
- [x] State tensor documentation updated (CLAUDE.md)
- [x] MCP shows commanders in game state (board.rs)
- [x] MCP shows commanders in deck listing (discovery.rs)
- [x] Bots function correctly with new system
- [x] All 668 tests pass

### Deliverable

AI agents see commander state, MCP displays commanders properly.

---

## ✅ Phase 8: Tauri UI [COMPLETE]

**Goal:** Visual representation of Command Zone

**Estimated Scope:** Svelte components, layout changes

### Tasks

- [x] **8.1** Design Command Zone component
  - Compact layout showing portrait, name, and ability text
  - Faction-specific styling with gradient backgrounds and borders
  - Active turn glow effect

- [x] **8.2** Create CommandZone Svelte component
  - Created `crates/essence-wars-ui/src/lib/components/board/CommandZone.svelte`
  - Displays commander portrait (48x48px, cropped from portrait WebP)
  - Displays commander name (semibold)
  - Displays ability description (truncated with full text on hover)
  - Uses faction-specific colors for gradient and border

- [x] **8.3** Update game board layout
  - Updated `GameBoard.svelte` to include CommandZone for both players
  - Added CommandZone in player info bar sections
  - Integrated into existing flex layout

- [x] **8.4** Implement hover/tooltip for ability
  - Full ability text shown via `title` attribute on component
  - Truncated text visible, full text on hover

- [x] **8.5** Style Command Zone
  - Faction-specific gradient backgrounds:
    - Argentum: brass/gold gradient
    - Symbiote: green/purple gradient
    - Obsidion: red/black gradient
    - Neutral: brown/tan gradient
  - Faction-specific border colors
  - Active turn glow effect with faction-appropriate shadow

- [ ] **8.6** Animate face attacks (deferred to Phase 9)
  - Not implemented in this phase
  - Face attacks still work, just no visual line to Command Zone

- [ ] **8.7** Handle commander ability triggers visually (deferred to Phase 9)
  - Not implemented in this phase
  - Triggers work mechanically, no visual feedback yet

- [x] **8.8** Test UI across different states
  - TypeScript/Svelte types checked: 0 errors
  - Rust backend builds successfully
  - All 663 tests pass
  - Added CommandZone to SpectatorPlayback for consistency

### Acceptance Criteria

- [x] Command Zone visible for both players
- [x] Commander name and ability displayed
- [ ] Life total prominently shown (shows via existing PlayerInfoWidget)
- [x] Hover shows full ability text
- [ ] Face attacks animate correctly (deferred)
- [ ] Triggered abilities have visual feedback (deferred)
- [x] UI is responsive and polished

### Notes

- Commander portraits use snake_case naming: `portrait/{name}.webp`
- Frontend types updated in `types.ts` with `CommanderDto` interface
- Backend Rust types added in `serialization.rs` with full DTO and conversion
- Added `start_game_with_commanders()` to `GameClient` API
- Fixed faction type mismatch between `core::cards::Faction` and `decks::Faction`

### Deliverable

Command Zone integrated into Tauri UI with faction-specific styling. Face attack animations and trigger visual feedback deferred to Phase 9.

---

## Phase 9: Testing & Validation

**Goal:** Ensure everything works correctly and is balanced

**Estimated Scope:** Testing, arena tournaments, polish

### Tasks

- [x] **9.1** Run full test suite
  ```bash
  cargo nextest run --status-level=fail
  ```
  - All ~629+ tests must pass
  - No new warnings

- [x] **9.2** Run Clippy
  ```bash
  ./scripts/run-clippy.sh
  ```
  - No new warnings or errors

- [ ] **9.3** Automated arena tournaments
  - Run all commander matchups using validate (Alpha-Beta Bot, MCTSBot)

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

- [x] **9.7** Performance validation
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
| Phase 1: Documentation | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 2: Card Data | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 3: Core Types | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 4: Passives | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 5: Triggers | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 6: Deck Migration | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 7: AI Integration | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 8: UI | **Complete** | 2026-01-25 | 2026-01-25 |
| Phase 9: Testing | Not Started | - | - |

## Appendix: File Inventory

### Files to Create

| File | Phase | Description |
|------|-------|-------------|
| `data/commanders/argentum.yaml` | 2 | Argentum commander definitions |
| `data/commanders/symbiote.yaml` | 2 | Symbiote commander definitions |
| `data/commanders/obsidion.yaml` | 2 | Obsidion commander definitions |
| `data/decks/argentum/sanctum_healer.toml` | 2 | Renamed from colossus_wall.toml |

### Files to Rename (Phase 2)

| Old Name | New Name |
|----------|----------|
| `static/portrait/1056.webp` | `static/portrait/the_high_artificer.webp` |
| `static/portrait/1057.webp` | `static/portrait/the_sanctum_healer.webp` |
| `static/portrait/1058.webp` | `static/portrait/siege_marshal_vex.webp` |
| `static/portrait/1059.webp` | `static/portrait/the_grand_architect.webp` |
| `static/portrait/2060.webp` | `static/portrait/the_broodmother.webp` |
| `static/portrait/2061.webp` | `static/portrait/plague_sovereign.webp` |
| `static/portrait/2062.webp` | `static/portrait/alpha_of_the_hunt.webp` |
| `static/portrait/2063.webp` | `static/portrait/the_eternal_grove.webp` |
| `static/portrait/3055.webp` | `static/portrait/the_blood_sovereign.webp` |
| `static/portrait/3056.webp` | `static/portrait/shadow_emperor_kael.webp` |
| `static/portrait/3057.webp` | `static/portrait/the_shadow_weaver.webp` |
| `static/portrait/3058.webp` | `static/portrait/void_archon.webp` |
| `data/decks/argentum/colossus_wall.toml` | (deleted, replaced by sanctum_healer.toml) |

### Files to Modify

| File | Phase | Changes |
|------|-------|---------|
| `docs/design-engine.md` | 1 | Add commander rules |
| `docs/CLAUDE.md` | 1 | Update context |
| `data/cards/core_set/argentum.yaml` | 2 | Replace commander creatures (1056-1059) with regular creatures |
| `data/cards/core_set/symbiote.yaml` | 2 | Replace commander creatures (2060-2063) with regular creatures |
| `data/cards/core_set/obsidion.yaml` | 2 | Replace commander creatures (3055-3058) with regular creatures |
| `crates/cardgame/src/core/cards.rs` | 3 | CardType::Commander |
| `crates/cardgame/src/core/state.rs` | 3 | Commander fields |
| `crates/cardgame/src/cards/database.rs` | 3 | Load commanders from data/commanders/ |
| `crates/cardgame/src/engine/effects.rs` | 4, 5 | Commander effects |
| `crates/cardgame/src/engine/triggers.rs` | 5 | New triggers |
| `data/decks/argentum/*.toml` | 6 | Add commander field, adjust card lists |
| `data/decks/symbiote/*.toml` | 6 | Add commander field, adjust card lists |
| `data/decks/obsidion/*.toml` | 6 | Add commander field, adjust card lists |
| `crates/cardgame/src/bin/arena.rs` | 6 | Handle new decks |
| `crates/cardgame/src/ai/tensor.rs` | 7 | Commander in tensor |
| `crates/essence-wars-mcp/src/tools.rs` | 7 | Show commanders |
| `crates/essence-wars-ui/src/components/` | 8 | CommandZone component |
| `crates/essence-wars-ui/src/routes/` | 8 | Layout update |
| `Cargo.toml` | 9 | Version 0.8.0 |

---

*End of Roadmap*
