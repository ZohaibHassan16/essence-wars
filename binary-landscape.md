#  Current Binary Landscape

  │       Binary        │ Lines │          Purpose           │ Complexity │
  │ generate_dataset    │ 711   │ ML training data           │ Very High  │
  │ tune                │ 704   │ CMA-ES weight optimization │ High       │
  │ arena               │ 452   │ Bot matches                │ Medium     │
  │ diagnose            │ 241   │ P1/P2 asymmetry analysis   │ Medium     │
  │ validate            │ 223   │ Balance testing            │ Medium     │
  │ insight_conditions  │ 210   │ Catch-up mechanic analysis │ Low        │
  │ profile_mcts        │ 192   │ Performance benchmarks     │ Low        │
  │ starvation_analysis │ 182   │ Hand starvation patterns   │ Low        │
  │ find_tutorial_seed  │ 142   │ Tutorial hand finder       │ Low        │
  │ validate_decks      │ 108   │ Deck integrity checks      │ Low        │

---
## Opportunities Identified

###  1. Cross-Binary Code Consolidation

Significant duplication in data loading, bot configuration, weight resolution, and output formatting. Could extract into a cli_common module.

### 2. Integration Gaps

  - validate and diagnose are siloed - validate could auto-invoke diagnose for outlier decks
  - tune doesn't connect to validate for post-training verification
  - No way to trace back from balance issues to specific cards

### 3. Missing Capabilities


  │       Idea        │                       Description                        │
  │ Replay binary     │ Replay games from seeds or dataset entries for debugging │
  │ Card statistics   │ Per-card win contribution, usage rates, synergy analysis │
  │ Balance diff      │ Compare validation runs across versions                  │
  │ Weight comparison │ Side-by-side weight file analysis                        │
  │ Dataset analysis  │ Statistics on existing datasets (distribution, coverage) │

### 4. Validate Enhancements

  - Per-card win rate contribution (identify problematic cards, not just decks)
  - Historical tracking (trend across versions)
  - HTML report generation with charts (in the experiments subfolder for the run)

### 5. Arena Enhancements

  - Tournament modes (Swiss, double elimination)
  - ELO rating persistence across sessions
  - Matchup matrix output
  - Game replay export (seeds + decisions)

### 6. Diagnose Enhancements

  - Comparative mode (deck A vs deck B side-by-side)
  - Action breakdown (% attacks vs plays vs abilities)
  - Critical turn identification
  - Essence (Mana) curve efficiency analysis

### 7. Tune Improvements

  - Resume from checkpoint
  - Post-tuning auto-validation
  - Multi-objective optimization (win rate + game length)
  - Hyperparameter auto-tuning
