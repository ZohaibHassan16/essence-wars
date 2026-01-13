# Changelog

All notable changes to the Essence Wars engine.

Format: `[version] - YYYY-MM-DD` with categories: Added, Changed, Fixed, Removed.

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
