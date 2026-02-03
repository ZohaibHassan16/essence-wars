# To Do

## (A) Deck Builder

- Custom Deck Builder
- Choose Faction -> Choose Commander
- Choose Cards from Faction + Neutral Cards 
- Save as Deck, add Description
- App should automatically detect which playstyle the deck falls into 
- Be able to play with custom decks
- Cannot edit or overwrite the prebuilt 12 Starter Decks, only custom decks

## (B) Update Lore and Other Documentation

Update `lore.md` with the new direction for Argentum, Symbiote and the Free Walkers in our Art Direction `docs/art-direction.md` and the prompts(Artwork). Free Walkers not using guns, Symbiote not being bio punk engineers but naturalists and druids etc. Make sure the whole Document is in sync, including `docs/essence-wars-design.md` Faction Descriptions.

Review all other existing Documentation and look for out of date statements/Info.

## (C) Update and Enhance Diagnosis and Analysis Capabilities of Spectator Mode

Update and Enhance the Diagnosis/Analysis Screen in Spectator Mode, using the now fully enhanced binary libraries we have at our hand, with plots, graphs, tables, statistics, probabilities etc.

## (D) Review and Update MCP Server

- Bring up to date
- Review `explain_rules()` , so that LLM Agents can play with confidence
- Review and Audit complete Package
- Discuss if AI Hint function needs to be Improved/Enhanced (Test it in a life game first) 

## (F) Review and Update Python Gym

- Look into the changes, update and integrate everything
- Create new, updated Github Action workflows - Fresh workflows with current best practices for Maturin/PyPI for Publishing to Github Packages / PyPI

## (G) Review if Asset Pipeline is still in sync

Search and review all Asset related Scripts, Data, Prompts, Artwork, Documention etc, and bring them up to date and in sync.

### (H) Tutorials

- Update Human vs AI Tutorial
- Create Spectator Mode Tutorial 
- Create Deck Builder Tutorial

### (I) Discuss Expansion

See `expansion.md`.

### (J) Essential Audit Checklist

Work through `essential-audit-checklist.md`.

### (K) Python Linting

Work through all ruff and mypy issues for the python modules.

### (L) Test Suite

Improve Test Coverage for vital Systems (Crates and Python)

## (M) Expansion Planning

| Question | Decision |
|----------|----------|
| Scripting vs Pure Rust | Pure Rust for performance |
| Custom YAML cards | Yes, sandboxed for casual play |
| Bot/Agent compatibility | Accept retuning; keep observation/action space stable |
| Expansion structure | 1 commander + deck per faction per expansion |

## (N) Win Conditions

Review Win Conditions implementation and Discussion in Code Base and `win-discussion.md`. Update Essence Wars Design Doc and other Documentation, including Game App `crates/essence-wars-ui`.

**Decision:** Keep life as "Tactical Stability" (zero = forced retreat), add Essence Extraction as parallel win condition.

**Implementation Details:**
- Renamed `GameMode::EssenceDuel` → `GameMode::EssenceWar` (now default)
- Renamed `WinReason::VictoryPointsReached` → `WinReason::EssenceExtractionReached`
- Renamed `WinReason::TurnLimitHigherLife` → `WinReason::TurnLimitTiebreaker`
- Turn limit tiebreaker uses VP for EssenceWar mode, life for Attrition mode
- Backwards compatibility maintained (CLI accepts "essence-duel", "essenceduel" etc.)
- All 759 tests passing

**Thresholds:** 50 essence extracted to win (unchanged from VictoryPoints).

## (O) Performance Optimization Plan

See `docs/design-performance-optimizations.md` for full details.

**Phase 1 (Sprint 1) - COMPLETED:**
1. ✅ Token data separation - Changed `token_abilities` + `token_name` to `Option<Box<TokenData>>` (2.2-2.4x speedup)
2. ⏭ Slot-indexed creature storage - Skipped (diminishing returns after token fix)
3. ⏭ Remove redundant fields - Skipped (complexity not justified)

**Phase 2 (Future - if needed):**
- Batch keyword evaluation (lookup table)
- Greedy delta scoring (skip forks for Attack/EndTurn)
- Legal action caching