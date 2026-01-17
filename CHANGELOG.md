# Changelog

All notable changes to the Essence Wars engine.

Format: `[version] - YYYY-MM-DD` with categories: Added, Changed, Fixed, Removed.

## [0.6.0] - 2026-01-17

### Added
- **Game Modes System**: Two distinct victory conditions for different research scenarios
  - **Attrition** (default): Reduce enemy to 0 life, or turn 30 → higher life wins
  - **Essence Duel** (experimental): First to 50 VP (cumulative face damage) or reduce to 0 life
  - `--mode` flag for arena CLI (`attrition` or `essence-duel`)
  - `GameEngine::start_game_with_mode()` API method
  - 10 unit tests in `tests/unit/game_mode_tests.rs`
- **Interactive Research Dashboards** (GitHub Pages):
  - [Balance Dashboard](https://christianwissmann85.github.io/essence-wars/dashboard/index.html): Faction matchups, deck rankings, P1/P2 analysis
  - [Training Dashboard](https://christianwissmann85.github.io/essence-wars/dashboard/training.html): MCTS weight tuning, fitness curves
  - [Performance Dashboard](https://christianwissmann85.github.io/essence-wars/dashboard/performance.html): Engine benchmarks, throughput metrics
  - Portal landing page at `docs/index.html`
- **Dashboard Generation Scripts**:
  - `python/cardgame/analysis/research_dashboard.py`: Balance validation dashboard generator
  - `python/cardgame/analysis/performance_dashboard.py`: Benchmark dashboard generator
  - `scripts/generate-all-dashboards.sh`: Generate all dashboards in one command
- **Documentation**:
  - `docs/game-modes.md`: Game modes documentation with API reference and research applications

### Changed
- `MatchConfig` now supports `with_game_mode()` builder method
- `GameRunner` now supports `with_game_mode()` for mode-aware game execution
- README.md updated with Game Modes section and dashboard links
- CLAUDE.md updated with Game Modes in Key Design Decisions

## [0.5.0] - 2026-01-14

### Added
- `src/bin/diagnose.rs`: P1/P2 diagnostic tool for analyzing game balance
  - Per-turn resource tracking (life, creatures, attack power, hand size)
  - Win rate breakdown by game phase (early/mid/late)
  - First blood statistics

### Changed
- **First Player Advantage (FPA) compensation redesigned**:
  - Old: P2 started with +1 essence (2 vs 1) - caused P2 to win 55%
  - New: P2 draws +2 extra cards at game start (6 vs 4 cards)
  - Both players now start with equal essence (1 each)
  - Achieves ~50% P1 win rate (validated with 10k games)
- `P2_BONUS_CARDS` constant added to `src/core/config.rs`

### Fixed
- **Critical**: MCTS now detects immediate wins before tree search
  - Previously missed lethal attacks, wasting simulations
  - Now checks all legal actions for instant victory first
- P1/P2 balance: P1 win rate improved from 29% to ~50%

## [0.4.0] - 2026-01-13

### Added
- **New Horizons Expansion**: 60 new faction-based cards (IDs 48-107)
  - **Argentum Combine** (15 cards): Guard, Piercing, Shield focus
  - **Symbiote Circles** (15 cards): Rush, Lethal, Regenerate focus
  - **Obsidion Syndicate** (15 cards): Lifesteal, Stealth, Ephemeral focus
  - **Free-Walkers** (15 cards): Ranged, Charge neutral cards
- Faction decks: `argentum_fortress.toml`, `symbiote_swarm.toml`, `obsidion_shadow.toml`
- `scripts/balance-test.sh` for symmetric faction matchup testing

### Changed
- **First Player Advantage (FPA) compensation**: P2 starts with 2 max essence (was 1)
  - P1 starts with 1 essence on turn 1, P2 starts with 2 essence
  - Brings cross-faction win rates into 45-55% target range
  - Advantage narrows as both approach 10 essence cap
- Design document updated to v1.1 with FPA mechanics documented
- Regression test golden values updated for FPA changes
- Bot validation thresholds adjusted for asymmetric starting resources

### Fixed
- Symbiote P1 vs Argentum dominance (79% → 49%) via FPA fix
- Cross-faction balance: all matchups now within 45-55% target

## [0.3.0] - 2026-01-13

### Added
- **Phase 1.5 Keywords**: 4 new keywords expanding from 8 to 12 total
  - **Ephemeral**: Creature dies at end of turn (triggers OnDeath effects)
  - **Regenerate**: Heals 2 HP at start of owner's turn
  - **Stealth**: Cannot be targeted by enemy attacks/spells (breaks on attack)
  - **Charge**: +2 attack damage when attacking
- 4 new test cards: Ghost Wolf, Frenzied Berserker, Swamp Troll, Shadow Agent
- Keyword weights for AI evaluation: `keyword_ephemeral`, `keyword_regenerate`, `keyword_stealth`, `keyword_charge`
- GreedyWeights now has 24 parameters (was 20)

### Changed
- Keywords upgraded from `u8` to `u16` bitfield (allows 16 keywords)
- Tensor normalization for keyword bitfield: `/65535.0` (was `/255.0`)
- Deck sizes increased to 22 cards (from 18) to include test cards
- Stealth masks Guard (stealthed Guards cannot be targeted)
- Regression test golden values updated for new deck compositions

## [0.2.1] - 2026-01-13

### Added
- `select_action_with_engine()` method to Bot trait for engine-aware bots
- `requires_engine()` method to Bot trait to indicate bots needing simulation access
- `arena_test_deck()` helper in test utilities matching arena's default deck
- Validation in `CardDatabase::load_from_directory()` for missing/empty directories

### Changed
- `GameRunner` now uses `select_action_with_engine()` for all bots
- MCTS stress tests use arena-compatible deck and rollout depth (100)
- Bot validation tests use consistent deck for fair comparisons

### Fixed
- **Critical**: MctsBot now panics with clear message if `select_action()` called without engine
- **Critical**: MCTS stress tests now properly test real tree search (was using wrong deck)
- `CardDatabase::load_from_directory()` no longer silently returns empty database on invalid paths

## [0.2.0] - 2026-01-13

### Added
- Essence/mana system: starts at 0, grows +1/turn, caps at 10, refills each turn
- Version tracking module (`src/version.rs`) for ML reproducibility
- `build.rs` captures git hash at compile time
- Experiments now save `version.toml` with engine version + git commit
- Resource system tests (`tests/unit/resource_system_tests.rs`)
- Property test for random deck compositions

### Changed
- Legal action generation now checks both AP (1 per action) and Essence (card cost)
- MCTS stress test thresholds adjusted for new game dynamics

### Fixed
- **Critical**: Cards can now be played according to mana curve (was broken in 0.1.0)
- Lifesteal creatures now properly enter combat
- Test warnings cleaned up across all test files
- Missing assertion in Ranged vs Guard edge case test

## [0.1.0] - 2026-01-01

Initial release.

### Added
- Core game engine with 8 keywords
- 43-card starter set
- Bot system: RandomBot, GreedyBot, MctsBot
- Arena CLI for running matches
- Weight tuning with CMA-ES optimizer
- Deck system with TOML definitions
- AI interface: tensor (326 floats), action mask (256), rewards
- Deterministic gameplay with seeded RNG
