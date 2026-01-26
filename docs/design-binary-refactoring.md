# Binary Infrastructure Refactoring Design

**Status:** Planned
**Created:** 2026-01-26
**Prerequisite:** Unified `start_game()` refactor (in progress)

## Overview

This document captures the architectural audit findings and proposed refactoring plan for the arena, validate, and diagnose binaries. The goal is to create a cohesive, unified execution framework that properly supports commander-level and deck-level balance testing.

## Problem Statement

### Current Issues

1. **Commander Integration Gap**: All binaries load commanders but don't use them in games
   - `start_game()` called instead of `start_game_with_commanders()`
   - Commander abilities are never activated in validation/arena runs

2. **Code Duplication**: Significant duplication across binaries
   - Card/deck/commander loading (~45 lines each, 3x)
   - Bot type parsing & weight resolution (~30 lines each, 3x)
   - Progress setup (inconsistent implementations)
   - Seed derivation logic (2 different implementations)

3. **Inconsistent Patterns**:
   | Aspect | Arena | Validate | Diagnose |
   |--------|-------|----------|----------|
   | Progress | `ProgressReporter::Rich` | `ProgressReporter::Simple` | Manual stderr |
   | Configuration | `MatchConfig` | `MatchupDefinition` | `DiagnosticConfig` |
   | Results | `MatchStats` | `MatchupResult` | `GameDiagnostics` |
   | Parallelism | Uses shared `run_batch_parallel` | Manual rayon | Custom runner |

4. **Balance Granularity**: Current system only aggregates at faction level
   - Need commander-level aggregation (12 commanders)
   - Need deck-level drill-down (12 decks)
   - Need to identify "anti-deck" dynamics

## Audit Findings

### Arena Binary (`src/bin/arena.rs`)

**Strengths:**
- Well-layered architecture (CLI → Config → Executor → Engine)
- Excellent parallel execution with rayon
- Good seed-based reproducibility
- Supports debug/trace modes

**Gaps:**
- Uses `start_game_with_mode()` - no commanders
- No commander statistics in output
- No `--list-commanders` option

**Key Files:**
- `src/bin/arena.rs` (419 lines)
- `src/arena/executor.rs` (422 lines)
- `src/arena/config.rs` (141 lines)
- `src/arena/stats.rs` (175 lines)

### Validate Binary (`src/bin/validate.rs`)

**Strengths:**
- Excellent infrastructure for faction balance
- Rich diagnostic collection (board advantage, efficiency, trade ratios)
- Statistical analysis (chi-square, Wilson CI)
- 95-99% CPU utilization (excellent parallelism)
- JSON + human-readable output

**Gaps:**
- `MatchupDefinition` lacks commander fields
- Uses `start_game()` - no commanders
- Only faction-level aggregation
- Reimplements parallel execution instead of using shared infrastructure

**Key Files:**
- `src/bin/validate.rs` (~300 lines)
- `src/validation/executor.rs` (~400 lines)
- `src/validation/types.rs` (~200 lines)
- `src/validation/analyzer.rs` (~300 lines)

### Diagnose Binary (`src/bin/diagnose.rs`)

**Strengths:**
- Per-turn asymmetry analysis
- Multiple export formats (human, JSON, CSV)
- Detailed timing metrics

**Gaps:**
- Uses `start_game()` - no commanders
- Manual progress reporting
- Different diagnostic structure from validate

**Key Files:**
- `src/bin/diagnose.rs` (~200 lines)
- `src/diagnostics/collector.rs` (~500 lines)

### Shared Infrastructure (`src/execution/`)

**Existing:**
- `run_batch_parallel()` - parallel game batch execution
- `GameSeeds` - deterministic seed derivation
- `ProgressReporter` - progress bar display

**Underutilized:**
- Validate reimplements parallel execution manually
- Diagnose doesn't use shared progress reporting

## Proposed Architecture

### Unified Execution Layer

```
┌─────────────────────────────────────────────────────────────┐
│                    Unified Execution Layer                   │
├─────────────────────────────────────────────────────────────┤
│  DataLoader         → Cards, Commanders, Decks, Weights     │
│  MatchupBuilder     → Deck-level, Commander-level, Faction  │
│  GameExecutor       → Parallel/Sequential with diagnostics  │
│  MetricsCollector   → Unified per-game metrics              │
│  ResultAggregator   → By faction, commander, deck, matchup  │
│  BalanceAnalyzer    → Statistical analysis at any level     │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
    ┌────▼────┐          ┌────▼────┐         ┌────▼────┐
    │  arena  │          │validate │         │diagnose │
    └─────────┘          └─────────┘         └─────────┘
```

### New Abstractions

#### 1. UnifiedMatchup
```rust
pub struct UnifiedMatchup {
    pub id: String,
    pub faction1: Faction,
    pub faction2: Faction,
    pub deck1: DeckDefinition,  // Includes commander
    pub deck2: DeckDefinition,  // Includes commander
}
```

#### 2. GameMetrics (unified for all binaries)
```rust
pub struct GameMetrics {
    // Outcome
    pub winner: Option<PlayerId>,
    pub turns: u32,
    pub duration: Duration,

    // Per-turn metrics
    pub board_advantage: Vec<i32>,
    pub essence_spent: [u32; 2],
    pub face_damage: [u32; 2],
    pub creatures_killed: [u32; 2],
    pub creatures_lost: [u32; 2],

    // First blood
    pub first_blood: Option<PlayerId>,
    pub first_blood_turn: Option<u32>,
}
```

#### 3. ExecutionConfig (shared CLI args)
```rust
pub struct ExecutionConfig {
    pub threads: usize,
    pub seed: u64,
    pub progress: ProgressStyle,
    pub bot_config: BotConfig,
}
```

#### 4. AggregationLevel
```rust
pub enum AggregationLevel {
    Faction,      // 3 factions
    Commander,    // 12 commanders
    Deck,         // 12 decks
    Matchup,      // Individual matchups
}
```

### Balance Analysis Output

Commander-level balance report:
```
=== Commander Balance (MCTS-500, 100 games/matchup) ===

Commander Win Rates (vs all opponents):
  The High Artificer (Argentum)     52.3% [49.8-54.8]  BALANCED
  Siege Marshal Vex (Argentum)      48.1% [45.6-50.6]  BALANCED
  The Broodmother (Symbiote)        55.2% [52.7-57.7]  WATCH
  Shadow Emperor Kael (Obsidion)    61.4% [58.9-63.9]  OUTLIER ⚠️

Matchup Matrix (rows = P1, columns = P2):
                    HighArt  Vex    Brood  Kael   ...
  High Artificer      -     48.2   44.1   38.5
  Siege Marshal Vex  51.8     -     46.3   41.2
  The Broodmother    55.9   53.7     -     48.8
  Shadow Emperor     61.5   58.8   51.2     -

Anti-Deck Dynamics:
  Broodmother >> High Artificer (55.9% - aggressive beats slow)
  Shadow Emperor >> All Argentum (60%+ - burst beats defense)
```

## Implementation Phases

### Phase 0: Prerequisites (Current)
- [x] Performance validation (9.7)
- [x] Architecture audit
- [ ] Unify `start_game()` with mandatory commanders

### Phase 1: Foundation
- [ ] Create `DataLoader` module (unified loading)
- [ ] Create `UnifiedMatchup` type
- [ ] Create `GameMetrics` type
- [ ] Refactor `MatchupBuilder` to support all granularities

### Phase 2: Arena Refactoring
- [ ] Update arena to use `UnifiedMatchup`
- [ ] Add commander info to output
- [ ] Add `--list-commanders` option

### Phase 3: Validate Refactoring
- [ ] Use shared `run_batch_parallel` instead of manual rayon
- [ ] Add commander/deck level aggregation
- [ ] Update `MatchupDefinition` → `UnifiedMatchup`
- [ ] Enhance output with commander statistics

### Phase 4: Diagnose Refactoring
- [ ] Align with `GameMetrics` structure
- [ ] Use shared progress reporting
- [ ] Add commander-aware diagnostics

### Phase 5: Integration & Testing
- [ ] Run full commander balance tournament
- [ ] Validate statistical significance
- [ ] Document balance thresholds

## Metrics & Success Criteria

### Balance Thresholds
- **Balanced**: 45-55% win rate
- **Watch**: 40-45% or 55-60%
- **Outlier**: <40% or >60%

### Acceptable Imbalance
- Individual matchups CAN be imbalanced (anti-deck dynamics are fun)
- Overall commander win rate SHOULD be 45-55%
- No faction should dominate all others

### Performance Targets
- Validate with 12×12 deck matchups at 100 games each = 14,400 games
- Should complete in <30 minutes on 8-core machine
- CPU utilization should remain >90%

## Open Questions

1. Should we add a "Test Commander" with no abilities for unit tests?
2. Should we support commander overrides in arena (for testing)?
3. How should we handle mirror matches (same commander)?

## Related Documents

- `docs/COMMANDER-ROADMAP.md` - Commander implementation roadmap
- `docs/design-commanders.md` - Commander system design
- `docs/tuning-pipeline.md` - Bot weight tuning guide
