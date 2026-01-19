//! Seed derivation utilities for deterministic parallel game execution.
//!
//! This module provides standardized seed derivation to ensure reproducible
//! results across parallel game execution in arena, validate, and tune binaries.

/// Seed offset constants used across all binaries.
pub mod offsets {
    /// Offset between consecutive games (implicit in wrapping_add)
    pub const GAME_OFFSET: u64 = 1;

    /// Offset between bot1 and bot2 seeds within a game
    pub const BOT_OFFSET: u64 = 1_000_000;

    /// Offset between matchups
    pub const MATCHUP_OFFSET: u64 = 1_000_000;

    /// Offset between directions (P1 vs P2) within a matchup
    pub const DIRECTION_OFFSET: u64 = 500_000;
}

/// Seeds for a single game, including separate seeds for each bot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameSeeds {
    /// Seed for the game engine (deck shuffling, etc.)
    pub game: u64,
    /// Seed for bot 1
    pub bot1: u64,
    /// Seed for bot 2
    pub bot2: u64,
}

impl GameSeeds {
    /// Create seeds for a simple game (arena-style).
    ///
    /// Used when running games without matchup structure.
    ///
    /// # Arguments
    /// * `base_seed` - The base seed for the entire run
    /// * `game_index` - Zero-based index of this game
    #[inline]
    pub fn for_game(base_seed: u64, game_index: usize) -> Self {
        let game_seed = base_seed.wrapping_add(game_index as u64);
        Self {
            game: game_seed,
            bot1: game_seed,
            bot2: game_seed.wrapping_add(offsets::BOT_OFFSET),
        }
    }

    /// Create seeds for a matchup game (validate-style).
    ///
    /// Used when running games within a matchup/direction structure.
    /// Each matchup gets its own seed space, and within a matchup,
    /// the reverse direction gets an additional offset.
    ///
    /// # Arguments
    /// * `base_seed` - The base seed for the entire run
    /// * `matchup_index` - Zero-based index of this matchup
    /// * `game_index` - Zero-based index of this game within the direction
    /// * `reverse_direction` - Whether this is the reverse direction (P2 as P1)
    #[inline]
    pub fn for_matchup(
        base_seed: u64,
        matchup_index: usize,
        game_index: usize,
        reverse_direction: bool,
    ) -> Self {
        let matchup_seed =
            base_seed.wrapping_add((matchup_index as u64) * offsets::MATCHUP_OFFSET);
        let direction_seed = if reverse_direction {
            matchup_seed.wrapping_add(offsets::DIRECTION_OFFSET)
        } else {
            matchup_seed
        };
        let game_seed = direction_seed.wrapping_add(game_index as u64);
        Self {
            game: game_seed,
            bot1: game_seed,
            bot2: game_seed.wrapping_add(offsets::BOT_OFFSET),
        }
    }

    /// Create seeds from a pre-computed game seed (for compatibility).
    ///
    /// Use this when the game seed has already been computed and you
    /// only need to derive bot seeds.
    #[inline]
    pub fn from_game_seed(game_seed: u64) -> Self {
        Self {
            game: game_seed,
            bot1: game_seed,
            bot2: game_seed.wrapping_add(offsets::BOT_OFFSET),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_seeds_for_game() {
        let seeds = GameSeeds::for_game(42, 0);
        assert_eq!(seeds.game, 42);
        assert_eq!(seeds.bot1, 42);
        assert_eq!(seeds.bot2, 42 + offsets::BOT_OFFSET);

        let seeds = GameSeeds::for_game(42, 5);
        assert_eq!(seeds.game, 47);
        assert_eq!(seeds.bot1, 47);
        assert_eq!(seeds.bot2, 47 + offsets::BOT_OFFSET);
    }

    #[test]
    fn test_game_seeds_for_matchup() {
        // First matchup, first direction, first game
        let seeds = GameSeeds::for_matchup(42, 0, 0, false);
        assert_eq!(seeds.game, 42);
        assert_eq!(seeds.bot1, 42);
        assert_eq!(seeds.bot2, 42 + offsets::BOT_OFFSET);

        // First matchup, reverse direction, first game
        let seeds = GameSeeds::for_matchup(42, 0, 0, true);
        assert_eq!(seeds.game, 42 + offsets::DIRECTION_OFFSET);
        assert_eq!(seeds.bot1, 42 + offsets::DIRECTION_OFFSET);
        assert_eq!(seeds.bot2, 42 + offsets::DIRECTION_OFFSET + offsets::BOT_OFFSET);

        // Second matchup, first direction, first game
        let seeds = GameSeeds::for_matchup(42, 1, 0, false);
        assert_eq!(seeds.game, 42 + offsets::MATCHUP_OFFSET);

        // Second matchup, first direction, 5th game
        let seeds = GameSeeds::for_matchup(42, 1, 4, false);
        assert_eq!(seeds.game, 42 + offsets::MATCHUP_OFFSET + 4);
    }

    #[test]
    fn test_game_seeds_determinism() {
        // Same inputs should always produce same outputs
        let seeds1 = GameSeeds::for_game(12345, 100);
        let seeds2 = GameSeeds::for_game(12345, 100);
        assert_eq!(seeds1, seeds2);

        let seeds1 = GameSeeds::for_matchup(12345, 2, 50, true);
        let seeds2 = GameSeeds::for_matchup(12345, 2, 50, true);
        assert_eq!(seeds1, seeds2);
    }

    #[test]
    fn test_from_game_seed() {
        let game_seed = 999;
        let seeds = GameSeeds::from_game_seed(game_seed);
        assert_eq!(seeds.game, game_seed);
        assert_eq!(seeds.bot1, game_seed);
        assert_eq!(seeds.bot2, game_seed + offsets::BOT_OFFSET);
    }

    #[test]
    fn test_seed_space_separation() {
        // Verify that matchups don't overlap in seed space
        let base = 42u64;

        // Matchup 0 uses seeds starting at 42
        // Matchup 1 uses seeds starting at 42 + 1_000_000
        // With 500 games per direction (1000 total), max seed in matchup 0 is 42 + 999 + 500_000 = 500_541

        let matchup0_max = GameSeeds::for_matchup(base, 0, 999, true);
        let matchup1_min = GameSeeds::for_matchup(base, 1, 0, false);

        // matchup0_max.game should be less than matchup1_min.game
        assert!(
            matchup0_max.game < matchup1_min.game,
            "Matchup seed spaces should not overlap"
        );
    }
}
